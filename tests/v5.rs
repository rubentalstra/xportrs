//! XPT v5 format integration tests.
//!
//! This module contains comprehensive tests for XPT v5 format compliance:
//! - `read` - Tests for reading XPT files from tests/data/
//! - `write` - Tests for writing and round-trip verification
//! - `byte_layout` - Byte-level spec compliance tests

#[path = "v5/byte_layout.rs"]
mod byte_layout;

#[path = "v5/read.rs"]
mod read;

#[path = "v5/write.rs"]
mod write;
