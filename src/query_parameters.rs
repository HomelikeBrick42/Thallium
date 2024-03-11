use std::any::TypeId;

use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLockReadGuard, RwLockWriteGuard,
};

use crate::{
    query::{Component, ComponentContainerTrait, Ref, RefMut},
    system::{ComponentContainer, RunState},
};

pub trait QueryParameter {
    type ComponentContainerLock<'a>;
    type ComponentContainer<'a>: ComponentContainerTrait<'a>;

    fn lock(state: RunState<'_>) -> Self::ComponentContainerLock<'_>;
    fn construct<'a>(
        lock: &'a mut Self::ComponentContainerLock<'_>,
    ) -> Self::ComponentContainer<'a>;
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
            |components| components.downcast_ref::<ComponentContainer<C>>().unwrap(),
        ))
    }

    fn construct<'a>(
        lock: &'a mut Self::ComponentContainerLock<'_>,
    ) -> Self::ComponentContainer<'a> {
        Some(lock.as_mut()?)
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
            |components| components.downcast_mut::<ComponentContainer<C>>().unwrap(),
        ))
    }

    fn construct<'a>(
        lock: &'a mut Self::ComponentContainerLock<'_>,
    ) -> Self::ComponentContainer<'a> {
        Some(lock.as_mut()?)
    }
}
