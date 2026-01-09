//! Configuration types for xportrs.
//!
//! This module provides configuration options for reading and writing XPT files,
//! including policies for text encoding, strictness levels, and verbosity.

mod types;

pub use types::{Config, ReadOptions, TextMode, Verbosity, WriteOptions};
