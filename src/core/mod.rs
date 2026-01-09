//! Low-level XPT format implementation.
//!
//! This module contains the core XPT format handling including:
//! - IEEE to IBM floating-point conversion
//! - Header parsing and building
//! - Binary reader and writer implementations
//!
//! Most users should use the high-level [`crate::io`] module instead.

pub mod float;
pub mod header;
pub mod reader;
pub mod writer;
