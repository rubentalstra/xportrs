//! Format application transform (xportr_format equivalent).
//!
//! Applies SAS formats from the specification to the dataset columns.

use crate::error::TransformError;
use crate::spec::DatasetSpec;
use crate::types::XptDataset;
use crate::validation::ActionLevel;

use super::config::{MismatchAction, TransformConfig};
use super::report::FormatChange;

/// Configuration for format application.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplyFormatConfig {
    /// Base transform configuration.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub base: TransformConfig,
}

impl Default for ApplyFormatConfig {
    fn default() -> Self {
        Self {
            base: TransformConfig::default(),
        }
    }
}

impl ApplyFormatConfig {
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

/// Result of format application operation.
#[derive(Debug)]
pub struct ApplyFormatResult {
    /// The modified dataset.
    pub dataset: XptDataset,
    /// Record of format changes.
    pub changes: Vec<FormatChange>,
    /// Warning messages.
    pub warnings: Vec<String>,
}

/// Apply SAS formats from specification to dataset columns.
///
/// This is equivalent to R's `xportr_format()` function. It sets the column
/// formats (and optionally informats) to match the specification.
///
/// # Arguments
///
/// * `dataset` - The dataset to transform
/// * `spec` - The specification defining expected formats
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
/// use xportrs::{XptDataset, XptColumn, DatasetSpec, VariableSpec, FormatSpec};
/// use xportrs::transform::{apply_format, ApplyFormatConfig};
///
/// let dataset = XptDataset::with_columns("DM", vec![
///     XptColumn::numeric("AGE"),
///     XptColumn::numeric("DTHDT"),
/// ]);
///
/// let spec = DatasetSpec::new("DM")
///     .add_variable(VariableSpec::numeric("AGE").with_format(FormatSpec::best(8)))
///     .add_variable(VariableSpec::numeric("DTHDT").with_format(FormatSpec::date9()));
///
/// let result = apply_format(dataset, &spec, ApplyFormatConfig::default()).unwrap();
/// assert_eq!(result.dataset.columns[0].format, Some("BEST".to_string()));
/// assert_eq!(result.dataset.columns[1].format, Some("DATE".to_string()));
/// ```
pub fn apply_format(
    mut dataset: XptDataset,
    spec: &DatasetSpec,
    config: ApplyFormatConfig,
) -> Result<ApplyFormatResult, TransformError> {
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
    for col in dataset.columns.iter_mut() {
        let Some(var_spec) = spec_vars.get(&col.name) else {
            continue;
        };

        // Apply format if specified in spec
        if let Some(format_spec) = &var_spec.format {
            let old_format = col.format.clone();
            let new_format_name = format_spec.name.clone();

            // Check if format change is needed
            if col.format != new_format_name {
                let change = FormatChange::new(
                    &col.name,
                    old_format,
                    format_spec.to_string(),
                );

                col.format = new_format_name;
                col.format_length = format_spec.width;
                col.format_decimals = format_spec.decimals;

                changes.push(change);
            }
        }

        // Apply informat if specified in spec
        if let Some(informat_spec) = &var_spec.informat {
            let old_informat = col.informat.clone();
            let new_informat_name = informat_spec.name.clone();

            // Only update if different (we don't track informat changes separately)
            if col.informat != new_informat_name {
                col.informat = new_informat_name;
                col.informat_length = informat_spec.width;
                col.informat_decimals = informat_spec.decimals;

                // Add to changes if there was a previous informat
                if old_informat.is_some() && config.base.should_report() {
                    warnings.push(format!(
                        "Variable '{}': informat changed from {:?} to {}",
                        col.name, old_informat, informat_spec
                    ));
                }
            }
        }
    }

    Ok(ApplyFormatResult {
        dataset,
        changes,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::VariableSpec;
    use crate::types::{FormatSpec, XptColumn};

    #[test]
    fn test_apply_format_basic() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![
                XptColumn::numeric("AGE"),
                XptColumn::numeric("DATE"),
            ],
        );

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::numeric("AGE").with_format(FormatSpec::best(8)))
            .add_variable(VariableSpec::numeric("DATE").with_format(FormatSpec::date9()));

        let result = apply_format(dataset, &spec, ApplyFormatConfig::default()).unwrap();

        assert_eq!(result.dataset.columns[0].format, Some("BEST".to_string()));
        assert_eq!(result.dataset.columns[0].format_length, 8);

        assert_eq!(result.dataset.columns[1].format, Some("DATE".to_string()));
        assert_eq!(result.dataset.columns[1].format_length, 9);

        assert_eq!(result.changes.len(), 2);
    }

    #[test]
    fn test_apply_format_with_decimals() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("VALUE")],
        );

        let spec = DatasetSpec::new("TEST").add_variable(
            VariableSpec::numeric("VALUE").with_format(FormatSpec::with_decimals("BEST", 12, 2)),
        );

        let result = apply_format(dataset, &spec, ApplyFormatConfig::default()).unwrap();

        assert_eq!(result.dataset.columns[0].format, Some("BEST".to_string()));
        assert_eq!(result.dataset.columns[0].format_length, 12);
        assert_eq!(result.dataset.columns[0].format_decimals, 2);
    }

    #[test]
    fn test_apply_format_replace_existing() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("DATE").with_format("DATE7", 7, 0)],
        );

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::numeric("DATE").with_format(FormatSpec::date9()));

        let result = apply_format(dataset, &spec, ApplyFormatConfig::default()).unwrap();

        assert_eq!(result.dataset.columns[0].format, Some("DATE".to_string()));
        assert_eq!(result.dataset.columns[0].format_length, 9);

        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].old_format, Some("DATE7".to_string()));
    }

    #[test]
    fn test_apply_format_no_change_needed() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE").with_format("BEST", 8, 0)],
        );

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::numeric("AGE").with_format(FormatSpec::best(8)));

        let result = apply_format(dataset, &spec, ApplyFormatConfig::default()).unwrap();

        // No changes should be recorded
        assert!(result.changes.is_empty());
    }

    #[test]
    fn test_apply_format_spec_without_format() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE").with_format("BEST", 8, 0)],
        );

        // Spec without format - should not change existing format
        let spec = DatasetSpec::new("TEST").add_variable(VariableSpec::numeric("AGE"));

        let result = apply_format(dataset, &spec, ApplyFormatConfig::default()).unwrap();

        // Format should remain unchanged
        assert_eq!(
            result.dataset.columns[0].format,
            Some("BEST".to_string())
        );
        assert!(result.changes.is_empty());
    }

    #[test]
    fn test_apply_format_variable_not_in_spec() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![
                XptColumn::numeric("AGE"),
                XptColumn::numeric("EXTRA"),
            ],
        );

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::numeric("AGE").with_format(FormatSpec::best(8)));

        let result = apply_format(dataset, &spec, ApplyFormatConfig::default()).unwrap();

        // Should warn about EXTRA
        assert!(result.warnings.iter().any(|w| w.contains("EXTRA")));
        // EXTRA format should remain None
        assert!(result.dataset.columns[1].format.is_none());
    }

    #[test]
    fn test_apply_format_character_format() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::character("NAME", 20)],
        );

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::character("NAME", 20).with_format(FormatSpec::char(20)));

        let result = apply_format(dataset, &spec, ApplyFormatConfig::default()).unwrap();

        assert_eq!(result.dataset.columns[0].format, Some("$CHAR".to_string()));
        assert_eq!(result.dataset.columns[0].format_length, 20);
    }

    #[test]
    fn test_apply_format_informat() {
        let dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("DATE")],
        );

        let spec = DatasetSpec::new("TEST").add_variable(
            VariableSpec::numeric("DATE")
                .with_format(FormatSpec::date9())
                .with_informat(FormatSpec::date9()),
        );

        let result = apply_format(dataset, &spec, ApplyFormatConfig::default()).unwrap();

        assert_eq!(result.dataset.columns[0].format, Some("DATE".to_string()));
        assert_eq!(result.dataset.columns[0].informat, Some("DATE".to_string()));
    }
}
