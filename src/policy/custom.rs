//! Custom policy builder.
//!
//! This module provides a [`CustomPolicy`] that allows users to define
//! their own agency-specific constraints. This is useful for:
//!
//! - Internal company standards
//! - Non-standard regulatory requirements
//! - Testing and development scenarios
//! - Regional variations not covered by built-in policies

use crate::XptVersion;

use super::agency::{Agency, AgencyPolicy};
use super::rules::FileNamingRules;

/// A custom policy with user-defined constraints.
///
/// Use the builder pattern to configure each constraint. Any constraints
/// not explicitly set will use sensible defaults based on V5 format limits.
///
/// # Example
///
/// ```
/// use xportrs::policy::{AgencyPolicy, CustomPolicy};
/// use xportrs::XptVersion;
///
/// // Create a custom policy for V8 format
/// let policy = CustomPolicy::new()
///     .with_required_version(XptVersion::V8)
///     .with_max_variable_name_length(32)
///     .with_max_variable_label_length(256)
///     .with_require_ascii(false);
///
/// assert_eq!(policy.max_variable_name_length(), 32);
/// assert!(!policy.require_ascii());
/// ```
///
/// # Example: Internal Standards
///
/// ```
/// use xportrs::policy::{AgencyPolicy, CustomPolicy, FileNamingRules};
///
/// // Company requires stricter naming
/// let policy = CustomPolicy::new()
///     .with_max_variable_name_length(6)
///     .with_require_uppercase_names(true)
///     .with_strict(true);
///
/// assert_eq!(policy.max_variable_name_length(), 6);
/// assert!(policy.require_uppercase_names());
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CustomPolicy {
    /// Required XPT version, if any.
    required_version: Option<XptVersion>,

    /// Maximum file size in bytes, if any.
    max_file_size: Option<u64>,

    /// Maximum variable name length.
    max_variable_name_length: usize,

    /// Maximum dataset name length.
    max_dataset_name_length: usize,

    /// Maximum variable label length.
    max_variable_label_length: usize,

    /// Maximum dataset label length.
    max_dataset_label_length: usize,

    /// Maximum format name length.
    max_format_name_length: usize,

    /// Whether to require uppercase names.
    require_uppercase_names: bool,

    /// Whether to require ASCII-only strings.
    require_ascii: bool,

    /// Whether to enforce strict validation.
    strict: bool,

    /// File naming rules.
    file_naming_rules: FileNamingRules,

    /// Optional custom description.
    description: Option<String>,
}

impl CustomPolicy {
    /// Create a new custom policy with default settings.
    ///
    /// Defaults are based on V5 format limits (8-char names, 40-char labels).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the required XPT version.
    #[must_use]
    pub fn with_required_version(mut self, version: XptVersion) -> Self {
        self.required_version = Some(version);
        // Update limits based on version
        match version {
            XptVersion::V5 => {
                self.max_variable_name_length = 8;
                self.max_dataset_name_length = 8;
                self.max_variable_label_length = 40;
                self.max_dataset_label_length = 40;
                self.max_format_name_length = 8;
            }
            XptVersion::V8 => {
                self.max_variable_name_length = 32;
                self.max_dataset_name_length = 32;
                self.max_variable_label_length = 256;
                self.max_dataset_label_length = 256;
                self.max_format_name_length = 32;
            }
        }
        self
    }

    /// Clear the required version (allow any version).
    #[must_use]
    pub fn with_any_version(mut self) -> Self {
        self.required_version = None;
        self
    }

    /// Set the maximum file size in bytes.
    #[must_use]
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = Some(size);
        self
    }

    /// Clear the maximum file size limit.
    #[must_use]
    pub fn with_no_file_size_limit(mut self) -> Self {
        self.max_file_size = None;
        self
    }

    /// Set the maximum variable name length.
    #[must_use]
    pub fn with_max_variable_name_length(mut self, length: usize) -> Self {
        self.max_variable_name_length = length;
        self
    }

    /// Set the maximum dataset name length.
    #[must_use]
    pub fn with_max_dataset_name_length(mut self, length: usize) -> Self {
        self.max_dataset_name_length = length;
        self
    }

    /// Set the maximum variable label length.
    #[must_use]
    pub fn with_max_variable_label_length(mut self, length: usize) -> Self {
        self.max_variable_label_length = length;
        self
    }

    /// Set the maximum dataset label length.
    #[must_use]
    pub fn with_max_dataset_label_length(mut self, length: usize) -> Self {
        self.max_dataset_label_length = length;
        self
    }

    /// Set the maximum format name length.
    #[must_use]
    pub fn with_max_format_name_length(mut self, length: usize) -> Self {
        self.max_format_name_length = length;
        self
    }

    /// Set whether uppercase names are required.
    #[must_use]
    pub fn with_require_uppercase_names(mut self, require: bool) -> Self {
        self.require_uppercase_names = require;
        self
    }

    /// Set whether ASCII-only strings are required.
    #[must_use]
    pub fn with_require_ascii(mut self, require: bool) -> Self {
        self.require_ascii = require;
        self
    }

    /// Set whether to enforce strict validation.
    #[must_use]
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Set custom file naming rules.
    #[must_use]
    pub fn with_file_naming_rules(mut self, rules: FileNamingRules) -> Self {
        self.file_naming_rules = rules;
        self
    }

    /// Set a custom description for this policy.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Create a custom policy starting from FDA defaults.
    #[must_use]
    pub fn from_fda_base() -> Self {
        Self::new()
            .with_required_version(XptVersion::V5)
            .with_max_file_size(5 * 1024 * 1024 * 1024)
            .with_require_ascii(true)
            .with_file_naming_rules(FileNamingRules::fda())
    }

    /// Create a custom policy for V8 extended format.
    #[must_use]
    pub fn v8_extended() -> Self {
        Self::new()
            .with_required_version(XptVersion::V8)
            .with_no_file_size_limit()
            .with_require_ascii(false)
            .with_file_naming_rules(FileNamingRules::permissive())
    }
}

impl Default for CustomPolicy {
    fn default() -> Self {
        Self {
            required_version: Some(XptVersion::V5),
            max_file_size: None,
            max_variable_name_length: 8,
            max_dataset_name_length: 8,
            max_variable_label_length: 40,
            max_dataset_label_length: 40,
            max_format_name_length: 8,
            require_uppercase_names: false,
            require_ascii: false,
            strict: false,
            file_naming_rules: FileNamingRules::default(),
            description: None,
        }
    }
}

impl AgencyPolicy for CustomPolicy {
    fn agency(&self) -> Agency {
        Agency::Custom
    }

    fn required_version(&self) -> Option<XptVersion> {
        self.required_version
    }

    fn max_file_size(&self) -> Option<u64> {
        self.max_file_size
    }

    fn max_variable_name_length(&self) -> usize {
        self.max_variable_name_length
    }

    fn max_dataset_name_length(&self) -> usize {
        self.max_dataset_name_length
    }

    fn max_variable_label_length(&self) -> usize {
        self.max_variable_label_length
    }

    fn max_dataset_label_length(&self) -> usize {
        self.max_dataset_label_length
    }

    fn max_format_name_length(&self) -> usize {
        self.max_format_name_length
    }

    fn require_uppercase_names(&self) -> bool {
        self.require_uppercase_names
    }

    fn require_ascii(&self) -> bool {
        self.require_ascii
    }

    fn file_naming_rules(&self) -> FileNamingRules {
        self.file_naming_rules.clone()
    }

    fn is_strict(&self) -> bool {
        self.strict
    }

    fn description(&self) -> String {
        if let Some(ref desc) = self.description {
            desc.clone()
        } else {
            let version_str = self
                .required_version
                .map_or("any version".to_string(), |v| format!("V{v}"));
            format!(
                "Custom Policy ({}) - {}, {}-char names",
                if self.strict { "strict" } else { "lenient" },
                version_str,
                self.max_variable_name_length
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_default() {
        let policy = CustomPolicy::new();
        assert_eq!(policy.agency(), Agency::Custom);
        assert_eq!(policy.required_version(), Some(XptVersion::V5));
        assert_eq!(policy.max_variable_name_length(), 8);
        assert!(!policy.is_strict());
    }

    #[test]
    fn test_custom_v8() {
        let policy = CustomPolicy::new().with_required_version(XptVersion::V8);

        assert_eq!(policy.required_version(), Some(XptVersion::V8));
        assert_eq!(policy.max_variable_name_length(), 32);
        assert_eq!(policy.max_variable_label_length(), 256);
    }

    #[test]
    fn test_custom_builder() {
        let policy = CustomPolicy::new()
            .with_max_variable_name_length(16)
            .with_max_variable_label_length(100)
            .with_require_uppercase_names(true)
            .with_require_ascii(true)
            .with_strict(true);

        assert_eq!(policy.max_variable_name_length(), 16);
        assert_eq!(policy.max_variable_label_length(), 100);
        assert!(policy.require_uppercase_names());
        assert!(policy.require_ascii());
        assert!(policy.is_strict());
    }

    #[test]
    fn test_custom_file_size() {
        let policy = CustomPolicy::new().with_max_file_size(1024 * 1024); // 1MB

        assert_eq!(policy.max_file_size(), Some(1024 * 1024));

        let unlimited = policy.with_no_file_size_limit();
        assert_eq!(unlimited.max_file_size(), None);
    }

    #[test]
    fn test_custom_from_fda_base() {
        let policy = CustomPolicy::from_fda_base();

        assert_eq!(policy.required_version(), Some(XptVersion::V5));
        assert!(policy.require_ascii());
        assert_eq!(policy.max_file_size(), Some(5 * 1024 * 1024 * 1024));
    }

    #[test]
    fn test_custom_v8_extended() {
        let policy = CustomPolicy::v8_extended();

        assert_eq!(policy.required_version(), Some(XptVersion::V8));
        assert!(!policy.require_ascii());
        assert_eq!(policy.max_file_size(), None);
        assert_eq!(policy.max_variable_name_length(), 32);
    }

    #[test]
    fn test_custom_description() {
        let policy = CustomPolicy::new();
        assert!(policy.description().contains("Custom"));

        let with_desc = policy.with_description("My Company Standard");
        assert_eq!(with_desc.description(), "My Company Standard");
    }

    #[test]
    fn test_custom_any_version() {
        let policy = CustomPolicy::new().with_any_version();
        assert_eq!(policy.required_version(), None);
    }

    #[test]
    fn test_custom_file_naming() {
        let policy = CustomPolicy::new().with_file_naming_rules(FileNamingRules::permissive());

        let rules = policy.file_naming_rules();
        assert!(rules.is_valid("LongDatasetName.xpt"));
    }
}
