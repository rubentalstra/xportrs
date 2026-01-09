//! Optional Polars `DataFrame` integration.
//!
//! This module provides conversion between XPT datasets and Polars `DataFrames`,
//! along with xportr-style transform methods for `DataFrame` pipelines.
//! Enable with the `polars` feature.
//!
//! # Reading XPT to `DataFrame`
//!
//! ```no_run
//! use std::path::Path;
//! use xportrs::read_xpt_to_dataframe;
//!
//! let df = read_xpt_to_dataframe(Path::new("dm.xpt")).unwrap();
//! println!("{}", df);
//! ```
//!
//! # Writing `DataFrame` to XPT
//!
//! ```no_run
//! use std::path::Path;
//! use polars::prelude::*;
//! use xportrs::write_dataframe_to_xpt;
//!
//! let df = df! {
//!     "USUBJID" => &["001", "002", "003"],
//!     "AGE" => &[25i64, 30, 35],
//! }.unwrap();
//!
//! write_dataframe_to_xpt(Path::new("out.xpt"), &df, "DM").unwrap();
//! ```
//!
//! # xportr-style Pipeline
//!
//! ```no_run
//! use polars::prelude::*;
//! use xportrs::polars::XportrTransforms;
//! use xportrs::spec::{DatasetSpec, VariableSpec};
//! use xportrs::ActionLevel;
//!
//! let spec = DatasetSpec::new("DM")
//!     .add_variable(VariableSpec::numeric("AGE").with_label("Age"))
//!     .add_variable(VariableSpec::character("SEX", 1).with_label("Sex"));
//!
//! let df = df! {
//!     "AGE" => &[25i64, 30],
//!     "SEX" => &["M", "F"],
//! }.unwrap();
//!
//! // Apply transforms using method chaining
//! let result = df
//!     .xportr_metadata(spec.clone())
//!     .xportr_label(&spec, ActionLevel::Warn).unwrap()
//!     .xportr_df_label("Demographics");
//!
//! // Access the DataFrame
//! assert_eq!(result.df().height(), 2);
//! ```

mod conversion;
mod metadata;
mod pipeline;
mod transforms;

use std::path::Path;

use polars::prelude::DataFrame;

use crate::core::reader::read_xpt;
use crate::core::writer::write_xpt_with_options;
use crate::error::Result;
use crate::types::{XptDataset, XptWriterOptions};

// Re-export MetadataFrame
pub use metadata::MetadataFrame;

// Re-export XportrTransforms trait
pub use transforms::XportrTransforms;

// Re-export pipeline functions
pub use pipeline::{write_df_fda_compliant, write_df_with_pipeline};

/// Read an XPT file directly to a Polars `DataFrame`.
///
/// # Arguments
/// * `path` - Path to the XPT file
///
/// # Returns
/// A Polars `DataFrame` containing the XPT data.
///
/// # Example
/// ```no_run
/// use std::path::Path;
/// use xportrs::read_xpt_to_dataframe;
///
/// let df = read_xpt_to_dataframe(Path::new("dm.xpt")).unwrap();
/// println!("{}", df);
/// ```
///
/// # Errors
///
/// Returns an error if the file cannot be read or converted to a `DataFrame`.
pub fn read_xpt_to_dataframe(path: &Path) -> Result<DataFrame> {
    let dataset = read_xpt(path)?;
    dataset.to_dataframe()
}

/// Write a Polars `DataFrame` to an XPT file.
///
/// # Arguments
/// * `path` - Output path for the XPT file
/// * `df` - The `DataFrame` to write
/// * `name` - Dataset name (1-8 characters)
///
/// # Returns
/// Ok(()) on success.
///
/// # Example
/// ```no_run
/// use std::path::Path;
/// use polars::prelude::*;
/// use xportrs::write_dataframe_to_xpt;
///
/// let df = df! {
///     "USUBJID" => &["001", "002", "003"],
///     "AGE" => &[25i64, 30, 35],
/// }.unwrap();
///
/// write_dataframe_to_xpt(Path::new("out.xpt"), &df, "DM").unwrap();
/// ```
///
/// # Errors
///
/// Returns an error if the `DataFrame` cannot be converted or written to the file.
pub fn write_dataframe_to_xpt(path: &Path, df: &DataFrame, name: &str) -> Result<()> {
    let dataset = XptDataset::from_dataframe(df, name)?;
    write_xpt_with_options(path, &dataset, &XptWriterOptions::default())
}
