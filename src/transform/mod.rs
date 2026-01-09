//! Metadata-driven transform operations.
//!
//! This module provides functions equivalent to R's xportr package for applying
//! metadata specifications to datasets. Each transform function can operate
//! independently or be chained together in a pipeline.
//!
//! # Transform Functions
//!
//! | Function | R Equivalent | Description |
//! |----------|--------------|-------------|
//! | [`coerce_type`] | `xportr_type()` | Coerce column types to match spec |
//! | [`apply_length`] | `xportr_length()` | Apply variable lengths from spec |
//! | [`apply_label`] | `xportr_label()` | Apply variable labels from spec |
//! | [`apply_order`] | `xportr_order()` | Reorder columns to match spec |
//! | [`apply_format`] | `xportr_format()` | Apply SAS formats from spec |
//! | [`apply_df_label`] | `xportr_df_label()` | Set dataset label |
//!
//! # Example
//!
//! ```
//! use xportrs::{XptDataset, XptColumn, DatasetSpec, VariableSpec};
//! use xportrs::transform::{apply_label, ApplyLabelConfig};
//! use xportrs::ActionLevel;
//!
//! // Create a dataset
//! let dataset = XptDataset::with_columns("DM", vec![
//!     XptColumn::numeric("AGE"),
//!     XptColumn::character("SEX", 1),
//! ]);
//!
//! // Create a specification with labels
//! let spec = DatasetSpec::new("DM")
//!     .add_variable(VariableSpec::numeric("AGE").with_label("Age in Years"))
//!     .add_variable(VariableSpec::character("SEX", 1).with_label("Sex"));
//!
//! // Apply labels from spec
//! let config = ApplyLabelConfig::default();
//! let result = apply_label(dataset, &spec, config).unwrap();
//! assert_eq!(result.dataset.columns[0].label, Some("Age in Years".to_string()));
//! ```

mod apply_df_label;
mod apply_format;
mod apply_label;
mod apply_length;
mod apply_order;
mod coerce_type;
mod config;
mod pipeline;
mod report;

// Re-export transform functions
pub use apply_df_label::apply_df_label;
pub use apply_format::{apply_format, ApplyFormatConfig, ApplyFormatResult};
pub use apply_label::{apply_label, ApplyLabelConfig, ApplyLabelResult};
pub use apply_length::{apply_length, ApplyLengthConfig, ApplyLengthResult};
pub use apply_order::{apply_order, ApplyOrderConfig, ApplyOrderResult, UnmatchedPosition};
pub use coerce_type::{coerce_type, CoerceTypeConfig, CoerceTypeResult};

// Re-export pipeline functions
pub use pipeline::{xportr, xportr_write, PipelineReport, XportrConfig, XportrResult};

// Re-export configuration types
pub use config::{MismatchAction, TransformConfig};

// Re-export report types
pub use report::{
    FormatChange, LabelChange, LengthChange, OrderChange, TransformReport, TypeConversion,
};
