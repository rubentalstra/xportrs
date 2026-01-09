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

use crate::error::{Result, XportrsError};
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
/// Always returns [`XportrsError::UnsupportedVersion`].
pub fn unsupported_error() -> XportrsError {
    XportrsError::UnsupportedVersion {
        version: XptVersion::V8,
    }
}

/// Placeholder for future V8 reader.
pub fn read_v8() -> Result<()> {
    Err(unsupported_error())
}

/// Placeholder for future V8 writer.
pub fn write_v8() -> Result<()> {
    Err(unsupported_error())
}
