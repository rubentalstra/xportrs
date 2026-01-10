//! Metadata types for xportrs.
//!
//! This module provides metadata structures that describe datasets and variables
//! for XPT file generation. Metadata is optional but improves determinism and
//! compliance.

mod dataset;
mod variable;

pub(crate) use dataset::DatasetMetadata;
pub(crate) use variable::VariableMetadata;
pub use variable::XptVarType;
