//! PMDA (Japan Pharmaceuticals and Medical Devices Agency) policy.
//!
//! The PMDA has requirements for clinical trial data submissions in Japan.
//! Key characteristics include:
//!
//! - Support for Japanese dataset names and labels
//! - Structure consistency requirements across datasets
//! - Alignment with CDISC standards with local adaptations

use crate::XptVersion;

use super::agency::{Agency, AgencyPolicy};
use super::rules::FileNamingRules;

/// PMDA maximum file size (5GB, per guidance).
pub const PMDA_MAX_FILE_SIZE: u64 = 5 * 1024 * 1024 * 1024;

/// PMDA policy for XPT file compliance.
///
/// The PMDA (Japan's regulatory agency for pharmaceuticals) has requirements
/// for clinical trial data submissions. PMDA follows CDISC standards with
/// some local adaptations for Japanese submissions.
///
/// # Features
///
/// - **Japanese support**: Allows UTF-8 for Japanese text in labels
/// - **Structure consistency**: Requires consistent structure across datasets
/// - **CDISC alignment**: Follows CDISC standards with local adaptations
///
/// # Example
///
/// ```
/// use xportrs::policy::{AgencyPolicy, PmdaPolicy};
/// use xportrs::XptVersion;
///
/// let policy = PmdaPolicy::default();
/// assert_eq!(policy.required_version(), Some(XptVersion::V5));
///
/// let strict = PmdaPolicy::strict();
/// assert!(strict.is_strict());
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PmdaPolicy {
    /// Whether to enforce strict validation.
    strict: bool,

    /// Whether to allow non-ASCII characters (Japanese text).
    allow_non_ascii: bool,
}

impl PmdaPolicy {
    /// Create a new PMDA policy in strict mode.
    ///
    /// Strict mode enforces all PMDA requirements as errors.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            strict: true,
            allow_non_ascii: true, // PMDA allows Japanese characters
        }
    }

    /// Create a new PMDA policy in lenient mode.
    ///
    /// Lenient mode treats some constraints as warnings.
    #[must_use]
    pub fn lenient() -> Self {
        Self {
            strict: false,
            allow_non_ascii: true,
        }
    }

    /// Create a new PMDA policy with specified options.
    #[must_use]
    pub fn new(strict: bool, allow_non_ascii: bool) -> Self {
        Self {
            strict,
            allow_non_ascii,
        }
    }

    /// Set whether non-ASCII characters are allowed.
    ///
    /// PMDA allows Japanese characters in labels for local submissions.
    #[must_use]
    pub fn with_allow_non_ascii(mut self, allow: bool) -> Self {
        self.allow_non_ascii = allow;
        self
    }

    /// Check if non-ASCII characters are allowed.
    #[must_use]
    pub fn allows_non_ascii(&self) -> bool {
        self.allow_non_ascii
    }
}

impl Default for PmdaPolicy {
    fn default() -> Self {
        Self {
            strict: false,
            allow_non_ascii: true, // Default allows Japanese text
        }
    }
}

impl AgencyPolicy for PmdaPolicy {
    fn agency(&self) -> Agency {
        Agency::Pmda
    }

    fn required_version(&self) -> Option<XptVersion> {
        // PMDA follows similar guidance to FDA on V5 format
        Some(XptVersion::V5)
    }

    fn max_file_size(&self) -> Option<u64> {
        Some(PMDA_MAX_FILE_SIZE)
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
        // CDISC convention
        false
    }

    fn require_ascii(&self) -> bool {
        // PMDA allows non-ASCII for Japanese text
        !self.allow_non_ascii
    }

    fn file_naming_rules(&self) -> FileNamingRules {
        FileNamingRules::pmda()
    }

    fn is_strict(&self) -> bool {
        self.strict
    }

    fn description(&self) -> String {
        let ascii_note = if self.allow_non_ascii {
            "Japanese text allowed"
        } else {
            "ASCII-only"
        };
        format!(
            "PMDA Policy ({}) - V5 required, {}, 5GB max",
            if self.strict { "strict" } else { "lenient" },
            ascii_note
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pmda_strict() {
        let policy = PmdaPolicy::strict();
        assert!(policy.is_strict());
        assert_eq!(policy.agency(), Agency::Pmda);
        assert!(policy.allows_non_ascii());
    }

    #[test]
    fn test_pmda_lenient() {
        let policy = PmdaPolicy::lenient();
        assert!(!policy.is_strict());
        assert!(policy.allows_non_ascii());
    }

    #[test]
    fn test_pmda_limits() {
        let policy = PmdaPolicy::default();
        assert_eq!(policy.max_variable_name_length(), 8);
        assert_eq!(policy.max_dataset_name_length(), 8);
        assert_eq!(policy.max_variable_label_length(), 40);
        assert_eq!(policy.max_dataset_label_length(), 40);
    }

    #[test]
    fn test_pmda_non_ascii_allowed() {
        let policy = PmdaPolicy::default();
        // By default, PMDA allows non-ASCII (Japanese text)
        assert!(!policy.require_ascii());

        // Can disable non-ASCII
        let ascii_only = PmdaPolicy::default().with_allow_non_ascii(false);
        assert!(ascii_only.require_ascii());
    }

    #[test]
    fn test_pmda_version() {
        let policy = PmdaPolicy::default();
        assert_eq!(policy.required_version(), Some(XptVersion::V5));
    }

    #[test]
    fn test_pmda_file_size() {
        let policy = PmdaPolicy::default();
        assert_eq!(policy.max_file_size(), Some(PMDA_MAX_FILE_SIZE));
    }

    #[test]
    fn test_pmda_description() {
        let policy = PmdaPolicy::strict();
        assert!(policy.description().contains("PMDA"));
        assert!(policy.description().contains("Japanese text allowed"));

        let ascii_only = PmdaPolicy::strict().with_allow_non_ascii(false);
        assert!(ascii_only.description().contains("ASCII-only"));
    }

    #[test]
    fn test_pmda_file_naming() {
        let policy = PmdaPolicy::default();
        let rules = policy.file_naming_rules();
        assert!(rules.is_valid("dm.xpt"));
    }
}
