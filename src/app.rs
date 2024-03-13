use crate::{
    entities::Entity,
    query::Component,
    resource::Resource,
    system::{
        Borrow, BorrowType, ComponentContainer, ComponentMap, ResourceMap, RunState, System,
        SystemFunction, SystemWrapper,
    },
};
use hashbrown::HashMap;
use parking_lot::RwLock;
use rayon::prelude::*;
use slotmap::SlotMap;
use std::{any::TypeId, marker::PhantomData};

struct SystemGroup {
    resources: HashMap<TypeId, BorrowType>,
    components: HashMap<TypeId, BorrowType>,
    systems: Vec<Box<dyn System>>,
}

pub struct App {
    resources: ResourceMap,
    entities: SlotMap<Entity, ()>,
    components: ComponentMap,
    system_groups: Vec<SystemGroup>,
}

impl App {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            entities: SlotMap::with_key(),
            components: HashMap::new(),
            system_groups: Vec::new(),
        }
    }

    pub fn add_resource<R>(&mut self, resource: R)
    where
        R: Resource,
    {
        self.resources
            .insert(TypeId::of::<R>(), RwLock::new(Box::new(resource)));
    }

    pub fn remove_resource<R>(&mut self) -> Option<R>
    where
        R: Resource,
    {
        self.resources
            .remove(&TypeId::of::<R>())
            .map(|resource| *resource.into_inner().downcast::<R>().unwrap())
    }

    pub fn create_entity(&mut self) -> Entity {
        self.entities.insert(())
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.entities.remove(entity);
    }

    pub fn add_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component,
    {
        if !self.entities.contains_key(entity) {
            return;
        }

        self.components
            .entry(TypeId::of::<C>())
            .or_insert_with(|| RwLock::new(Box::new(ComponentContainer::<C>::new())))
            .get_mut()
            .downcast_mut::<ComponentContainer<C>>()
            .unwrap()
            .insert(entity, component);
    }

    pub fn remove_component<C>(&mut self, entity: Entity) -> Option<C>
    where
        C: Component,
    {
        if !self.entities.contains_key(entity) {
            return None;
        }

        self.components
            .get_mut(&TypeId::of::<C>())?
            .get_mut()
            .downcast_mut::<ComponentContainer<C>>()
            .unwrap()
            .remove(entity)
    }

    pub fn register_system<S, Marker>(&mut self, system: S)
    where
        SystemWrapper<S, Marker>: System + 'static,
        S: SystemFunction<Marker>,
    {
        let system = Box::new(SystemWrapper(system, PhantomData));
        let (resources, components) = Self::check_system::<S, Marker>();
        for system_group in &mut self.system_groups {
            if system_group.resources.iter().any(|(id, borrow_type)| {
                let Some(other_borrow_type) = resources.get(id) else {
                    return false;
                };
                match (borrow_type, other_borrow_type) {
                    (BorrowType::Immutable, BorrowType::Immutable) => false,
                    (_, _) => true,
                }
            }) {
                continue;
            }

            if system_group.components.iter().any(|(id, borrow_type)| {
                let Some(other_borrow_type) = components.get(id) else {
                    return false;
                };
                match (borrow_type, other_borrow_type) {
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

    pub fn run_once<S, Marker>(&mut self, mut system: S)
    where
        S: SystemFunction<Marker>,
    {
        Self::check_system::<S, Marker>();
        system.run(RunState {
            resources: &self.resources,
            entities: &self.entities,
            components: &self.components,
        });
    }

    pub fn run_registered(&mut self) {
        for system_group in &mut self.system_groups {
            system_group.systems.par_iter_mut().for_each(|system| {
                system.run(RunState {
                    resources: &self.resources,
                    entities: &self.entities,
                    components: &self.components,
                })
            });
        }
    }

    fn check_system<S, Marker>() -> (HashMap<TypeId, BorrowType>, HashMap<TypeId, BorrowType>)
    where
        S: SystemFunction<Marker>,
    {
        let mut seen_resource_types: HashMap<TypeId, BorrowType> = HashMap::new();
        for Borrow {
            id,
            name,
            borrow_type,
        } in S::get_resource_types()
        {
            if let Some(old_borrow_type) = seen_resource_types.insert(id, borrow_type) {
                match (old_borrow_type, borrow_type) {
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

        let mut seen_component_types: HashMap<TypeId, BorrowType> = HashMap::new();
        for Borrow {
            id,
            name,
            borrow_type,
        } in S::get_component_types()
        {
            if let Some(old_borrow_type) = seen_component_types.insert(id, borrow_type) {
                match (old_borrow_type, borrow_type) {
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

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
