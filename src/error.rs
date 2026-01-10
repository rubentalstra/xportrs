//! Error types for xportrs.
//!
//! This module defines the [`Error`] enum which represents all possible
//! errors that can occur during XPT file reading, writing, and validation.

use std::path::PathBuf;

use crate::xpt::XptVersion;

/// The main error type for xportrs operations.
///
/// This enum covers all error conditions that can arise when working with XPT files,
/// including I/O errors, format violations, validation failures, and unsupported features.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// An I/O error occurred while reading or writing.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// The requested XPT version is not supported.
    #[error("unsupported XPT version: {version:?}")]
    UnsupportedVersion {
        /// The unsupported version that was requested.
        version: XptVersion,
    },

    /// The XPT file is corrupt or malformed.
    #[error("corrupt xpt: {message}")]
    Corrupt {
        /// A description of what makes the file corrupt.
        message: String,
    },

    /// The schema or structure is invalid.
    #[error("invalid schema: {message}")]
    InvalidSchema {
        /// A description of what makes the schema invalid.
        message: String,
    },

    /// Validation failed with one or more errors.
    #[error("validation failed: {message}")]
    ValidationFailed {
        /// A description of the validation failure.
        message: String,
    },

    /// A metadata-related error occurred.
    #[error("metadata error: {message}")]
    Metadata {
        /// A description of the metadata error.
        message: String,
    },

    /// The requested member (dataset) was not found in the XPT file.
    #[error("member not found: {domain_code}")]
    MemberNotFound {
        /// The domain code that was not found.
        domain_code: String,
    },

    /// Column length mismatch - all columns must have the same number of rows.
    #[error(
        "column length mismatch: column '{column_name}' has {actual} rows, expected {expected}"
    )]
    ColumnLengthMismatch {
        /// Name of the column with mismatched length.
        column_name: String,
        /// The actual number of rows in the column.
        actual: usize,
        /// The expected number of rows.
        expected: usize,
    },

    /// A file path error occurred.
    #[error("path error: {message} (path: {path:?})")]
    Path {
        /// A description of the path error.
        message: String,
        /// The problematic path.
        path: PathBuf,
    },

    /// An encoding error occurred during text conversion.
    #[error("encoding error: {message}")]
    Encoding {
        /// A description of the encoding error.
        message: String,
    },
}

impl Error {
    /// Creates a new [`Error::Corrupt`] error.
    #[must_use]
    pub fn corrupt(message: impl Into<String>) -> Self {
        Self::Corrupt {
            message: message.into(),
        }
    }

    /// Creates a new [`Error::InvalidSchema`] error.
    #[must_use]
    pub fn invalid_schema(message: impl Into<String>) -> Self {
        Self::InvalidSchema {
            message: message.into(),
        }
    }

    /// Creates a new [`Error::ValidationFailed`] error.
    #[must_use]
    pub fn validation_failed(message: impl Into<String>) -> Self {
        Self::ValidationFailed {
            message: message.into(),
        }
    }

    /// Creates a new [`Error::Metadata`] error.
    #[must_use]
    pub fn metadata(message: impl Into<String>) -> Self {
        Self::Metadata {
            message: message.into(),
        }
    }

    /// Creates a new [`Error::Encoding`] error.
    #[must_use]
    pub fn encoding(message: impl Into<String>) -> Self {
        Self::Encoding {
            message: message.into(),
        }
    }
}

/// A type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
