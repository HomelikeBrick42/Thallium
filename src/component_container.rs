use crate::{entities::Entity, query::Component};
use std::num::NonZeroUsize;

pub struct ComponentContainer<C>
where
    C: Component,
{
    pub(crate) components: Vec<Option<(NonZeroUsize, C)>>,
}

impl<C> ComponentContainer<C>
where
    C: Component,
{
    pub(crate) fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub(crate) fn insert(&mut self, entity: Entity, component: C) {
        if entity.id >= self.components.len() {
            self.components.resize_with(entity.id + 1, || None);
        }
        self.components[entity.id] = Some((entity.generation, component));
    }

    pub(crate) fn remove(&mut self, entity: Entity) -> Option<C> {
        if let Some(&Some((generation, _))) = self.components.get(entity.id) {
            if generation == entity.generation {
                return self.components[entity.id]
                    .take()
                    .map(|(_, component)| component);
            }
        }
        None
    }

    pub(crate) fn get(&self, entity: Entity) -> Option<&C> {
        self.components
            .get(entity.id)
            .and_then(|slot| slot.as_ref())
            .and_then(|&(generation, ref component)| {
                (generation == entity.generation).then_some(component)
            })
    }

    pub(crate) fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        self.components
            .get_mut(entity.id)
            .and_then(|slot| slot.as_mut())
            .and_then(|&mut (generation, ref mut component)| {
                (generation == entity.generation).then_some(component)
            })
    }

    pub(crate) fn get_many_mut<const N: usize>(
        &mut self,
        mut entities: [Entity; N],
    ) -> Option<[&mut C; N]> {
        entities.sort_unstable();

        // Make sure all entities are referencing valid components
        {
            let mut previous_entity_id = usize::MAX;
            for entity in entities {
                if entity.id == previous_entity_id {
                    return None;
                }
                if self.components.get(entity.id)?.as_ref()?.0 != entity.generation {
                    return None;
                }
                previous_entity_id = entity.id;
            }
        }

        let mut previous_id = 0;
        let mut components = self.components.as_mut_slice();
        Some(entities.map(|entity| {
            let (component, rest) = std::mem::take(&mut components)
                [entity.id.saturating_sub(previous_id + 1)..]
                .split_first_mut()
                .unwrap();
            components = rest;
            previous_id = entity.id;
            &mut component.as_mut().unwrap().1
        }))
    }
}
