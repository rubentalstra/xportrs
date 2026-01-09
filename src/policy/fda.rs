//! FDA (U.S. Food and Drug Administration) policy.
//!
//! The FDA requires XPT V5 format for regulatory submissions to CDER and CBER.
//! This policy enforces FDA-specific constraints including:
//!
//! - XPT V5 format required
//! - 8-character maximum for variable/dataset names
//! - 40-character maximum for labels
//! - ASCII-only string data
//! - 5GB maximum file size (guidance)
//! - Lowercase filenames matching dataset names

use crate::XptVersion;

use super::agency::{Agency, AgencyPolicy};
use super::rules::FileNamingRules;

/// FDA maximum file size (5GB, per guidance).
pub const FDA_MAX_FILE_SIZE: u64 = 5 * 1024 * 1024 * 1024;

/// FDA policy for XPT file compliance.
///
/// The FDA requires XPT V5 format for submissions to CDER (drugs) and CBER (biologics).
/// This policy can operate in strict or lenient mode:
///
/// - **Strict mode**: All constraints are enforced as errors
/// - **Lenient mode**: Some constraints generate warnings instead of errors
///
/// # Example
///
/// ```
/// use xportrs::policy::{AgencyPolicy, FdaPolicy};
/// use xportrs::XptVersion;
///
/// // Strict mode (recommended for final submissions)
/// let strict = FdaPolicy::strict();
/// assert_eq!(strict.required_version(), Some(XptVersion::V5));
/// assert!(strict.is_strict());
///
/// // Lenient mode (for development/testing)
/// let lenient = FdaPolicy::lenient();
/// assert!(!lenient.is_strict());
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FdaPolicy {
    /// Whether to enforce strict validation.
    strict: bool,
}

impl FdaPolicy {
    /// Create a new FDA policy in strict mode.
    ///
    /// Strict mode enforces all FDA requirements as errors.
    /// This is recommended for final submission packages.
    #[must_use]
    pub fn strict() -> Self {
        Self { strict: true }
    }

    /// Create a new FDA policy in lenient mode.
    ///
    /// Lenient mode treats some constraints as warnings instead of errors.
    /// This is useful during development and testing.
    #[must_use]
    pub fn lenient() -> Self {
        Self { strict: false }
    }

    /// Create a new FDA policy with specified strictness.
    #[must_use]
    pub fn new(strict: bool) -> Self {
        Self { strict }
    }
}

impl Default for FdaPolicy {
    fn default() -> Self {
        Self::strict()
    }
}

impl AgencyPolicy for FdaPolicy {
    fn agency(&self) -> Agency {
        Agency::Fda
    }

    fn required_version(&self) -> Option<XptVersion> {
        // FDA requires V5 format
        Some(XptVersion::V5)
    }

    fn max_file_size(&self) -> Option<u64> {
        // FDA guidance recommends 5GB max
        Some(FDA_MAX_FILE_SIZE)
    }

    fn max_variable_name_length(&self) -> usize {
        // V5 format limit
        8
    }

    fn max_dataset_name_length(&self) -> usize {
        // V5 format limit
        8
    }

    fn max_variable_label_length(&self) -> usize {
        // V5 format limit
        40
    }

    fn max_dataset_label_length(&self) -> usize {
        // V5 format limit
        40
    }

    fn require_uppercase_names(&self) -> bool {
        // FDA recommends but does not require uppercase
        // SAS convention is uppercase for CDISC datasets
        false
    }

    fn require_ascii(&self) -> bool {
        // FDA strongly recommends ASCII-only
        // Non-ASCII can cause issues with SAS transcoding
        true
    }

    fn file_naming_rules(&self) -> FileNamingRules {
        FileNamingRules::fda()
    }

    fn is_strict(&self) -> bool {
        self.strict
    }

    fn description(&self) -> String {
        format!(
            "FDA Policy ({}) - V5 required, 8-char names, 40-char labels, ASCII-only, 5GB max",
            if self.strict { "strict" } else { "lenient" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fda_strict_defaults() {
        let policy = FdaPolicy::strict();
        assert!(policy.is_strict());
        assert_eq!(policy.agency(), Agency::Fda);
        assert_eq!(policy.required_version(), Some(XptVersion::V5));
    }

    #[test]
    fn test_fda_lenient_defaults() {
        let policy = FdaPolicy::lenient();
        assert!(!policy.is_strict());
        assert_eq!(policy.agency(), Agency::Fda);
        assert_eq!(policy.required_version(), Some(XptVersion::V5));
    }

    #[test]
    fn test_fda_limits() {
        let policy = FdaPolicy::strict();
        assert_eq!(policy.max_variable_name_length(), 8);
        assert_eq!(policy.max_dataset_name_length(), 8);
        assert_eq!(policy.max_variable_label_length(), 40);
        assert_eq!(policy.max_dataset_label_length(), 40);
    }

    #[test]
    fn test_fda_file_size() {
        let policy = FdaPolicy::strict();
        assert_eq!(policy.max_file_size(), Some(FDA_MAX_FILE_SIZE));
        assert_eq!(policy.max_file_size(), Some(5 * 1024 * 1024 * 1024));
    }

    #[test]
    fn test_fda_ascii_required() {
        let policy = FdaPolicy::strict();
        assert!(policy.require_ascii());
    }

    #[test]
    fn test_fda_file_naming() {
        let policy = FdaPolicy::strict();
        let rules = policy.file_naming_rules();
        assert!(rules.is_valid("dm.xpt"));
        assert!(!rules.is_valid("DM.xpt")); // Not lowercase
        assert!(!rules.is_valid("demographics.xpt")); // Too long
    }

    #[test]
    fn test_fda_description() {
        let strict = FdaPolicy::strict();
        assert!(strict.description().contains("FDA"));
        assert!(strict.description().contains("strict"));

        let lenient = FdaPolicy::lenient();
        assert!(lenient.description().contains("lenient"));
    }

    #[test]
    fn test_fda_default() {
        let policy = FdaPolicy::default();
        assert!(policy.is_strict()); // Default is strict
    }
}
