# `thallium_ecs`

[![Latest Version](https://img.shields.io/crates/v/thallium_ecs.svg)](https://crates.io/crates/thallium_ecs)
[![Rust Documentation](https://docs.rs/thallium_ecs/badge.svg)](https://docs.rs/thallium_ecs)
![GitHub license](https://img.shields.io/badge/license-MIT-blue.svg)

A basic ECS that ive been working on

## Example code

```rust
use thallium_ecs::{
    app::App,
    component::Component,
    entities::Entities,
    query::{Query, Ref, RefMut},
    system_set::SystemSet,
};

struct Person {
    name: String,
    age: i32,
}
impl Component for Person {}

let mut app = App::new();

let person1 = app.create_entity();
app.add_component(person1, Person {
    name: "Alice".into(),
    age: 23,
});

let person2 = app.create_entity();
app.add_component(person2, Person {
    name: "Bob".into(),
    age: 25,
});

// create a system set that prints all people
let mut print_people = SystemSet::new();
print_people.register_system(|q: Query<'_, Ref<Person>>| {
    for (_, person) in q.iter() {
        println!("'{}' is {} years old", person.name, person.age);
    }
});

// print out all the people
// should print:
//
// 'Alice' is 23 years old
// 'Bob' is 25 years old
app.run(&mut print_people);

// increment the ages of all people
app.run(|mut q: Query<'_, RefMut<Person>>| {
    for (_, person) in q.iter_mut() {
        person.age += 1;
    }
});

// another way to increment the ages of all people would be
app.run(|entities: Entities<'_>, mut q: Query<'_, RefMut<Person>>| {
    for entity in entities.iter() {
        if let Some(person) = q.get_mut(entity) {
            person.age += 1;
        }
    }
});

// print out all the people again
// should print:
//
// 'Alice' is 25 years old
// 'Bob' is 27 years old
app.run(&mut print_people);
```
