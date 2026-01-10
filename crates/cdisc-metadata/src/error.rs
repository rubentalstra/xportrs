//! Error types for CDISC metadata parsing.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for CDISC metadata operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when parsing CDISC metadata.
#[derive(Debug, Error)]
pub enum Error {
    /// IO error reading files.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// CSV parsing error.
    #[error("CSV parsing error: {0}")]
    Csv(#[from] csv::Error),

    /// TOML parsing error.
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Missing required file.
    #[error("missing required file: {0}")]
    MissingFile(PathBuf),

    /// Invalid metadata format.
    #[error("invalid metadata format: {0}")]
    InvalidFormat(String),

    /// Unknown standard type.
    #[error("unknown standard: {0}")]
    UnknownStandard(String),
}
