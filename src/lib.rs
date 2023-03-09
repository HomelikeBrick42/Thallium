use rayon::prelude::*;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    num::NonZeroUsize,
};

pub type SystemType<T> = Box<dyn Fn(Entity, &mut T) + Send + Sync>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    id: usize,
    gen: std::num::NonZeroUsize,
}

pub trait Component: Clone + Sized + Send + Sync + 'static {}

trait ComponentContainerTrait: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove_entity(&mut self, entity: Entity);
    fn run_systems(&mut self);
}

impl dyn ComponentContainerTrait {
    pub fn as_component_container<T: Component>(&self) -> Option<&ComponentContainer<T>> {
        self.as_any().downcast_ref()
    }

    pub fn as_component_container_mut<T: Component>(
        &mut self,
    ) -> Option<&mut ComponentContainer<T>> {
        self.as_any_mut().downcast_mut()
    }
}

struct ComponentContainer<T: Component> {
    // TODO: we are trimming the end off when removing components, but what about trimming the front?
    components: Vec<Option<(NonZeroUsize, T)>>,
    systems: Vec<SystemType<T>>,
}

impl<T: Component> Default for ComponentContainer<T> {
    fn default() -> Self {
        Self {
            components: Vec::default(),
            systems: Vec::default(),
        }
    }
}

impl<T: Component> ComponentContainerTrait for ComponentContainer<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn run_systems(&mut self) {
        self.systems.iter_mut().for_each(|system| {
            self.components
                .par_iter_mut()
                .enumerate()
                .for_each(|(id, data)| {
                    if let &mut Some((gen, ref mut component)) = data {
                        system(Entity { id, gen }, component);
                    }
                });
        });
    }

    fn remove_entity(&mut self, entity: Entity) {
        if let Some((gen, _)) = self.components[entity.id] {
            assert!(entity.gen == gen);
            self.components[entity.id] = None;
        }

        // Free up unused components
        while let Some(&None) = self.components.last() {
            self.components.pop();
        }
        self.components.shrink_to_fit();
    }
}

pub trait System {
    fn run_system(&self, ecs: &mut ECS);
}

impl<T: Component, F: Fn(Entity, &mut T)> System for F {
    fn run_system(&self, ecs: &mut ECS) {
        todo!()
    }
}

pub struct ECS {
    // TODO: we are trimming the end off when removing entities, but what about trimming the front?
    entities: Vec<(bool, NonZeroUsize)>,
    next_free_entity: usize,
    component_containers: HashMap<TypeId, Box<dyn ComponentContainerTrait>>,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            next_free_entity: 0,
            component_containers: HashMap::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        while self.next_free_entity < self.entities.len() {
            let (alive, gen) = &mut self.entities[self.next_free_entity];
            if !*alive {
                *alive = true;
                *gen = gen.checked_add(1).unwrap();
                return Entity {
                    id: self.next_free_entity,
                    gen: *gen,
                };
            }
            self.next_free_entity += 1;
        }

        let id = self.entities.len();
        let gen = NonZeroUsize::new(1).unwrap();
        self.entities.push((true, gen));
        Entity { id, gen }
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        if !self.is_entity_valid(entity) {
            return false;
        }

        self.component_containers
            .par_iter_mut()
            .for_each(|(_, component_container)| {
                component_container.remove_entity(entity);
            });

        self.entities[entity.id].0 = false;
        if entity.id < self.next_free_entity {
            self.next_free_entity = entity.id;
        }

        // Free up unused entities
        while let Some(&(false, _)) = self.entities.last() {
            self.entities.pop();
        }
        self.entities.shrink_to_fit();

        true
    }

    pub fn is_entity_valid(&self, entity: Entity) -> bool {
        self.entities
            .get(entity.id)
            .map(|&(alive, gen)| alive && entity.gen == gen)
            .unwrap_or(false)
    }

    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> Option<&mut T> {
        if !self.is_entity_valid(entity) {
            return None;
        }

        let components = &mut self
            .component_containers
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<ComponentContainer<T>>::default())
            .as_component_container_mut()
            .unwrap()
            .components;

        if entity.id >= components.len() {
            let count = components.len() - entity.id + 1;
            components.reserve(count);
            components.extend(std::iter::repeat(None).take(count));
        }
        if components[entity.id].is_some() {
            return None;
        }
        components[entity.id] = Some((entity.gen, component));
        Some(&mut components[entity.id].as_mut().unwrap().1)
    }

    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> {
        if !self.is_entity_valid(entity) {
            return None;
        }

        let components = &mut self
            .component_containers
            .get_mut(&TypeId::of::<T>())?
            .as_component_container_mut()
            .unwrap()
            .components;

        let component = components[entity.id].take().map(|(gen, component)| {
            assert!(entity.gen == gen);
            component
        });

        // Free up unused components
        while let Some(&None) = components.last() {
            components.pop();
        }
        components.shrink_to_fit();

        component
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.is_entity_valid(entity) {
            return None;
        }

        let components = &self
            .component_containers
            .get(&TypeId::of::<T>())?
            .as_component_container()
            .unwrap()
            .components;
        components.get(entity.id).and_then(|data| {
            let &(gen, ref component) = data.as_ref()?;
            assert!(entity.gen == gen);
            Some(component)
        })
    }

    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.is_entity_valid(entity) {
            return None;
        }

        let components = &mut self
            .component_containers
            .get_mut(&TypeId::of::<T>())?
            .as_component_container_mut()
            .unwrap()
            .components;
        components.get_mut(entity.id).and_then(|data| {
            let &mut (gen, ref mut component) = data.as_mut()?;
            assert!(entity.gen == gen);
            Some(component)
        })
    }

    pub fn add_system<T: Component>(&mut self, system: SystemType<T>) {
        let component_container = self
            .component_containers
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<ComponentContainer<T>>::default());
        component_container
            .as_any_mut()
            .downcast_mut::<ComponentContainer<T>>()
            .unwrap()
            .systems
            .push(system);
    }

    pub fn run_systems(&mut self) {
        self.component_containers
            .par_iter_mut()
            .for_each(|(_, component_container)| {
                component_container.run_systems();
            });
    }
}

impl Default for ECS {
    fn default() -> Self {
        Self::new()
    }
}
