use crate::{
    system::{Borrow, RunState},
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
    entities: &'a Vec<NonZeroUsize>,
}

impl<'a> Entities<'a> {
    pub fn iter(&self) -> impl ExactSizeIterator<Item = Entity> + 'a {
        self.entities
            .iter()
            .enumerate()
            .map(|(id, &generation)| Entity { id, generation })
    }
}

impl<'a> SystemParameter for Entities<'a> {
    type This<'this> = Entities<'this>;
    type Lock<'state> = &'state Vec<NonZeroUsize>;

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
