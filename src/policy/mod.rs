//! Agency policy layer.
//!
//! This module provides agency-specific validation policies for XPT files.
//! Different regulatory agencies (FDA, NMPA, PMDA) have different requirements
//! for clinical trial data submissions.
//!
//! # Built-in Policies
//!
//! | Policy | Agency | Description |
//! |--------|--------|-------------|
//! | [`FdaPolicy`] | FDA (US) | V5 required, 8-char names, ASCII-only, 5GB max |
//! | [`NmpaPolicy`] | NMPA (China) | V5, allows Chinese text for bilingual datasets |
//! | [`PmdaPolicy`] | PMDA (Japan) | V5, allows Japanese text |
//! | [`CustomPolicy`] | Custom | User-defined constraints |
//!
//! # Example
//!
//! ```
//! use xportrs::policy::{AgencyPolicy, FdaPolicy, NmpaPolicy, CustomPolicy};
//! use xportrs::XptVersion;
//!
//! // FDA strict mode (for final submissions)
//! let fda = FdaPolicy::strict();
//! assert_eq!(fda.required_version(), Some(XptVersion::V5));
//! assert!(fda.require_ascii());
//!
//! // NMPA allows Chinese text
//! let nmpa = NmpaPolicy::default();
//! assert!(!nmpa.require_ascii());
//!
//! // Custom policy for V8 format
//! let custom = CustomPolicy::new()
//!     .with_required_version(XptVersion::V8)
//!     .with_max_variable_name_length(32);
//! assert_eq!(custom.max_variable_name_length(), 32);
//! ```
//!
//! # Strict vs Lenient Mode
//!
//! Each built-in policy supports strict and lenient modes:
//!
//! - **Strict mode**: All violations are treated as errors
//! - **Lenient mode**: Some violations may be treated as warnings
//!
//! ```
//! use xportrs::policy::{AgencyPolicy, FdaPolicy};
//!
//! // Strict mode for final submission
//! let strict = FdaPolicy::strict();
//! assert!(strict.is_strict());
//!
//! // Lenient mode for development
//! let lenient = FdaPolicy::lenient();
//! assert!(!lenient.is_strict());
//! ```

mod agency;
mod custom;
mod fda;
mod nmpa;
mod pmda;
mod rules;

// Re-export agency types
pub use agency::{Agency, AgencyPolicy};

// Re-export policy implementations
pub use custom::CustomPolicy;
pub use fda::{FdaPolicy, FDA_MAX_FILE_SIZE};
pub use nmpa::{NmpaPolicy, NMPA_MAX_FILE_SIZE};
pub use pmda::{PmdaPolicy, PMDA_MAX_FILE_SIZE};

// Re-export rules
pub use rules::{FileNamingIssue, FileNamingRules};
