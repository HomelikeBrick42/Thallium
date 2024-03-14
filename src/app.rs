use crate::{
    component_container::ComponentContainer,
    entities::Entity,
    query::Component,
    resource::Resource,
    system::{ComponentMap, ResourceMap, RunState, System, SystemWrapper},
};
use hashbrown::HashMap;
use parking_lot::RwLock;
use std::{any::TypeId, marker::PhantomData, num::NonZeroUsize};

pub struct App {
    resources: ResourceMap,
    next_free_entity: usize,
    entities: Vec<NonZeroUsize>,
    components: ComponentMap,
}

impl App {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            next_free_entity: 0,
            entities: Vec::new(),
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
        let id = self.next_free_entity;
        if id < self.entities.len() {
            let generation = self.entities[id];
            self.entities[id] = NonZeroUsize::new(generation.get() + 1).unwrap();

            while let Some(&generation) = self.entities.get(self.next_free_entity) {
                self.next_free_entity += 1;
                if generation.get() & 1 != 0 {
                    break;
                }
            }

            Entity { id, generation }
        } else {
            const NEW_GENERATION: NonZeroUsize = match NonZeroUsize::new(2) {
                Some(generation) => generation,
                None => unreachable!(),
            };

            self.entities.push(NEW_GENERATION);
            self.next_free_entity = self.entities.len();
            Entity {
                id,
                generation: NEW_GENERATION,
            }
        }
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        if self.entities.get(entity.id) == Some(&entity.generation) {
            self.entities[entity.id] |= NonZeroUsize::MIN;
            if self.next_free_entity > entity.id {}
        }
    }

    pub fn add_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component,
    {
        if self.entities.get(entity.id) != Some(&entity.generation) {
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
