//! XPT v5 structural validation.
//!
//! This module provides validation checks specific to XPT v5 format requirements.

use crate::schema::SchemaPlan;

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
pub fn validate_v5_schema(plan: &SchemaPlan) -> Vec<Issue> {
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
    if let Some(ref label) = plan.dataset_label
        && label.len() > constraints::MAX_LABEL_BYTES
    {
        issues.push(Issue::DatasetLabelTooLong {
            dataset: plan.domain_code.clone(),
            max: constraints::MAX_LABEL_BYTES,
            actual: label.len(),
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

        // Variable label length
        if var.label.len() > constraints::MAX_LABEL_BYTES {
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
        assert!(matches!(issues[0], Issue::DatasetNameTooLong { .. }));
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
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, Issue::NumericWrongLength { .. }))
        );
    }
}
