//! Type coercion transform (xportr_type equivalent).
//!
//! Coerces column types to match the specification. This handles converting
//! between numeric and character types as needed.

use crate::error::TransformError;
use crate::spec::DatasetSpec;
use crate::types::{MissingValue, NumericValue, XptDataset, XptType, XptValue};
use crate::validation::ActionLevel;

use super::config::{MismatchAction, TransformConfig};
use super::report::TypeConversion;

/// Configuration for type coercion.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CoerceTypeConfig {
    /// Base transform configuration.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub base: TransformConfig,
}

impl Default for CoerceTypeConfig {
    fn default() -> Self {
        Self {
            base: TransformConfig::default(),
        }
    }
}

impl CoerceTypeConfig {
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

/// Result of type coercion operation.
#[derive(Debug)]
pub struct CoerceTypeResult {
    /// The modified dataset.
    pub dataset: XptDataset,
    /// Record of type conversions performed.
    pub conversions: Vec<TypeConversion>,
    /// Warning messages.
    pub warnings: Vec<String>,
}

/// Coerce column types in a dataset to match the specification.
///
/// This is equivalent to R's `xportr_type()` function. It converts values
/// between numeric and character types as needed to match the spec.
///
/// # Type Conversions
///
/// - **Char → Num**: Attempts to parse string as number. Unparseable strings become missing.
/// - **Num → Char**: Converts number to string representation.
///
/// # Arguments
///
/// * `dataset` - The dataset to transform
/// * `spec` - The specification defining expected types
/// * `config` - Configuration for the transform
///
/// # Errors
///
/// Returns error if `config.base.action` is `ActionLevel::Stop` and type
/// mismatches are found.
///
/// # Example
///
/// ```
/// use xportrs::{XptDataset, XptColumn, XptValue, DatasetSpec, VariableSpec};
/// use xportrs::transform::{coerce_type, CoerceTypeConfig};
///
/// let mut dataset = XptDataset::with_columns("DM", vec![
///     XptColumn::character("AGE", 8),  // Wrong type - should be numeric
/// ]);
/// dataset.add_row(vec![XptValue::character("25")]);
///
/// let spec = DatasetSpec::new("DM")
///     .add_variable(VariableSpec::numeric("AGE"));
///
/// let result = coerce_type(dataset, &spec, CoerceTypeConfig::default()).unwrap();
/// assert!(result.dataset.columns[0].is_numeric());
/// ```
pub fn coerce_type(
    mut dataset: XptDataset,
    spec: &DatasetSpec,
    config: CoerceTypeConfig,
) -> Result<CoerceTypeResult, TransformError> {
    let mut conversions = Vec::new();
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

    // Check for variables in spec not in data
    for var in &spec.variables {
        let var_upper = var.name.to_uppercase();
        if dataset.column_by_name(&var_upper).is_none() {
            let msg = format!("Variable '{}' in specification not found in data", var.name);
            match config.base.variable_not_in_data {
                MismatchAction::Error => {
                    if config.base.should_stop() {
                        return Err(TransformError::variable_not_in_data(&var.name));
                    }
                }
                MismatchAction::Warn => {
                    warnings.push(msg);
                }
                MismatchAction::Ignore | MismatchAction::Remove => {}
            }
        }
    }

    // Process each column that exists in both data and spec
    for (col_idx, col) in dataset.columns.iter_mut().enumerate() {
        let Some(var_spec) = spec_vars.get(&col.name) else {
            continue;
        };

        // Check if type conversion is needed
        if col.data_type != var_spec.data_type {
            let from_type = type_description(col.data_type);
            let to_type = type_description(var_spec.data_type);

            let mut conversion = TypeConversion::new(&col.name, from_type, to_type);

            // Convert all values in this column
            let (converted, failed) =
                convert_column_values(&mut dataset.rows, col_idx, col.data_type, var_spec.data_type);

            conversion.values_converted = converted;
            conversion.values_failed = failed;

            // Update column type
            col.data_type = var_spec.data_type;

            // Update length for character columns if specified
            if var_spec.data_type == XptType::Char {
                if let Some(len) = var_spec.length {
                    col.length = len;
                }
            } else {
                // Numeric columns are always 8 bytes
                col.length = 8;
            }

            if failed > 0 && config.base.should_report() {
                warnings.push(format!(
                    "Variable '{}': {} value(s) could not be converted and became missing",
                    col.name, failed
                ));
            }

            conversions.push(conversion);
        }
    }

    Ok(CoerceTypeResult {
        dataset,
        conversions,
        warnings,
    })
}

/// Get a human-readable type description.
fn type_description(xpt_type: XptType) -> &'static str {
    match xpt_type {
        XptType::Num => "Num",
        XptType::Char => "Char",
    }
}

/// Convert values in a column from one type to another.
///
/// Returns (converted_count, failed_count).
fn convert_column_values(
    rows: &mut [Vec<XptValue>],
    col_idx: usize,
    from_type: XptType,
    to_type: XptType,
) -> (usize, usize) {
    let mut converted = 0;
    let mut failed = 0;

    for row in rows.iter_mut() {
        if col_idx >= row.len() {
            continue;
        }

        let old_value = std::mem::take(&mut row[col_idx]);
        let (new_value, success) = convert_value(old_value, from_type, to_type);
        row[col_idx] = new_value;

        if success {
            converted += 1;
        } else {
            failed += 1;
        }
    }

    (converted, failed)
}

/// Convert a single value from one type to another.
///
/// Returns (converted_value, success).
fn convert_value(value: XptValue, from_type: XptType, to_type: XptType) -> (XptValue, bool) {
    match (from_type, to_type) {
        (XptType::Char, XptType::Num) => char_to_num(value),
        (XptType::Num, XptType::Char) => num_to_char(value),
        _ => (value, true), // Same type, no conversion needed
    }
}

/// Convert a character value to numeric.
fn char_to_num(value: XptValue) -> (XptValue, bool) {
    match value {
        XptValue::Char(s) => {
            let trimmed = s.trim();

            // Handle empty strings as missing
            if trimmed.is_empty() {
                return (XptValue::numeric_missing(), true);
            }

            // Handle SAS missing value notation
            if trimmed == "." {
                return (XptValue::numeric_missing(), true);
            }

            if trimmed.starts_with('.') && trimmed.len() == 2 {
                let ch = trimmed.chars().nth(1).unwrap_or('_');
                if ch == '_' {
                    return (
                        XptValue::numeric_missing_with(MissingValue::Underscore),
                        true,
                    );
                } else if ch.is_ascii_uppercase() {
                    return (
                        XptValue::numeric_missing_with(MissingValue::Special(ch)),
                        true,
                    );
                }
            }

            // Try to parse as number
            match trimmed.parse::<f64>() {
                Ok(n) => (XptValue::numeric(n), true),
                Err(_) => {
                    // Conversion failed - return missing
                    (XptValue::numeric_missing(), false)
                }
            }
        }
        XptValue::Num(n) => (XptValue::Num(n), true), // Already numeric
    }
}

/// Convert a numeric value to character.
fn num_to_char(value: XptValue) -> (XptValue, bool) {
    match value {
        XptValue::Num(n) => {
            let s = match n {
                NumericValue::Value(v) => {
                    // Format the number, avoiding unnecessary trailing zeros
                    if v.fract() == 0.0 && v.abs() < 1e15 {
                        format!("{}", v as i64)
                    } else {
                        format!("{v}")
                    }
                }
                NumericValue::Missing(MissingValue::Standard) => ".".to_string(),
                NumericValue::Missing(MissingValue::Underscore) => "._".to_string(),
                NumericValue::Missing(MissingValue::Special(ch)) => format!(".{ch}"),
            };
            (XptValue::character(s), true)
        }
        XptValue::Char(s) => (XptValue::Char(s), true), // Already character
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::VariableSpec;
    use crate::types::XptColumn;

    fn create_test_dataset() -> XptDataset {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![
                XptColumn::character("AGE", 8),
                XptColumn::numeric("NAME"),
            ],
        );
        dataset.add_row(vec![XptValue::character("25"), XptValue::numeric(42.0)]);
        dataset.add_row(vec![
            XptValue::character("30"),
            XptValue::numeric_missing(),
        ]);
        dataset
    }

    fn create_test_spec() -> DatasetSpec {
        DatasetSpec::new("TEST")
            .add_variable(VariableSpec::numeric("AGE"))
            .add_variable(VariableSpec::character("NAME", 20))
    }

    #[test]
    fn test_coerce_type_char_to_num() {
        let dataset = create_test_dataset();
        let spec = create_test_spec();

        let result = coerce_type(dataset, &spec, CoerceTypeConfig::default()).unwrap();

        // AGE should now be numeric
        assert_eq!(result.dataset.columns[0].data_type, XptType::Num);
        assert_eq!(result.dataset.rows[0][0].as_f64(), Some(25.0));
        assert_eq!(result.dataset.rows[1][0].as_f64(), Some(30.0));

        // Should have recorded the conversion
        assert_eq!(result.conversions.len(), 2);
        let age_conv = result.conversions.iter().find(|c| c.variable == "AGE");
        assert!(age_conv.is_some());
    }

    #[test]
    fn test_coerce_type_num_to_char() {
        let dataset = create_test_dataset();
        let spec = create_test_spec();

        let result = coerce_type(dataset, &spec, CoerceTypeConfig::default()).unwrap();

        // NAME should now be character
        assert_eq!(result.dataset.columns[1].data_type, XptType::Char);
        assert_eq!(result.dataset.rows[0][1].as_str(), Some("42"));
        assert_eq!(result.dataset.rows[1][1].as_str(), Some(".")); // Missing converts to "."
    }

    #[test]
    fn test_coerce_type_unparseable_string() {
        let mut dataset = XptDataset::with_columns("TEST", vec![XptColumn::character("AGE", 8)]);
        dataset.add_row(vec![XptValue::character("not a number")]);
        dataset.add_row(vec![XptValue::character("25")]);

        let spec = DatasetSpec::new("TEST").add_variable(VariableSpec::numeric("AGE"));

        let result = coerce_type(dataset, &spec, CoerceTypeConfig::default()).unwrap();

        // First row should be missing (conversion failed)
        assert!(result.dataset.rows[0][0].is_missing());
        // Second row should be converted
        assert_eq!(result.dataset.rows[1][0].as_f64(), Some(25.0));

        // Should report the failure
        let conv = &result.conversions[0];
        assert_eq!(conv.values_failed, 1);
        assert_eq!(conv.values_converted, 1);
    }

    #[test]
    fn test_coerce_type_missing_values() {
        let mut dataset = XptDataset::with_columns("TEST", vec![XptColumn::character("AGE", 8)]);
        dataset.add_row(vec![XptValue::character(".")]);
        dataset.add_row(vec![XptValue::character(".A")]);
        dataset.add_row(vec![XptValue::character("")]);

        let spec = DatasetSpec::new("TEST").add_variable(VariableSpec::numeric("AGE"));

        let result = coerce_type(dataset, &spec, CoerceTypeConfig::default()).unwrap();

        assert!(result.dataset.rows[0][0].is_missing());
        assert!(result.dataset.rows[1][0].is_missing());
        assert!(result.dataset.rows[2][0].is_missing());
    }

    #[test]
    fn test_coerce_type_variable_not_in_spec() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE"), XptColumn::numeric("EXTRA")],
        );
        dataset.add_row(vec![XptValue::numeric(25.0), XptValue::numeric(1.0)]);

        let spec = DatasetSpec::new("TEST").add_variable(VariableSpec::numeric("AGE"));

        let result = coerce_type(dataset, &spec, CoerceTypeConfig::default()).unwrap();

        // Should have a warning about EXTRA
        assert!(result.warnings.iter().any(|w| w.contains("EXTRA")));
    }

    #[test]
    fn test_coerce_type_variable_not_in_data() {
        let mut dataset = XptDataset::with_columns("TEST", vec![XptColumn::numeric("AGE")]);
        dataset.add_row(vec![XptValue::numeric(25.0)]);

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::numeric("AGE"))
            .add_variable(VariableSpec::character("MISSING", 10));

        let result = coerce_type(dataset, &spec, CoerceTypeConfig::default()).unwrap();

        // Should have a warning about MISSING
        assert!(result.warnings.iter().any(|w| w.contains("MISSING")));
    }

    #[test]
    fn test_coerce_type_no_changes_needed() {
        let mut dataset = XptDataset::with_columns("TEST", vec![XptColumn::numeric("AGE")]);
        dataset.add_row(vec![XptValue::numeric(25.0)]);

        let spec = DatasetSpec::new("TEST").add_variable(VariableSpec::numeric("AGE"));

        let result = coerce_type(dataset, &spec, CoerceTypeConfig::default()).unwrap();

        // No conversions should have been made
        assert!(result.conversions.is_empty());
    }
}
