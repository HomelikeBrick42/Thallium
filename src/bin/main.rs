use thallium_ecs::*;

#[derive(Clone)]
struct TestComponent {
    value: i32,
}

impl Component for TestComponent {}

fn main() {
    let mut ecs = ECS::new();
    ecs.add_system(|_entity: Entity, test_component: &mut TestComponent| {
        println!("{}", test_component.value);
        test_component.value += 1;
    });

    let a = ecs.create_entity();
    ecs.add_component(a, TestComponent { value: 5 });

    let b = ecs.create_entity();
    ecs.add_component(b, TestComponent { value: -42 });

    ecs.run_systems();
    println!();
    ecs.run_systems();
    println!();
    ecs.run_systems();
}
