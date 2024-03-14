use crate::{
    component_container::ComponentContainer,
    entities::Entity,
    query::Component,
    resource::Resource,
    system::{ComponentMap, EntityMap, ResourceMap, RunState, System, SystemWrapper},
};
use hashbrown::{HashMap, HashSet};
use parking_lot::RwLock;
use std::{any::TypeId, marker::PhantomData, num::NonZeroUsize};

pub struct App {
    resources: ResourceMap,
    next_free_entity: usize,
    entities: EntityMap,
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
            let (generation, _) = self.entities[id];
            let generation = NonZeroUsize::new(generation.get() + 1).unwrap();
            self.entities[id].0 = generation;

            while let Some(&(generation, _)) = self.entities.get(self.next_free_entity) {
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

            self.entities.push((NEW_GENERATION, HashSet::new()));
            self.next_free_entity = self.entities.len();
            Entity {
                id,
                generation: NEW_GENERATION,
            }
        }
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        if !self
            .entities
            .get(entity.id)
            .map_or(false, |&(generation, _)| generation == entity.generation)
        {
            self.entities[entity.id].0 |= NonZeroUsize::MIN;
            for id in self.entities[entity.id].1.drain() {
                self.components[&id].write().remove(entity);
            }
            if entity.id < self.next_free_entity {
                self.next_free_entity = entity.id;
            }
        }
    }

    pub fn add_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component,
    {
        if !self
            .entities
            .get(entity.id)
            .map_or(false, |&(generation, _)| generation == entity.generation)
        {
            return;
        }

        let component_id = TypeId::of::<C>();
        self.components
            .entry(component_id)
            .or_insert_with(|| RwLock::new(Box::new(ComponentContainer::<C>::new())))
            .get_mut()
            .downcast_mut::<C>()
            .insert(entity, component);

        self.entities[entity.id].1.insert(component_id);
    }

    pub fn remove_component<C>(&mut self, entity: Entity) -> Option<C>
    where
        C: Component,
    {
        if self.entities.get(entity.id)?.0 != entity.generation {
            return None;
        }

        let component_id = TypeId::of::<C>();
        self.entities[entity.id].1.remove(&component_id);

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
