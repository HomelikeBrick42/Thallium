#![doc = include_str!("../README.md")]

/// An alias for [`thallium_ecs`], also with the related deives from [`thallium_derive`]
#[cfg(feature = "ecs")]
#[cfg_attr(docsrs, doc(cfg(feature = "ecs")))]
pub mod ecs {
    #[cfg(feature = "derive")]
    #[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
    pub use thallium_derive::{Component, Resource};

    #[cfg_attr(docsrs, doc(cfg(feature = "ecs")))]
    pub use thallium_ecs::*;
}

/// An alias for [`thallium_derive`]
#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub mod derive {
    #[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
    pub use thallium_derive::*;
}
