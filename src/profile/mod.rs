//! Compliance profiles for xportrs.
//!
//! This module provides compliance profiles that define validation rules
//! for XPT files. Built-in presets are available for FDA, NMPA, and PMDA
//! requirements.

pub mod presets;
mod rules;

pub use presets::{
    ComplianceProfileBuilder, FDA_PROFILE, NMPA_PROFILE, PMDA_PROFILE, custom_profile,
};
pub use rules::{ComplianceProfile, Rule};
