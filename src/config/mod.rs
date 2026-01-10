//! Configuration types for xportrs.
//!
//! This module provides configuration options for reading and writing XPT files,
//! including policies for text encoding, strictness levels, and verbosity.

mod types;

pub(crate) use types::{Config, ReadOptions, WriteOptions};
pub use types::{TextMode, Verbosity};
