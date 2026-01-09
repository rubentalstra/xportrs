//! Compliance profile presets.
//!
//! This module provides pre-defined compliance profiles for common regulatory
//! requirements (FDA, NMPA, PMDA) and a builder for custom profiles.

use std::sync::LazyLock;

use crate::xpt::XptVersion;

use super::rules::{ComplianceProfile, Rule};

/// FDA compliance profile preset.
///
/// This profile enforces requirements commonly expected for FDA submissions:
/// - XPT v5 format
/// - ASCII-only names and labels
/// - 8-byte dataset/variable names
/// - 40-byte labels
/// - 200-byte character value policy limit
pub static FDA_PROFILE: LazyLock<ComplianceProfile> = LazyLock::new(|| {
    ComplianceProfile::new("FDA", XptVersion::V5).with_rules([
        Rule::RequireAsciiNames,
        Rule::RequireAsciiLabels,
        Rule::RequireAsciiCharacterValues,
        Rule::DatasetNameMaxBytes(8),
        Rule::VariableNameMaxBytes(8),
        Rule::LabelMaxBytes(40),
        Rule::CharacterValueMaxBytes(200),
        Rule::DatasetNameMatchesFileStem,
        Rule::dataset_name_pattern(r"^[A-Z][A-Z0-9]{0,7}$"),
        Rule::variable_name_pattern(r"^[A-Z_][A-Z0-9_]{0,7}$"),
    ])
});

/// NMPA (China) compliance profile preset.
///
/// This profile follows NMPA requirements, which are largely aligned with FDA:
/// - XPT v5 format
/// - ASCII-only names and labels
/// - 8-byte dataset/variable names
/// - 40-byte labels
pub static NMPA_PROFILE: LazyLock<ComplianceProfile> = LazyLock::new(|| {
    ComplianceProfile::new("NMPA", XptVersion::V5).with_rules([
        Rule::RequireAsciiNames,
        Rule::RequireAsciiLabels,
        Rule::RequireAsciiCharacterValues,
        Rule::DatasetNameMaxBytes(8),
        Rule::VariableNameMaxBytes(8),
        Rule::LabelMaxBytes(40),
        Rule::CharacterValueMaxBytes(200),
        Rule::DatasetNameMatchesFileStem,
        Rule::dataset_name_pattern(r"^[A-Z][A-Z0-9]{0,7}$"),
        Rule::variable_name_pattern(r"^[A-Z_][A-Z0-9_]{0,7}$"),
    ])
});

/// PMDA (Japan) compliance profile preset.
///
/// This profile follows PMDA requirements:
/// - XPT v5 format
/// - ASCII-only names and labels
/// - 8-byte dataset/variable names
/// - 40-byte labels
pub static PMDA_PROFILE: LazyLock<ComplianceProfile> = LazyLock::new(|| {
    ComplianceProfile::new("PMDA", XptVersion::V5).with_rules([
        Rule::RequireAsciiNames,
        Rule::RequireAsciiLabels,
        Rule::RequireAsciiCharacterValues,
        Rule::DatasetNameMaxBytes(8),
        Rule::VariableNameMaxBytes(8),
        Rule::LabelMaxBytes(40),
        Rule::CharacterValueMaxBytes(200),
        Rule::DatasetNameMatchesFileStem,
        Rule::dataset_name_pattern(r"^[A-Z][A-Z0-9]{0,7}$"),
        Rule::variable_name_pattern(r"^[A-Z_][A-Z0-9_]{0,7}$"),
    ])
});

/// Creates a custom compliance profile builder.
///
/// # Example
///
/// ```
/// use xportrs::profile::{custom_profile, Rule};
///
/// let profile = custom_profile("MyOrg")
///     .with_rule(Rule::RequireAsciiNames)
///     .with_rule(Rule::DatasetNameMaxBytes(8))
///     .build();
/// ```
#[must_use]
pub fn custom_profile(name: &'static str) -> ComplianceProfileBuilder {
    ComplianceProfileBuilder::new(name)
}

/// Builder for custom compliance profiles.
#[derive(Debug)]
pub struct ComplianceProfileBuilder {
    name: &'static str,
    version: XptVersion,
    rules: Vec<Rule>,
}

impl ComplianceProfileBuilder {
    /// Creates a new builder with the given name.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            version: XptVersion::V5,
            rules: Vec::new(),
        }
    }

    /// Sets the default XPT version.
    #[must_use]
    pub const fn with_version(mut self, version: XptVersion) -> Self {
        self.version = version;
        self
    }

    /// Adds a rule to the profile.
    #[must_use]
    pub fn with_rule(mut self, rule: Rule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Adds multiple rules to the profile.
    #[must_use]
    pub fn with_rules(mut self, rules: impl IntoIterator<Item = Rule>) -> Self {
        self.rules.extend(rules);
        self
    }

    /// Starts from an existing profile (copies its rules).
    #[must_use]
    pub fn extend_from(mut self, profile: &ComplianceProfile) -> Self {
        self.rules.extend(profile.rules.iter().cloned());
        self
    }

    /// Builds the compliance profile.
    #[must_use]
    pub fn build(self) -> ComplianceProfile {
        ComplianceProfile {
            name: self.name,
            default_version: self.version,
            rules: self.rules,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fda_profile() {
        let profile = &*FDA_PROFILE;
        assert_eq!(profile.name, "FDA");
        assert!(profile.requires_ascii_names());
        assert_eq!(profile.max_dataset_name_bytes(), Some(8));
        assert_eq!(profile.max_variable_name_bytes(), Some(8));
        assert_eq!(profile.max_label_bytes(), Some(40));
    }

    #[test]
    fn test_custom_profile() {
        let profile = custom_profile("Custom")
            .with_rule(Rule::RequireAsciiNames)
            .with_rule(Rule::LabelMaxBytes(50))
            .build();

        assert_eq!(profile.name, "Custom");
        assert!(profile.requires_ascii_names());
        assert_eq!(profile.max_label_bytes(), Some(50));
    }
}
