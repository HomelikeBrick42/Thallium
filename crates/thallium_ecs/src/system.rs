use crate::{
    component_container::DynComponentContainer, entities::EntityMap, App, SystemParameter,
};
use parking_lot::RwLock;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    sync::mpsc::Sender,
};

pub(crate) type ResourceMap = HashMap<TypeId, RwLock<Box<dyn Any + Send + Sync>>>;
pub(crate) type ComponentMap = HashMap<TypeId, RwLock<Box<dyn DynComponentContainer>>>;
pub(crate) type CommandSender = Sender<Box<dyn FnOnce(&mut App) + Send>>;

#[derive(Clone, Copy)]
pub struct RunState<'a> {
    pub(crate) resources: &'a ResourceMap,
    pub(crate) entities: &'a EntityMap,
    pub(crate) components: &'a ComponentMap,
    pub(crate) command_sender: &'a CommandSender,
    pub(crate) current_tick: u64,
}

#[derive(Clone, Copy)]
pub enum BorrowType {
    Immutable,
    Mutable,
}

#[derive(Clone, Copy)]
pub struct Borrow {
    pub id: TypeId,
    pub name: &'static str,
    pub borrow_type: BorrowType,
}

/// An ECS system that can be added to a [`SystemSet`](crate::SystemSet)
pub trait System: Send + Sync {
    /// Runs the system
    fn run(&mut self, state: &RunState<'_>);
    /// Returns an iterator over all [`Resource`](crate::Resource) types that this [`System`] will use
    fn get_resource_types(&self) -> impl Iterator<Item = Borrow> + '_
    where
        Self: Sized;
    /// Returns an iterator over all [`Component`](crate::Component) types that this [`System`] will use
    fn get_component_types(&self) -> impl Iterator<Item = Borrow> + '_
    where
        Self: Sized;
}

/// Trait for converting things into a [`System`]
pub trait IntoSystem<Marker> {
    /// The type returned from [`IntoSystem::into_system`]
    type System: System;

    /// Converts `self` into a [`System`]
    fn into_system(self) -> Self::System;
}

pub struct SystemFunctionWrapper<F, Marker>(pub(crate) F, pub(crate) PhantomData<fn(Marker)>)
where
    F: SystemFunction<Marker>;
impl<F, Marker> System for SystemFunctionWrapper<F, Marker>
where
    F: SystemFunction<Marker>,
{
    fn run(&mut self, state: &RunState<'_>) {
        F::run(&mut self.0, state);
    }

    fn get_resource_types(&self) -> impl Iterator<Item = Borrow> + '_
    where
        Self: Sized,
    {
        F::get_resource_types()
    }

    fn get_component_types(&self) -> impl Iterator<Item = Borrow> + '_
    where
        Self: Sized,
    {
        F::get_component_types()
    }
}

impl<F, Marker> IntoSystem<Marker> for F
where
    F: SystemFunction<Marker>,
{
    type System = SystemFunctionWrapper<F, Marker>;

    fn into_system(self) -> Self::System {
        SystemFunctionWrapper(self, PhantomData)
    }
}

/// The trait for functions which can be used as [`System`]s
pub trait SystemFunction<Marker>: Send + Sync {
    /// Runs the system
    fn run(&mut self, state: &RunState<'_>);
    /// Gets the [`Resource`](crate::Component) types that this [`SystemFunction`] will use
    fn get_resource_types() -> impl Iterator<Item = Borrow>;
    /// Gets the [`Component`](crate::Component) types that this [`SystemFunction`] will use
    fn get_component_types() -> impl Iterator<Item = Borrow>;
}

macro_rules! system_function_impl {
    ($($param:ident),*) => {
        impl<Func, $($param),*> SystemFunction<fn($($param),*)> for Func
        where
            for<'a> Func: FnMut($($param),*) + FnMut($($param::This<'a>),*) + Send + Sync,
            $($param: SystemParameter,)*
        {
            fn run(&mut self, state: &RunState<'_>) {
                _ = state;
                $(
                    #[allow(non_snake_case)]
                    let mut $param = $param::lock(state);
                )*
                self($($param::construct(&mut $param)),*)
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

system_function_impl!();
system_function_impl!(A);
system_function_impl!(A, B);
system_function_impl!(A, B, C);
system_function_impl!(A, B, C, D);
system_function_impl!(A, B, C, D, E);
system_function_impl!(A, B, C, D, E, F);
system_function_impl!(A, B, C, D, E, F, G);
system_function_impl!(A, B, C, D, E, F, G, H);
system_function_impl!(A, B, C, D, E, F, G, H, I);
system_function_impl!(A, B, C, D, E, F, G, H, I, J);
system_function_impl!(A, B, C, D, E, F, G, H, I, J, K);
system_function_impl!(A, B, C, D, E, F, G, H, I, J, K, L);
system_function_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M);
system_function_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
system_function_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
system_function_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
