use crate::{
    resource::{Res, ResMut, Resource},
    system::{Borrow, BorrowType, RunState},
};
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLockReadGuard, RwLockWriteGuard,
};
use std::any::TypeId;

pub trait SystemParameter: Send + Sync {
    type This<'this>;
    type Lock<'state>;

    fn lock(state: RunState<'_>) -> Self::Lock<'_>;
    fn construct<'this>(state: &'this mut Self::Lock<'_>) -> Self::This<'this>;
    fn get_resource_types() -> impl Iterator<Item = Borrow>;
    fn get_component_types() -> impl Iterator<Item = Borrow>;
}

impl<'a, R> SystemParameter for Res<'a, R>
where
    R: Resource,
{
    type This<'this> = Res<'this, R>;
    type Lock<'state> = MappedRwLockReadGuard<'state, R>;

    fn lock(state: RunState<'_>) -> Self::Lock<'_> {
        RwLockReadGuard::map(
            state
                .resources
                .get(&TypeId::of::<R>())
                .expect("Non-Option Res expects the resource to always exist")
                .try_read()
                .expect("the lock should always be available"),
            |resource| resource.downcast_ref::<R>().unwrap(),
        )
    }

    fn construct<'this>(state: &'this mut Self::Lock<'_>) -> Self::This<'this> {
        Res { inner: state }
    }

    fn get_resource_types() -> impl Iterator<Item = Borrow> {
        std::iter::once(Borrow {
            id: TypeId::of::<R>(),
            name: std::any::type_name::<R>(),
            borrow_type: BorrowType::Immutable,
        })
    }

    fn get_component_types() -> impl Iterator<Item = Borrow> {
        std::iter::empty()
    }
}

impl<'a, R> SystemParameter for ResMut<'a, R>
where
    R: Resource,
{
    type This<'this> = ResMut<'this, R>;
    type Lock<'state> = MappedRwLockWriteGuard<'state, R>;

    fn lock(state: RunState<'_>) -> Self::Lock<'_> {
        RwLockWriteGuard::map(
            state
                .resources
                .get(&TypeId::of::<R>())
                .expect("Non-Option ResMut expects the resource to always exist")
                .try_write()
                .expect("the lock should always be available"),
            |resource| resource.downcast_mut::<R>().unwrap(),
        )
    }

    fn construct<'this>(state: &'this mut Self::Lock<'_>) -> Self::This<'this> {
        ResMut { inner: state }
    }

    fn get_resource_types() -> impl Iterator<Item = Borrow> {
        std::iter::once(Borrow {
            id: TypeId::of::<R>(),
            name: std::any::type_name::<R>(),
            borrow_type: BorrowType::Mutable,
        })
    }

    fn get_component_types() -> impl Iterator<Item = Borrow> {
        std::iter::empty()
    }
}

impl<'a, R> SystemParameter for Option<Res<'a, R>>
where
    R: Resource,
{
    type This<'this> = Option<Res<'this, R>>;
    type Lock<'state> = Option<MappedRwLockReadGuard<'state, R>>;

    fn lock(state: RunState<'_>) -> Self::Lock<'_> {
        Some(RwLockReadGuard::map(
            state
                .resources
                .get(&TypeId::of::<R>())?
                .try_read()
                .expect("the lock should always be available"),
            |resource| resource.downcast_ref::<R>().unwrap(),
        ))
    }

    fn construct<'this>(state: &'this mut Self::Lock<'_>) -> Self::This<'this> {
        Some(Res {
            inner: state.as_ref()?,
        })
    }

    fn get_resource_types() -> impl Iterator<Item = Borrow> {
        std::iter::once(Borrow {
            id: TypeId::of::<R>(),
            name: std::any::type_name::<R>(),
            borrow_type: BorrowType::Immutable,
        })
    }

    fn get_component_types() -> impl Iterator<Item = Borrow> {
        std::iter::empty()
    }
}

impl<'a, R> SystemParameter for Option<ResMut<'a, R>>
where
    R: Resource,
{
    type This<'this> = Option<ResMut<'this, R>>;
    type Lock<'state> = Option<MappedRwLockWriteGuard<'state, R>>;

    fn lock(state: RunState<'_>) -> Self::Lock<'_> {
        Some(RwLockWriteGuard::map(
            state
                .resources
                .get(&TypeId::of::<R>())?
                .try_write()
                .expect("the lock should always be available"),
            |resource| resource.downcast_mut::<R>().unwrap(),
        ))
    }

    fn construct<'this>(state: &'this mut Self::Lock<'_>) -> Self::This<'this> {
        Some(ResMut {
            inner: state.as_mut()?,
        })
    }

    fn get_resource_types() -> impl Iterator<Item = Borrow> {
        std::iter::once(Borrow {
            id: TypeId::of::<R>(),
            name: std::any::type_name::<R>(),
            borrow_type: BorrowType::Mutable,
        })
    }

    fn get_component_types() -> impl Iterator<Item = Borrow> {
        std::iter::empty()
    }
}

macro_rules! system_parameter_tuple {
    ($($param:ident),*) => {
        impl<$($param),*> SystemParameter for ($($param,)*)
        where
            $($param: SystemParameter,)*
        {
            type This<'this> = ($($param::This<'this>,)*);
            type Lock<'state> = ($($param::Lock<'state>,)*);

            #[allow(clippy::unused_unit)]
            fn lock(state: RunState<'_>) -> Self::Lock<'_> {
                _ = state;
                ($($param::lock(state),)*)
            }

            #[allow(clippy::unused_unit)]
            fn construct<'this>(state: &'this mut Self::Lock<'_>) -> Self::This<'this> {
                #[allow(non_snake_case)]
                let ($($param,)*) = state;
                ($($param::construct($param),)*)
            }

            fn get_resource_types() -> impl Iterator<Item = Borrow> {
                std::iter::empty()
                    $(
                        .chain($param::get_resource_types())
                    )*
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

system_parameter_tuple!();
system_parameter_tuple!(A);
system_parameter_tuple!(A, B);
system_parameter_tuple!(A, B, C);
system_parameter_tuple!(A, B, C, D);
system_parameter_tuple!(A, B, C, D, E);
system_parameter_tuple!(A, B, C, D, E, F);
system_parameter_tuple!(A, B, C, D, E, F, G);
system_parameter_tuple!(A, B, C, D, E, F, G, H);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I, J);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
system_parameter_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
