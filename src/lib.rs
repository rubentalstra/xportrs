//! # xportrs
//!
//! Pure Rust SAS XPORT (XPT) reader and writer for CDISC clinical trial data submissions.
//!
//! `xportrs` provides a safe, DataFrame-agnostic implementation of XPT v5 I/O
//! with built-in regulatory compliance validation for FDA, PMDA, and NMPA submissions.
//!
//! ## Quick Start
//!
//! ### Reading an XPT file
//!
//! ```no_run
//! use xportrs::Xpt;
//!
//! // Read the first dataset from a file
//! let dataset = Xpt::read("ae.xpt")?;
//! println!("Domain: {}", dataset.domain_code());
//! println!("Rows: {}", dataset.nrows());
//!
//! // Read a specific member from a multi-dataset file
//! let dm = Xpt::reader("study.xpt")?.read_member("DM")?;
//! # Ok::<(), xportrs::Error>(())
//! ```
//!
//! ### Writing an XPT file
//!
//! ```no_run
//! use xportrs::{Xpt, Agency, Dataset, Column, ColumnData};
//!
//! let dataset = Dataset::new(
//!     "AE",  // Domain code (accepts &str, String, or DomainCode)
//!     vec![
//!         Column::new("USUBJID", ColumnData::String(vec![
//!             Some("01-001".into()),
//!             Some("01-002".into()),
//!         ])),
//!         Column::new("AESEQ", ColumnData::I64(vec![Some(1), Some(1)])),
//!     ],
//! )?;
//!
//! // Write with structural validation only
//! Xpt::writer(dataset.clone()).finalize()?.write_path("ae.xpt")?;
//!
//! // Write with FDA agency compliance validation
//! let mut builder = Xpt::writer(dataset);
//! builder.agency(Agency::FDA);
//! builder.finalize()?.write_path("ae_fda.xpt")?;
//! # Ok::<(), xportrs::Error>(())
//! ```
//!
//! ## Entry Points
//!
//! The [`Xpt`] struct provides all main functionality:
//!
//! - [`Xpt::read`] - Read a file in one line
//! - [`Xpt::reader`] - Read with options (member selection, etc.)
//! - [`Xpt::writer`] - Build a validated write plan
//! - [`Xpt::inspect`] - Examine file metadata without loading data
//!
//! ## Data Types
//!
//! - [`Dataset`] - Tabular data container with domain code and columns
//! - [`Column`] - Single variable with name and data
//! - [`ColumnData`] - Type-safe column values (numeric, character, date/time)
//! - [`DomainCode`] - Type-safe domain identifier (e.g., "AE", "DM", "LB")
//! - [`Label`] - Type-safe label string for datasets and variables
//!
//! ## Validation & Compliance
//!
//! - [`Agency`] - Regulatory agencies ([`Agency::FDA`], [`Agency::PMDA`], [`Agency::NMPA`])
//! - [`Issue`] - Validation problems with severity and context
//! - [`Severity`] - [`Severity::Error`] (blocking) or [`Severity::Warning`] (informational)
//!
//! When an agency is specified, the following rules are enforced:
//!
//! - ASCII-only names, labels, and character values
//! - Dataset names: max 8 bytes, uppercase alphanumeric
//! - Variable names: max 8 bytes, uppercase alphanumeric with underscores
//! - Labels: max 40 bytes
//! - Character values: max 200 bytes
//! - Automatic file splitting for files exceeding 5GB
//!
//! ## Feature Flags
//!
//! | Feature   | Description                                        |
//! |-----------|---------------------------------------------------|
//! | `serde`   | Serialization/deserialization support              |
//! | `tracing` | Structured logging with the `tracing` crate       |
//! | `polars`  | Polars `DataFrame` integration                     |
//! | `full`    | All optional features                              |
//!
//! ## CDISC Terminology
//!
//! This crate uses CDISC SDTM vocabulary:
//!
//! - **Domain dataset**: A table identified by a [`DomainCode`] (e.g., "AE", "DM", "LB")
//! - **Observation**: One row/record in the [`Dataset`]
//! - **Variable**: One [`Column`]; may have a [`VariableRole`]

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]

// Core modules
pub mod agency;
mod api;
pub mod config;
pub mod dataset;
mod error;
pub mod metadata;
mod schema;
pub mod validate;
mod write_plan;
pub mod xpt;

// Optional integrations
#[cfg(feature = "polars")]
pub mod polars;

// Main entry point - the unified API
pub use api::{Xpt, XptReaderBuilder};

// Agency for compliance validation
pub use agency::Agency;

// Configuration types users may need
pub use config::{TextMode, Verbosity};

// Dataset types - needed to construct data
pub use dataset::{
    Column, ColumnData, ColumnNames, Dataset, DomainCode, Format, FormatParseError, IntoIter, Iter,
    IterMut, Justification, Label, VariableName, VariableRole,
};

// Error types
pub use error::{Error, Result};

// Metadata types - for advanced usage
pub use metadata::XptVarType;

// Validation types
pub use validate::{Issue, Severity};

// Write plan types
pub use write_plan::{ValidatedWrite, XptWriterBuilder};

// XPT version enum
pub use xpt::XptVersion;

// XPT file info (for Xpt::inspect)
pub use xpt::v5::read::XptInfo;

/// Temporal conversion utilities.
///
/// These functions convert between Rust chrono types and SAS date/time values.
pub mod temporal {
    pub use crate::xpt::v5::timestamp::{
        date_from_sas_days, datetime_from_sas_seconds, sas_days_since_1960, sas_seconds_since_1960,
        sas_seconds_since_midnight, time_from_sas_seconds,
    };
}
