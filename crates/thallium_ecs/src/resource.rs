use crate::{
    resource_container::ResourceContainer,
    system::{Borrow, BorrowType, SystemRunState},
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
    pub(crate) resource: &'a T,
    pub(crate) last_modified_tick: u64,
    pub(crate) last_run_tick: u64,
}

impl<'a, T> Res<'a, T>
where
    T: Resource,
{
    /// Returns whether this [`Resource`] has been modified since the last time this system was run
    pub fn get_modified(&self) -> bool {
        self.last_run_tick < self.last_modified_tick
    }
}

impl<'a, T> Deref for Res<'a, T>
where
    T: Resource,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.resource
    }
}

impl<'a, R> SystemParameter for Res<'a, R>
where
    R: Resource,
{
    type This<'this> = Res<'this, R>;
    type Lock<'state> = MappedRwLockReadGuard<'state, ResourceContainer<R>>;

    fn lock<'state>(state: &SystemRunState<'state>) -> Self::Lock<'state> {
        RwLockReadGuard::map(
            state
                .resources
                .get(&TypeId::of::<R>())
                .expect("Non-Option Res expects the resource to always exist")
                .try_read()
                .expect("the lock should always be available"),
            |resource| resource.downcast_ref().unwrap(),
        )
    }

    fn construct<'this>(state: &'this mut Self::Lock<'_>, last_run_tick: u64) -> Self::This<'this> {
        Res {
            resource: &state.resource,
            last_modified_tick: state.last_modified_tick,
            last_run_tick,
        }
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
    type Lock<'state> = Option<MappedRwLockReadGuard<'state, ResourceContainer<R>>>;

    fn lock<'state>(state: &SystemRunState<'state>) -> Self::Lock<'state> {
        Some(RwLockReadGuard::map(
            state
                .resources
                .get(&TypeId::of::<R>())?
                .try_read()
                .expect("the lock should always be available"),
            |resource| resource.downcast_ref().unwrap(),
        ))
    }

    fn construct<'this>(state: &'this mut Self::Lock<'_>, last_run_tick: u64) -> Self::This<'this> {
        let state = state.as_ref()?;
        Some(Res {
            resource: &state.resource,
            last_modified_tick: state.last_modified_tick,
            last_run_tick,
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
pub struct ResMut<'a, R>
where
    R: Resource,
{
    pub(crate) resource: &'a mut R,
    pub(crate) last_modified_tick: &'a mut u64,
    pub(crate) last_run_tick: u64,
    pub(crate) current_tick: u64,
}

impl<'a, R> ResMut<'a, R>
where
    R: Resource,
{
    /// Returns whether this [`Resource`] has been modified since the last time this system was run
    pub fn get_modified(&self) -> bool {
        self.last_run_tick < *self.last_modified_tick
    }

    /// Gets a reference to the inner [`Resource`] without triggering the modification detection
    pub fn silently_modify(&mut self) -> &mut R {
        self.resource
    }
}

impl<'a, R> Deref for ResMut<'a, R>
where
    R: Resource,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.resource
    }
}

impl<'a, R> DerefMut for ResMut<'a, R>
where
    R: Resource,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        *self.last_modified_tick = self.current_tick;
        self.resource
    }
}

impl<'a, R> SystemParameter for ResMut<'a, R>
where
    R: Resource,
{
    type This<'this> = ResMut<'this, R>;
    type Lock<'state> = (MappedRwLockWriteGuard<'state, ResourceContainer<R>>, u64);

    fn lock<'state>(state: &SystemRunState<'state>) -> Self::Lock<'state> {
        (
            RwLockWriteGuard::map(
                state
                    .resources
                    .get(&TypeId::of::<R>())
                    .expect("Non-Option ResMut expects the resource to always exist")
                    .try_write()
                    .expect("the lock should always be available"),
                |resource| resource.downcast_mut().unwrap(),
            ),
            state.current_tick,
        )
    }

    fn construct<'this>(state: &'this mut Self::Lock<'_>, last_run_tick: u64) -> Self::This<'this> {
        let (state, current_tick) = state;
        let state = &mut **state;
        ResMut {
            resource: &mut state.resource,
            last_modified_tick: &mut state.last_modified_tick,
            last_run_tick,
            current_tick: *current_tick,
        }
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
    type Lock<'state> = Option<(MappedRwLockWriteGuard<'state, ResourceContainer<R>>, u64)>;

    fn lock<'state>(state: &SystemRunState<'state>) -> Self::Lock<'state> {
        Some((
            RwLockWriteGuard::map(
                state
                    .resources
                    .get(&TypeId::of::<R>())?
                    .try_write()
                    .expect("the lock should always be available"),
                |resource| resource.downcast_mut().unwrap(),
            ),
            state.current_tick,
        ))
    }

    fn construct<'this>(state: &'this mut Self::Lock<'_>, last_run_tick: u64) -> Self::This<'this> {
        let (state, current_tick) = state.as_mut()?;
        let state = &mut **state;
        Some(ResMut {
            resource: &mut state.resource,
            last_modified_tick: &mut state.last_modified_tick,
            last_run_tick,
            current_tick: *current_tick,
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
