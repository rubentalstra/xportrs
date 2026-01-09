//! Compliance profile validation.
//!
//! This module provides validation checks based on compliance profiles.

use std::path::Path;

use crate::profile::{ComplianceProfile, Rule};
use crate::schema::SchemaPlan;

use super::issues::{Issue, Severity};

/// Validates a schema plan against a compliance profile.
///
/// Returns a list of issues found during validation.
#[must_use]
pub fn validate_profile(
    plan: &SchemaPlan,
    profile: &ComplianceProfile,
    file_path: Option<&Path>,
) -> Vec<Issue> {
    let mut issues = Vec::new();

    for rule in profile.iter_rules() {
        match rule {
            Rule::DatasetNamePattern { regex } => {
                if !regex.is_match(&plan.domain_code) {
                    issues.push(
                        Issue::new(
                            Severity::Error,
                            "PROFILE_001",
                            format!(
                                "dataset name '{}' does not match required pattern '{}'",
                                plan.domain_code,
                                regex.as_str()
                            ),
                        )
                        .with_dataset(&plan.domain_code),
                    );
                }
            }

            Rule::VariableNamePattern { regex } => {
                for var in &plan.variables {
                    if !regex.is_match(&var.name) {
                        issues.push(
                            Issue::new(
                                Severity::Error,
                                "PROFILE_002",
                                format!(
                                    "variable name '{}' does not match required pattern '{}'",
                                    var.name,
                                    regex.as_str()
                                ),
                            )
                            .with_variable(&var.name),
                        );
                    }
                }
            }

            Rule::DatasetNameMatchesFileStem => {
                if let Some(path) = file_path {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if !stem.eq_ignore_ascii_case(&plan.domain_code) {
                            issues.push(
                                Issue::new(
                                    Severity::Error,
                                    "PROFILE_003",
                                    format!(
                                        "dataset name '{}' does not match file stem '{}'",
                                        plan.domain_code, stem
                                    ),
                                )
                                .with_dataset(&plan.domain_code),
                            );
                        }
                    }
                }
            }

            Rule::RequireAsciiNames => {
                if !plan.domain_code.is_ascii() {
                    issues.push(
                        Issue::new(
                            Severity::Error,
                            "PROFILE_004",
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
                                "PROFILE_004",
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

            Rule::RequireAsciiLabels => {
                if let Some(ref label) = plan.dataset_label {
                    if !label.is_ascii() {
                        issues.push(
                            Issue::new(
                                Severity::Error,
                                "PROFILE_005",
                                "dataset label contains non-ASCII characters",
                            )
                            .with_dataset(&plan.domain_code),
                        );
                    }
                }

                for var in &plan.variables {
                    if !var.label.is_ascii() {
                        issues.push(
                            Issue::new(
                                Severity::Error,
                                "PROFILE_005",
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

            Rule::RequireAsciiCharacterValues => {
                // Note: This would need to check actual data, not just schema
                // For now, we record this as a profile requirement that gets checked
                // during data coercion
            }

            Rule::DatasetNameMaxBytes(max) => {
                if plan.domain_code.len() > *max {
                    issues.push(
                        Issue::new(
                            Severity::Error,
                            "PROFILE_006",
                            format!(
                                "dataset name '{}' exceeds {} bytes (has {} bytes)",
                                plan.domain_code,
                                max,
                                plan.domain_code.len()
                            ),
                        )
                        .with_dataset(&plan.domain_code),
                    );
                }
            }

            Rule::VariableNameMaxBytes(max) => {
                for var in &plan.variables {
                    if var.name.len() > *max {
                        issues.push(
                            Issue::new(
                                Severity::Error,
                                "PROFILE_007",
                                format!(
                                    "variable name '{}' exceeds {} bytes (has {} bytes)",
                                    var.name,
                                    max,
                                    var.name.len()
                                ),
                            )
                            .with_variable(&var.name),
                        );
                    }
                }
            }

            Rule::LabelMaxBytes(max) => {
                if let Some(ref label) = plan.dataset_label {
                    if label.len() > *max {
                        issues.push(
                            Issue::new(
                                Severity::Error,
                                "PROFILE_008",
                                format!(
                                    "dataset label exceeds {} bytes (has {} bytes)",
                                    max,
                                    label.len()
                                ),
                            )
                            .with_dataset(&plan.domain_code),
                        );
                    }
                }

                for var in &plan.variables {
                    if var.label.len() > *max {
                        issues.push(
                            Issue::new(
                                Severity::Error,
                                "PROFILE_008",
                                format!(
                                    "variable '{}' label exceeds {} bytes (has {} bytes)",
                                    var.name,
                                    max,
                                    var.label.len()
                                ),
                            )
                            .with_variable(&var.name),
                        );
                    }
                }
            }

            Rule::CharacterValueMaxBytes(max) => {
                for var in &plan.variables {
                    if var.xpt_type.is_character() && var.length > *max {
                        issues.push(
                            Issue::new(
                                Severity::Warning,
                                "PROFILE_009",
                                format!(
                                    "character variable '{}' length {} exceeds policy limit of {} bytes",
                                    var.name, var.length, max
                                ),
                            )
                            .with_variable(&var.name),
                        );
                    }
                }
            }

            Rule::MaxFileSizeGb(_max) => {
                // File size check would be done during write, not schema validation
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::presets::FDA_PROFILE;
    use crate::schema::plan::PlannedVariable;

    #[test]
    fn test_fda_profile_valid() {
        let mut plan = SchemaPlan::new("AE".into());
        plan.variables = vec![
            PlannedVariable::numeric("AESEQ"),
            PlannedVariable::character("USUBJID", 20),
        ];
        plan.recalculate_positions();

        let issues = validate_profile(&plan, &*FDA_PROFILE, None);
        // Should have no errors (warnings about pattern might exist)
        assert!(!issues.iter().any(|i| i.is_error()));
    }

    #[test]
    fn test_non_ascii_name() {
        let mut plan = SchemaPlan::new("AÉ".into());
        plan.variables = vec![PlannedVariable::numeric("AESEQ")];
        plan.recalculate_positions();

        let issues = validate_profile(&plan, &*FDA_PROFILE, None);
        assert!(issues.iter().any(|i| i.code == "PROFILE_004"));
    }
}
