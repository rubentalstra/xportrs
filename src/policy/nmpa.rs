//! NMPA (China National Medical Products Administration) policy.
//!
//! The NMPA has specific requirements for clinical trial data submissions
//! in China. Key characteristics include:
//!
//! - Support for bilingual datasets (Chinese and English)
//! - 5GB file size guidance (similar to FDA)
//! - Structure consistency requirements
//! - Same basic format constraints as FDA for CDISC compliance

use crate::XptVersion;

use super::agency::{Agency, AgencyPolicy};
use super::rules::FileNamingRules;

/// NMPA maximum file size (5GB, per guidance).
pub const NMPA_MAX_FILE_SIZE: u64 = 5 * 1024 * 1024 * 1024;

/// NMPA policy for XPT file compliance.
///
/// The NMPA (China's FDA equivalent) has requirements for clinical trial
/// data submissions. While many requirements align with FDA, NMPA has
/// specific needs around bilingual support and structure consistency.
///
/// # Features
///
/// - **Bilingual support**: Allows UTF-8 for Chinese text in labels
/// - **Structure consistency**: Requires identical structure across datasets
/// - **Format alignment**: Follows CDISC standards with local adaptations
///
/// # Example
///
/// ```
/// use xportrs::policy::{AgencyPolicy, NmpaPolicy};
/// use xportrs::XptVersion;
///
/// // Standard mode (recommended)
/// let policy = NmpaPolicy::default();
/// assert_eq!(policy.required_version(), Some(XptVersion::V5));
///
/// // Strict mode for final submissions
/// let strict = NmpaPolicy::strict();
/// assert!(strict.is_strict());
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NmpaPolicy {
    /// Whether to enforce strict validation.
    strict: bool,

    /// Whether to allow non-ASCII characters (Chinese text).
    allow_non_ascii: bool,
}

impl NmpaPolicy {
    /// Create a new NMPA policy in strict mode.
    ///
    /// Strict mode enforces all NMPA requirements as errors.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            strict: true,
            allow_non_ascii: true, // NMPA allows Chinese characters
        }
    }

    /// Create a new NMPA policy in lenient mode.
    ///
    /// Lenient mode treats some constraints as warnings.
    #[must_use]
    pub fn lenient() -> Self {
        Self {
            strict: false,
            allow_non_ascii: true,
        }
    }

    /// Create a new NMPA policy with specified options.
    #[must_use]
    pub fn new(strict: bool, allow_non_ascii: bool) -> Self {
        Self {
            strict,
            allow_non_ascii,
        }
    }

    /// Set whether non-ASCII characters are allowed.
    ///
    /// NMPA allows Chinese characters in labels for bilingual datasets.
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

impl Default for NmpaPolicy {
    fn default() -> Self {
        Self {
            strict: false,
            allow_non_ascii: true, // Default allows Chinese text
        }
    }
}

impl AgencyPolicy for NmpaPolicy {
    fn agency(&self) -> Agency {
        Agency::Nmpa
    }

    fn required_version(&self) -> Option<XptVersion> {
        // NMPA follows FDA guidance on V5 format
        Some(XptVersion::V5)
    }

    fn max_file_size(&self) -> Option<u64> {
        // NMPA follows similar guidance to FDA
        Some(NMPA_MAX_FILE_SIZE)
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
        // NMPA allows non-ASCII for bilingual datasets
        !self.allow_non_ascii
    }

    fn file_naming_rules(&self) -> FileNamingRules {
        FileNamingRules::nmpa()
    }

    fn is_strict(&self) -> bool {
        self.strict
    }

    fn description(&self) -> String {
        let ascii_note = if self.allow_non_ascii {
            "Chinese text allowed"
        } else {
            "ASCII-only"
        };
        format!(
            "NMPA Policy ({}) - V5 required, {}, 5GB max",
            if self.strict { "strict" } else { "lenient" },
            ascii_note
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nmpa_strict() {
        let policy = NmpaPolicy::strict();
        assert!(policy.is_strict());
        assert_eq!(policy.agency(), Agency::Nmpa);
        assert!(policy.allows_non_ascii());
    }

    #[test]
    fn test_nmpa_lenient() {
        let policy = NmpaPolicy::lenient();
        assert!(!policy.is_strict());
        assert!(policy.allows_non_ascii());
    }

    #[test]
    fn test_nmpa_limits() {
        let policy = NmpaPolicy::default();
        assert_eq!(policy.max_variable_name_length(), 8);
        assert_eq!(policy.max_dataset_name_length(), 8);
        assert_eq!(policy.max_variable_label_length(), 40);
        assert_eq!(policy.max_dataset_label_length(), 40);
    }

    #[test]
    fn test_nmpa_non_ascii_allowed() {
        let policy = NmpaPolicy::default();
        // By default, NMPA allows non-ASCII (Chinese text)
        assert!(!policy.require_ascii());

        // Can disable non-ASCII
        let ascii_only = NmpaPolicy::default().with_allow_non_ascii(false);
        assert!(ascii_only.require_ascii());
    }

    #[test]
    fn test_nmpa_version() {
        let policy = NmpaPolicy::default();
        assert_eq!(policy.required_version(), Some(XptVersion::V5));
    }

    #[test]
    fn test_nmpa_file_size() {
        let policy = NmpaPolicy::default();
        assert_eq!(policy.max_file_size(), Some(NMPA_MAX_FILE_SIZE));
    }

    #[test]
    fn test_nmpa_description() {
        let policy = NmpaPolicy::strict();
        assert!(policy.description().contains("NMPA"));
        assert!(policy.description().contains("Chinese text allowed"));

        let ascii_only = NmpaPolicy::strict().with_allow_non_ascii(false);
        assert!(ascii_only.description().contains("ASCII-only"));
    }

    #[test]
    fn test_nmpa_file_naming() {
        let policy = NmpaPolicy::default();
        let rules = policy.file_naming_rules();
        assert!(rules.is_valid("dm.xpt"));
    }
}
