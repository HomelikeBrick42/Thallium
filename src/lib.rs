use std::{
    collections::HashMap,
    any::TypeId,
};
use rayon::prelude::*;

type SystemType<T> = Box<dyn Fn(Entity, &mut T) + Send + Sync>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    id: usize,
    gen: std::num::NonZeroUsize, // hopefully Option<Entity> will use the 0 value as the discriminant
}

pub trait Component: Clone + Sized + Send + Sync + 'static {}

// For any implementations of this trait where a function takes a *mut ()
// the pointer given should be read immediately to avoid memory leaks
unsafe trait ComponentContainerTrait: Send + Sync {
    // a pointer to something that implements Component
    // the pointer passed should be to an object in a ManuallyDrop, this function takes ownership immediately
    unsafe fn add_component(&mut self, entity: Entity, component: *mut ()) -> *mut ();
    // supposed to be a pointer to SystemType
    // the pointer passed should be to an object in a ManuallyDrop, this function takes ownership immediately
    unsafe fn add_system(&mut self, system: *mut ());

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

unsafe impl<T: Component> ComponentContainerTrait for ComponentContainer<T> {
    unsafe fn add_component(&mut self, entity: Entity, component: *mut ()) -> *mut () {
        let component = unsafe { std::ptr::read(component as *mut T) };
        self.components.insert(entity, component);
        self.components.get_mut(&entity).unwrap() as *mut T as *mut ()
    }

    unsafe fn add_system(&mut self, system: *mut ()) {
        let system = unsafe { std::ptr::read(system as *mut SystemType<T>) };
        self.systems.push(system);
    }

    fn run_systems(&mut self) {
        self.systems.iter_mut().for_each(|system| {
            self.components.par_iter_mut().for_each(|(&entity, component)| {
                system(entity, component);
            });
        });
    }
}

pub struct ECS {
    components: HashMap<TypeId, Box<dyn ComponentContainerTrait>>,
}

impl ECS {
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> &mut T {
        let component_container = self.components
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<ComponentContainer<T>>::default());
        // using ManuallyDrop so that if the function call happens to panic, nothing is `drop`ped twice
        // `add_component` should immediately read the pointer before doing anything else
        // so it should be on the stack and `drop`ped even in the case of panic
        unsafe {
            let mut component = std::mem::ManuallyDrop::new(component);
            let component_ptr = component_container
                .add_component(entity, &mut *component as *mut T as *mut ()) as *mut T;
            &mut *component_ptr
        }
    }

    pub fn add_system<T: Component>(&mut self, system: SystemType<T>) {
        let component_container = self.components
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<ComponentContainer<T>>::default());
        // using ManuallyDrop so that if the function call happens to panic, nothing is `drop`ped twice
        // `add_system` should immediately read the pointer before doing anything else
        // so it should be on the stack and `drop`ped even in the case of panic
        unsafe {
            let mut system = std::mem::ManuallyDrop::new(system);
            component_container
                .add_system(&mut *system as *mut SystemType<T> as *mut ());
        }
    }

    pub fn run_systems(&mut self) {
        self.components.par_iter_mut().for_each(|(_, component_container)| {
            component_container.run_systems();
        });
    }
}
