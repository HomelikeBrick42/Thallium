use crate::{Component, Entity};
use std::{any::Any, num::NonZeroUsize};

pub(crate) trait DynComponentContainer: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, entity: Entity);
}

impl<C> DynComponentContainer for ComponentContainer<C>
where
    C: Component,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove(&mut self, entity: Entity) {
        self.remove(entity);
    }
}

impl dyn DynComponentContainer {
    pub fn downcast_ref<C>(&self) -> &ComponentContainer<C>
    where
        C: Component,
    {
        self.as_any()
            .downcast_ref::<ComponentContainer<C>>()
            .unwrap()
    }

    pub fn downcast_mut<C>(&mut self) -> &mut ComponentContainer<C>
    where
        C: Component,
    {
        self.as_any_mut()
            .downcast_mut::<ComponentContainer<C>>()
            .unwrap()
    }
}

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

    #[allow(unsafe_code)]
    pub(crate) fn get_many_mut<const N: usize>(
        &mut self,
        entities: [Entity; N],
    ) -> Option<[&mut C; N]> {
        // check that there are no invalid entities
        {
            let mut i = 0;
            while i < entities.len() {
                let entity = entities[i];
                if entities[i].id >= self.components.len() {
                    break;
                }
                if !self.components[entity.id]
                    .as_ref()
                    .map_or(false, |&(generation, _)| generation == entity.generation)
                {
                    break;
                }

                unsafe {
                    self.components[entity.id].as_mut().unwrap_unchecked().0 |= NonZeroUsize::MIN;
                }

                i += 1;
            }

            for &entity in &entities[..i] {
                unsafe {
                    let generation = &mut self.components[entity.id].as_mut().unwrap_unchecked().0;
                    *generation = NonZeroUsize::new_unchecked(generation.get() & !1usize);
                }
            }

            if i != entities.len() {
                return None;
            }
        }

        unsafe {
            let components_ptr = self.components.as_mut_ptr();
            Some(entities.map(|entity| {
                &mut (*components_ptr.add(entity.id))
                    .as_mut()
                    .unwrap_unchecked()
                    .1
            }))
        }
    }
}
