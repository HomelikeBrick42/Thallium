use rayon::prelude::*;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    num::NonZeroUsize,
};

type SystemType<T> = Box<dyn Fn(Entity, &mut T) + Send + Sync>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    id: usize,
    gen: std::num::NonZeroUsize,
}

pub trait Component: Clone + Sized + Send + Sync + 'static {}

trait ComponentContainerTrait: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn run_systems(&mut self);
}

struct ComponentContainer<T: Component> {
    components: HashMap<Entity, T>,
    systems: Vec<SystemType<T>>,
}

impl<T: Component> Default for ComponentContainer<T> {
    fn default() -> Self {
        Self {
            components: HashMap::default(),
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
                .for_each(|(&entity, component)| {
                    system(entity, component);
                });
        });
    }
}

pub struct ECS {
    entities: HashMap<usize, NonZeroUsize>,
    component_containers: HashMap<TypeId, Box<dyn ComponentContainerTrait>>,
}

impl ECS {
    pub fn is_valid(&self, entity: Entity) -> bool {
        self.entities
            .get(&entity.id)
            .map(|gen| &entity.gen == gen)
            .unwrap_or(false)
    }

    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> Option<&mut T> {
        if !self.is_valid(entity) {
            return None;
        }

        let component_container = self
            .component_containers
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<ComponentContainer<T>>::default())
            .as_any_mut()
            .downcast_mut::<ComponentContainer<T>>()
            .unwrap();
        component_container.components.insert(entity, component);
        component_container.components.get_mut(&entity)
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.is_valid(entity) {
            return None;
        }

        let component_container = self.component_containers.get(&TypeId::of::<T>())?;
        component_container
            .as_any()
            .downcast_ref::<ComponentContainer<T>>()
            .unwrap()
            .components
            .get(&entity)
    }

    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.is_valid(entity) {
            return None;
        }

        let component_container = self.component_containers.get_mut(&TypeId::of::<T>())?;
        component_container
            .as_any_mut()
            .downcast_mut::<ComponentContainer<T>>()
            .unwrap()
            .components
            .get_mut(&entity)
    }

    pub fn add_system<T: Component>(&mut self, system: SystemType<T>) {
        let component_container = (&mut *self
            .component_containers
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<ComponentContainer<T>>::default())
            as &mut dyn Any)
            .downcast_mut::<ComponentContainer<T>>()
            .unwrap();
        component_container.systems.push(system);
    }

    pub fn run_systems(&mut self) {
        self.component_containers
            .par_iter_mut()
            .for_each(|(_, component_container)| {
                component_container.run_systems();
            });
    }
}
