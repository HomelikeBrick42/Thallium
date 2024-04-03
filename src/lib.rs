#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

/// An alias for [`thallium_ecs`], also with the related deives from [`thallium_derive`]
#[cfg(feature = "ecs")]
pub mod ecs {
    #[cfg(feature = "derive")]
    pub use thallium_derive::{Component, Resource};

    pub use thallium_ecs::*;
}

/// An alias for [`thallium_derive`]
#[cfg(feature = "derive")]
pub mod derive {
    pub use thallium_derive::*;
}
