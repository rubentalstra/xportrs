//! Label application transform (`xportr_label` equivalent).
//!
//! Applies variable labels from the specification to the dataset columns.

use crate::error::TransformError;
use crate::spec::DatasetSpec;
use crate::types::XptDataset;
use crate::validation::ActionLevel;

use super::config::{MismatchAction, TransformConfig};
use super::report::LabelChange;

/// Configuration for label application.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplyLabelConfig {
    /// Base transform configuration.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub base: TransformConfig,
}

impl ApplyLabelConfig {
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
}

/// Result of label application operation.
#[derive(Debug)]
pub struct ApplyLabelResult {
    /// The modified dataset.
    pub dataset: XptDataset,
    /// Record of label changes.
    pub changes: Vec<LabelChange>,
    /// Warning messages.
    pub warnings: Vec<String>,
}

/// Apply labels from specification to dataset columns.
///
/// This is equivalent to R's `xportr_label()` function. It sets the column
/// labels to match the specification.
///
/// # Arguments
///
/// * `dataset` - The dataset to transform
/// * `spec` - The specification defining expected labels
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
/// use xportrs::{XptDataset, XptColumn, DatasetSpec, VariableSpec};
/// use xportrs::transform::{apply_label, ApplyLabelConfig};
///
/// let dataset = XptDataset::with_columns("DM", vec![
///     XptColumn::numeric("AGE"),
///     XptColumn::character("SEX", 1),
/// ]);
///
/// let spec = DatasetSpec::new("DM")
///     .add_variable(VariableSpec::numeric("AGE").with_label("Age in Years"))
///     .add_variable(VariableSpec::character("SEX", 1).with_label("Sex"));
///
/// let result = apply_label(dataset, &spec, ApplyLabelConfig::default()).unwrap();
/// assert_eq!(result.dataset.columns[0].label, Some("Age in Years".to_string()));
/// assert_eq!(result.dataset.columns[1].label, Some("Sex".to_string()));
/// ```
pub fn apply_label(
    mut dataset: XptDataset,
    spec: &DatasetSpec,
    config: ApplyLabelConfig,
) -> Result<ApplyLabelResult, TransformError> {
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
    for col in &mut dataset.columns {
        let Some(var_spec) = spec_vars.get(&col.name) else {
            continue;
        };

        // Only apply label if specified in spec
        let Some(new_label) = &var_spec.label else {
            continue;
        };

        let old_label = col.label.clone();

        // Check if label change is needed
        if col.label.as_ref() != Some(new_label) {
            let change = LabelChange::new(&col.name, old_label, new_label);
            col.label = Some(new_label.clone());
            changes.push(change);
        }
    }

    Ok(ApplyLabelResult {
        dataset,
        changes,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::VariableSpec;
    use crate::types::XptColumn;

    #[test]
    fn test_apply_label_basic() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE"), XptColumn::character("NAME", 20)],
        );

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::numeric("AGE").with_label("Age in Years"))
            .add_variable(VariableSpec::character("NAME", 20).with_label("Subject Name"));

        let result = apply_label(dataset, &spec, ApplyLabelConfig::default()).unwrap();

        assert_eq!(
            result.dataset.columns[0].label,
            Some("Age in Years".to_string())
        );
        assert_eq!(
            result.dataset.columns[1].label,
            Some("Subject Name".to_string())
        );
        assert_eq!(result.changes.len(), 2);
    }

    #[test]
    fn test_apply_label_replace_existing() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE").with_label("Old Label")],
        );

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::numeric("AGE").with_label("New Label"));

        let result = apply_label(dataset, &spec, ApplyLabelConfig::default()).unwrap();

        assert_eq!(
            result.dataset.columns[0].label,
            Some("New Label".to_string())
        );
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].old_label, Some("Old Label".to_string()));
        assert_eq!(result.changes[0].new_label, "New Label");
    }

    #[test]
    fn test_apply_label_no_change_needed() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE").with_label("Age in Years")],
        );

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::numeric("AGE").with_label("Age in Years"));

        let result = apply_label(dataset, &spec, ApplyLabelConfig::default()).unwrap();

        // No changes should be recorded
        assert!(result.changes.is_empty());
    }

    #[test]
    fn test_apply_label_spec_without_label() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE").with_label("Old Label")],
        );

        // Spec without label - should not change existing label
        let spec = DatasetSpec::new("TEST").add_variable(VariableSpec::numeric("AGE"));

        let result = apply_label(dataset, &spec, ApplyLabelConfig::default()).unwrap();

        // Label should remain unchanged
        assert_eq!(
            result.dataset.columns[0].label,
            Some("Old Label".to_string())
        );
        assert!(result.changes.is_empty());
    }

    #[test]
    fn test_apply_label_variable_not_in_spec() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE"), XptColumn::numeric("EXTRA")],
        );

        let spec =
            DatasetSpec::new("TEST").add_variable(VariableSpec::numeric("AGE").with_label("Age"));

        let result = apply_label(dataset, &spec, ApplyLabelConfig::default()).unwrap();

        // Should warn about EXTRA
        assert!(result.warnings.iter().any(|w| w.contains("EXTRA")));
        // EXTRA label should remain None
        assert!(result.dataset.columns[1].label.is_none());
    }

    #[test]
    fn test_apply_label_case_insensitive() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE")], // Uppercase
        );

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::numeric("age").with_label("Age in Years")); // Lowercase

        let result = apply_label(dataset, &spec, ApplyLabelConfig::default()).unwrap();

        // Should match case-insensitively
        assert_eq!(
            result.dataset.columns[0].label,
            Some("Age in Years".to_string())
        );
    }
}
