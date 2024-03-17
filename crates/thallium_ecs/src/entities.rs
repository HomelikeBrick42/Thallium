use crate::{
    system::{Borrow, RunState},
    SystemParameter,
};
use std::{any::TypeId, collections::HashSet, num::NonZeroUsize};

/// A handle for components to be attached to
///
/// The behaviour of using an [`Entity`] with the wrong [`App`](crate::App) is unspecified but not UB
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    pub(crate) id: usize,
    pub(crate) generation: NonZeroUsize,
}

pub struct EntityMap {
    entities: Vec<(NonZeroUsize, HashSet<TypeId>)>,
    next_free_entity: usize,
}

impl EntityMap {
    pub(crate) fn new() -> Self {
        Self {
            entities: Vec::new(),
            next_free_entity: 0,
        }
    }

    pub(crate) fn create_entity(&mut self) -> Entity {
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

    pub(crate) fn destroy_entity(&mut self, entity: Entity) -> Option<HashSet<TypeId>> {
        if self.entity_exists(entity) {
            self.entities[entity.id].0 |= NonZeroUsize::MIN;
            if entity.id < self.next_free_entity {
                self.next_free_entity = entity.id;
            }
            Some(std::mem::take(&mut self.entities[entity.id].1))
        } else {
            None
        }
    }

    pub(crate) fn entity_exists(&self, entity: Entity) -> bool {
        self.entities
            .get(entity.id)
            .map_or(false, |&(generation, _)| generation == entity.generation)
    }

    pub(crate) fn add_component(&mut self, entity: Entity, component_type: TypeId) {
        debug_assert_eq!(self.entities[entity.id].0, entity.generation);
        self.entities[entity.id].1.insert(component_type);
    }

    pub(crate) fn remove_component(&mut self, entity: Entity, component_type: TypeId) {
        debug_assert_eq!(self.entities[entity.id].0, entity.generation);
        self.entities[entity.id].1.remove(&component_type);
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = Option<Entity>> + '_ {
        self.entities
            .iter()
            .enumerate()
            .map(|(id, &(generation, _))| {
                (generation.get() & 1 == 0).then_some(Entity { id, generation })
            })
    }
}

impl Default for EntityMap {
    fn default() -> Self {
        Self::new()
    }
}

/// A [`SystemParameter`] for getting a list of all alive [`Entity`]s
#[derive(Clone, Copy)]
pub struct Entities<'a> {
    entities: &'a EntityMap,
}

impl<'a> Entities<'a> {
    /// Checks if an [`Entity`] exists
    pub fn entity_exists(&self, entity: Entity) -> bool {
        self.entities.entity_exists(entity)
    }

    /// Returns an iterator over all alive [`Entity`]s
    pub fn iter(&self) -> impl Iterator<Item = Entity> + 'a {
        self.entities.iter().flatten()
    }
}

impl<'a> SystemParameter for Entities<'a> {
    type This<'this> = Entities<'this>;
    type Lock<'state> = &'state EntityMap;

    fn lock(state: RunState<'_>) -> Self::Lock<'_> {
        state.entities
    }

    fn construct<'this>(state: &'this mut Self::Lock<'_>) -> Self::This<'this> {
        Entities { entities: state }
    }

    fn get_resource_types() -> impl Iterator<Item = Borrow> {
        std::iter::empty()
    }

    fn get_component_types() -> impl Iterator<Item = Borrow> {
        std::iter::empty()
    }
}
