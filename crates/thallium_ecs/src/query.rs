use crate::{
    component_container::{ComponentContainer, ComponentSlot},
    entities::EntityMap,
    query_parameters::OptionalComponentContainer,
    system::{Borrow, SystemRunState},
    Component, Entity, QueryParameter, SystemParameter,
};
use std::ops::{Deref, DerefMut};

/// The [`QueryParameter`] for getting a reference to a [`Component`]
pub struct Ref<'a, C>
where
    C: Component,
{
    pub(crate) component: &'a C,
    pub(crate) last_modified_tick: u64,
    pub(crate) last_run_tick: u64,
}

impl<'a, C> Ref<'a, C>
where
    C: Component,
{
    /// Returns whether this [`Component`] has been modified since the last system has run
    pub fn get_modified(&self) -> bool {
        self.last_run_tick < self.last_modified_tick
    }
}

impl<'a, C> Deref for Ref<'a, C>
where
    C: Component,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.component
    }
}

/// The [`QueryParameter`] for getting a mutable reference to a [`Component`]
pub struct RefMut<'a, C>
where
    C: Component,
{
    pub(crate) component: &'a mut C,
    pub(crate) last_modified_tick: &'a mut u64,
    pub(crate) last_run_tick: u64,
    pub(crate) current_tick: u64,
}

impl<'a, C> RefMut<'a, C>
where
    C: Component,
{
    /// Returns whether this [`Component`] has been modified since the last time this system was run
    pub fn get_modified(&self) -> bool {
        self.last_run_tick < *self.last_modified_tick
    }

    /// Gets a reference to the inner [`Component`] without triggering the modification detection
    pub fn silently_modify(&mut self) -> &mut C {
        self.component
    }
}

impl<'a, C> Deref for RefMut<'a, C>
where
    C: Component,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.component
    }
}

impl<'a, C> DerefMut for RefMut<'a, C>
where
    C: Component,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        *self.last_modified_tick = self.current_tick;
        self.component
    }
}

/// A [`SystemParameter`] that lets you get references to [`Component`]s specified in `Q`
pub struct Query<'a, Q>
where
    Q: QueryParameter,
{
    entities: &'a EntityMap,
    container: Q::ComponentContainer<'a>,
    current_tick: u64,
    last_run_tick: u64,
}

impl<'a, Q> SystemParameter for Query<'a, Q>
where
    Q: QueryParameter,
{
    type This<'this> = Query<'this, Q>;
    type Lock<'state> = (&'state EntityMap, Q::ComponentContainerLock<'state>, u64);

    fn lock<'state>(state: &SystemRunState<'state>) -> Self::Lock<'state> {
        (state.entities, Q::lock(state), state.current_tick)
    }

    fn construct<'this>(state: &'this mut Self::Lock<'_>, last_run_tick: u64) -> Self::This<'this> {
        let (entities, state, current_tick) = state;
        Query {
            entities,
            container: Q::construct(state),
            current_tick: *current_tick,
            last_run_tick,
        }
    }

    fn get_resource_types() -> impl Iterator<Item = Borrow> {
        std::iter::empty()
    }

    fn get_component_types() -> impl Iterator<Item = Borrow> {
        Q::get_component_types()
    }
}

impl<'a, Q> Query<'a, Q>
where
    Q: QueryParameter,
{
    /// Gets access to the [`Component`]s that are attached to `entity`
    pub fn get<'b>(
        &'b self,
        entity: Entity,
    ) -> Option<<Q::ComponentContainer<'a> as ComponentContainerTrait<'a>>::Parameter<'b>> {
        self.container.get(self.last_run_tick, entity)
    }

    /// Gets mutable access to the [`Component`]s that are attached to `entity`
    pub fn get_mut<'b>(
        &'b mut self,
        entity: Entity,
    ) -> Option<<Q::ComponentContainer<'a> as ComponentContainerTrait<'a>>::ParameterMut<'b>> {
        self.container
            .get_mut(self.last_run_tick, self.current_tick, entity)
    }

    /// Semantically the same as calling [`Query::get_mut`] multiple times for each [`Entity`] in the `entities` array, also returns [`None`] if there are any duplicates in `entities`
    pub fn get_many_mut<'b, const N: usize>(
        &'b mut self,
        entities: [Entity; N],
    ) -> Option<[<Q::ComponentContainer<'a> as ComponentContainerTrait<'a>>::ParameterMut<'b>; N]>
    {
        self.container
            .get_many_mut(self.last_run_tick, self.current_tick, entities)
    }

    /// Returns an iterator over all the [`Component`]s, also gives the [`Entity`] that the [`Component`]s are attached to
    pub fn iter<'b>(
        &'b self,
    ) -> impl Iterator<
        Item = (
            Entity,
            <Q::ComponentContainer<'a> as ComponentContainerTrait<'a>>::Parameter<'b>,
        ),
    > + 'b {
        self.entities
            .iter()
            .zip(self.container.iter(self.last_run_tick))
            .filter_map(|(entity, parameter)| entity.zip(parameter))
    }

    /// Returns a mutable iterator over all the [`Component`]s, also gives the [`Entity`] that the [`Component`]s are attached to
    pub fn iter_mut<'b>(
        &'b mut self,
    ) -> impl Iterator<
        Item = (
            Entity,
            <Q::ComponentContainer<'a> as ComponentContainerTrait<'a>>::ParameterMut<'b>,
        ),
    > + 'b {
        self.entities
            .iter()
            .zip(
                self.container
                    .iter_mut(self.last_run_tick, self.current_tick),
            )
            .filter_map(|(entity, parameter)| entity.zip(parameter))
    }
}

pub trait ComponentContainerTrait<'a>: Send + Sync {
    type Parameter<'param>
    where
        Self: 'param;
    type ParameterMut<'param>
    where
        Self: 'param;

    fn get(&self, last_run_tick: u64, entity: Entity) -> Option<Self::Parameter<'_>>;
    fn get_mut(
        &mut self,
        last_run_tick: u64,
        current_tick: u64,
        entity: Entity,
    ) -> Option<Self::ParameterMut<'_>>;
    fn get_many_mut<const N: usize>(
        &mut self,
        last_run_tick: u64,
        current_tick: u64,
        entities: [Entity; N],
    ) -> Option<[Self::ParameterMut<'_>; N]>;

    fn iter(&self, last_run_tick: u64) -> impl Iterator<Item = Option<Self::Parameter<'_>>>;
    fn iter_mut(
        &mut self,
        last_run_tick: u64,
        current_tick: u64,
    ) -> impl Iterator<Item = Option<Self::ParameterMut<'_>>>;
}

impl<'a, C> ComponentContainerTrait<'a> for Option<&'a ComponentContainer<C>>
where
    C: Component,
{
    type Parameter<'param> = Ref<'param, C>
    where
        Self: 'param;
    type ParameterMut<'param> = Ref<'param, C>
    where
        Self: 'param;

    fn get(&self, last_run_tick: u64, entity: Entity) -> Option<Self::Parameter<'_>> {
        ComponentContainer::<C>::get(self.as_ref()?, last_run_tick, entity)
    }

    fn get_mut(
        &mut self,
        last_run_tick: u64,
        current_tick: u64,
        entity: Entity,
    ) -> Option<Self::ParameterMut<'_>> {
        _ = current_tick;
        ComponentContainer::<C>::get(self.as_mut()?, last_run_tick, entity)
    }

    fn get_many_mut<const N: usize>(
        &mut self,
        last_run_tick: u64,
        current_tick: u64,
        entities: [Entity; N],
    ) -> Option<[Self::ParameterMut<'_>; N]> {
        _ = current_tick;
        let container = self.as_mut()?;

        // this could be replaced with `.try_map` when its stablized which would remove the double-iteration
        if entities
            .iter()
            .all(|&entity| container.get(last_run_tick, entity).is_some())
        {
            Some(entities.map(|entity| container.get(last_run_tick, entity).unwrap()))
        } else {
            None
        }
    }

    fn iter(&self, last_run_tick: u64) -> impl Iterator<Item = Option<Self::Parameter<'_>>> {
        self.as_ref().into_iter().flat_map(move |this| {
            this.components.iter().map(move |slot| {
                slot.as_ref().map(
                    |&ComponentSlot {
                         ref component,
                         last_modified_tick,
                         ..
                     }| Ref {
                        component,
                        last_modified_tick,
                        last_run_tick,
                    },
                )
            })
        })
    }

    fn iter_mut(
        &mut self,
        last_run_tick: u64,
        current_tick: u64,
    ) -> impl Iterator<Item = Option<Self::ParameterMut<'_>>> {
        _ = current_tick;
        self.as_mut().into_iter().flat_map(move |this| {
            this.components.iter().map(move |slot| {
                slot.as_ref().map(
                    |&ComponentSlot {
                         ref component,
                         last_modified_tick,
                         ..
                     }| Ref {
                        component,
                        last_modified_tick,
                        last_run_tick,
                    },
                )
            })
        })
    }
}

impl<'a, C> ComponentContainerTrait<'a> for Option<&'a mut ComponentContainer<C>>
where
    C: Component,
{
    type Parameter<'param> = Ref<'param, C>
    where
        Self: 'param;
    type ParameterMut<'param> = RefMut<'param, C>
    where
        Self: 'param;

    fn get(&self, last_run_tick: u64, entity: Entity) -> Option<Self::Parameter<'_>> {
        ComponentContainer::<C>::get(self.as_ref()?, last_run_tick, entity)
    }

    fn get_mut(
        &mut self,
        last_run_tick: u64,
        current_tick: u64,
        entity: Entity,
    ) -> Option<Self::ParameterMut<'_>> {
        ComponentContainer::<C>::get_mut(self.as_mut()?, last_run_tick, current_tick, entity)
    }

    fn get_many_mut<const N: usize>(
        &mut self,
        last_run_tick: u64,
        current_tick: u64,
        entities: [Entity; N],
    ) -> Option<[Self::ParameterMut<'_>; N]> {
        ComponentContainer::<C>::get_many_mut(self.as_mut()?, last_run_tick, current_tick, entities)
    }

    fn iter(&self, last_run_tick: u64) -> impl Iterator<Item = Option<Self::Parameter<'_>>> {
        self.as_ref().into_iter().flat_map(move |this| {
            this.components.iter().map(move |slot| {
                slot.as_ref().map(
                    |&ComponentSlot {
                         ref component,
                         last_modified_tick,
                         ..
                     }| Ref {
                        component,
                        last_modified_tick,
                        last_run_tick,
                    },
                )
            })
        })
    }

    fn iter_mut(
        &mut self,
        last_run_tick: u64,
        current_tick: u64,
    ) -> impl Iterator<Item = Option<Self::ParameterMut<'_>>> {
        self.as_mut().into_iter().flat_map(move |this| {
            this.components.iter_mut().map(move |slot| {
                slot.as_mut().map(
                    |ComponentSlot {
                         component,
                         last_modified_tick,
                         ..
                     }| RefMut {
                        component,
                        last_modified_tick,
                        last_run_tick,
                        current_tick,
                    },
                )
            })
        })
    }
}

impl<'a, T> ComponentContainerTrait<'a> for OptionalComponentContainer<T>
where
    T: ComponentContainerTrait<'a>,
{
    type Parameter<'param> = Option<T::Parameter<'param>>
    where
        Self: 'param;

    type ParameterMut<'param> = Option<T::ParameterMut<'param>>
    where
        Self: 'param;

    fn get(&self, last_run_tick: u64, entity: Entity) -> Option<Self::Parameter<'_>> {
        Some(self.0.get(last_run_tick, entity))
    }

    fn get_mut(
        &mut self,
        last_run_tick: u64,
        current_tick: u64,
        entity: Entity,
    ) -> Option<Self::ParameterMut<'_>> {
        Some(self.0.get_mut(last_run_tick, current_tick, entity))
    }

    fn get_many_mut<const N: usize>(
        &mut self,
        last_run_tick: u64,
        current_tick: u64,
        entities: [Entity; N],
    ) -> Option<[Self::ParameterMut<'_>; N]> {
        let mut parameters = self
            .0
            .get_many_mut(last_run_tick, current_tick, entities)
            .map(IntoIterator::into_iter);
        Some(std::array::from_fn(|_| {
            parameters
                .as_mut()
                .map(|parameter| parameter.next().unwrap())
        }))
    }

    fn iter(&self, last_run_tick: u64) -> impl Iterator<Item = Option<Self::Parameter<'_>>> {
        self.0.iter(last_run_tick).map(Some)
    }

    fn iter_mut(
        &mut self,
        last_run_tick: u64,
        current_tick: u64,
    ) -> impl Iterator<Item = Option<Self::ParameterMut<'_>>> {
        self.0.iter_mut(last_run_tick, current_tick).map(Some)
    }
}

macro_rules! component_container_tuple {
    ($($param:ident),*) => {
        impl<'a, $($param),*> ComponentContainerTrait<'a> for ($($param,)*)
        where
            $($param: ComponentContainerTrait<'a>,)*
        {
            type Parameter<'param> = ($($param::Parameter<'param>,)*)
            where
                Self: 'param;
            type ParameterMut<'param> = ($($param::ParameterMut<'param>,)*)
            where
                Self: 'param;

            fn get(&self, last_run_tick: u64, entity: Entity) -> Option<Self::Parameter<'_>> {
                _ = last_run_tick;
                _ = entity;
                #[allow(non_snake_case)]
                let ($($param,)*) = self;
                Some(($($param.get(last_run_tick, entity)?,)*))
            }

            fn get_mut(
                &mut self,
                last_run_tick: u64,
                current_tick: u64,
                entity: Entity,
            ) -> Option<Self::ParameterMut<'_>> {
                _ = last_run_tick;
                _ = current_tick;
                _ = entity;
                #[allow(non_snake_case)]
                let ($($param,)*) = self;
                Some(($($param.get_mut(last_run_tick, current_tick, entity)?,)*))
            }

            fn get_many_mut<const LEN: usize>(
                &mut self,
                last_run_tick: u64,
                current_tick: u64,
                entities: [Entity; LEN],
            ) -> Option<[Self::ParameterMut<'_>; LEN]> {
                _ = last_run_tick;
                _ = current_tick;
                _ = entities;
                #[allow(non_snake_case)]
                let ($($param,)*) = self;
                $(
                    #[allow(non_snake_case)]
                    let mut $param = $param.get_many_mut(last_run_tick, current_tick, entities)?.into_iter();
                )*
                Some(std::array::from_fn(|_| ($($param.next().unwrap(),)*)))
            }

            fn iter(&self, last_run_tick: u64) -> impl Iterator<Item = Option<Self::Parameter<'_>>> {
                _ = last_run_tick;
                #[allow(non_snake_case)]
                let ($($param,)*) = self;
                $(
                    #[allow(non_snake_case)]
                    let mut $param = $param.iter(last_run_tick);
                )*
                std::iter::from_fn(move || {
                    $(
                        #[allow(non_snake_case)]
                        let $param = $param.next()?;
                    )*
                    $(
                        #[allow(non_snake_case)]
                        let Some($param) = $param else {
                            return Some(None);
                        };
                    )*
                    Some(Some(($($param,)*)))
                })
            }

            fn iter_mut(
                &mut self,
                last_run_tick: u64,
                current_tick: u64,
            ) -> impl Iterator<Item = Option<Self::ParameterMut<'_>>> {
                _ = last_run_tick;
                _ = current_tick;
                #[allow(non_snake_case)]
                let ($($param,)*) = self;
                $(
                    #[allow(non_snake_case)]
                    let mut $param = $param.iter_mut(last_run_tick, current_tick);
                )*
                std::iter::from_fn(move || {
                    $(
                        #[allow(non_snake_case)]
                        let $param = $param.next()?;
                    )*
                    $(
                        #[allow(non_snake_case)]
                        let Some($param) = $param else {
                            return Some(None);
                        };
                    )*
                    Some(Some(($($param,)*)))
                })
            }
        }
    };
}

component_container_tuple!();
component_container_tuple!(A);
component_container_tuple!(A, B);
component_container_tuple!(A, B, C);
component_container_tuple!(A, B, C, D);
component_container_tuple!(A, B, C, D, E);
component_container_tuple!(A, B, C, D, E, F);
component_container_tuple!(A, B, C, D, E, F, G);
component_container_tuple!(A, B, C, D, E, F, G, H);
component_container_tuple!(A, B, C, D, E, F, G, H, I);
component_container_tuple!(A, B, C, D, E, F, G, H, I, J);
component_container_tuple!(A, B, C, D, E, F, G, H, I, J, K);
component_container_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
component_container_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
component_container_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
component_container_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
component_container_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
