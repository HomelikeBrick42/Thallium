use crate::{system::RunState, system_parameters::SystemParameter};
use slotmap::SlotMap;

slotmap::new_key_type! {
    pub struct Entity;
}

#[derive(Clone, Copy)]
pub struct Entities<'a> {
    entities: &'a SlotMap<Entity, ()>,
}

impl<'a> Entities<'a> {
    pub fn iter(&self) -> impl ExactSizeIterator<Item = Entity> + 'a {
        self.entities.keys()
    }
}

impl<'a> SystemParameter for Entities<'a> {
    type This<'this> = Entities<'this>;
    type Lock<'state> = &'state SlotMap<Entity, ()>;

    fn lock(state: RunState<'_>) -> Self::Lock<'_> {
        state.entities
    }

    fn construct<'this>(state: &'this mut Self::Lock<'_>) -> Self::This<'this> {
        Entities { entities: state }
    }
}
