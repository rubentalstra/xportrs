//! Schema planning for xportrs.
//!
//! This module handles the derivation of XPT transport schemas from datasets
//! and metadata. The schema plan defines the exact byte layout for XPT files.

mod derive;
pub mod plan;

pub use derive::derive_schema_plan;
pub use plan::{PlannedVariable, SchemaPlan};
