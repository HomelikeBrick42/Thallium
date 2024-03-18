use crate::{Component, Entity, Ref, RefMut};
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

pub struct ComponentSlot<C>
where
    C: Component,
{
    pub(crate) generation: NonZeroUsize,
    pub(crate) component: C,
    pub(crate) last_modified_tick: u64,
}

pub struct ComponentContainer<C>
where
    C: Component,
{
    pub(crate) components: Vec<Option<ComponentSlot<C>>>,
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

    pub(crate) fn insert(&mut self, current_tick: u64, entity: Entity, component: C) {
        if entity.id >= self.components.len() {
            self.components.resize_with(entity.id + 1, || None);
        }
        self.components[entity.id] = Some(ComponentSlot {
            generation: entity.generation,
            component,
            last_modified_tick: current_tick,
        });
    }

    pub(crate) fn remove(&mut self, entity: Entity) -> Option<C> {
        if let Some(&Some(ComponentSlot { generation, .. })) = self.components.get(entity.id) {
            if generation == entity.generation {
                return self.components[entity.id]
                    .take()
                    .map(|ComponentSlot { component, .. }| component);
            }
        }
        None
    }

    pub(crate) fn get(&self, current_tick: u64, entity: Entity) -> Option<Ref<'_, C>> {
        self.components
            .get(entity.id)
            .and_then(|slot| slot.as_ref())
            .and_then(
                |&ComponentSlot {
                     generation,
                     ref component,
                     last_modified_tick,
                 }| {
                    (generation == entity.generation).then_some(Ref {
                        component,
                        last_modified_tick,
                        current_tick,
                    })
                },
            )
    }

    pub(crate) fn get_mut(&mut self, current_tick: u64, entity: Entity) -> Option<RefMut<'_, C>> {
        self.components
            .get_mut(entity.id)
            .and_then(|slot| slot.as_mut())
            .and_then(
                |&mut ComponentSlot {
                     generation,
                     ref mut component,
                     ref mut last_modified_tick,
                 }| {
                    (generation == entity.generation).then_some(RefMut {
                        component,
                        last_modified_tick,
                        current_tick,
                    })
                },
            )
    }

    #[allow(unsafe_code)]
    pub(crate) fn get_many_mut<const N: usize>(
        &mut self,
        current_tick: u64,
        entities: [Entity; N],
    ) -> Option<[RefMut<'_, C>; N]> {
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
                    .map_or(false, |&ComponentSlot { generation, .. }| {
                        generation == entity.generation
                    })
                {
                    break;
                }

                // an odd generation number signals that its invalid, so later iterations cant see that this component is valid
                unsafe {
                    self.components[entity.id]
                        .as_mut()
                        .unwrap_unchecked()
                        .generation |= NonZeroUsize::MIN;
                }

                i += 1;
            }

            // unset all the modified components to make them valid again
            for &entity in &entities[..i] {
                unsafe {
                    let generation = &mut self.components[entity.id]
                        .as_mut()
                        .unwrap_unchecked()
                        .generation;
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
                let ComponentSlot {
                    component,
                    last_modified_tick,
                    ..
                } = (*components_ptr.add(entity.id)).as_mut().unwrap_unchecked();
                RefMut {
                    component,
                    last_modified_tick,
                    current_tick,
                }
            }))
        }
    }
}
