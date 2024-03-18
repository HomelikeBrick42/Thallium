use crate::{
    component_container::ComponentContainer,
    entities::EntityMap,
    system::{ComponentMap, ResourceMap, RunState},
    Component, Entity, IntoSystem, Resource, System,
};
use parking_lot::RwLock;
use std::{any::TypeId, collections::HashMap};

/// The main struct that you will create for holding entities, components, and resources
pub struct App {
    resources: ResourceMap,
    entities: EntityMap,
    components: ComponentMap,
    current_tick: u64,
}

impl App {
    /// Constructs an empty [`App`]
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            entities: EntityMap::new(),
            components: HashMap::new(),
            current_tick: 0,
        }
    }

    /// Adds a [`Resource`] to the [`App`], currently if you add the same type of [`Resource`] twice it will replace the previous one
    pub fn add_resource<R>(&mut self, resource: R)
    where
        R: Resource,
    {
        self.resources
            .insert(TypeId::of::<R>(), RwLock::new(Box::new(resource)));
    }

    /// Removes a [`Resource`] from the [`App`] and returns it
    pub fn remove_resource<R>(&mut self) -> Option<R>
    where
        R: Resource,
    {
        self.resources
            .remove(&TypeId::of::<R>())
            .map(|resource| *resource.into_inner().downcast::<R>().unwrap())
    }

    /// Creates an [`Entity`]
    pub fn create_entity(&mut self) -> Entity {
        self.entities.create_entity()
    }

    /// Destroys an [`Entity`] along with all its attached [`Component`]s
    pub fn destroy_entity(&mut self, entity: Entity) {
        for component in self.entities.destroy_entity(entity).into_iter().flatten() {
            self.components
                .get_mut(&component)
                .unwrap()
                .get_mut()
                .remove(entity);
        }
    }

    /// Checks if an [`Entity`] exists
    pub fn entity_exists(&self, entity: Entity) -> bool {
        self.entities.entity_exists(entity)
    }

    /// Adds a [`Component`] to an [`Entity`], currently if you add the same [`Component`] twice it will replace the previous one
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
            .insert(entity, self.current_tick, component);

        self.entities.add_component(entity, component_id);
    }

    /// Removes a [`Component`] from an [`Entity`] and returns it
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

    /// Runs a system with access to the [`App`]
    pub fn run<S, Marker>(&mut self, system: S)
    where
        S: IntoSystem<Marker>,
    {
        let (command_sender, command_receiver) = std::sync::mpsc::channel();
        system.into_system().run(&RunState {
            resources: &self.resources,
            entities: &self.entities,
            components: &self.components,
            command_sender: &command_sender,
            current_tick: self.current_tick,
        });
        drop(command_sender);
        for command in command_receiver {
            command(self);
        }
    }

    /// Advances to the next tick, this effects stuff like modification checking
    pub fn next_tick(&mut self) {
        self.current_tick += 1;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
