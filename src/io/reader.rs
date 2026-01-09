//! High-level XPT file reading with metadata support.
//!
//! This module provides an ergonomic API for reading XPT files with
//! integrated validation and metadata extraction.
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! use xportrs::io::{read_xpt, read_xpt_with_validation};
//! use xportrs::validation::Validator;
//! use xportrs::XptVersion;
//!
//! // Simple read
//! let dataset = read_xpt(Path::new("dm.xpt")).unwrap();
//! println!("Read {} rows", dataset.num_rows());
//!
//! // Read with validation
//! let result = read_xpt_with_validation(Path::new("dm.xpt"), XptVersion::V5).unwrap();
//! if result.validation.has_issues() {
//!     eprintln!("Validation issues: {}", result.validation);
//! }
//! ```

use std::fs::File;
use std::path::Path;

use crate::core::reader::{self, StreamingReader};
use crate::error::{ValidationResult, XptError};
use crate::types::{XptDataset, XptReaderOptions};
use crate::validation::{ValidationMode, Validator};
use crate::XptVersion;

// Re-export core reader types for convenience
pub use crate::core::reader::{ObservationIter, XptReader};

/// Result of reading an XPT file with validation.
#[derive(Debug)]
pub struct ReadResult {
    /// The loaded dataset.
    pub dataset: XptDataset,
    /// Detected XPT version.
    pub version: XptVersion,
    /// Validation result (may have warnings).
    pub validation: ValidationResult,
}

impl ReadResult {
    /// Check if the read was fully successful (no errors or warnings).
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.validation.is_fda_compliant()
    }

    /// Check if the read was valid (no errors, warnings allowed).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.validation.is_valid()
    }
}

/// Read an XPT file from path.
///
/// This is the simplest way to read an XPT file. For more control,
/// use [`read_xpt_with_options`] or [`read_xpt_with_validation`].
///
/// # Errors
///
/// Returns an error if the file cannot be opened or parsed.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use xportrs::io::read_xpt;
///
/// let dataset = read_xpt(Path::new("dm.xpt")).unwrap();
/// println!("Dataset: {}", dataset.name);
/// ```
pub fn read_xpt(path: &Path) -> crate::error::Result<XptDataset> {
    reader::read_xpt(path)
}

/// Read an XPT file with custom options.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or parsed.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use xportrs::io::read_xpt_with_options;
/// use xportrs::XptReaderOptions;
///
/// let options = XptReaderOptions::new().strict();
/// let dataset = read_xpt_with_options(Path::new("dm.xpt"), options).unwrap();
/// ```
pub fn read_xpt_with_options(
    path: &Path,
    options: XptReaderOptions,
) -> crate::error::Result<XptDataset> {
    reader::read_xpt_with_options(path, options)
}

/// Read an XPT file with automatic validation.
///
/// This function reads the file and validates it against the specified
/// XPT version's rules. The validation result is returned alongside the dataset.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or parsed.
/// Validation issues are returned in the `ReadResult`, not as errors.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use xportrs::io::read_xpt_with_validation;
/// use xportrs::XptVersion;
///
/// let result = read_xpt_with_validation(Path::new("dm.xpt"), XptVersion::V5).unwrap();
/// if !result.is_valid() {
///     for error in &result.validation.errors {
///         eprintln!("Error: {}", error);
///     }
/// }
/// ```
pub fn read_xpt_with_validation(path: &Path, version: XptVersion) -> crate::error::Result<ReadResult> {
    let file = File::open(path).map_err(|_| XptError::file_not_found(path))?;
    let mut reader = StreamingReader::new(file)?;
    let detected_version = reader.version();

    let dataset = reader_to_dataset(&mut reader)?;

    let validator = Validator::new(version);
    let validation = validator.validate(&dataset);

    Ok(ReadResult {
        dataset,
        version: detected_version,
        validation,
    })
}

/// Read an XPT file with FDA compliance validation.
///
/// This is a convenience function that reads a file and validates it
/// against FDA submission requirements.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or parsed.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use xportrs::io::read_xpt_fda_compliant;
///
/// let result = read_xpt_fda_compliant(Path::new("dm.xpt")).unwrap();
/// if !result.validation.is_fda_compliant() {
///     eprintln!("Not FDA compliant!");
///     for warning in &result.validation.warnings {
///         eprintln!("Warning: {}", warning);
///     }
/// }
/// ```
pub fn read_xpt_fda_compliant(path: &Path) -> crate::error::Result<ReadResult> {
    let file = File::open(path).map_err(|_| XptError::file_not_found(path))?;
    let mut reader = StreamingReader::new(file)?;
    let detected_version = reader.version();

    let dataset = reader_to_dataset(&mut reader)?;

    let validator = Validator::fda_compliant(XptVersion::V5)
        .with_mode(ValidationMode::FdaCompliant);
    let validation = validator.validate(&dataset);

    Ok(ReadResult {
        dataset,
        version: detected_version,
        validation,
    })
}

/// Create a streaming reader for large files.
///
/// Use this when you need to process a large file without loading
/// all data into memory.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or the header is invalid.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use xportrs::io::read_xpt_streaming;
///
/// let mut reader = read_xpt_streaming(Path::new("large.xpt")).unwrap();
/// println!("Dataset: {}", reader.meta().name);
///
/// for observation in reader.observations() {
///     let obs = observation.unwrap();
///     // Process each row...
/// }
/// ```
pub fn read_xpt_streaming(path: &Path) -> crate::error::Result<StreamingReader<File>> {
    reader::read_xpt_streaming(path)
}

/// Create a streaming reader with custom options.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or the header is invalid.
pub fn read_xpt_streaming_with_options(
    path: &Path,
    options: XptReaderOptions,
) -> crate::error::Result<StreamingReader<File>> {
    reader::read_xpt_streaming_with_options(path, options)
}

/// Get metadata from an XPT file without reading all data.
///
/// This is useful for inspecting a file's structure without loading
/// all observations into memory.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or parsed.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use xportrs::io::get_xpt_metadata;
///
/// let meta = get_xpt_metadata(Path::new("dm.xpt")).unwrap();
/// println!("Dataset: {}", meta.name);
/// println!("Version: {:?}", meta.version);
/// println!("Columns: {:?}", meta.columns.iter().map(|c| &c.name).collect::<Vec<_>>());
/// ```
pub fn get_xpt_metadata(path: &Path) -> crate::error::Result<DatasetMeta> {
    let file = File::open(path).map_err(|_| XptError::file_not_found(path))?;
    let reader = StreamingReader::new(file)?;
    Ok(reader.meta().clone())
}

/// Helper function to convert a streaming reader to a dataset.
fn reader_to_dataset<R: std::io::Read + std::io::Seek>(
    reader: &mut StreamingReader<R>,
) -> crate::error::Result<XptDataset> {
    let meta = reader.meta().clone();
    let observations = reader.read_all_observations()?;

    let mut dataset = XptDataset::new(&meta.name);
    dataset.label = meta.label;
    dataset.dataset_type = meta.dataset_type;
    dataset.columns = meta.columns;

    // Convert observations to rows
    for obs in observations {
        dataset.rows.push(obs.into_values());
    }

    Ok(dataset)
}

// Re-export commonly needed types
pub use crate::core::reader::DatasetMeta;

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests require actual XPT files.
    // These tests verify the API surface compiles correctly.

    #[test]
    fn test_read_result_methods() {
        let result = ReadResult {
            dataset: XptDataset::new("TEST"),
            version: XptVersion::V5,
            validation: ValidationResult::new(),
        };

        assert!(result.is_clean());
        assert!(result.is_valid());
    }
}
