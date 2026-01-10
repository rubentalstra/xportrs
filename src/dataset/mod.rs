//! Dataset types for xportrs.
//!
//! This module defines the core data structures used to represent CDISC domain
//! datasets in memory. The design is columnar and DataFrame-agnostic.

mod domain_dataset;
mod newtypes;

pub use domain_dataset::{Column, ColumnData, Dataset, VariableRole};
pub use newtypes::{DomainCode, Label};
