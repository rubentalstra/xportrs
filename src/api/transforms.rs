//! Transform functions for type, length, label, and format operations.

use polars::prelude::*;

use crate::config::ActionLevel;
use crate::error::XptError;
use crate::report::{
    FormatChange, FormatReport, LabelChange, LabelReport, LengthChange, LengthReport,
    TypeConversion, TypeReport,
};
use crate::spec::DatasetSpec;

/// Coerce column types to match specification.
///
/// This function converts DataFrame column types to match the types defined
/// in the specification. Numeric columns are converted to Float64, and
/// character columns are converted to String.
///
/// # Arguments
///
/// * `df` - DataFrame to transform
/// * `spec` - Dataset specification defining expected types
/// * `action` - How to handle type mismatches
///
/// # Returns
///
/// A tuple of (transformed DataFrame, TypeReport).
///
/// # Errors
///
/// Returns an error if `action` is `ActionLevel::Stop` and type mismatches exist.
///
/// # Example
///
/// ```no_run
/// use xportrs::{xportrs_type, ActionLevel, DatasetSpec, VariableSpec};
/// use polars::prelude::*;
///
/// let df = DataFrame::default();
/// let spec = DatasetSpec::new("DM")
///     .add_variable(VariableSpec::numeric("AGE"));
///
/// let (df, report) = xportrs_type(df, &spec, ActionLevel::Warn).unwrap();
/// ```
pub fn xportrs_type(
    df: DataFrame,
    spec: &DatasetSpec,
    action: ActionLevel,
) -> Result<(DataFrame, TypeReport), XptError> {
    let mut report = TypeReport::new();
    let mut new_columns: Vec<Column> = Vec::with_capacity(df.width());

    // Build spec lookup
    let spec_vars: std::collections::HashMap<_, _> = spec
        .variables
        .iter()
        .map(|v| (v.name.to_uppercase(), v))
        .collect();

    for col in df.get_columns() {
        let col_name = col.name().as_str().to_uppercase();
        let col_name_str = col.name().as_str();

        if let Some(var_spec) = spec_vars.get(&col_name) {
            let is_numeric_in_spec = var_spec.data_type == crate::types::XptType::Num;
            let is_numeric_in_data = matches!(
                col.dtype(),
                DataType::Float64 | DataType::Float32 | DataType::Int64 | DataType::Int32
            );

            if is_numeric_in_spec != is_numeric_in_data {
                // Type mismatch - need conversion
                let from_type = format!("{:?}", col.dtype());
                let to_type = if is_numeric_in_spec {
                    "Numeric"
                } else {
                    "Character"
                };

                if action.is_error() {
                    return Err(XptError::InvalidFormat {
                        message: format!(
                            "Type mismatch for '{}': expected {}, got {}",
                            col_name_str, to_type, from_type
                        ),
                    });
                }

                let mut conversion =
                    TypeConversion::new(col_name_str.to_string(), from_type, to_type.to_string());

                // Perform conversion
                let series = col.as_materialized_series();
                let converted = if is_numeric_in_spec {
                    // Convert to numeric
                    convert_to_numeric(series, &mut conversion)?
                } else {
                    // Convert to string
                    convert_to_string(series, &mut conversion)?
                };

                report.conversions.push(conversion);
                new_columns.push(converted.into_column());
            } else {
                new_columns.push(col.clone());
            }
        } else {
            // Variable not in spec - warn if needed
            if action.should_report() {
                report.warnings.push(format!(
                    "Variable '{}' not found in specification",
                    col_name_str
                ));
            }
            new_columns.push(col.clone());
        }
    }

    // Check for variables in spec not in data
    for var in &spec.variables {
        let var_upper = var.name.to_uppercase();
        if !df
            .get_column_names()
            .iter()
            .any(|n| n.as_str().to_uppercase() == var_upper)
        {
            if action.should_report() {
                report.warnings.push(format!(
                    "Variable '{}' in specification not found in data",
                    var.name
                ));
            }
        }
    }

    let result_df = DataFrame::new(new_columns).map_err(|e| XptError::InvalidFormat {
        message: e.to_string(),
    })?;

    Ok((result_df, report))
}

/// Convert a series to numeric (Float64).
fn convert_to_numeric(
    series: &Series,
    conversion: &mut TypeConversion,
) -> Result<Series, XptError> {
    let mut converted = 0usize;
    let mut failed = 0usize;

    let result = match series.dtype() {
        DataType::String => {
            // Parse strings to floats
            let ca = series.str().map_err(|e| XptError::InvalidFormat {
                message: e.to_string(),
            })?;

            let values: Float64Chunked = ca
                .into_iter()
                .map(|opt| {
                    opt.and_then(|s| {
                        let trimmed = s.trim();
                        if trimmed.is_empty() || trimmed == "." {
                            converted += 1;
                            None // Missing
                        } else {
                            match trimmed.parse::<f64>() {
                                Ok(v) => {
                                    converted += 1;
                                    Some(v)
                                }
                                Err(_) => {
                                    failed += 1;
                                    None // Failed conversion becomes missing
                                }
                            }
                        }
                    })
                })
                .collect();

            values.into_series().with_name(series.name().clone())
        }
        _ => {
            // Already numeric-like, just cast
            converted = series.len();
            series
                .cast(&DataType::Float64)
                .map_err(|e| XptError::InvalidFormat {
                    message: e.to_string(),
                })?
        }
    };

    conversion.values_converted = converted;
    conversion.values_failed = failed;

    Ok(result)
}

/// Convert a series to string.
fn convert_to_string(series: &Series, conversion: &mut TypeConversion) -> Result<Series, XptError> {
    conversion.values_converted = series.len();
    conversion.values_failed = 0;

    series
        .cast(&DataType::String)
        .map_err(|e| XptError::InvalidFormat {
            message: e.to_string(),
        })
}

/// Apply variable lengths from specification.
///
/// This function adjusts character column lengths to match the specification.
/// Values that exceed the specified length will be truncated.
///
/// # Arguments
///
/// * `df` - DataFrame to transform
/// * `spec` - Dataset specification defining expected lengths
/// * `action` - How to handle length mismatches
///
/// # Returns
///
/// A tuple of (transformed DataFrame, LengthReport).
pub fn xportrs_length(
    df: DataFrame,
    spec: &DatasetSpec,
    action: ActionLevel,
) -> Result<(DataFrame, LengthReport), XptError> {
    let mut report = LengthReport::new();

    // Build spec lookup
    let spec_vars: std::collections::HashMap<_, _> = spec
        .variables
        .iter()
        .map(|v| (v.name.to_uppercase(), v))
        .collect();

    // For now, we just record what would change
    // Actual truncation happens during write
    for col in df.get_columns() {
        let col_name = col.name().as_str().to_uppercase();
        let col_name_str = col.name().as_str();

        if let Some(var_spec) = spec_vars.get(&col_name) {
            if let Some(spec_length) = var_spec.length {
                // Check if any values exceed the spec length
                if col.dtype() == &DataType::String {
                    let series = col.as_materialized_series();
                    if let Ok(ca) = series.str() {
                        let max_len = ca
                            .into_iter()
                            .filter_map(|s| s.map(str::len))
                            .max()
                            .unwrap_or(0);
                        let old_length = max_len as u16;

                        if old_length > spec_length {
                            let truncated = ca
                                .into_iter()
                                .filter(|s| s.is_some_and(|v| v.len() > spec_length as usize))
                                .count();

                            if action.is_error() && truncated > 0 {
                                return Err(XptError::InvalidFormat {
                                    message: format!(
                                        "Variable '{}' has {} values exceeding length {}",
                                        col_name_str, truncated, spec_length
                                    ),
                                });
                            }

                            let mut change = LengthChange::new(
                                col_name_str.to_string(),
                                old_length,
                                spec_length,
                            );
                            change.truncated_values = truncated;
                            report.changes.push(change);
                        }
                    }
                }
            }
        }
    }

    // Return unchanged - actual truncation happens during write
    Ok((df, report))
}

/// Apply variable labels from specification.
///
/// This function records label assignments from the specification.
/// Labels are applied during the write operation.
///
/// # Arguments
///
/// * `df` - DataFrame to transform
/// * `spec` - Dataset specification defining labels
/// * `action` - How to handle label mismatches
///
/// # Returns
///
/// A tuple of (DataFrame unchanged, LabelReport).
pub fn xportrs_label(
    df: DataFrame,
    spec: &DatasetSpec,
    action: ActionLevel,
) -> Result<(DataFrame, LabelReport), XptError> {
    let mut report = LabelReport::new();

    // Build spec lookup
    let spec_vars: std::collections::HashMap<_, _> = spec
        .variables
        .iter()
        .map(|v| (v.name.to_uppercase(), v))
        .collect();

    for col in df.get_columns() {
        let col_name = col.name().as_str().to_uppercase();
        let col_name_str = col.name().as_str();

        if let Some(var_spec) = spec_vars.get(&col_name) {
            if let Some(ref new_label) = var_spec.label {
                // Record the label change
                let change = LabelChange::new(col_name_str.to_string(), None, new_label.clone());
                report.changes.push(change);
            }
        } else if action.should_report() {
            report.warnings.push(format!(
                "Variable '{}' not found in specification",
                col_name_str
            ));
        }
    }

    // DataFrame unchanged - labels applied during write
    Ok((df, report))
}

/// Apply SAS formats from specification.
///
/// This function records format assignments from the specification.
/// Formats are applied during the write operation.
///
/// # Arguments
///
/// * `df` - DataFrame to transform
/// * `spec` - Dataset specification defining formats
/// * `action` - How to handle format mismatches
///
/// # Returns
///
/// A tuple of (DataFrame unchanged, FormatReport).
pub fn xportrs_format(
    df: DataFrame,
    spec: &DatasetSpec,
    action: ActionLevel,
) -> Result<(DataFrame, FormatReport), XptError> {
    let mut report = FormatReport::new();

    // Build spec lookup
    let spec_vars: std::collections::HashMap<_, _> = spec
        .variables
        .iter()
        .map(|v| (v.name.to_uppercase(), v))
        .collect();

    for col in df.get_columns() {
        let col_name = col.name().as_str().to_uppercase();
        let col_name_str = col.name().as_str();

        if let Some(var_spec) = spec_vars.get(&col_name) {
            if let Some(ref format) = var_spec.format {
                if let Some(ref name) = format.name {
                    // Record the format change
                    let format_str = if format.decimals > 0 {
                        format!("{}{}.{}", name, format.width, format.decimals)
                    } else if format.width > 0 {
                        format!("{}{}.", name, format.width)
                    } else {
                        format!("{}.", name)
                    };
                    let change = FormatChange::new(col_name_str.to_string(), None, format_str);
                    report.changes.push(change);
                }
            }
        } else if action.should_report() {
            report.warnings.push(format!(
                "Variable '{}' not found in specification",
                col_name_str
            ));
        }
    }

    // DataFrame unchanged - formats applied during write
    Ok((df, report))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_report_empty() {
        let df = DataFrame::default();
        let spec = DatasetSpec::new("TEST");
        let (result, report) = xportrs_type(df, &spec, ActionLevel::Warn).unwrap();
        assert!(!report.has_changes());
        assert_eq!(result.height(), 0);
    }
}
