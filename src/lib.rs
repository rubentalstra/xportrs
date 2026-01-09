//! CDISC-compliant XPT file generation, inspired by R's xportr package.
//!
//! This crate provides functionality to read and write SAS Transport V5 and V8 format files,
//! commonly used for SDTM datasets in regulatory submissions. It also includes a metadata-driven
//! transformation system inspired by the R xportr package.
//!
//! # Features
//!
//! - Full SAS Transport V5 format support (default, for maximum compatibility)
//! - SAS Transport V8 format support (longer names, labels, and formats)
//! - IEEE to IBM mainframe floating-point conversion
//! - Support for all 28 SAS missing value codes (`.`, `._`, `.A`-`.Z`)
//! - Variable metadata including formats and informats
//! - Metadata specification types for xportr-style workflows
//! - Optional Polars DataFrame integration (with `polars` feature)
//!
//! # Format Versions
//!
//! | Feature | V5 Limit | V8 Limit |
//! |---------|----------|----------|
//! | Variable name | 8 chars | 32 chars |
//! | Variable label | 40 chars | 256 chars |
//! | Format name | 8 chars | 32 chars |
//! | Dataset name | 8 chars | 32 chars |
//!
//! By default, files are written in V5 format. Use [`XptWriterOptions::with_version`]
//! to write V8 format files. When reading, the format is auto-detected.
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! use xportrs::{XptDataset, XptColumn, XptValue, read_xpt, write_xpt};
//!
//! // Read an XPT file
//! let dataset = read_xpt(Path::new("dm.xpt")).unwrap();
//! println!("Dataset: {} ({} rows)", dataset.name, dataset.num_rows());
//!
//! // Create a new dataset
//! let mut ds = XptDataset::with_columns(
//!     "DM",
//!     vec![
//!         XptColumn::character("USUBJID", 20).with_label("Unique Subject ID"),
//!         XptColumn::numeric("AGE").with_label("Age in Years"),
//!     ],
//! );
//! ds.add_row(vec![
//!     XptValue::character("STUDY-001"),
//!     XptValue::numeric(35.0),
//! ]);
//!
//! // Write to XPT file
//! write_xpt(Path::new("dm_out.xpt"), &ds).unwrap();
//! ```
//!
//! # Missing Values
//!
//! SAS supports 28 different missing value codes:
//!
//! ```
//! use xportrs::{MissingValue, NumericValue, XptValue};
//!
//! // Standard missing (.)
//! let missing = XptValue::numeric_missing();
//!
//! // Special missing (.A through .Z)
//! let missing_a = XptValue::numeric_missing_with(MissingValue::Special('A'));
//!
//! // Check for missing
//! assert!(missing.is_missing());
//! ```
//!
//! # Metadata Specification (xportr-style)
//!
//! ```
//! use xportrs::spec::{DatasetSpec, VariableSpec};
//! use xportrs::FormatSpec;
//!
//! let spec = DatasetSpec::new("DM")
//!     .with_label("Demographics")
//!     .add_variable(
//!         VariableSpec::character("USUBJID", 20)
//!             .with_label("Unique Subject Identifier")
//!             .with_order(1)
//!     )
//!     .add_variable(
//!         VariableSpec::numeric("AGE")
//!             .with_label("Age")
//!             .with_format(FormatSpec::best(8))
//!             .with_order(2)
//!     );
//! ```

mod error;
pub mod float;
pub mod header;
pub mod reader;
pub mod spec;
mod types;
pub mod validation;
mod version;
pub mod writer;

#[cfg(feature = "polars")]
pub mod polars;

// Re-export error types
pub use error::{
    ErrorLocation, Result, Severity, SpecError, SpecResult, TransformError, TransformResult,
    ValidationError, ValidationErrorCode, ValidationResult, XptError,
};

// Re-export version
pub use version::XptVersion;

// Re-export core types
pub use types::{
    FormatSpec, InformatSpec, Justification, MissingValue, NumericValue, Observation,
    RowLengthError, XptColumn, XptDataset, XptLibrary, XptReaderOptions, XptType, XptValue,
    XptWriterOptions,
};

// Re-export spec types at top level for convenience
pub use spec::{Core, DatasetSpec, VariableSpec};

// Re-export validation types
pub use validation::ActionLevel;

// Re-export reader functionality
pub use reader::{
    read_xpt, read_xpt_streaming, read_xpt_streaming_with_options, read_xpt_with_options,
    DatasetMeta, ObservationIter, StreamingReader, XptReader,
};

// Re-export writer functionality
pub use writer::{
    write_xpt, write_xpt_with_options, DatasetInfo, StreamingWriter, ValidatedWriter, XptWriter,
    XptWriterBuilder,
};

// Re-export Polars integration
#[cfg(feature = "polars")]
pub use polars::{read_xpt_to_dataframe, write_dataframe_to_xpt};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
