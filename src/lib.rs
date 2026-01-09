//! CDISC-compliant XPT file generation, inspired by R's xportr package.
//!
//! `xportrs` provides a simple, DataFrame-first API for reading and writing SAS Transport
//! (XPT) V5/V8 files with metadata-driven transformations.
//!
//! # Quick Start
//!
//! ```no_run
//! use xportrs::*;
//!
//! // Read XPT file
//! let df = xportrs_read("input.xpt").unwrap();
//!
//! // Create specification
//! let spec = DatasetSpec::new("DM")
//!     .with_label("Demographics")
//!     .add_variable(VariableSpec::character("USUBJID", 20).with_label("Subject ID"))
//!     .add_variable(VariableSpec::numeric("AGE").with_label("Age"));
//!
//! // Write with FDA-compliant settings
//! let report = xportrs(df, "output.xpt", &spec, XportrsConfig::fda()).unwrap();
//! println!("{}", report.summary());
//! ```
//!
//! # Primary API
//!
//! The main API consists of DataFrame-based functions matching R's xportr design:
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`xportrs_read`] | Read XPT file into `DataFrame` |
//! | [`xportrs_write`] | Write `DataFrame` to XPT file |
//! | [`xportrs_type`] | Coerce column types to match spec |
//! | [`xportrs_length`] | Apply variable lengths from spec |
//! | [`xportrs_label`] | Apply variable labels from spec |
//! | [`xportrs_format`] | Apply SAS formats from spec |
//! | [`xportrs_order`] | Reorder columns to match spec |
//! | [`xportrs`] | All-in-one pipeline (transform + write) |
//! | [`xportrs_validate`] | Validate `DataFrame` against policy |
//!
//! # Configuration
//!
//! Use [`XportrsConfig`] to control behavior:
//!
//! ```
//! use xportrs::{XportrsConfig, ActionLevel, XptVersion};
//!
//! // FDA-compliant (V5, strict validation)
//! let config = XportrsConfig::fda();
//!
//! // Custom
//! let config = XportrsConfig::default()
//!     .with_version(XptVersion::V5)
//!     .with_type_action(ActionLevel::Stop);
//! ```
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
//! V5 is the default (required for FDA submissions). V8 allows longer names and labels.

// Internal modules
mod api;
mod config;
mod report;

// Public modules
pub mod core;
pub mod error;
pub mod io;
pub mod polars;
pub mod policy;
pub mod spec;
pub mod transform;
pub mod types;
pub mod validation;

// Primary API
pub use api::{
    xportrs, xportrs_df_label, xportrs_format, xportrs_label, xportrs_length, xportrs_metadata,
    xportrs_order, xportrs_read, xportrs_type, xportrs_validate, xportrs_write,
};
pub use config::{ActionLevel, XportrsConfig, XptVersion};
pub use error::{Result, ValidationError, ValidationResult, XptError};
pub use policy::{AgencyPolicy, FdaPolicy, NmpaPolicy, PmdaPolicy};
pub use report::{
    FormatChange, FormatReport, LabelChange, LabelReport, LengthChange, LengthReport, OrderChange,
    OrderReport, TypeConversion, TypeReport, WriteReport, XportrsReport,
};
pub use spec::{DatasetSpec, VariableSpec};

pub use ::polars::prelude::DataFrame;

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
