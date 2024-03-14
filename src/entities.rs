use crate::{
    system::{Borrow, EntityMap, RunState},
    system_parameters::SystemParameter,
};
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    pub(crate) id: usize,
    pub(crate) generation: NonZeroUsize,
}

#[derive(Clone, Copy)]
pub struct Entities<'a> {
    entities: &'a EntityMap,
}

impl<'a> Entities<'a> {
    pub fn entity_exists(&self, entity: Entity) -> bool {
        self.entities
            .get(entity.id)
            .map_or(false, |&(generation, _)| generation == entity.generation)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = Entity> + 'a {
        self.entities
            .iter()
            .enumerate()
            .map(|(id, &(generation, _))| Entity { id, generation })
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
