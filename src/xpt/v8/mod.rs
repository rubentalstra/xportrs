//! XPT Version 8 implementation (not yet implemented).
//!
//! This module is a placeholder for future XPT v8 support.
//! V8 differs from V5 in several ways:
//!
//! - Variable names can be up to 32 bytes (vs 8)
//! - Labels can be up to 256 bytes (vs 40)
//! - Different header structure
//! - Different NAMESTR format
//!
//! See `README.md` in this module for more details.

// Allow dead code for placeholder implementation - these functions
// are part of the planned v8 API but not yet used.
#![allow(dead_code)]

use crate::error::{Result, Error};
use crate::xpt::XptVersion;

/// Checks if V8 operations are supported.
///
/// Currently returns `false` as V8 is not implemented.
#[must_use]
pub const fn is_supported() -> bool {
    false
}

/// Returns an error indicating V8 is not supported.
///
/// # Errors
///
/// Always returns [`Error::UnsupportedVersion`].
#[must_use]
pub fn unsupported_error() -> Error {
    Error::UnsupportedVersion {
        version: XptVersion::V8,
    }
}

/// Placeholder for future V8 reader.
///
/// # Errors
///
/// Always returns [`Error::UnsupportedVersion`] as V8 is not yet implemented.
pub fn read_v8() -> Result<()> {
    Err(unsupported_error())
}

/// Placeholder for future V8 writer.
///
/// # Errors
///
/// Always returns [`Error::UnsupportedVersion`] as V8 is not yet implemented.
pub fn write_v8() -> Result<()> {
    Err(unsupported_error())
}
