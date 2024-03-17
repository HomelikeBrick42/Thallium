use crate::{
    system::{Borrow, BorrowType, RunState},
    SystemParameter,
};
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLockReadGuard, RwLockWriteGuard,
};
use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
};

/// The trait implemented for all types that will be used as resources
pub trait Resource: Sized + Send + Sync + 'static {}

/// The [`SystemParameter`] for getting a reference to a [`Resource`]
pub struct Res<'a, T>
where
    T: Resource,
{
    pub(crate) inner: &'a T,
}

impl<'a, T> Deref for Res<'a, T>
where
    T: Resource,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
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

/// The [`SystemParameter`] for getting a mutable reference to a [`Resource`]
pub struct ResMut<'a, T>
where
    T: Resource,
{
    pub(crate) inner: &'a mut T,
}

impl<'a, T> Deref for ResMut<'a, T>
where
    T: Resource,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> DerefMut for ResMut<'a, T>
where
    T: Resource,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
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
