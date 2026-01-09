//! High-level XPT file writing with validation support.
//!
//! This module provides an ergonomic API for writing XPT files with
//! integrated validation and metadata specification support.
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! use xportrs::io::{write_xpt_validated};
//! use xportrs::types::{XptDataset, XptColumn, XptVersion};
//!
//! let mut dataset = XptDataset::new("DM");
//! dataset.columns.push(XptColumn::character("USUBJID", 20));
//!
//! // Write with validation
//! let result = write_xpt_validated(Path::new("dm.xpt"), &dataset, XptVersion::V5).unwrap();
//! println!("Validation: {}", result.validation);
//! ```

use std::path::Path;

use crate::XptVersion;
use crate::core::writer;
use crate::error::{Result, ValidationResult, XptError};
use crate::spec::DatasetSpec;
use crate::types::{XptDataset, XptWriterOptions};
use crate::validation::Validator;

// Re-export core writer types
pub use crate::core::writer::{
    DatasetInfo, StreamingWriter, ValidatedWriter, XptWriter, XptWriterBuilder,
};

/// Result of writing an XPT file with validation.
#[derive(Debug)]
pub struct WriteResult {
    /// Number of rows written.
    pub rows_written: usize,
    /// Validation result (may have warnings).
    pub validation: ValidationResult,
}

impl WriteResult {
    /// Check if the write was fully clean (no warnings).
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.validation.is_fda_compliant()
    }

    /// Check if the write was valid (no errors).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.validation.is_valid()
    }
}

/// Write a dataset to an XPT file with custom options.
///
/// # Errors
///
/// Returns an error if the file cannot be created or written.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use xportrs::io::write_xpt_with_options;
/// use xportrs::types::{XptDataset, XptColumn, XptWriterOptions, XptVersion};
///
/// let mut dataset = XptDataset::new("DM");
/// dataset.columns.push(XptColumn::character("USUBJID", 20));
///
/// let options = XptWriterOptions::new().with_version(XptVersion::V8);
/// write_xpt_with_options(Path::new("dm.xpt"), &dataset, &options).unwrap();
/// ```
pub fn write_xpt_with_options(
    path: &Path,
    dataset: &XptDataset,
    options: &XptWriterOptions,
) -> Result<()> {
    writer::write_xpt_with_options(path, dataset, options)
}

/// Write a dataset with automatic validation.
///
/// This function validates the dataset before writing and returns
/// both the write result and any validation issues.
///
/// # Errors
///
/// Returns an error if validation fails with errors (not just warnings),
/// or if the file cannot be written.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use xportrs::io::write_xpt_validated;
/// use xportrs::types::{XptDataset, XptColumn, XptVersion};
///
/// let mut dataset = XptDataset::new("DM");
/// dataset.columns.push(XptColumn::character("USUBJID", 20));
///
/// let result = write_xpt_validated(Path::new("dm.xpt"), &dataset, XptVersion::V5).unwrap();
/// if result.validation.has_issues() {
///     for warning in &result.validation.warnings {
///         eprintln!("Warning: {}", warning);
///     }
/// }
/// ```
pub fn write_xpt_validated(
    path: &Path,
    dataset: &XptDataset,
    version: XptVersion,
) -> Result<WriteResult> {
    let validator = Validator::basic(version);
    let validation = validator.validate(dataset);

    // Fail on errors, allow warnings
    if !validation.is_valid() {
        return Err(XptError::Validation(validation.errors.clone()));
    }

    let options = XptWriterOptions::new().with_version(version);
    XptWriter::create_with_options(path, options)?.write_dataset(dataset)?;

    Ok(WriteResult {
        rows_written: dataset.num_rows(),
        validation,
    })
}

/// Write a dataset with FDA compliance validation.
///
/// This is a convenience function that validates and writes a dataset
/// with strict FDA submission requirements.
///
/// # Errors
///
/// Returns an error if the dataset is not FDA compliant or cannot be written.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use xportrs::io::write_xpt_fda_compliant;
/// use xportrs::types::{XptDataset, XptColumn};
///
/// let mut dataset = XptDataset::new("DM");
/// dataset.columns.push(XptColumn::character("USUBJID", 20));
///
/// // Will fail if not FDA compliant
/// write_xpt_fda_compliant(Path::new("dm.xpt"), &dataset).unwrap();
/// ```
pub fn write_xpt_fda_compliant(path: &Path, dataset: &XptDataset) -> Result<WriteResult> {
    let validator = Validator::fda();
    let validation = validator.validate(dataset);

    // For FDA compliance, we fail on both errors AND warnings
    if !validation.is_fda_compliant() {
        let mut all_issues = validation.errors.clone();
        all_issues.extend(validation.warnings.clone());
        return Err(XptError::Validation(all_issues));
    }

    let options = XptWriterOptions::new().with_version(XptVersion::V5);
    XptWriter::create_with_options(path, options)?.write_dataset(dataset)?;

    Ok(WriteResult {
        rows_written: dataset.num_rows(),
        validation,
    })
}

/// Write a dataset validated against a specification.
///
/// This function validates the dataset against both XPT format rules
/// and the provided specification before writing.
///
/// # Errors
///
/// Returns an error if validation fails or the file cannot be written.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use xportrs::io::write_xpt_against_spec;
/// use xportrs::spec::{DatasetSpec, VariableSpec};
/// use xportrs::types::{XptDataset, XptColumn, XptVersion};
///
/// let spec = DatasetSpec::new("DM")
///     .add_variable(VariableSpec::character("USUBJID", 20));
///
/// let mut dataset = XptDataset::new("DM");
/// dataset.columns.push(XptColumn::character("USUBJID", 20));
///
/// let result = write_xpt_against_spec(
///     Path::new("dm.xpt"),
///     &dataset,
///     &spec,
///     XptVersion::V5,
/// ).unwrap();
/// ```
pub fn write_xpt_against_spec(
    path: &Path,
    dataset: &XptDataset,
    spec: &DatasetSpec,
    version: XptVersion,
) -> Result<WriteResult> {
    let validator = Validator::basic(version);
    let validation = validator.validate_against_spec(dataset, spec);

    // Fail on errors, allow warnings
    if !validation.is_valid() {
        return Err(XptError::Validation(validation.errors.clone()));
    }

    let options = XptWriterOptions::new().with_version(version);
    XptWriter::create_with_options(path, options)?.write_dataset(dataset)?;

    Ok(WriteResult {
        rows_written: dataset.num_rows(),
        validation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_result_methods() {
        let result = WriteResult {
            rows_written: 10,
            validation: ValidationResult::new(),
        };

        assert!(result.is_clean());
        assert!(result.is_valid());
        assert_eq!(result.rows_written, 10);
    }
}
