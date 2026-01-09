//! # xportrs
//!
//! Pure Rust SAS XPORT (XPT) reader and writer for CDISC domain datasets.
//!
//! `xportrs` provides a safe, DataFrame-agnostic implementation of XPT v5 I/O
//! and compliance tooling for clinical trial data submissions.
//!
//! ## Features
//!
//! - **Pure Rust**: No unsafe code (`#![forbid(unsafe_code)]`)
//! - **DataFrame-agnostic**: Works with any in-memory table representation
//! - **CDISC-compliant**: Built-in compliance profiles for FDA, NMPA, and PMDA
//! - **XPT v5**: Full read and write support
//! - **XPT v8**: API-ready (not yet implemented)
//!
//! ## Quick Start
//!
//! ### Reading an XPT file
//!
//! ```no_run
//! use xportrs::Xpt;
//!
//! // Simple: read first dataset
//! let dataset = Xpt::read("ae.xpt")?;
//! println!("Domain: {}", dataset.domain_code);
//! println!("Rows: {}", dataset.nrows);
//!
//! // With options or specific member
//! let dm = Xpt::reader("study.xpt")?.read_member("DM")?;
//! # Ok::<(), xportrs::XportrsError>(())
//! ```
//!
//! ### Writing an XPT file
//!
//! ```no_run
//! use xportrs::{Xpt, DomainDataset, Column, ColumnData};
//!
//! let dataset = DomainDataset::new(
//!     "AE".to_string(),
//!     vec![
//!         Column::new("USUBJID", ColumnData::String(vec![
//!             Some("01-001".into()),
//!             Some("01-002".into()),
//!         ])),
//!         Column::new("AESEQ", ColumnData::I64(vec![Some(1), Some(1)])),
//!     ],
//! )?;
//!
//! Xpt::writer(dataset)
//!     .finalize()?
//!     .write_path("ae.xpt")?;
//! # Ok::<(), xportrs::XportrsError>(())
//! ```
//!
//! ## Modules
//!
//! - [`config`]: Configuration options for reading and writing
//! - [`dataset`]: Core data structures (`DomainDataset`, `Column`, `ColumnData`)
//! - [`metadata`]: Variable and dataset metadata
//! - [`profile`]: Compliance profiles (FDA, NMPA, PMDA)
//! - [`schema`]: Schema planning for XPT generation
//! - [`validate`]: Validation logic and issue reporting
//! - [`xpt`]: XPT format implementation details
//!
//! ## CDISC Terminology
//!
//! This crate uses CDISC SDTM vocabulary:
//!
//! - **Domain dataset**: A table identified by a domain code (e.g., "AE", "DM", "LB")
//! - **Observation**: One row/record in the dataset
//! - **Variable**: One column; may have a role (Identifier/Topic/Timing/Qualifier/Rule)
//!
//! ## Safety
//!
//! This crate is built with `#![forbid(unsafe_code)]`. All binary parsing and
//! encoding uses safe Rust constructs.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]

// Core modules
mod api;
pub mod config;
pub mod dataset;
mod error;
pub mod metadata;
pub mod profile;
pub mod schema;
pub mod validate;
mod write_plan;
pub mod xpt;

// Main entry point - the unified API
pub use api::{Xpt, XptReaderBuilder};

// Configuration types users may need
pub use config::{Config, ReadOptions, TextMode, Verbosity, WriteOptions};

// Dataset types - needed to construct data
pub use dataset::{Column, ColumnData, DomainDataset, VariableRole};

// Error types
pub use error::{Result, XportrsError};

// Metadata types - for advanced usage
pub use metadata::{DatasetMetadata, VariableMetadata, XptVarType};

// Schema types - for advanced usage
pub use schema::{PlannedVariable, SchemaPlan};

// Validation types
pub use validate::{Issue, Severity, Target};

// Write plan types
pub use write_plan::{FinalizedWritePlan, XptWritePlan};

// XPT version enum
pub use xpt::XptVersion;

// Compliance profile types
pub use profile::{ComplianceProfile, Rule};

// XPT file info (for Xpt::inspect)
pub use xpt::v5::read::XptFile;

/// Temporal conversion utilities.
///
/// These functions convert between Rust chrono types and SAS date/time values.
pub mod temporal {
    pub use crate::xpt::v5::timestamp::{
        date_from_sas_days, datetime_from_sas_seconds, sas_days_since_1960, sas_seconds_since_1960,
        sas_seconds_since_midnight, time_from_sas_seconds,
    };
}
