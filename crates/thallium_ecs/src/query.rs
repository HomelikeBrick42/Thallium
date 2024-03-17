use crate::{
    component_container::ComponentContainer,
    entities::EntityMap,
    query_parameters::OptionalComponentContainer,
    system::{Borrow, RunState},
    Component, Entity, QueryParameter, SystemParameter,
};
use std::marker::PhantomData;

/// The [`QueryParameter`] for getting a reference to a [`Component`]
pub struct Ref<C>(PhantomData<fn() -> C>)
where
    C: Component;

/// The [`QueryParameter`] for getting a mutable reference to a [`Component`]
pub struct RefMut<C>(PhantomData<fn() -> C>)
where
    C: Component;

/// A [`SystemParameter`] that lets you get references to [`Component`]s specified in `Q`
pub struct Query<'a, Q>
where
    Q: QueryParameter,
{
    entities: &'a EntityMap,
    container: Q::ComponentContainer<'a>,
}

impl<'a, Q> SystemParameter for Query<'a, Q>
where
    Q: QueryParameter,
{
    type This<'this> = Query<'this, Q>;
    type Lock<'state> = (&'state EntityMap, Q::ComponentContainerLock<'state>);

    fn lock<'state>(state: &RunState<'state>) -> Self::Lock<'state> {
        (state.entities, Q::lock(state))
    }

    fn construct<'this>(state: &'this mut Self::Lock<'_>) -> Self::This<'this> {
        let (entities, state) = state;
        Query {
            entities,
            container: Q::construct(state),
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
        self.container.get(entity)
    }

    /// Gets mutable access to the [`Component`]s that are attached to `entity`
    pub fn get_mut<'b>(
        &'b mut self,
        entity: Entity,
    ) -> Option<<Q::ComponentContainer<'a> as ComponentContainerTrait<'a>>::ParameterMut<'b>> {
        self.container.get_mut(entity)
    }

    /// Semantically the same as calling [`Query::get_mut`] multiple times for each [`Entity`] in the `entities` array, also returns [`None`] if there are any duplicates in `entities`
    pub fn get_many_mut<'b, const N: usize>(
        &'b mut self,
        entities: [Entity; N],
    ) -> Option<[<Q::ComponentContainer<'a> as ComponentContainerTrait<'a>>::ParameterMut<'b>; N]>
    {
        self.container.get_many_mut(entities)
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
            .zip(self.container.iter())
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
            .zip(self.container.iter_mut())
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

    fn get(&self, entity: Entity) -> Option<Self::Parameter<'_>>;
    fn get_mut(&mut self, entity: Entity) -> Option<Self::ParameterMut<'_>>;
    fn get_many_mut<const N: usize>(
        &mut self,
        entities: [Entity; N],
    ) -> Option<[Self::ParameterMut<'_>; N]>;

    fn iter(&self) -> impl Iterator<Item = Option<Self::Parameter<'_>>>;
    fn iter_mut(&mut self) -> impl Iterator<Item = Option<Self::ParameterMut<'_>>>;
}

impl<'a, C> ComponentContainerTrait<'a> for Option<&'a ComponentContainer<C>>
where
    C: Component,
{
    type Parameter<'param> = &'param C
    where
        Self: 'param;
    type ParameterMut<'param> = &'param C
    where
        Self: 'param;

    fn get(&self, entity: Entity) -> Option<Self::Parameter<'_>> {
        ComponentContainer::<C>::get(self.as_ref()?, entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<Self::ParameterMut<'_>> {
        ComponentContainer::<C>::get(self.as_mut()?, entity)
    }

    fn get_many_mut<const N: usize>(
        &mut self,
        entities: [Entity; N],
    ) -> Option<[Self::ParameterMut<'_>; N]> {
        let container = self.as_mut()?;

        // this could be replaced with `.try_map` when its stablized which would remove the double-iteration
        if entities
            .iter()
            .all(|&entity| container.get(entity).is_some())
        {
            Some(entities.map(|entity| container.get(entity).unwrap()))
        } else {
            None
        }
    }

    fn iter(&self) -> impl Iterator<Item = Option<Self::Parameter<'_>>> {
        self.as_ref().into_iter().flat_map(|this| {
            this.components.iter().map(|slot| {
                if let Some((_, component)) = slot {
                    Some(component)
                } else {
                    None
                }
            })
        })
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = Option<Self::ParameterMut<'_>>> {
        self.as_mut().into_iter().flat_map(|this| {
            this.components.iter().map(|slot| {
                if let Some((_, component)) = slot {
                    Some(component)
                } else {
                    None
                }
            })
        })
    }
}

impl<'a, C> ComponentContainerTrait<'a> for Option<&'a mut ComponentContainer<C>>
where
    C: Component,
{
    type Parameter<'param> = &'param C
    where
        Self: 'param;
    type ParameterMut<'param> = &'param mut C
    where
        Self: 'param;

    fn get(&self, entity: Entity) -> Option<Self::Parameter<'_>> {
        ComponentContainer::<C>::get(self.as_ref()?, entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<Self::ParameterMut<'_>> {
        ComponentContainer::<C>::get_mut(self.as_mut()?, entity)
    }

    fn get_many_mut<const N: usize>(
        &mut self,
        entities: [Entity; N],
    ) -> Option<[Self::ParameterMut<'_>; N]> {
        ComponentContainer::<C>::get_many_mut(self.as_mut()?, entities)
    }

    fn iter(&self) -> impl Iterator<Item = Option<Self::Parameter<'_>>> {
        self.as_ref().into_iter().flat_map(|this| {
            this.components.iter().map(|slot| {
                if let Some((_, component)) = slot {
                    Some(component)
                } else {
                    None
                }
            })
        })
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = Option<Self::ParameterMut<'_>>> {
        self.as_mut().into_iter().flat_map(|this| {
            this.components.iter_mut().map(|slot| {
                if let Some((_, component)) = slot {
                    Some(component)
                } else {
                    None
                }
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

    fn get(&self, entity: Entity) -> Option<Self::Parameter<'_>> {
        Some(self.0.get(entity))
    }

    fn get_mut(&mut self, entity: Entity) -> Option<Self::ParameterMut<'_>> {
        Some(self.0.get_mut(entity))
    }

    fn get_many_mut<const N: usize>(
        &mut self,
        entities: [Entity; N],
    ) -> Option<[Self::ParameterMut<'_>; N]> {
        let mut parameters = self.0.get_many_mut(entities).map(IntoIterator::into_iter);
        Some(std::array::from_fn(|_| {
            parameters
                .as_mut()
                .map(|parameter| parameter.next().unwrap())
        }))
    }

    fn iter(&self) -> impl Iterator<Item = Option<Self::Parameter<'_>>> {
        self.0.iter().map(Some)
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = Option<Self::ParameterMut<'_>>> {
        self.0.iter_mut().map(Some)
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

            fn get(&self, entity: Entity) -> Option<Self::Parameter<'_>> {
                _ = entity;
                #[allow(non_snake_case)]
                let ($($param,)*) = self;
                Some(($($param.get(entity)?,)*))
            }

            fn get_mut(&mut self, entity: Entity) -> Option<Self::ParameterMut<'_>> {
                _ = entity;
                #[allow(non_snake_case)]
                let ($($param,)*) = self;
                Some(($($param.get_mut(entity)?,)*))
            }

            fn get_many_mut<const LEN: usize>(
                &mut self,
                entities: [Entity; LEN],
            ) -> Option<[Self::ParameterMut<'_>; LEN]> {
                _ = entities;
                #[allow(non_snake_case)]
                let ($($param,)*) = self;
                $(
                    #[allow(non_snake_case)]
                    let mut $param = $param.get_many_mut(entities)?.into_iter();
                )*
                Some(std::array::from_fn(|_| ($($param.next().unwrap(),)*)))
            }

            fn iter(&self) -> impl Iterator<Item = Option<Self::Parameter<'_>>> {
                #[allow(non_snake_case)]
                let ($($param,)*) = self;
                $(
                    #[allow(non_snake_case)]
                    let mut $param = $param.iter();
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

            fn iter_mut(&mut self) -> impl Iterator<Item = Option<Self::ParameterMut<'_>>> {
                #[allow(non_snake_case)]
                let ($($param,)*) = self;
                $(
                    #[allow(non_snake_case)]
                    let mut $param = $param.iter_mut();
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
