use crate::{
    entities::Entity,
    query::Component,
    resource::Resource,
    system::{ComponentContainer, ComponentMap, ResourceMap, RunState, System, SystemWrapper},
};
use hashbrown::HashMap;
use parking_lot::RwLock;
use slotmap::SlotMap;
use std::{any::TypeId, marker::PhantomData};

pub struct App {
    resources: ResourceMap,
    entities: SlotMap<Entity, ()>,
    components: ComponentMap,
}

impl App {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            entities: SlotMap::with_key(),
            components: HashMap::new(),
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

    pub fn run<S, Marker>(&mut self, system: S)
    where
        SystemWrapper<S, Marker>: System,
    {
        SystemWrapper(system, PhantomData).run(RunState {
            resources: &self.resources,
            entities: &self.entities,
            components: &self.components,
        });
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
