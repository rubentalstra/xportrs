//! XPT Version 5 implementation.
//!
//! This module contains the complete implementation for reading and writing
//! XPT v5 (SAS Transport) format files.

// Allow dead code in internal format implementation modules.
// Many functions are part of a complete XPT v5 API but may not be used yet.
#[allow(dead_code)]
pub(crate) mod constants;
#[allow(dead_code)]
pub(crate) mod encoding;
#[allow(dead_code)]
pub(crate) mod namestr;
#[allow(dead_code)]
pub(crate) mod read;
#[allow(dead_code)]
pub(crate) mod record;
#[allow(dead_code)]
pub(crate) mod timestamp;
#[allow(dead_code)]
pub(crate) mod write;
