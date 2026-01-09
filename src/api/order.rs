//! Column ordering operations.

use polars::prelude::*;

use crate::config::ActionLevel;
use crate::error::XptError;
use crate::report::{OrderChange, OrderReport};
use crate::spec::DatasetSpec;

/// Reorder columns to match specification order.
///
/// This function reorders the DataFrame columns to match the order defined
/// in the specification. Variables not in the spec are placed at the end.
///
/// # Arguments
///
/// * `df` - DataFrame to transform
/// * `spec` - Dataset specification defining expected order
/// * `action` - How to handle order mismatches
///
/// # Returns
///
/// A tuple of (reordered DataFrame, OrderReport).
///
/// # Errors
///
/// Returns an error if `action` is `ActionLevel::Stop` and there are variables
/// in the spec that don't exist in the data.
///
/// # Example
///
/// ```no_run
/// use xportrs::{xportrs_order, ActionLevel, DatasetSpec, VariableSpec};
/// use polars::prelude::*;
///
/// let df = DataFrame::default();
/// let spec = DatasetSpec::new("DM")
///     .add_variable(VariableSpec::character("USUBJID", 20).with_order(1))
///     .add_variable(VariableSpec::numeric("AGE").with_order(2));
///
/// let (df, report) = xportrs_order(df, &spec, ActionLevel::Warn).unwrap();
/// ```
pub fn xportrs_order(
    df: DataFrame,
    spec: &DatasetSpec,
    action: ActionLevel,
) -> Result<(DataFrame, OrderReport), XptError> {
    let mut report = OrderReport::new();

    // Get current column names (uppercase for matching)
    let current_cols: Vec<_> = df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let current_upper: Vec<_> = current_cols.iter().map(|s| s.to_uppercase()).collect();

    // Build ordered list from spec
    let mut ordered_vars: Vec<_> = spec
        .variables
        .iter()
        .filter(|v| v.order.is_some())
        .collect();
    ordered_vars.sort_by_key(|v| v.order);

    // Build target order
    let mut target_order: Vec<String> = Vec::new();
    let mut used_cols: std::collections::HashSet<String> = std::collections::HashSet::new();

    // First, add columns in spec order
    for var in &ordered_vars {
        let var_upper = var.name.to_uppercase();

        // Find the original column name (preserving case)
        if let Some(idx) = current_upper.iter().position(|n| *n == var_upper) {
            let original_name = &current_cols[idx];
            if !used_cols.contains(original_name) {
                target_order.push(original_name.clone());
                used_cols.insert(original_name.clone());
            }
        } else {
            // Variable in spec but not in data
            report.missing_from_data.push(var.name.clone());
            if action.is_error() {
                return Err(XptError::InvalidFormat {
                    message: format!("Variable '{}' in specification not found in data", var.name),
                });
            }
        }
    }

    // Add columns from spec without order
    for var in &spec.variables {
        if var.order.is_none() {
            let var_upper = var.name.to_uppercase();
            if let Some(idx) = current_upper.iter().position(|n| *n == var_upper) {
                let original_name = &current_cols[idx];
                if !used_cols.contains(original_name) {
                    target_order.push(original_name.clone());
                    used_cols.insert(original_name.clone());
                }
            }
        }
    }

    // Add remaining columns not in spec
    for col_name in &current_cols {
        if !used_cols.contains(col_name) {
            report.missing_from_spec.push(col_name.clone());
            target_order.push(col_name.clone());
            used_cols.insert(col_name.clone());
        }
    }

    // Record changes
    for (new_pos, col_name) in target_order.iter().enumerate() {
        if let Some(old_pos) = current_cols.iter().position(|n| n == col_name) {
            if old_pos != new_pos {
                report
                    .changes
                    .push(OrderChange::new(col_name, old_pos, new_pos));
            }
        }
    }

    // Reorder DataFrame
    let result = if report.has_changes() {
        df.select(target_order.iter().map(|s| s.as_str()))
            .map_err(|e| XptError::InvalidFormat {
                message: e.to_string(),
            })?
    } else {
        df
    };

    Ok((result, report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::VariableSpec;

    #[test]
    fn test_order_empty_df() {
        let df = DataFrame::default();
        let spec = DatasetSpec::new("TEST");
        let (result, report) = xportrs_order(df, &spec, ActionLevel::Warn).unwrap();
        assert!(!report.has_changes());
        assert_eq!(result.width(), 0);
    }

    #[test]
    fn test_order_with_spec() {
        let df = df![
            "B" => [1, 2],
            "A" => [3, 4],
        ]
        .unwrap();

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::numeric("A").with_order(1))
            .add_variable(VariableSpec::numeric("B").with_order(2));

        let (result, report) = xportrs_order(df, &spec, ActionLevel::Warn).unwrap();

        assert!(report.has_changes());
        assert_eq!(result.get_column_names()[0].as_str(), "A");
        assert_eq!(result.get_column_names()[1].as_str(), "B");
    }
}
