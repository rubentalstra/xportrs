//! Metadata specification types for XPT datasets.
//!
//! This module provides types for defining variable and dataset specifications
//! that can be used to transform and validate data before writing to XPT format.
//!
//! The specification types are inspired by the R xportr package's metadata-driven
//! approach to CDISC-compliant data transformation.
//!
//! # Example
//!
//! ```
//! use xportrs::spec::{DatasetSpec, VariableSpec};
//! use xportrs::XptType;
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
//!             .with_order(2)
//!     );
//!
//! assert_eq!(spec.name, "DM");
//! assert_eq!(spec.variables.len(), 2);
//! ```
//!
//! # Loading Specs from DataFrames
//!
//! With the `polars` feature enabled, you can load specifications from DataFrames:
//!
//! ```ignore
//! use polars::prelude::*;
//! use xportrs::spec::{DataFrameMetadataSource, MetadataSource, ColumnMapping};
//!
//! // Load spec from CSV via Polars
//! let spec_df = CsvReadOptions::default()
//!     .try_into_reader_with_file_path(Some("specs/var_spec.csv".into()))?
//!     .finish()?;
//!
//! // Create metadata source with default column names
//! let source = DataFrameMetadataSource::new(spec_df);
//! let dm_spec = source.load_dataset_spec("DM")?;
//!
//! // Or with custom column mapping for different naming conventions
//! let source = DataFrameMetadataSource::new(spec_df)
//!     .with_mapping(ColumnMapping::pinnacle21());
//! ```

mod dataset;
mod mapping;
mod source;
mod variable;

pub use dataset::DatasetSpec;
pub use mapping::ColumnMapping;
pub use source::MetadataSource;
pub use variable::{Core, VariableSpec};

#[cfg(feature = "polars")]
pub use source::DataFrameMetadataSource;
