//! Order application transform (`xportr_order` equivalent).
//!
//! Reorders variables (columns) in the dataset to match the specification order.

use crate::error::TransformError;
use crate::spec::DatasetSpec;
use crate::types::XptDataset;
use crate::validation::ActionLevel;

use super::config::{MismatchAction, TransformConfig};
use super::report::OrderChange;

/// Configuration for order application.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplyOrderConfig {
    /// Base transform configuration.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub base: TransformConfig,

    /// Where to place variables not in the specification.
    pub unmatched_position: UnmatchedPosition,
}

/// Where to place variables that are in the data but not in the specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnmatchedPosition {
    /// Place unmatched variables at the end (after all spec variables).
    #[default]
    End,
    /// Place unmatched variables at the beginning (before all spec variables).
    Start,
    /// Remove unmatched variables from the output.
    Remove,
}

impl Default for ApplyOrderConfig {
    fn default() -> Self {
        Self {
            base: TransformConfig::default(),
            unmatched_position: UnmatchedPosition::End,
        }
    }
}

impl ApplyOrderConfig {
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

    /// Set where to place unmatched variables.
    #[must_use]
    pub fn with_unmatched_position(mut self, position: UnmatchedPosition) -> Self {
        self.unmatched_position = position;
        self
    }
}

/// Result of order application operation.
#[derive(Debug)]
pub struct ApplyOrderResult {
    /// The modified dataset.
    pub dataset: XptDataset,
    /// Record of order changes.
    pub changes: Vec<OrderChange>,
    /// Warning messages.
    pub warnings: Vec<String>,
}

/// Apply variable ordering from specification to dataset.
///
/// This is equivalent to R's `xportr_order()` function. It reorders the columns
/// to match the order specified in the specification (using the `order` field
/// or the order of variables in the spec).
///
/// # Arguments
///
/// * `dataset` - The dataset to transform
/// * `spec` - The specification defining expected order
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
/// use xportrs::transform::{apply_order, ApplyOrderConfig};
///
/// let mut dataset = XptDataset::with_columns("DM", vec![
///     XptColumn::character("SEX", 1),  // Wrong order
///     XptColumn::numeric("AGE"),
/// ]);
/// dataset.add_row(vec![XptValue::character("M"), XptValue::numeric(25.0)]);
///
/// let spec = DatasetSpec::new("DM")
///     .add_variable(VariableSpec::numeric("AGE").with_order(1))
///     .add_variable(VariableSpec::character("SEX", 1).with_order(2));
///
/// let result = apply_order(dataset, &spec, ApplyOrderConfig::default()).unwrap();
/// assert_eq!(result.dataset.columns[0].name, "AGE");
/// assert_eq!(result.dataset.columns[1].name, "SEX");
/// ```
pub fn apply_order(
    mut dataset: XptDataset,
    spec: &DatasetSpec,
    config: ApplyOrderConfig,
) -> Result<ApplyOrderResult, TransformError> {
    let mut changes = Vec::new();
    let mut warnings = Vec::new();

    // Build a map of current positions
    let current_positions: std::collections::HashMap<_, _> = dataset
        .columns
        .iter()
        .enumerate()
        .map(|(i, c)| (c.name.clone(), i))
        .collect();

    // Build ordering from spec
    // If variables have explicit order, use that; otherwise use position in spec
    let mut spec_order: Vec<_> = spec
        .variables
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let order = v.order.unwrap_or(i + 1);
            (v.name.to_uppercase(), order)
        })
        .collect();

    // Sort by order value
    spec_order.sort_by_key(|(_, order)| *order);

    // Build new column order
    let mut new_order: Vec<usize> = Vec::new();
    let mut used_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut unmatched_indices: Vec<usize> = Vec::new();

    // First, add columns in spec order
    for (var_name, _) in &spec_order {
        if let Some(&idx) = current_positions.get(var_name) {
            new_order.push(idx);
            used_indices.insert(idx);
        } else {
            // Variable in spec but not in data
            let msg = format!("Variable '{}' in specification not found in data", var_name);
            match config.base.variable_not_in_data {
                MismatchAction::Error => {
                    if config.base.should_stop() {
                        return Err(TransformError::variable_not_in_data(var_name));
                    }
                }
                MismatchAction::Warn => {
                    warnings.push(msg);
                }
                MismatchAction::Ignore | MismatchAction::Remove => {}
            }
        }
    }

    // Identify columns not in spec
    for (idx, col) in dataset.columns.iter().enumerate() {
        if !used_indices.contains(&idx) {
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

            if config.unmatched_position != UnmatchedPosition::Remove {
                unmatched_indices.push(idx);
            }
        }
    }

    // Add unmatched columns based on config
    match config.unmatched_position {
        UnmatchedPosition::Start => {
            new_order = unmatched_indices.into_iter().chain(new_order).collect();
        }
        UnmatchedPosition::End => {
            new_order.extend(unmatched_indices);
        }
        UnmatchedPosition::Remove => {
            // Already excluded
        }
    }

    // Check if reordering is needed
    let needs_reorder = new_order
        .iter()
        .enumerate()
        .any(|(new_pos, &old_pos)| new_pos != old_pos);

    if !needs_reorder && new_order.len() == dataset.columns.len() {
        // No changes needed
        return Ok(ApplyOrderResult {
            dataset,
            changes,
            warnings,
        });
    }

    // Record changes
    for (new_pos, &old_pos) in new_order.iter().enumerate() {
        if new_pos != old_pos {
            let change = OrderChange::new(&dataset.columns[old_pos].name, old_pos, new_pos);
            changes.push(change);
        }
    }

    // Reorder columns
    let new_columns: Vec<_> = new_order
        .iter()
        .map(|&idx| dataset.columns[idx].clone())
        .collect();

    // Reorder values in each row
    let new_rows: Vec<Vec<_>> = dataset
        .rows
        .iter()
        .map(|row| new_order.iter().map(|&idx| row[idx].clone()).collect())
        .collect();

    dataset.columns = new_columns;
    dataset.rows = new_rows;

    Ok(ApplyOrderResult {
        dataset,
        changes,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::VariableSpec;
    use crate::types::{XptColumn, XptValue};

    #[test]
    fn test_apply_order_basic() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![
                XptColumn::character("C", 1),
                XptColumn::character("A", 1),
                XptColumn::character("B", 1),
            ],
        );
        dataset.add_row(vec![
            XptValue::character("c"),
            XptValue::character("a"),
            XptValue::character("b"),
        ]);

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::character("A", 1).with_order(1))
            .add_variable(VariableSpec::character("B", 1).with_order(2))
            .add_variable(VariableSpec::character("C", 1).with_order(3));

        let result = apply_order(dataset, &spec, ApplyOrderConfig::default()).unwrap();

        assert_eq!(result.dataset.columns[0].name, "A");
        assert_eq!(result.dataset.columns[1].name, "B");
        assert_eq!(result.dataset.columns[2].name, "C");

        // Check values are also reordered
        assert_eq!(result.dataset.rows[0][0].as_str(), Some("a"));
        assert_eq!(result.dataset.rows[0][1].as_str(), Some("b"));
        assert_eq!(result.dataset.rows[0][2].as_str(), Some("c"));
    }

    #[test]
    fn test_apply_order_by_spec_position() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::character("B", 1), XptColumn::character("A", 1)],
        );
        dataset.add_row(vec![XptValue::character("b"), XptValue::character("a")]);

        // No explicit order - use position in spec
        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::character("A", 1))
            .add_variable(VariableSpec::character("B", 1));

        let result = apply_order(dataset, &spec, ApplyOrderConfig::default()).unwrap();

        assert_eq!(result.dataset.columns[0].name, "A");
        assert_eq!(result.dataset.columns[1].name, "B");
    }

    #[test]
    fn test_apply_order_no_change_needed() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::character("A", 1), XptColumn::character("B", 1)],
        );
        dataset.add_row(vec![XptValue::character("a"), XptValue::character("b")]);

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::character("A", 1).with_order(1))
            .add_variable(VariableSpec::character("B", 1).with_order(2));

        let result = apply_order(dataset, &spec, ApplyOrderConfig::default()).unwrap();

        // No changes should be recorded
        assert!(result.changes.is_empty());
    }

    #[test]
    fn test_apply_order_unmatched_at_end() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![
                XptColumn::character("EXTRA", 1),
                XptColumn::character("A", 1),
            ],
        );
        dataset.add_row(vec![XptValue::character("x"), XptValue::character("a")]);

        let spec = DatasetSpec::new("TEST").add_variable(VariableSpec::character("A", 1));

        let config = ApplyOrderConfig::default().with_unmatched_position(UnmatchedPosition::End);
        let result = apply_order(dataset, &spec, config).unwrap();

        // A should be first, EXTRA at end
        assert_eq!(result.dataset.columns[0].name, "A");
        assert_eq!(result.dataset.columns[1].name, "EXTRA");
    }

    #[test]
    fn test_apply_order_unmatched_at_start() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![
                XptColumn::character("A", 1),
                XptColumn::character("EXTRA", 1),
            ],
        );
        dataset.add_row(vec![XptValue::character("a"), XptValue::character("x")]);

        let spec = DatasetSpec::new("TEST").add_variable(VariableSpec::character("A", 1));

        let config = ApplyOrderConfig::default().with_unmatched_position(UnmatchedPosition::Start);
        let result = apply_order(dataset, &spec, config).unwrap();

        // EXTRA should be first, A after
        assert_eq!(result.dataset.columns[0].name, "EXTRA");
        assert_eq!(result.dataset.columns[1].name, "A");
    }

    #[test]
    fn test_apply_order_remove_unmatched() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![
                XptColumn::character("A", 1),
                XptColumn::character("EXTRA", 1),
            ],
        );
        dataset.add_row(vec![XptValue::character("a"), XptValue::character("x")]);

        let spec = DatasetSpec::new("TEST").add_variable(VariableSpec::character("A", 1));

        let config = ApplyOrderConfig::default().with_unmatched_position(UnmatchedPosition::Remove);
        let result = apply_order(dataset, &spec, config).unwrap();

        // Only A should remain
        assert_eq!(result.dataset.columns.len(), 1);
        assert_eq!(result.dataset.columns[0].name, "A");
        assert_eq!(result.dataset.rows[0].len(), 1);
    }

    #[test]
    fn test_apply_order_variable_not_in_data() {
        let mut dataset = XptDataset::with_columns("TEST", vec![XptColumn::character("A", 1)]);
        dataset.add_row(vec![XptValue::character("a")]);

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::character("A", 1))
            .add_variable(VariableSpec::character("MISSING", 1));

        let result = apply_order(dataset, &spec, ApplyOrderConfig::default()).unwrap();

        // Should warn about MISSING
        assert!(result.warnings.iter().any(|w| w.contains("MISSING")));
    }

    #[test]
    fn test_apply_order_case_insensitive() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::character("AGE", 1)], // Uppercase
        );
        dataset.add_row(vec![XptValue::character("25")]);

        let spec = DatasetSpec::new("TEST").add_variable(VariableSpec::character("age", 1)); // Lowercase

        let result = apply_order(dataset, &spec, ApplyOrderConfig::default()).unwrap();

        // Should match case-insensitively
        assert_eq!(result.dataset.columns[0].name, "AGE");
    }
}
