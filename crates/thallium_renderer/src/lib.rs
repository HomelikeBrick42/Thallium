#![doc = include_str!("../README.md")]

use thallium_derive::Component;

#[derive(Component)]
pub struct Material {
    pub color: [f32; 3],
}
