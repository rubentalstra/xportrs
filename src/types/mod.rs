//! Core types for XPT file handling.
//!
//! This module provides the fundamental data structures for representing
//! XPT datasets, columns, values, and file handling options.
//!
//! # Type Overview
//!
//! - [`XptValue`]: A cell value (numeric or character)
//! - [`NumericValue`]: A numeric value that may be missing
//! - [`MissingValue`]: One of 28 SAS missing value codes
//! - [`XptColumn`]: Column/variable definition (metadata)
//! - [`XptDataset`]: Dataset with columns and rows
//! - [`XptLibrary`]: Container for multiple datasets
//! - [`Observation`]: Single row for streaming operations
//! - [`FormatSpec`]: Output format specification

mod column;
mod dataset;
mod format;
mod missing;
mod observation;
mod options;
mod value;
mod version;

pub use column::{Justification, XptColumn, XptType};
pub use dataset::{RowLengthError, XptDataset, XptLibrary};
pub use format::{FormatSpec, InformatSpec};
pub use missing::MissingValue;
pub use observation::Observation;
pub use options::{XptReaderOptions, XptWriterOptions};
pub use value::{NumericValue, XptValue};
pub use version::XptVersion;
