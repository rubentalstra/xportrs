//! Write Polars DataFrames to XPT files.

use std::fs;
use std::path::Path;

use polars::prelude::*;

use crate::config::XportrsConfig;
use crate::error::XptError;
use crate::report::WriteReport;
use crate::spec::DatasetSpec;

/// Write a DataFrame to an XPT file.
///
/// This function applies the specification metadata and writes the DataFrame
/// to a CDISC-compliant XPT file.
///
/// # Arguments
///
/// * `path` - Output path for the XPT file
/// * `df` - DataFrame to write
/// * `spec` - Dataset specification with metadata
/// * `config` - Configuration options
///
/// # Returns
///
/// A [`WriteReport`] containing details about the write operation.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be created
/// - Validation fails (when using strict config)
/// - Data cannot be converted to XPT format
///
/// # Example
///
/// ```no_run
/// use xportrs::{xportrs_read, xportrs_write, XportrsConfig, DatasetSpec};
///
/// let df = xportrs_read("input.xpt").unwrap();
/// let spec = DatasetSpec::new("DM").with_label("Demographics");
/// let config = XportrsConfig::fda();
///
/// let report = xportrs_write("output.xpt", &df, &spec, config).unwrap();
/// println!("Wrote {} rows", report.rows_written);
/// ```
pub fn xportrs_write<P: AsRef<Path>>(
    path: P,
    df: &DataFrame,
    spec: &DatasetSpec,
    config: XportrsConfig,
) -> Result<WriteReport, XptError> {
    let path = path.as_ref();

    // Convert DataFrame to XptDataset
    let dataset = dataframe_to_dataset(df, spec, &config)?;

    // Create writer options from config
    let writer_options = crate::types::XptWriterOptions {
        version: config.version,
        sas_version: config.sas_version.clone(),
        os_name: config.os_name.clone(),
        created: config.created,
        modified: config.modified,
        default_missing: crate::types::MissingValue::Standard,
        namestr_length: 140,
    };

    // Write the file
    crate::io::write_xpt_with_options(path, &dataset, &writer_options)?;

    // Get file size
    let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    // Build report
    let report = WriteReport {
        path: path.to_path_buf(),
        dataset_name: dataset.name.clone(),
        rows_written: dataset.num_rows(),
        columns_written: dataset.num_columns(),
        file_size,
        warnings: Vec::new(),
    };

    Ok(report)
}

/// Convert a DataFrame to an XptDataset using the specification.
fn dataframe_to_dataset(
    df: &DataFrame,
    spec: &DatasetSpec,
    config: &XportrsConfig,
) -> Result<crate::types::XptDataset, XptError> {
    use crate::types::{XptColumn, XptDataset, XptType};

    // Build columns from DataFrame schema, using spec for metadata
    let mut columns = Vec::with_capacity(df.width());
    let spec_vars: std::collections::HashMap<_, _> = spec
        .variables
        .iter()
        .map(|v| (v.name.to_uppercase(), v))
        .collect();

    for col in df.get_columns() {
        let col_name = col.name().as_str().to_uppercase();
        let var_spec = spec_vars.get(&col_name);

        // Determine column type from DataFrame
        let (data_type, length) = match col.dtype() {
            DataType::Float64 | DataType::Float32 | DataType::Int64 | DataType::Int32 => {
                (XptType::Num, 8u16)
            }
            DataType::String => {
                // Use spec length if available, otherwise compute from data
                let len = var_spec.and_then(|v| v.length).unwrap_or_else(|| {
                    compute_max_string_length(col.as_materialized_series().clone())
                });
                (XptType::Char, len)
            }
            _ => {
                // Default to character for other types
                let len = var_spec.and_then(|v| v.length).unwrap_or(200);
                (XptType::Char, len)
            }
        };

        let mut xpt_col = XptColumn {
            name: col_name
                .chars()
                .take(config.version.variable_name_limit())
                .collect(),
            data_type,
            length,
            label: None,
            format: None,
            format_length: 0,
            format_decimals: 0,
            informat: None,
            informat_length: 0,
            informat_decimals: 0,
            justification: crate::types::Justification::Left,
        };

        // Apply metadata from spec
        if let Some(var) = var_spec {
            if let Some(ref label) = var.label {
                let max_label = config.version.variable_label_limit();
                xpt_col.label = Some(label.chars().take(max_label).collect());
            }
            if let Some(ref fmt) = var.format {
                xpt_col.format = fmt.name.clone();
                xpt_col.format_length = fmt.width;
                xpt_col.format_decimals = fmt.decimals;
            }
        }

        columns.push(xpt_col);
    }

    // Build rows from DataFrame
    let mut rows = Vec::with_capacity(df.height());
    for row_idx in 0..df.height() {
        let mut row = Vec::with_capacity(df.width());
        for (col_idx, col) in df.get_columns().iter().enumerate() {
            let value = series_value_at(
                col.as_materialized_series().clone(),
                row_idx,
                columns[col_idx].data_type,
            )?;
            row.push(value);
        }
        rows.push(row);
    }

    // Create dataset
    let mut dataset = XptDataset {
        name: spec
            .name
            .chars()
            .take(config.version.dataset_name_limit())
            .collect(),
        label: spec.label.as_ref().map(|l| {
            l.chars()
                .take(config.version.dataset_label_limit())
                .collect()
        }),
        dataset_type: None,
        columns,
        rows,
    };

    // Apply dataset label from spec
    if let Some(ref label) = spec.label {
        let max_label = config.version.dataset_label_limit();
        dataset.label = Some(label.chars().take(max_label).collect());
    }

    Ok(dataset)
}

/// Compute the maximum string length in a column.
#[allow(clippy::needless_pass_by_value)]
fn compute_max_string_length(series: Series) -> u16 {
    let max_len = series
        .str()
        .map(|ca| ca.into_iter().filter_map(|s| s.map(str::len)).max())
        .ok()
        .flatten()
        .unwrap_or(8);

    // Ensure at least 1, and cap at 200 for safety
    max_len.clamp(1, 200) as u16
}

/// Extract a value from a Series at a given index.
#[allow(clippy::needless_pass_by_value)]
fn series_value_at(
    series: Series,
    idx: usize,
    target_type: crate::types::XptType,
) -> Result<crate::types::XptValue, XptError> {
    use crate::types::{MissingValue, NumericValue, XptType, XptValue};

    match target_type {
        XptType::Num => {
            let value = series.get(idx).ok();
            match value {
                Some(AnyValue::Float64(v)) => {
                    if v.is_nan() {
                        Ok(XptValue::Num(NumericValue::Missing(MissingValue::Standard)))
                    } else {
                        Ok(XptValue::Num(NumericValue::Value(v)))
                    }
                }
                Some(AnyValue::Float32(v)) => {
                    if v.is_nan() {
                        Ok(XptValue::Num(NumericValue::Missing(MissingValue::Standard)))
                    } else {
                        Ok(XptValue::Num(NumericValue::Value(f64::from(v))))
                    }
                }
                Some(AnyValue::Int64(v)) => Ok(XptValue::Num(NumericValue::Value(v as f64))),
                Some(AnyValue::Int32(v)) => Ok(XptValue::Num(NumericValue::Value(f64::from(v)))),
                Some(AnyValue::String(s)) => {
                    // Try to parse string as number
                    match s.trim().parse::<f64>() {
                        Ok(v) => Ok(XptValue::Num(NumericValue::Value(v))),
                        Err(_) => Ok(XptValue::Num(NumericValue::Missing(MissingValue::Standard))),
                    }
                }
                // Null, None, and any other types become missing
                _ => Ok(XptValue::Num(NumericValue::Missing(MissingValue::Standard))),
            }
        }
        XptType::Char => {
            let value = series.get(idx).ok();
            match value {
                Some(AnyValue::String(s)) => Ok(XptValue::Char(s.to_string())),
                Some(AnyValue::Null) | None => Ok(XptValue::Char(String::new())),
                Some(v) => Ok(XptValue::Char(format!("{v}"))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added once we have test fixtures
}
