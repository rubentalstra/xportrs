//! Validation for xportrs.
//!
//! This module provides validation logic for XPT files. Structural
//! checks for XPT v5 format are always applied. Agency-specific
//! validation is handled by the [`Agency`](crate::Agency) enum.

mod checks_v5;
mod issues;

pub(crate) use checks_v5::validate_v5_schema;
pub use issues::{Issue, IssueCollection, Severity, Target};
