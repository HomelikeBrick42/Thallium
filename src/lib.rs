#![doc = include_str!("../README.md")]

/// An alias for [`thallium_ecs`], also with the related deives from [`thallium_derive`]
pub mod ecs {
    pub use thallium_derive::{Component, Resource};
    pub use thallium_ecs::*;
}

/// An alias for [`thallium_derive`]
pub mod derive {
    pub use thallium_derive::*;
}

/// An alias for [`thallium_renderer`]
pub mod renderer {
    pub use thallium_renderer::*;
}
