use crate::{
    component::Component,
    component_container::ComponentContainer,
    query::{ComponentContainerTrait, Ref, RefMut},
    system::{Borrow, BorrowType, RunState},
};
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLockReadGuard, RwLockWriteGuard,
};
use std::any::TypeId;

pub trait QueryParameter {
    type ComponentContainerLock<'a>;
    type ComponentContainer<'a>: ComponentContainerTrait<'a>;

    fn lock(state: RunState<'_>) -> Self::ComponentContainerLock<'_>;
    fn construct<'a>(
        lock: &'a mut Self::ComponentContainerLock<'_>,
    ) -> Self::ComponentContainer<'a>;
    fn get_component_types() -> impl Iterator<Item = Borrow>;
}

impl<C> QueryParameter for Ref<C>
where
    C: Component,
{
    type ComponentContainerLock<'a> = Option<MappedRwLockReadGuard<'a, ComponentContainer<C>>>;
    type ComponentContainer<'a> = Option<&'a ComponentContainer<C>>;

    fn lock(state: RunState<'_>) -> Self::ComponentContainerLock<'_> {
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

    fn lock(state: RunState<'_>) -> Self::ComponentContainerLock<'_> {
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

    fn lock(state: RunState<'_>) -> Self::ComponentContainerLock<'_> {
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
            fn lock(state: RunState<'_>) -> Self::ComponentContainerLock<'_> {
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
