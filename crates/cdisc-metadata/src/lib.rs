//! CDISC metadata parser for SDTM, SEND, and ADaM standards.
//!
//! This crate provides parsers for CDISC Implementation Guide metadata files:
//! - SDTM-IG (Study Data Tabulation Model)
//! - SEND-IG (Standard for Exchange of Nonclinical Data)
//! - ADaM-IG (Analysis Data Model)

mod adam;
mod error;
mod loader;
mod sdtm;
mod send;
mod types;

pub use error::{Error, Result};
pub use loader::load_standard;
pub use types::{DatasetDef, Standard, VarType, Variable};

// Re-export standard-specific loaders for direct access
pub use adam::load_adam;
pub use sdtm::load_sdtm;
pub use send::load_send;
