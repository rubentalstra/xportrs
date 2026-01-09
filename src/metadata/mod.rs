//! Metadata types for xportrs.
//!
//! This module provides metadata structures that describe datasets and variables
//! for XPT file generation. Metadata is optional but improves determinism and
//! compliance.

mod dataset;
mod variable;

pub use dataset::DatasetMetadata;
pub use variable::{VariableMetadata, XptVarType};
