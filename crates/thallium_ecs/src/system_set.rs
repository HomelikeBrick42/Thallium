use crate::{
    system::{Borrow, BorrowType, RunState},
    System, SystemFunction, SystemWrapper,
};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::{any::TypeId, collections::HashMap, marker::PhantomData};

pub(crate) struct SystemGroup<'a> {
    resources: HashMap<TypeId, Borrow>,
    components: HashMap<TypeId, Borrow>,
    systems: Vec<Box<dyn System + 'a>>,
}

/// A set of [`System`]s that can be run in parallel
pub struct SystemSet<'a> {
    system_groups: Vec<SystemGroup<'a>>,
}

impl<'a> SystemSet<'a> {
    /// Constructs an empty [`SystemSet`]
    pub fn new() -> Self {
        SystemSet {
            system_groups: Vec::new(),
        }
    }

    /// Registers a [`System`] with this [`SystemSet`]
    pub fn register_system<S, Marker>(&mut self, system: S)
    where
        SystemWrapper<S, Marker>: System + 'a,
        S: SystemFunction<Marker>,
    {
        let system = Box::new(SystemWrapper(system, PhantomData));
        let (resources, components) = Self::check_system::<S, Marker>();
        for system_group in &mut self.system_groups {
            if system_group.resources.iter().any(|(id, borrow)| {
                let Some(other_borrow) = resources.get(id) else {
                    return false;
                };
                match (borrow.borrow_type, other_borrow.borrow_type) {
                    (BorrowType::Immutable, BorrowType::Immutable) => false,
                    (_, _) => true,
                }
            }) {
                continue;
            }

            if system_group.components.iter().any(|(id, borrow)| {
                let Some(other_borrow) = components.get(id) else {
                    return false;
                };
                match (borrow.borrow_type, other_borrow.borrow_type) {
                    (BorrowType::Immutable, BorrowType::Immutable) => false,
                    (_, _) => true,
                }
            }) {
                continue;
            }

            system_group.systems.push(system);
            system_group.resources.extend(resources);
            system_group.components.extend(components);
            return;
        }
        self.system_groups.push(SystemGroup {
            resources,
            components,
            systems: vec![system],
        });
    }

    fn check_system<S, Marker>() -> (HashMap<TypeId, Borrow>, HashMap<TypeId, Borrow>)
    where
        S: SystemFunction<Marker>,
    {
        let mut seen_resource_types: HashMap<TypeId, Borrow> = HashMap::new();
        for borrow @ Borrow {
            id,
            name,
            borrow_type,
        } in S::get_resource_types()
        {
            if let Some(old_borrow) = seen_resource_types.insert(id, borrow) {
                match (old_borrow.borrow_type, borrow_type) {
                    (BorrowType::Mutable, _) => {
                        panic!("tried to borrow resource `{name}`, but it has already been mutably borrowed")
                    }
                    (_, BorrowType::Mutable) => {
                        panic!("tried to borrow resource `{name}` as mutable, but it has already been borrowed")
                    }
                    (_, _) => {}
                }
            }
        }

        let mut seen_component_types: HashMap<TypeId, Borrow> = HashMap::new();
        for borrow @ Borrow {
            id,
            name,
            borrow_type,
        } in S::get_component_types()
        {
            if let Some(borrow) = seen_component_types.insert(id, borrow) {
                match (borrow.borrow_type, borrow_type) {
                    (BorrowType::Mutable, _) => {
                        panic!("tried to borrow component `{name}`, but it has already been mutably borrowed")
                    }
                    (_, BorrowType::Mutable) => {
                        panic!("tried to borrow component `{name}` as mutable, but it has already been borrowed")
                    }
                    (_, _) => {}
                }
            }
        }

        (seen_resource_types, seen_component_types)
    }
}

impl<'a> Default for SystemSet<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> System for SystemWrapper<SystemSet<'a>, ()> {
    fn run(&mut self, state: &RunState<'_>) {
        for system_group in &mut self.0.system_groups {
            system_group
                .systems
                .par_iter_mut()
                .for_each(|system| system.run(state));
        }
    }
}

impl<'a> System for SystemWrapper<&mut SystemSet<'a>, ()> {
    fn run(&mut self, state: &RunState<'_>) {
        for system_group in &mut self.0.system_groups {
            system_group
                .systems
                .par_iter_mut()
                .for_each(|system| system.run(state));
        }
    }
}
