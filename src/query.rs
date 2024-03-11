use crate::{
    entities::Entity,
    query_parameters::{OptionalComponentContainer, QueryParameter},
    system::{Borrow, RunState},
    system_parameters::SystemParameter,
};
use slotmap::{SecondaryMap, SlotMap};
use std::marker::PhantomData;

pub trait Component: Sized + Send + Sync + 'static {}

pub struct Ref<C>(PhantomData<fn() -> C>)
where
    C: Component;

pub struct RefMut<C>(PhantomData<fn() -> C>)
where
    C: Component;

pub struct Query<'a, Q>
where
    Q: QueryParameter,
{
    entities: &'a SlotMap<Entity, ()>,
    container: Q::ComponentContainer<'a>,
}

impl<'a, Q> SystemParameter for Query<'a, Q>
where
    Q: QueryParameter,
{
    type This<'this> = Query<'this, Q>;
    type Lock<'state> = (
        &'state SlotMap<Entity, ()>,
        Q::ComponentContainerLock<'state>,
    );

    fn lock(state: RunState<'_>) -> Self::Lock<'_> {
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
    pub fn get<'b>(
        &'b self,
        entity: Entity,
    ) -> Option<<Q::ComponentContainer<'a> as ComponentContainerTrait<'a>>::Parameter<'b>> {
        if self.entities.contains_key(entity) {
            self.container.get(entity)
        } else {
            None
        }
    }

    pub fn get_mut<'b>(
        &'b mut self,
        entity: Entity,
    ) -> Option<<Q::ComponentContainer<'a> as ComponentContainerTrait<'a>>::ParameterMut<'b>> {
        if self.entities.contains_key(entity) {
            self.container.get_mut(entity)
        } else {
            None
        }
    }

    pub fn get_many_mut<'b, const N: usize>(
        &'b mut self,
        entities: [Entity; N],
    ) -> Option<[<Q::ComponentContainer<'a> as ComponentContainerTrait<'a>>::ParameterMut<'b>; N]>
    {
        if entities
            .iter()
            .all(|&entity| self.entities.contains_key(entity))
        {
            self.container.get_many_mut(entities)
        } else {
            None
        }
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
}

impl<'a, C> ComponentContainerTrait<'a> for Option<&'a SecondaryMap<Entity, C>>
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
        SecondaryMap::get(self.as_ref()?, entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<Self::ParameterMut<'_>> {
        SecondaryMap::get(self.as_mut()?, entity)
    }

    fn get_many_mut<const N: usize>(
        &mut self,
        entities: [Entity; N],
    ) -> Option<[Self::ParameterMut<'_>; N]> {
        let container = self.as_mut()?;

        // this could be replaced with `.try_map` when its stablized which would remove the double-iteration
        if entities
            .iter()
            .all(|&entity| container.contains_key(entity))
        {
            Some(entities.map(|entity| container.get(entity).unwrap()))
        } else {
            None
        }
    }
}

impl<'a, C> ComponentContainerTrait<'a> for Option<&'a mut SecondaryMap<Entity, C>>
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
        SecondaryMap::get(self.as_ref()?, entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<Self::ParameterMut<'_>> {
        SecondaryMap::get_mut(self.as_mut()?, entity)
    }

    fn get_many_mut<const N: usize>(
        &mut self,
        entities: [Entity; N],
    ) -> Option<[Self::ParameterMut<'_>; N]> {
        SecondaryMap::get_disjoint_mut(self.as_mut()?, entities)
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
