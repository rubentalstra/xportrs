//! Compliance rules.
//!
//! This module defines the [`ComplianceProfile`] and [`Rule`] types that
//! specify validation requirements for XPT files.

use regex::Regex;

use crate::xpt::XptVersion;

/// A compliance profile defining validation rules and defaults.
///
/// Profiles are sets of composable rules plus a default transport version.
/// They represent policy presets, not authoritative regulatory requirements.
#[derive(Debug, Clone)]
pub struct ComplianceProfile {
    /// The name of this profile (e.g., "FDA", "NMPA", "Custom").
    pub name: &'static str,

    /// The default XPT version for this profile.
    pub default_version: XptVersion,

    /// The validation rules that comprise this profile.
    pub rules: Vec<Rule>,
}

impl ComplianceProfile {
    /// Creates a new compliance profile.
    #[must_use]
    pub const fn new(name: &'static str, default_version: XptVersion) -> Self {
        Self {
            name,
            default_version,
            rules: Vec::new(),
        }
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

    /// Returns an iterator over the rules.
    pub fn iter_rules(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter()
    }

    /// Checks if the profile requires ASCII-only variable names.
    #[must_use]
    pub fn requires_ascii_names(&self) -> bool {
        self.rules
            .iter()
            .any(|r| matches!(r, Rule::RequireAsciiNames))
    }

    /// Checks if the profile requires ASCII-only labels.
    #[must_use]
    pub fn requires_ascii_labels(&self) -> bool {
        self.rules
            .iter()
            .any(|r| matches!(r, Rule::RequireAsciiLabels))
    }

    /// Gets the maximum dataset name length from rules, if specified.
    #[must_use]
    pub fn max_dataset_name_bytes(&self) -> Option<usize> {
        self.rules.iter().find_map(|r| {
            if let Rule::DatasetNameMaxBytes(n) = r {
                Some(*n)
            } else {
                None
            }
        })
    }

    /// Gets the maximum variable name length from rules, if specified.
    #[must_use]
    pub fn max_variable_name_bytes(&self) -> Option<usize> {
        self.rules.iter().find_map(|r| {
            if let Rule::VariableNameMaxBytes(n) = r {
                Some(*n)
            } else {
                None
            }
        })
    }

    /// Gets the maximum label length from rules, if specified.
    #[must_use]
    pub fn max_label_bytes(&self) -> Option<usize> {
        self.rules.iter().find_map(|r| {
            if let Rule::LabelMaxBytes(n) = r {
                Some(*n)
            } else {
                None
            }
        })
    }
}

/// A single validation rule within a compliance profile.
///
/// Rules define specific constraints and policies for XPT file generation.
#[derive(Debug, Clone)]
pub enum Rule {
    /// Dataset name must match the given regex pattern.
    DatasetNamePattern {
        /// The regex pattern to match.
        regex: Regex,
    },

    /// Variable names must match the given regex pattern.
    VariableNamePattern {
        /// The regex pattern to match.
        regex: Regex,
    },

    /// Dataset name must match the file stem (filename without extension).
    DatasetNameMatchesFileStem,

    /// Variable and dataset names must be ASCII only.
    RequireAsciiNames,

    /// Labels must be ASCII only.
    RequireAsciiLabels,

    /// Character values must be ASCII only.
    RequireAsciiCharacterValues,

    /// Maximum byte length for dataset names.
    DatasetNameMaxBytes(usize),

    /// Maximum byte length for variable names.
    VariableNameMaxBytes(usize),

    /// Maximum byte length for labels.
    LabelMaxBytes(usize),

    /// Maximum byte length for character values (policy limit).
    CharacterValueMaxBytes(usize),

    /// Maximum file size in GB.
    MaxFileSizeGb(f64),
}

impl Rule {
    /// Creates a dataset name pattern rule from a regex string.
    ///
    /// # Panics
    ///
    /// Panics if the regex is invalid.
    #[must_use]
    pub fn dataset_name_pattern(pattern: &str) -> Self {
        Self::DatasetNamePattern {
            regex: Regex::new(pattern).expect("invalid regex pattern"),
        }
    }

    /// Creates a variable name pattern rule from a regex string.
    ///
    /// # Panics
    ///
    /// Panics if the regex is invalid.
    #[must_use]
    pub fn variable_name_pattern(pattern: &str) -> Self {
        Self::VariableNamePattern {
            regex: Regex::new(pattern).expect("invalid regex pattern"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let profile = ComplianceProfile::new("Test", XptVersion::V5)
            .with_rule(Rule::RequireAsciiNames)
            .with_rule(Rule::DatasetNameMaxBytes(8));

        assert_eq!(profile.name, "Test");
        assert!(profile.requires_ascii_names());
        assert_eq!(profile.max_dataset_name_bytes(), Some(8));
    }
}
