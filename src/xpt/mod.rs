//! XPT format implementation.
//!
//! This module contains the core XPT format handling, including version
//! definitions and version-specific implementations.

pub(crate) mod v5;
pub(crate) mod v8;
mod version;

pub use version::XptVersion;
