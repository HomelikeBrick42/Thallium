use crate::{
    component_container::ComponentContainer,
    entities::EntityMap,
    system::{ComponentMap, ResourceMap, RunState},
    Component, Entity, Resource, System, SystemWrapper,
};
use parking_lot::RwLock;
use std::{any::TypeId, collections::HashMap, marker::PhantomData};

pub struct App {
    resources: ResourceMap,
    entities: EntityMap,
    components: ComponentMap,
}

impl App {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            entities: EntityMap::new(),
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
        self.entities.create_entity()
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        for component in self.entities.destroy_entity(entity).into_iter().flatten() {
            self.components
                .get_mut(&component)
                .unwrap()
                .get_mut()
                .remove(entity);
        }
    }

    pub fn entity_exists(&self, entity: Entity) -> bool {
        self.entities.entity_exists(entity)
    }

    pub fn add_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component,
    {
        if !self.entity_exists(entity) {
            return;
        }

        let component_id = TypeId::of::<C>();
        self.components
            .entry(component_id)
            .or_insert_with(|| RwLock::new(Box::new(ComponentContainer::<C>::new())))
            .get_mut()
            .downcast_mut::<C>()
            .insert(entity, component);

        self.entities.add_component(entity, component_id);
    }

    pub fn remove_component<C>(&mut self, entity: Entity) -> Option<C>
    where
        C: Component,
    {
        if !self.entity_exists(entity) {
            return None;
        }

        let component_id = TypeId::of::<C>();
        self.entities.remove_component(entity, component_id);

        self.components
            .get_mut(&component_id)?
            .get_mut()
            .downcast_mut::<C>()
            .remove(entity)
    }

    pub fn run<S, Marker>(&mut self, system: S)
    where
        SystemWrapper<S, Marker>: System,
    {
        let (command_sender, command_receiver) = std::sync::mpsc::channel();
        SystemWrapper(system, PhantomData).run(RunState {
            resources: &self.resources,
            entities: &self.entities,
            components: &self.components,
            command_sender: &command_sender,
        });
        drop(command_sender);
        for command in command_receiver {
            command(self);
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
