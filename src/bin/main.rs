use thallium_ecs::*;

struct TestComponent {
    value: i32,
}

impl Component for TestComponent {}

struct TestComponent2 {
    some_other_value: Option<f64>,
}

impl Component for TestComponent2 {}

fn main() {
    let mut ecs = ECS::new();
    ecs.register_system(|_entity, test_component: &mut TestComponent| {
        println!("System 1: {}", test_component.value);
        test_component.value += 1;
    });

    ecs.register_system(
        |_entity, test_component: &mut TestComponent, test_component_2: &mut TestComponent2| {
            println!("System 2: {}", test_component.value);
            if let Some(value) = test_component_2.some_other_value {
                println!("System 2 value: {value}");
            }
        },
    );

    let a = ecs.create_entity();
    ecs.add_component(a, TestComponent { value: 5 });

    let b = ecs.create_entity();
    ecs.add_component(b, TestComponent { value: -42 });
    ecs.add_component(
        b,
        TestComponent2 {
            some_other_value: Some(1.23),
        },
    );

    ecs.run_registered_systems();
    println!();
    ecs.run_registered_systems();
    println!();
    ecs.run_registered_systems();
}
