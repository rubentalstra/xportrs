//! XPT v5 structural validation.
//!
//! This module provides validation checks specific to XPT v5 format requirements.

use crate::schema::DatasetSchema;

use super::issues::Issue;

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
pub(crate) fn validate_v5_schema(plan: &DatasetSchema) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check dataset name length
    if plan.domain_code.len() > constraints::MAX_DATASET_NAME_BYTES {
        issues.push(Issue::DatasetNameTooLong {
            dataset: plan.domain_code.clone(),
            max: constraints::MAX_DATASET_NAME_BYTES,
            actual: plan.domain_code.len(),
        });
    }

    // Check dataset label length
    if let Some(ref label) = plan.dataset_label {
        if label.len() > constraints::MAX_LABEL_BYTES {
            issues.push(Issue::DatasetLabelTooLong {
                dataset: plan.domain_code.clone(),
                max: constraints::MAX_LABEL_BYTES,
                actual: label.len(),
            });
        }
    } else {
        // Missing dataset label - warning per Pinnacle 21 SD0063A
        issues.push(Issue::MissingDatasetLabel {
            dataset: plan.domain_code.clone(),
        });
    }

    // Check each variable
    for var in &plan.variables {
        // Variable name length
        if var.name.len() > constraints::MAX_VARIABLE_NAME_BYTES {
            issues.push(Issue::VariableNameTooLong {
                variable: var.name.clone(),
                max: constraints::MAX_VARIABLE_NAME_BYTES,
                actual: var.name.len(),
            });
        }

        // Variable label length and missing label check
        if var.label.is_empty() {
            // Missing variable label - warning per Pinnacle 21 SD0063
            issues.push(Issue::MissingVariableLabel {
                variable: var.name.clone(),
            });
        } else if var.label.len() > constraints::MAX_LABEL_BYTES {
            issues.push(Issue::VariableLabelTooLong {
                variable: var.name.clone(),
                max: constraints::MAX_LABEL_BYTES,
                actual: var.label.len(),
            });
        }

        // Numeric length must be 8
        if var.xpt_type.is_numeric() && var.length != constraints::NUMERIC_LENGTH {
            issues.push(Issue::NumericWrongLength {
                variable: var.name.clone(),
                expected: constraints::NUMERIC_LENGTH,
                actual: var.length,
            });
        }

        // Character length must be >= 1
        if var.xpt_type.is_character() && var.length < constraints::MIN_CHARACTER_LENGTH {
            issues.push(Issue::CharacterLengthTooShort {
                variable: var.name.clone(),
                min: constraints::MIN_CHARACTER_LENGTH,
                actual: var.length,
            });
        }

        // Character length must be <= 200
        if var.xpt_type.is_character() && var.length > constraints::MAX_CHARACTER_LENGTH {
            issues.push(Issue::CharacterLengthTooLong {
                variable: var.name.clone(),
                max: constraints::MAX_CHARACTER_LENGTH,
                actual: var.length,
            });
        }
    }

    // Verify row_len consistency
    let expected_row_len: usize = plan.variables.iter().map(|v| v.length).sum();
    if plan.row_len != expected_row_len {
        issues.push(Issue::RowLenInconsistent {
            recorded: plan.row_len,
            computed: expected_row_len,
        });
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::plan::VariableSpec;

    #[test]
    fn test_valid_schema_with_labels() {
        let mut plan = DatasetSchema::new("AE").with_label(Some("Adverse Events".into()));
        plan.variables = vec![
            VariableSpec::numeric("AESEQ").with_label("Sequence Number"),
            VariableSpec::character("USUBJID", 20).with_label("Unique Subject Identifier"),
        ];
        plan.recalculate_positions();

        let issues = validate_v5_schema(&plan);
        // Should have no errors or warnings when labels are provided
        assert!(issues.is_empty());
    }

    #[test]
    fn test_missing_labels_warnings() {
        let mut plan = DatasetSchema::new("AE");
        plan.variables = vec![
            VariableSpec::numeric("AESEQ"),
            VariableSpec::character("USUBJID", 20),
        ];
        plan.recalculate_positions();

        let issues = validate_v5_schema(&plan);
        // Should have warnings for missing dataset label and variable labels
        assert!(!issues.is_empty());

        // All issues should be warnings, not errors
        for issue in &issues {
            assert!(issue.is_warning(), "Expected warning, got error: {:?}", issue);
        }

        // Check for specific warning types
        assert!(issues.iter().any(|i| matches!(i, Issue::MissingDatasetLabel { .. })));
        assert!(issues.iter().any(|i| matches!(i, Issue::MissingVariableLabel { variable } if variable == "AESEQ")));
        assert!(issues.iter().any(|i| matches!(i, Issue::MissingVariableLabel { variable } if variable == "USUBJID")));
    }

    #[test]
    fn test_name_too_long() {
        let mut plan = DatasetSchema::new("TOOLONGNAME");
        plan.variables = vec![VariableSpec::numeric("AESEQ")];
        plan.recalculate_positions();

        let issues = validate_v5_schema(&plan);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| matches!(i, Issue::DatasetNameTooLong { .. })));
    }

    #[test]
    fn test_numeric_wrong_length() {
        let mut plan = DatasetSchema::new("AE");
        plan.variables = vec![VariableSpec::new(
            "AESEQ".into(),
            crate::metadata::XptVarType::Numeric,
            4, // Wrong!
        )];
        plan.recalculate_positions();

        let issues = validate_v5_schema(&plan);
        assert!(!issues.is_empty());
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, Issue::NumericWrongLength { .. }))
        );
    }
}
