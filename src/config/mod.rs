//! Configuration types for xportrs.
//!
//! This module provides configuration options for reading and writing XPT files,
//! including policies for text encoding, strictness levels, and verbosity.

mod config;

pub use config::{Config, ReadOptions, TextMode, Verbosity, WriteOptions};
