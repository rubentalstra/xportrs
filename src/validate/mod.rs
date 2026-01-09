//! Validation for xportrs.
//!
//! This module provides validation logic for XPT files, including structural
//! checks for XPT v5 format and compliance profile checks.

mod checks_profile;
mod checks_v5;
mod issues;

pub use checks_profile::validate_profile;
pub use checks_v5::validate_v5_schema;
pub use issues::{Issue, IssueCollection, Severity, Target};
