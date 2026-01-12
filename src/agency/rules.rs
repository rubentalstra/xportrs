//! Validation rules for agency compliance.
//!
//! This module defines the [`Rule`] enum that specifies validation
//! requirements for XPT files.

use std::path::Path;

use regex::Regex;

use crate::schema::DatasetSchema;
use crate::validate::Issue;

/// A single validation rule within an agency's requirements.
///
/// Rules define specific constraints and policies for XPT file generation.
/// Each agency has a set of rules that are applied during validation.
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

    /// Validates a schema plan against this rule.
    ///
    /// Returns a list of issues found during validation.
    #[must_use]
    pub(crate) fn validate(
        &self,
        plan: &DatasetSchema,
        file_path: Option<&Path>,
        agency_name: &'static str,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        match self {
            Self::DatasetNamePattern { regex } => {
                if !regex.is_match(&plan.domain_code) {
                    issues.push(Issue::DatasetNamePatternMismatch {
                        dataset: plan.domain_code.clone(),
                        agency: agency_name,
                        pattern: regex.as_str().to_string(),
                    });
                }
            }

            Self::VariableNamePattern { regex } => {
                for var in &plan.variables {
                    if !regex.is_match(&var.name) {
                        issues.push(Issue::VariableNamePatternMismatch {
                            variable: var.name.clone(),
                            agency: agency_name,
                            pattern: regex.as_str().to_string(),
                        });
                    }
                }
            }

            Self::DatasetNameMatchesFileStem => {
                if let Some(path) = file_path
                    && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                    && !stem.eq_ignore_ascii_case(&plan.domain_code)
                {
                    issues.push(Issue::DatasetNameFileStemMismatch {
                        dataset: plan.domain_code.clone(),
                        stem: stem.to_string(),
                    });
                }
            }

            Self::RequireAsciiNames => {
                if !plan.domain_code.is_ascii() {
                    issues.push(Issue::NonAsciiDatasetName {
                        dataset: plan.domain_code.clone(),
                    });
                }

                for var in &plan.variables {
                    if !var.name.is_ascii() {
                        issues.push(Issue::NonAsciiVariableName {
                            variable: var.name.clone(),
                        });
                    }
                }
            }

            Self::RequireAsciiLabels => {
                if let Some(ref label) = plan.dataset_label
                    && !label.is_ascii()
                {
                    issues.push(Issue::NonAsciiDatasetLabel {
                        dataset: plan.domain_code.clone(),
                    });
                }

                for var in &plan.variables {
                    if !var.label.is_ascii() {
                        issues.push(Issue::NonAsciiVariableLabel {
                            variable: var.name.clone(),
                        });
                    }
                }
            }

            Self::RequireAsciiCharacterValues => {
                // Note: This is checked during data encoding, not schema validation
            }

            Self::DatasetNameMaxBytes(max) => {
                if plan.domain_code.len() > *max {
                    issues.push(Issue::AgencyDatasetNameTooLong {
                        dataset: plan.domain_code.clone(),
                        max: *max,
                        actual: plan.domain_code.len(),
                    });
                }
            }

            Self::VariableNameMaxBytes(max) => {
                for var in &plan.variables {
                    if var.name.len() > *max {
                        issues.push(Issue::AgencyVariableNameTooLong {
                            variable: var.name.clone(),
                            max: *max,
                            actual: var.name.len(),
                        });
                    }
                }
            }

            Self::LabelMaxBytes(max) => {
                // Check dataset label
                if let Some(ref label) = plan.dataset_label {
                    let byte_len = label.len();
                    if byte_len > *max {
                        issues.push(Issue::AgencyLabelTooLong {
                            name: plan.domain_code.clone(),
                            is_dataset: true,
                            max: *max,
                            actual: byte_len,
                        });
                    } else if !label.is_ascii() && byte_len >= (*max * 80 / 100) {
                        // Warn when multi-byte label is >= 80% of limit
                        issues.push(Issue::MultiByteLabelNearLimit {
                            name: plan.domain_code.clone(),
                            is_dataset: true,
                            byte_count: byte_len,
                            max_bytes: *max,
                            char_count: label.chars().count(),
                        });
                    }
                }

                // Check variable labels
                for var in &plan.variables {
                    let byte_len = var.label.len();
                    if byte_len > *max {
                        issues.push(Issue::AgencyLabelTooLong {
                            name: var.name.clone(),
                            is_dataset: false,
                            max: *max,
                            actual: byte_len,
                        });
                    } else if !var.label.is_ascii() && byte_len >= (*max * 80 / 100) {
                        // Warn when multi-byte label is >= 80% of limit
                        issues.push(Issue::MultiByteLabelNearLimit {
                            name: var.name.clone(),
                            is_dataset: false,
                            byte_count: byte_len,
                            max_bytes: *max,
                            char_count: var.label.chars().count(),
                        });
                    }
                }
            }

            Self::CharacterValueMaxBytes(max) => {
                for var in &plan.variables {
                    if var.xpt_type.is_character() && var.length > *max {
                        issues.push(Issue::CharacterValueLengthExceeded {
                            variable: var.name.clone(),
                            length: var.length,
                            agency: agency_name,
                            max: *max,
                        });
                    }
                }
            }

            Self::MaxFileSizeGb(_max) => {
                // File size check is done during write, not schema validation
            }
        }

        issues
    }
}
