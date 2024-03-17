use crate::{
    component_container::ComponentContainer,
    query::ComponentContainerTrait,
    system::{Borrow, BorrowType, RunState},
    Component, Ref, RefMut,
};
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLockReadGuard, RwLockWriteGuard,
};
use std::any::TypeId;

/// The type parameter for a [`Query`](crate::Query)
pub trait QueryParameter {
    /// The lock returned from [`QueryParameter::lock`]
    type ComponentContainerLock<'a>;
    /// The component container that is used to get [`Component`]s from
    type ComponentContainer<'a>: ComponentContainerTrait<'a>;

    /// Locks any needed state, the first step to creating a [`Query`](crate::Query)
    fn lock<'a>(state: &RunState<'a>) -> Self::ComponentContainerLock<'a>;
    /// Constructs the component container from the locked state, the final state to creating a [`Query`](crate::Query)
    fn construct<'a>(
        lock: &'a mut Self::ComponentContainerLock<'_>,
    ) -> Self::ComponentContainer<'a>;

    /// Returns an iterator over all the [`Component`] types that will be locked
    fn get_component_types() -> impl Iterator<Item = Borrow>;
}

impl<C> QueryParameter for Ref<C>
where
    C: Component,
{
    type ComponentContainerLock<'a> = Option<MappedRwLockReadGuard<'a, ComponentContainer<C>>>;
    type ComponentContainer<'a> = Option<&'a ComponentContainer<C>>;

    fn lock<'a>(state: &RunState<'a>) -> Self::ComponentContainerLock<'a> {
        Some(RwLockReadGuard::map(
            state
                .components
                .get(&TypeId::of::<C>())?
                .try_read()
                .expect("the lock should always be available"),
            |components| components.downcast_ref::<C>(),
        ))
    }

    fn construct<'a>(
        lock: &'a mut Self::ComponentContainerLock<'_>,
    ) -> Self::ComponentContainer<'a> {
        Some(lock.as_mut()?)
    }

    fn get_component_types() -> impl Iterator<Item = Borrow> {
        std::iter::once(Borrow {
            id: TypeId::of::<C>(),
            name: std::any::type_name::<C>(),
            borrow_type: BorrowType::Immutable,
        })
    }
}

impl<C> QueryParameter for RefMut<C>
where
    C: Component,
{
    type ComponentContainerLock<'a> = Option<MappedRwLockWriteGuard<'a, ComponentContainer<C>>>;
    type ComponentContainer<'a> = Option<&'a mut ComponentContainer<C>>;

    fn lock<'a>(state: &RunState<'a>) -> Self::ComponentContainerLock<'a> {
        Some(RwLockWriteGuard::map(
            state
                .components
                .get(&TypeId::of::<C>())?
                .try_write()
                .expect("the lock should always be available"),
            |components| components.downcast_mut::<C>(),
        ))
    }

    fn construct<'a>(
        lock: &'a mut Self::ComponentContainerLock<'_>,
    ) -> Self::ComponentContainer<'a> {
        Some(lock.as_mut()?)
    }

    fn get_component_types() -> impl Iterator<Item = Borrow> {
        std::iter::once(Borrow {
            id: TypeId::of::<C>(),
            name: std::any::type_name::<C>(),
            borrow_type: BorrowType::Mutable,
        })
    }
}

pub struct OptionalComponentContainer<T>(pub(crate) T);

impl<P> QueryParameter for Option<P>
where
    P: QueryParameter,
{
    type ComponentContainerLock<'a> = P::ComponentContainerLock<'a>;
    type ComponentContainer<'a> = OptionalComponentContainer<P::ComponentContainer<'a>>;

    fn lock<'a>(state: &RunState<'a>) -> Self::ComponentContainerLock<'a> {
        P::lock(state)
    }

    fn construct<'a>(
        lock: &'a mut Self::ComponentContainerLock<'_>,
    ) -> Self::ComponentContainer<'a> {
        OptionalComponentContainer(P::construct(lock))
    }

    fn get_component_types() -> impl Iterator<Item = Borrow> {
        P::get_component_types()
    }
}

macro_rules! query_parameter_tuple {
    ($($param:ident),*) => {
        impl<$($param),*> QueryParameter for ($($param,)*)
        where
            $($param: QueryParameter,)*
        {
            type ComponentContainerLock<'a> = ($($param::ComponentContainerLock<'a>,)*);
            type ComponentContainer<'a> = ($($param::ComponentContainer<'a>,)*);

            #[allow(clippy::unused_unit)]
            fn lock<'a>(state: &RunState<'a>) -> Self::ComponentContainerLock<'a> {
                _ = state;
                ($($param::lock(state),)*)
            }

            #[allow(clippy::unused_unit)]
            fn construct<'this>(state: &'this mut Self::ComponentContainerLock<'_>) -> Self::ComponentContainer<'this> {
                #[allow(non_snake_case)]
                let ($($param,)*) = state;
                ($($param::construct($param),)*)
            }

            fn get_component_types() -> impl Iterator<Item = Borrow> {
                std::iter::empty()
                    $(
                        .chain($param::get_component_types())
                    )*
            }
        }
    };
}

query_parameter_tuple!();
query_parameter_tuple!(A);
query_parameter_tuple!(A, B);
query_parameter_tuple!(A, B, C);
query_parameter_tuple!(A, B, C, D);
query_parameter_tuple!(A, B, C, D, E);
query_parameter_tuple!(A, B, C, D, E, F);
query_parameter_tuple!(A, B, C, D, E, F, G);
query_parameter_tuple!(A, B, C, D, E, F, G, H);
query_parameter_tuple!(A, B, C, D, E, F, G, H, I);
query_parameter_tuple!(A, B, C, D, E, F, G, H, I, J);
query_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K);
query_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
query_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
query_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
query_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
query_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
