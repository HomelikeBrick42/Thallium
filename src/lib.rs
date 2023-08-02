use hashbrown::{HashMap, HashSet};
use rayon::prelude::*;
use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    marker::PhantomData,
    num::NonZeroUsize,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    id: usize,
    gen: std::num::NonZeroUsize,
}

pub trait Component: Sized + Send + Sync + 'static {}

trait ComponentContainerTrait: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove_entity(&mut self, entity: Entity);
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
}

impl<T: Component> Default for ComponentContainer<T> {
    fn default() -> Self {
        Self {
            components: Vec::default(),
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

struct ComponentContainers {
    containers: HashMap<TypeId, Box<UnsafeCell<dyn ComponentContainerTrait>>>,
}
unsafe impl Sync for ComponentContainers {}

type Entities = Vec<(bool, NonZeroUsize)>;
pub struct SystemParam<'a> {
    component_containers: &'a ComponentContainers,
    entities: &'a Entities,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Mutability {
    Unique,
    Shared,
}

pub trait System {
    fn get_type_ids(&self, ids: &mut Vec<(TypeId, Mutability)>);
    fn run_system(&self, ecs: SystemParam<'_>);
}

/// # Safety
/// no safety docs yet
unsafe trait SystemParameter: Sync + Sized {
    fn get_type_ids(ids: &mut Vec<(TypeId, Mutability)>);
    unsafe fn from_system_param<'a>(entity: Entity, ecs: &'a SystemParam<'a>) -> Option<Self>
    where
        Self: 'a;
}

macro_rules! define_system_parameter {
    ($($generic:ident),*) => {
        unsafe impl<$($generic,)*> SystemParameter for ($($generic,)*)
        where
            $($generic: SystemParameter,)*
        {
            fn get_type_ids(ids: &mut Vec<(TypeId, Mutability)>) {
                let _ = ids;
                $($generic::get_type_ids(ids);)*
            }

            unsafe fn from_system_param<'a>(
                entity: Entity,
                ecs: &'a SystemParam<'a>,
            ) -> Option<Self>
            where
                Self: 'a,
            {
                let _ = entity;
                let _ = ecs;
                Some(($($generic::from_system_param(entity, ecs)?,)*))
            }
        }
    };
}

define_system_parameter!();
define_system_parameter!(A);
define_system_parameter!(A, B);
define_system_parameter!(A, B, C);
define_system_parameter!(A, B, C, D);
define_system_parameter!(A, B, C, D, E);
define_system_parameter!(A, B, C, D, E, F);
define_system_parameter!(A, B, C, D, E, F, G);
define_system_parameter!(A, B, C, D, E, F, G, H);
define_system_parameter!(A, B, C, D, E, F, G, H, I);
define_system_parameter!(A, B, C, D, E, F, G, H, I, J);

unsafe impl<'a, T> SystemParameter for &'a mut T
where
    T: Component,
{
    fn get_type_ids(ids: &mut Vec<(TypeId, Mutability)>) {
        ids.push((TypeId::of::<T>(), Mutability::Unique));
    }

    unsafe fn from_system_param<'b>(entity: Entity, ecs: &'b SystemParam<'b>) -> Option<Self>
    where
        'a: 'b,
    {
        let components = &mut (*ecs
            .component_containers
            .containers
            .get(&TypeId::of::<T>())?
            .get())
        .as_component_container_mut()
        .unwrap()
        .components;
        components.get_mut(entity.id).and_then(|data| {
            let &mut (gen, ref mut component) = data.as_mut()?;
            assert!(entity.gen == gen);
            Some(component)
        })
    }
}

unsafe impl<'a, T> SystemParameter for &'a T
where
    T: Component,
{
    fn get_type_ids(ids: &mut Vec<(TypeId, Mutability)>) {
        ids.push((TypeId::of::<T>(), Mutability::Unique));
    }

    unsafe fn from_system_param<'b>(entity: Entity, ecs: &'b SystemParam<'b>) -> Option<Self>
    where
        'a: 'b,
    {
        let components = &(*ecs
            .component_containers
            .containers
            .get(&TypeId::of::<T>())?
            .get())
        .as_component_container()
        .unwrap()
        .components;
        components.get(entity.id).and_then(|data| {
            let &(gen, ref component) = data.as_ref()?;
            assert!(entity.gen == gen);
            Some(component)
        })
    }
}

pub struct SystemWrapper<Args, F>(F, PhantomData<Args>);
impl<Args, F> From<F> for SystemWrapper<Args, F>
where
    Self: System,
{
    fn from(f: F) -> Self {
        SystemWrapper(f, PhantomData)
    }
}

macro_rules! define_system {
    ($($generic:ident),*) => {
        impl<$($generic,)* Func> System for SystemWrapper<($($generic,)*), Func>
        where
            $($generic: SystemParameter,)*
            Func: Fn(Entity, $($generic,)*) + Sync,
        {
            fn get_type_ids(&self, ids: &mut Vec<(TypeId, Mutability)>) {
                let _ = ids;
                $($generic::get_type_ids(ids);)*
            }

            fn run_system(&self, ecs: SystemParam<'_>) {
                ecs.entities
                    .par_iter()
                    .enumerate()
                    .for_each(|(id, &(alive, gen))| {
                        if !alive {
                            return;
                        }
                        let entity = Entity { id, gen };
                        $(
                            #[allow(non_snake_case)]
                            let Some($generic) = (unsafe { $generic::from_system_param(entity, &ecs) }) else {
                                return;
                            };
                        )*
                        self.0(entity, $($generic,)*);
                    });
            }
        }
    };
}

define_system!();
define_system!(A);
define_system!(A, B);
define_system!(A, B, C);
define_system!(A, B, C, D);
define_system!(A, B, C, D, E);
define_system!(A, B, C, D, E, F);
define_system!(A, B, C, D, E, F, G);
define_system!(A, B, C, D, E, F, G, H);
define_system!(A, B, C, D, E, F, G, H, I);
define_system!(A, B, C, D, E, F, G, H, I, J);

pub struct ECS {
    // TODO: we are trimming the end off when removing entities, but what about trimming the front?
    entities: Entities,
    next_free_entity: usize,
    component_containers: ComponentContainers,
    systems: Vec<Box<dyn System>>,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            next_free_entity: 0,
            component_containers: ComponentContainers {
                containers: HashMap::new(),
            },
            systems: Vec::new(),
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
            .containers
            .par_iter_mut()
            .for_each(|(_, component_container)| {
                component_container.get_mut().remove_entity(entity);
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
            .containers
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<UnsafeCell<ComponentContainer<T>>>::default())
            .get_mut()
            .as_component_container_mut()
            .unwrap()
            .components;

        if entity.id >= components.len() {
            let count = entity.id - components.len() + 1;
            components.reserve(count);
            components.extend(std::iter::repeat_with(|| None).take(count));
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
            .containers
            .get_mut(&TypeId::of::<T>())?
            .get_mut()
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
            .containers
            .get(&TypeId::of::<T>())?;
        // safe, all the methods that modify components take &mut self
        let components = unsafe {
            &(*components.get())
                .as_component_container()
                .unwrap()
                .components
        };
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
            .containers
            .get_mut(&TypeId::of::<T>())?
            .get_mut()
            .as_component_container_mut()
            .unwrap()
            .components;
        components.get_mut(entity.id).and_then(|data| {
            let &mut (gen, ref mut component) = data.as_mut()?;
            assert!(entity.gen == gen);
            Some(component)
        })
    }

    pub fn register_system<T: 'static, F: 'static>(
        &mut self,
        system: impl Into<SystemWrapper<T, F>>,
    ) where
        SystemWrapper<T, F>: System,
    {
        let system = system.into();
        let mut types = vec![];
        system.get_type_ids(&mut types);
        assert!(Self::is_system_parameter_types_valid(&types));
        self.systems.push(Box::new(system));
    }

    pub fn run_system<T: 'static, F: 'static>(&mut self, system: impl Into<SystemWrapper<T, F>>)
    where
        SystemWrapper<T, F>: System,
    {
        let system = system.into();
        let mut types = vec![];
        system.get_type_ids(&mut types);
        assert!(Self::is_system_parameter_types_valid(&types));
        system.run_system(SystemParam {
            component_containers: &self.component_containers,
            entities: &self.entities,
        });
    }

    pub fn run_registered_systems(&mut self) {
        for system in &self.systems {
            system.run_system(SystemParam {
                component_containers: &self.component_containers,
                entities: &self.entities,
            });
        }
    }

    fn is_system_parameter_types_valid(types: &[(TypeId, Mutability)]) -> bool {
        let mut set = HashSet::new();
        for &(typ, mutability) in types {
            if !set.insert(typ) {
                if let Mutability::Unique = mutability {
                    return false;
                }
            }
        }
        true
    }
}

impl Default for ECS {
    fn default() -> Self {
        Self::new()
    }
}
