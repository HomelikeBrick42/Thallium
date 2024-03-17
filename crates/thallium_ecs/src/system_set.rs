use crate::{
    system::{Borrow, BorrowType, RunState},
    IntoSystem, System,
};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::{any::TypeId, collections::HashMap};

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
        S: IntoSystem<Marker>,
        S::System: 'a,
    {
        let system = system.into_system();
        let (resources, components) = Self::check_system(&system);
        let system = Box::new(system);
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

    fn check_system<S>(system: &S) -> (HashMap<TypeId, Borrow>, HashMap<TypeId, Borrow>)
    where
        S: System,
    {
        let mut seen_resource_types: HashMap<TypeId, Borrow> = HashMap::new();
        for borrow @ Borrow {
            id,
            name,
            borrow_type,
        } in system.get_resource_types()
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
        } in system.get_component_types()
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

impl<'a> System for SystemSet<'a> {
    fn run(&mut self, state: &RunState<'_>) {
        for system_group in &mut self.system_groups {
            system_group
                .systems
                .par_iter_mut()
                .for_each(|system| system.run(state));
        }
    }

    fn get_resource_types(&self) -> impl Iterator<Item = Borrow> + '_
    where
        Self: Sized,
    {
        self.system_groups
            .iter()
            .fold(HashMap::new(), |mut a, b| {
                for (&id, &borrow) in &b.resources {
                    if let (
                        Borrow {
                            borrow_type: borrow_type @ BorrowType::Immutable,
                            ..
                        },
                        BorrowType::Mutable,
                    ) = (a.entry(id).or_insert(borrow), borrow.borrow_type)
                    {
                        *borrow_type = BorrowType::Mutable
                    }
                }
                a
            })
            .into_values()
    }

    fn get_component_types(&self) -> impl Iterator<Item = Borrow> + '_
    where
        Self: Sized,
    {
        self.system_groups
            .iter()
            .fold(HashMap::new(), |mut a, b| {
                for (&id, &borrow) in &b.components {
                    if let (
                        Borrow {
                            borrow_type: borrow_type @ BorrowType::Immutable,
                            ..
                        },
                        BorrowType::Mutable,
                    ) = (a.entry(id).or_insert(borrow), borrow.borrow_type)
                    {
                        *borrow_type = BorrowType::Mutable
                    }
                }
                a
            })
            .into_values()
    }
}

impl<'a> System for &mut SystemSet<'a> {
    fn run(&mut self, state: &RunState<'_>) {
        for system_group in &mut self.system_groups {
            system_group
                .systems
                .par_iter_mut()
                .for_each(|system| system.run(state));
        }
    }

    fn get_resource_types(&self) -> impl Iterator<Item = Borrow> + '_
    where
        Self: Sized,
    {
        SystemSet::get_resource_types(self)
    }

    fn get_component_types(&self) -> impl Iterator<Item = Borrow> + '_
    where
        Self: Sized,
    {
        SystemSet::get_component_types(self)
    }
}

impl<'a> IntoSystem<()> for SystemSet<'a> {
    type System = Self;

    fn into_system(self) -> Self::System {
        self
    }
}

impl<'a> IntoSystem<()> for &mut SystemSet<'a> {
    type System = Self;

    fn into_system(self) -> Self::System {
        self
    }
}
