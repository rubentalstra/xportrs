//! XPT v5 structural validation.
//!
//! This module provides validation checks specific to XPT v5 format requirements.

use crate::schema::SchemaPlan;

use super::issues::{Issue, Severity};

/// XPT v5 constraints.
pub mod constraints {
    /// Maximum bytes for dataset name in v5.
    pub const MAX_DATASET_NAME_BYTES: usize = 8;
    /// Maximum bytes for variable name in v5.
    pub const MAX_VARIABLE_NAME_BYTES: usize = 8;
    /// Maximum bytes for label in v5.
    pub const MAX_LABEL_BYTES: usize = 40;
    /// Fixed length for numeric variables in v5.
    pub const NUMERIC_LENGTH: usize = 8;
    /// Minimum length for character variables in v5.
    pub const MIN_CHARACTER_LENGTH: usize = 1;
    /// Maximum length for character variables in v5.
    pub const MAX_CHARACTER_LENGTH: usize = 200;
}

/// Validates a schema plan against XPT v5 structural requirements.
///
/// Returns a list of issues found during validation.
#[must_use]
pub fn validate_v5_schema(plan: &SchemaPlan) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check dataset name length
    if plan.domain_code.len() > constraints::MAX_DATASET_NAME_BYTES {
        issues.push(
            Issue::new(
                Severity::Error,
                "XPT_V5_001",
                format!(
                    "dataset name '{}' exceeds {} bytes (has {} bytes)",
                    plan.domain_code,
                    constraints::MAX_DATASET_NAME_BYTES,
                    plan.domain_code.len()
                ),
            )
            .with_dataset(&plan.domain_code),
        );
    }

    // Check dataset label length
    if let Some(ref label) = plan.dataset_label {
        if label.len() > constraints::MAX_LABEL_BYTES {
            issues.push(
                Issue::new(
                    Severity::Error,
                    "XPT_V5_002",
                    format!(
                        "dataset label exceeds {} bytes (has {} bytes)",
                        constraints::MAX_LABEL_BYTES,
                        label.len()
                    ),
                )
                .with_dataset(&plan.domain_code),
            );
        }
    }

    // Check each variable
    for var in &plan.variables {
        // Variable name length
        if var.name.len() > constraints::MAX_VARIABLE_NAME_BYTES {
            issues.push(
                Issue::new(
                    Severity::Error,
                    "XPT_V5_003",
                    format!(
                        "variable name '{}' exceeds {} bytes (has {} bytes)",
                        var.name,
                        constraints::MAX_VARIABLE_NAME_BYTES,
                        var.name.len()
                    ),
                )
                .with_variable(&var.name),
            );
        }

        // Variable label length
        if var.label.len() > constraints::MAX_LABEL_BYTES {
            issues.push(
                Issue::new(
                    Severity::Error,
                    "XPT_V5_004",
                    format!(
                        "variable label exceeds {} bytes (has {} bytes)",
                        constraints::MAX_LABEL_BYTES,
                        var.label.len()
                    ),
                )
                .with_variable(&var.name),
            );
        }

        // Numeric length must be 8
        if var.xpt_type.is_numeric() && var.length != constraints::NUMERIC_LENGTH {
            issues.push(
                Issue::new(
                    Severity::Error,
                    "XPT_V5_005",
                    format!(
                        "numeric variable '{}' must have length {} (has {})",
                        var.name,
                        constraints::NUMERIC_LENGTH,
                        var.length
                    ),
                )
                .with_variable(&var.name),
            );
        }

        // Character length must be >= 1
        if var.xpt_type.is_character() && var.length < constraints::MIN_CHARACTER_LENGTH {
            issues.push(
                Issue::new(
                    Severity::Error,
                    "XPT_V5_006",
                    format!(
                        "character variable '{}' must have length >= {} (has {})",
                        var.name,
                        constraints::MIN_CHARACTER_LENGTH,
                        var.length
                    ),
                )
                .with_variable(&var.name),
            );
        }
    }

    // Verify row_len consistency
    let expected_row_len: usize = plan.variables.iter().map(|v| v.length).sum();
    if plan.row_len != expected_row_len {
        issues.push(Issue::new(
            Severity::Error,
            "XPT_V5_007",
            format!(
                "row_len inconsistency: recorded {} but computed {}",
                plan.row_len, expected_row_len
            ),
        ));
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::plan::PlannedVariable;

    #[test]
    fn test_valid_schema() {
        let mut plan = SchemaPlan::new("AE".into());
        plan.variables = vec![
            PlannedVariable::numeric("AESEQ"),
            PlannedVariable::character("USUBJID", 20),
        ];
        plan.recalculate_positions();

        let issues = validate_v5_schema(&plan);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_name_too_long() {
        let mut plan = SchemaPlan::new("TOOLONGNAME".into());
        plan.variables = vec![PlannedVariable::numeric("AESEQ")];
        plan.recalculate_positions();

        let issues = validate_v5_schema(&plan);
        assert!(!issues.is_empty());
        assert!(issues[0].code == "XPT_V5_001");
    }

    #[test]
    fn test_numeric_wrong_length() {
        let mut plan = SchemaPlan::new("AE".into());
        plan.variables = vec![PlannedVariable::new(
            "AESEQ".into(),
            crate::metadata::XptVarType::Numeric,
            4, // Wrong!
        )];
        plan.recalculate_positions();

        let issues = validate_v5_schema(&plan);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.code == "XPT_V5_005"));
    }
}
