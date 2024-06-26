#![doc = include_str!("../README.md")]

mod app;
mod commands;
mod component;
mod component_container;
mod entities;
mod query;
mod query_parameters;
mod resource;
mod resource_container;
mod system;
mod system_parameters;
mod system_set;
mod tick;

pub use app::App;
pub use commands::Commands;
pub use component::Component;
pub use entities::{Entities, Entity};
pub use query::{Query, Ref, RefMut};
pub use query_parameters::QueryParameter;
pub use resource::{Res, ResMut, Resource};
pub use system::{IntoSystem, System, SystemFunction};
pub use system_parameters::SystemParameter;
pub use system_set::SystemSet;
pub use tick::{CurrentTick, LastRunTick};

#[cfg(test)]
mod tests {
    use crate::{App, Commands, Component, Entities, Query, Ref, RefMut, SystemSet};

    #[test]
    fn test() {
        struct TestComponent {
            value: i32,
        }
        impl Component for TestComponent {}

        struct TestComponent2 {
            value: i32,
        }
        impl Component for TestComponent2 {}

        let mut app = App::new();

        let entity1 = app.create_entity();
        app.add_component(entity1, TestComponent { value: 42 });

        let entity2 = app.create_entity();
        app.add_component(entity2, TestComponent { value: 44 });
        app.add_component(entity2, TestComponent2 { value: 0 });

        app.run(|mut q: Query<'_, RefMut<'_, TestComponent>>| {
            let [mut c1, mut c2] = q.get_many_mut([entity1, entity2]).unwrap();
            assert_eq!(c1.value, 42);
            assert_eq!(c2.value, 44);
            c1.value += 1;
            c2.value -= 1;
        });

        let mut set = SystemSet::new();
        set.register_system(
            |q1: Query<'_, Ref<'_, TestComponent>>,
             q2: Query<'_, Option<(Ref<'_, TestComponent2>, Ref<'_, TestComponent>)>>| {
                for (entity, c) in q1.iter() {
                    assert_eq!(c.value, 43);
                    if let Some((c2, c)) = q2.get(entity).unwrap() {
                        assert_eq!(c2.value, 0);
                        assert_eq!(c.value, 43);
                    }
                }
            },
        );
        app.run(&mut set);

        app.run(|mut commands: Commands<'_>| {
            commands.create_entity(());
            commands.create_entity(TestComponent { value: 5 });
        });

        app.run(
            |entities: Entities<'_>, q: Query<'_, Ref<'_, TestComponent>>| {
                assert_eq!(entities.iter().count(), 4);
                assert_eq!(
                    entities
                        .iter()
                        .filter(|&entity| q.get(entity).is_some())
                        .count(),
                    3
                );
            },
        );
    }
}
