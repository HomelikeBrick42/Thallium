# `thallium_ecs`

[![Latest Version](https://img.shields.io/crates/v/thallium_ecs.svg)](https://crates.io/crates/thallium_ecs)
[![Rust Documentation](https://docs.rs/thallium_ecs/badge.svg)](https://docs.rs/thallium_ecs)
[![GitHub license](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/HomelikeBrick42/thallium_ecs/master/LICENSE)

A basic ECS that ive been working on

## Example code

```rust
use thallium_ecs::{
    app::App,
    entities::Entities,
    query::{Component, Query, Ref, RefMut},
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
    age: 20,
});

let person1 = app.create_entity();
app.add_component(person1, Person {
    name: "Bob".into(),
    age: 25,
});

// increment the ages of all people
app.run_once(|entities: Entities<'_>, mut q: Query<'_, RefMut<Person>>| {
    for entity in entities.iter() {
        if let Some(person) = q.get_mut(entity) {
            person.age += 1;
        }
    }
});

app.register_system(|entities: Entities<'_>, q: Query<'_, Ref<Person>>| {
    for entity in entities.iter() {
        if let Some(person) = q.get(entity) {
            println!("'{}' is {} years old", person.name, person.age);
        }
    }
});

app.run_registered();
```
