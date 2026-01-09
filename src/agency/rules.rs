//! Validation rules for agency compliance.
//!
//! This module defines the [`Rule`] enum that specifies validation
//! requirements for XPT files.

use std::path::Path;

use regex::Regex;

use crate::schema::SchemaPlan;
use crate::validate::{Issue, Severity};

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
    pub fn validate(
        &self,
        plan: &SchemaPlan,
        file_path: Option<&Path>,
        agency_name: &str,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        match self {
            Self::DatasetNamePattern { regex } => {
                if !regex.is_match(&plan.domain_code) {
                    issues.push(
                        Issue::new(
                            Severity::Error,
                            "AGENCY_001",
                            format!(
                                "dataset name '{}' does not match {} required pattern '{}'",
                                plan.domain_code,
                                agency_name,
                                regex.as_str()
                            ),
                        )
                        .with_dataset(&plan.domain_code),
                    );
                }
            }

            Self::VariableNamePattern { regex } => {
                for var in &plan.variables {
                    if !regex.is_match(&var.name) {
                        issues.push(
                            Issue::new(
                                Severity::Error,
                                "AGENCY_002",
                                format!(
                                    "variable name '{}' does not match {} required pattern '{}'",
                                    var.name,
                                    agency_name,
                                    regex.as_str()
                                ),
                            )
                            .with_variable(&var.name),
                        );
                    }
                }
            }

            Self::DatasetNameMatchesFileStem => {
                if let Some(path) = file_path
                    && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                    && !stem.eq_ignore_ascii_case(&plan.domain_code)
                {
                    issues.push(
                        Issue::new(
                            Severity::Error,
                            "AGENCY_003",
                            format!(
                                "dataset name '{}' does not match file stem '{}'",
                                plan.domain_code, stem
                            ),
                        )
                        .with_dataset(&plan.domain_code),
                    );
                }
            }

            Self::RequireAsciiNames => {
                if !plan.domain_code.is_ascii() {
                    issues.push(
                        Issue::new(
                            Severity::Error,
                            "AGENCY_004",
                            format!(
                                "dataset name '{}' contains non-ASCII characters",
                                plan.domain_code
                            ),
                        )
                        .with_dataset(&plan.domain_code),
                    );
                }

                for var in &plan.variables {
                    if !var.name.is_ascii() {
                        issues.push(
                            Issue::new(
                                Severity::Error,
                                "AGENCY_004",
                                format!(
                                    "variable name '{}' contains non-ASCII characters",
                                    var.name
                                ),
                            )
                            .with_variable(&var.name),
                        );
                    }
                }
            }

            Self::RequireAsciiLabels => {
                if let Some(ref label) = plan.dataset_label
                    && !label.is_ascii()
                {
                    issues.push(
                        Issue::new(
                            Severity::Error,
                            "AGENCY_005",
                            "dataset label contains non-ASCII characters",
                        )
                        .with_dataset(&plan.domain_code),
                    );
                }

                for var in &plan.variables {
                    if !var.label.is_ascii() {
                        issues.push(
                            Issue::new(
                                Severity::Error,
                                "AGENCY_005",
                                format!(
                                    "variable '{}' label contains non-ASCII characters",
                                    var.name
                                ),
                            )
                            .with_variable(&var.name),
                        );
                    }
                }
            }

            Self::RequireAsciiCharacterValues => {
                // Note: This is checked during data encoding, not schema validation
            }

            Self::DatasetNameMaxBytes(max) => {
                if plan.domain_code.len() > *max {
                    issues.push(
                        Issue::new(
                            Severity::Error,
                            "AGENCY_006",
                            format!(
                                "dataset name '{}' exceeds {} bytes (has {} bytes)",
                                plan.domain_code, max,
                                plan.domain_code.len()
                            ),
                        )
                        .with_dataset(&plan.domain_code),
                    );
                }
            }

            Self::VariableNameMaxBytes(max) => {
                for var in &plan.variables {
                    if var.name.len() > *max {
                        issues.push(
                            Issue::new(
                                Severity::Error,
                                "AGENCY_007",
                                format!(
                                    "variable name '{}' exceeds {} bytes (has {} bytes)",
                                    var.name, max,
                                    var.name.len()
                                ),
                            )
                            .with_variable(&var.name),
                        );
                    }
                }
            }

            Self::LabelMaxBytes(max) => {
                if let Some(ref label) = plan.dataset_label
                    && label.len() > *max
                {
                    issues.push(
                        Issue::new(
                            Severity::Error,
                            "AGENCY_008",
                            format!(
                                "dataset label exceeds {} bytes (has {} bytes)",
                                max,
                                label.len()
                            ),
                        )
                        .with_dataset(&plan.domain_code),
                    );
                }

                for var in &plan.variables {
                    if var.label.len() > *max {
                        issues.push(
                            Issue::new(
                                Severity::Error,
                                "AGENCY_008",
                                format!(
                                    "variable '{}' label exceeds {} bytes (has {} bytes)",
                                    var.name, max,
                                    var.label.len()
                                ),
                            )
                            .with_variable(&var.name),
                        );
                    }
                }
            }

            Self::CharacterValueMaxBytes(max) => {
                for var in &plan.variables {
                    if var.xpt_type.is_character() && var.length > *max {
                        issues.push(
                            Issue::new(
                                Severity::Warning,
                                "AGENCY_009",
                                format!(
                                    "character variable '{}' length {} exceeds {} policy limit of {} bytes",
                                    var.name, var.length, agency_name, max
                                ),
                            )
                            .with_variable(&var.name),
                        );
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
