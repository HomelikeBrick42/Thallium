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
use slotmap::SlotMap;
use std::{any::TypeId, marker::PhantomData};

pub struct App {
    resources: ResourceMap,
    entities: SlotMap<Entity, ()>,
    components: ComponentMap,
    systems: Vec<Box<dyn System>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            entities: SlotMap::with_key(),
            components: HashMap::new(),
            systems: Vec::new(),
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
        SystemWrapper<S, Marker>: System,
        S: SystemFunction<Marker>,
    {
        Self::check_system::<S, Marker>();
        self.systems
            .push(Box::new(SystemWrapper(system, PhantomData)));
    }

    pub fn run_once<S, Marker>(&mut self, system: S)
    where
        SystemWrapper<S, Marker>: System,
        S: SystemFunction<Marker>,
    {
        Self::check_system::<S, Marker>();
        SystemWrapper(system, PhantomData).run(RunState {
            resources: &self.resources,
            entities: &self.entities,
            components: &self.components,
        });
    }

    pub fn run_registered(&mut self) {
        for system in &mut self.systems {
            system.run(RunState {
                resources: &self.resources,
                entities: &self.entities,
                components: &self.components,
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
