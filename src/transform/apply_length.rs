//! Length application transform (xportr_length equivalent).
//!
//! Applies variable lengths from the specification to the dataset columns.
//! For character variables, this may involve truncating values that exceed
//! the specified length.

use crate::error::TransformError;
use crate::spec::DatasetSpec;
use crate::types::{XptDataset, XptType, XptValue};
use crate::validation::ActionLevel;

use super::config::{MismatchAction, TransformConfig};
use super::report::LengthChange;

/// Configuration for length application.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplyLengthConfig {
    /// Base transform configuration.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub base: TransformConfig,

    /// Whether to truncate character values that exceed the specified length.
    pub truncate_values: bool,
}

impl Default for ApplyLengthConfig {
    fn default() -> Self {
        Self {
            base: TransformConfig::default(),
            truncate_values: true,
        }
    }
}

impl ApplyLengthConfig {
    /// Create a new config with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific action level.
    #[must_use]
    pub fn with_action(mut self, action: ActionLevel) -> Self {
        self.base.action = action;
        self
    }

    /// Set whether to truncate values.
    #[must_use]
    pub fn with_truncate(mut self, truncate: bool) -> Self {
        self.truncate_values = truncate;
        self
    }
}

/// Result of length application operation.
#[derive(Debug)]
pub struct ApplyLengthResult {
    /// The modified dataset.
    pub dataset: XptDataset,
    /// Record of length changes.
    pub changes: Vec<LengthChange>,
    /// Warning messages.
    pub warnings: Vec<String>,
}

/// Apply lengths from specification to dataset columns.
///
/// This is equivalent to R's `xportr_length()` function. It sets the column
/// lengths to match the specification and optionally truncates values that
/// exceed the new length.
///
/// # Arguments
///
/// * `dataset` - The dataset to transform
/// * `spec` - The specification defining expected lengths
/// * `config` - Configuration for the transform
///
/// # Errors
///
/// Returns error if `config.base.action` is `ActionLevel::Stop` and issues
/// are found.
///
/// # Example
///
/// ```
/// use xportrs::{XptDataset, XptColumn, XptValue, DatasetSpec, VariableSpec};
/// use xportrs::transform::{apply_length, ApplyLengthConfig};
///
/// let mut dataset = XptDataset::with_columns("DM", vec![
///     XptColumn::character("NAME", 100),  // Too long
/// ]);
/// dataset.add_row(vec![XptValue::character("John Smith")]);
///
/// let spec = DatasetSpec::new("DM")
///     .add_variable(VariableSpec::character("NAME", 20));
///
/// let result = apply_length(dataset, &spec, ApplyLengthConfig::default()).unwrap();
/// assert_eq!(result.dataset.columns[0].length, 20);
/// ```
pub fn apply_length(
    mut dataset: XptDataset,
    spec: &DatasetSpec,
    config: ApplyLengthConfig,
) -> Result<ApplyLengthResult, TransformError> {
    let mut changes = Vec::new();
    let mut warnings = Vec::new();

    // Build a map of spec variables by name for quick lookup
    let spec_vars: std::collections::HashMap<_, _> = spec
        .variables
        .iter()
        .map(|v| (v.name.to_uppercase(), v))
        .collect();

    // Check for variables in data not in spec
    for col in &dataset.columns {
        if !spec_vars.contains_key(&col.name) {
            let msg = format!("Variable '{}' not found in specification", col.name);
            match config.base.variable_not_in_spec {
                MismatchAction::Error => {
                    if config.base.should_stop() {
                        return Err(TransformError::variable_not_in_spec(&col.name));
                    }
                }
                MismatchAction::Warn => {
                    warnings.push(msg);
                }
                MismatchAction::Ignore | MismatchAction::Remove => {}
            }
        }
    }

    // Process each column that exists in spec
    for (col_idx, col) in dataset.columns.iter_mut().enumerate() {
        let Some(var_spec) = spec_vars.get(&col.name) else {
            continue;
        };

        // Only apply length if specified in spec
        let Some(new_length) = var_spec.length else {
            continue;
        };

        let old_length = col.length;

        // Check if length change is needed
        if old_length != new_length {
            let mut change = LengthChange::new(&col.name, old_length, new_length);

            // For character variables, potentially truncate values
            if col.data_type == XptType::Char && config.truncate_values && new_length < old_length {
                let truncated = truncate_column_values(&mut dataset.rows, col_idx, new_length);
                change.truncated_values = truncated;

                if truncated > 0 && config.base.should_report() {
                    warnings.push(format!(
                        "Variable '{}': {} value(s) truncated to {} characters",
                        col.name, truncated, new_length
                    ));
                }
            }

            col.length = new_length;
            changes.push(change);
        }
    }

    Ok(ApplyLengthResult {
        dataset,
        changes,
        warnings,
    })
}

/// Truncate character values in a column to the specified length.
///
/// Returns the number of values that were truncated.
fn truncate_column_values(rows: &mut [Vec<XptValue>], col_idx: usize, max_length: u16) -> usize {
    let max_len = max_length as usize;
    let mut truncated = 0;

    for row in rows.iter_mut() {
        if col_idx >= row.len() {
            continue;
        }

        if let XptValue::Char(ref mut s) = row[col_idx] {
            if s.len() > max_len {
                // Truncate to max_length characters
                // Be careful with UTF-8: truncate at char boundaries
                let truncated_str: String = s.chars().take(max_len).collect();
                *s = truncated_str;
                truncated += 1;
            }
        }
    }

    truncated
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::VariableSpec;
    use crate::types::XptColumn;

    #[test]
    fn test_apply_length_basic() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::character("NAME", 100)],
        );
        dataset.add_row(vec![XptValue::character("John")]);

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::character("NAME", 20));

        let result = apply_length(dataset, &spec, ApplyLengthConfig::default()).unwrap();

        assert_eq!(result.dataset.columns[0].length, 20);
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].old_length, 100);
        assert_eq!(result.changes[0].new_length, 20);
    }

    #[test]
    fn test_apply_length_with_truncation() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::character("NAME", 100)],
        );
        dataset.add_row(vec![XptValue::character("This is a very long name that will be truncated")]);
        dataset.add_row(vec![XptValue::character("Short")]);

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::character("NAME", 10));

        let result = apply_length(dataset, &spec, ApplyLengthConfig::default()).unwrap();

        // First value should be truncated
        assert_eq!(result.dataset.rows[0][0].as_str(), Some("This is a "));
        // Second value should be unchanged
        assert_eq!(result.dataset.rows[1][0].as_str(), Some("Short"));
        // Should report truncation
        assert_eq!(result.changes[0].truncated_values, 1);
    }

    #[test]
    fn test_apply_length_no_truncation() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::character("NAME", 100)],
        );
        dataset.add_row(vec![XptValue::character("Long value here")]);

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::character("NAME", 10));

        let config = ApplyLengthConfig::default().with_truncate(false);
        let result = apply_length(dataset, &spec, config).unwrap();

        // Length metadata should change, but value should not be truncated
        assert_eq!(result.dataset.columns[0].length, 10);
        assert_eq!(result.dataset.rows[0][0].as_str(), Some("Long value here"));
        assert_eq!(result.changes[0].truncated_values, 0);
    }

    #[test]
    fn test_apply_length_numeric_column() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE")],
        );
        dataset.add_row(vec![XptValue::numeric(25.0)]);

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::numeric("AGE").with_length(4));

        let result = apply_length(dataset, &spec, ApplyLengthConfig::default()).unwrap();

        // Numeric column length should change (even though it's unusual)
        assert_eq!(result.dataset.columns[0].length, 4);
    }

    #[test]
    fn test_apply_length_no_change_needed() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::character("NAME", 20)],
        );
        dataset.add_row(vec![XptValue::character("John")]);

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::character("NAME", 20));

        let result = apply_length(dataset, &spec, ApplyLengthConfig::default()).unwrap();

        // No changes should be recorded
        assert!(result.changes.is_empty());
    }

    #[test]
    fn test_apply_length_variable_not_in_spec() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![
                XptColumn::character("NAME", 20),
                XptColumn::character("EXTRA", 10),
            ],
        );
        dataset.add_row(vec![XptValue::character("John"), XptValue::character("X")]);

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::character("NAME", 30));

        let result = apply_length(dataset, &spec, ApplyLengthConfig::default()).unwrap();

        // Should warn about EXTRA
        assert!(result.warnings.iter().any(|w| w.contains("EXTRA")));
        // EXTRA length should be unchanged
        assert_eq!(result.dataset.columns[1].length, 10);
    }

    #[test]
    fn test_apply_length_length_increase() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::character("NAME", 10)],
        );
        dataset.add_row(vec![XptValue::character("John")]);

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::character("NAME", 50));

        let result = apply_length(dataset, &spec, ApplyLengthConfig::default()).unwrap();

        // Length should increase without any truncation
        assert_eq!(result.dataset.columns[0].length, 50);
        assert_eq!(result.changes[0].truncated_values, 0);
    }
}
