//! Value-level validation rules.
//!
//! This module contains validation rules that check individual values
//! within rows, such as character value lengths and ASCII compliance.

use crate::error::{ErrorLocation, Severity, ValidationError, ValidationErrorCode};
use crate::types::{XptColumn, XptDataset, XptValue};
use crate::validation::{ValidationContext, ValidationRule};

/// Rule that validates character values don't exceed their column's defined length.
///
/// This rule checks each character value in the dataset and ensures it doesn't
/// exceed the maximum length defined for its column.
pub struct CharacterLengthRule;

impl ValidationRule for CharacterLengthRule {
    fn name(&self) -> &'static str {
        "CharacterLengthRule"
    }

    fn validate_dataset(
        &self,
        dataset: &XptDataset,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (row_idx, row) in dataset.rows.iter().enumerate() {
            for (col_idx, value) in row.iter().enumerate() {
                if col_idx >= dataset.columns.len() {
                    continue;
                }

                let column = &dataset.columns[col_idx];
                if let XptValue::Char(s) = value {
                    let max_len = column.length as usize;
                    if s.len() > max_len {
                        errors.push(ValidationError::new(
                            ValidationErrorCode::CharacterValueTooLong,
                            format!(
                                "Character value in column '{}' at row {} has length {} but maximum is {}",
                                column.name, row_idx, s.len(), max_len
                            ),
                            ErrorLocation::Value {
                                dataset: dataset.name.clone(),
                                column: column.name.clone(),
                                row: row_idx,
                            },
                            Severity::Error,
                        ));
                    }
                }
            }
        }

        errors
    }
}

/// Rule that validates character values contain only ASCII characters.
///
/// This rule is particularly important for FDA submissions where non-ASCII
/// characters may cause issues with SAS processing.
pub struct AsciiValueRule;

impl ValidationRule for AsciiValueRule {
    fn name(&self) -> &'static str {
        "AsciiValueRule"
    }

    fn validate_dataset(
        &self,
        dataset: &XptDataset,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (row_idx, row) in dataset.rows.iter().enumerate() {
            for (col_idx, value) in row.iter().enumerate() {
                if col_idx >= dataset.columns.len() {
                    continue;
                }

                let column = &dataset.columns[col_idx];
                if let XptValue::Char(s) = value {
                    if !s.is_ascii() {
                        // Find the first non-ASCII character for the error message
                        let non_ascii_char = s.chars().find(|c| !c.is_ascii());
                        let message = if let Some(c) = non_ascii_char {
                            format!(
                                "Character value in column '{}' at row {} contains non-ASCII character '{}'",
                                column.name, row_idx, c
                            )
                        } else {
                            format!(
                                "Character value in column '{}' at row {} contains non-ASCII characters",
                                column.name, row_idx
                            )
                        };

                        errors.push(ValidationError::new(
                            ValidationErrorCode::NonAsciiValue,
                            message,
                            ErrorLocation::Value {
                                dataset: dataset.name.clone(),
                                column: column.name.clone(),
                                row: row_idx,
                            },
                            Severity::Warning, // Warning because UTF-8 is technically allowed
                        ));
                    }
                }
            }
        }

        errors
    }
}

/// Rule that validates numeric values are within valid IEEE 754 range for IBM conversion.
///
/// IBM floating-point format has different range limitations than IEEE 754.
/// This rule warns about values that may lose precision or be out of range.
pub struct NumericRangeRule;

impl ValidationRule for NumericRangeRule {
    fn name(&self) -> &'static str {
        "NumericRangeRule"
    }

    fn validate_column(
        &self,
        _column: &XptColumn,
        _index: usize,
        _dataset_name: &str,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        // Currently no specific column-level numeric range checks
        // The actual value checks happen in validate_dataset
        Vec::new()
    }

    fn validate_dataset(
        &self,
        dataset: &XptDataset,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // IBM float has a smaller exponent range than IEEE 754
        // Maximum magnitude is approximately 7.2e75
        // Minimum non-zero magnitude is approximately 5.4e-79
        const IBM_MAX_MAGNITUDE: f64 = 7.2e75;
        const IBM_MIN_MAGNITUDE: f64 = 5.4e-79;

        for (row_idx, row) in dataset.rows.iter().enumerate() {
            for (col_idx, value) in row.iter().enumerate() {
                if col_idx >= dataset.columns.len() {
                    continue;
                }

                let column = &dataset.columns[col_idx];
                if let XptValue::Num(num_val) = value {
                    if let Some(n) = num_val.value() {
                        let abs_val = n.abs();

                        if abs_val > IBM_MAX_MAGNITUDE {
                            errors.push(ValidationError::new(
                                ValidationErrorCode::CharacterValueTooLong, // Reusing code for now
                                format!(
                                    "Numeric value {} in column '{}' at row {} exceeds IBM float maximum magnitude",
                                    n, column.name, row_idx
                                ),
                                ErrorLocation::Value {
                                    dataset: dataset.name.clone(),
                                    column: column.name.clone(),
                                    row: row_idx,
                                },
                                Severity::Error,
                            ));
                        } else if abs_val > 0.0 && abs_val < IBM_MIN_MAGNITUDE {
                            errors.push(ValidationError::new(
                                ValidationErrorCode::CharacterValueTooLong, // Reusing code for now
                                format!(
                                    "Numeric value {} in column '{}' at row {} is below IBM float minimum magnitude (will become zero)",
                                    n, column.name, row_idx
                                ),
                                ErrorLocation::Value {
                                    dataset: dataset.name.clone(),
                                    column: column.name.clone(),
                                    row: row_idx,
                                },
                                Severity::Warning,
                            ));
                        }
                    }
                }
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::ActionLevel;

    fn create_test_dataset(values: Vec<XptValue>) -> XptDataset {
        let mut dataset = XptDataset::new("TEST");
        dataset.columns.push(XptColumn::character("VAR1", 10));
        dataset.rows.push(values);
        dataset
    }

    fn make_context() -> ValidationContext {
        ValidationContext::new(crate::XptVersion::V5, ActionLevel::Warn)
    }

    #[test]
    fn test_character_length_valid() {
        let rule = CharacterLengthRule;
        let dataset = create_test_dataset(vec![XptValue::character("short")]);
        let ctx = make_context();

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_character_length_too_long() {
        let rule = CharacterLengthRule;
        let dataset = create_test_dataset(vec![XptValue::character("this is way too long")]);
        let ctx = make_context();

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationErrorCode::CharacterValueTooLong);
    }

    #[test]
    fn test_ascii_value_valid() {
        let rule = AsciiValueRule;
        let dataset = create_test_dataset(vec![XptValue::character("ASCII text")]);
        let ctx = make_context();

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_ascii_value_non_ascii() {
        let rule = AsciiValueRule;
        let dataset = create_test_dataset(vec![XptValue::character("Héllo")]);
        let ctx = make_context();

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationErrorCode::NonAsciiValue);
        assert!(errors[0].message.contains("é"));
    }
}
