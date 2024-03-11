use std::ops::{Deref, DerefMut};

pub trait Resource: Sized + Send + Sync + 'static {}

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
