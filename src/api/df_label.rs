//! Dataset label assignment.

use polars::prelude::*;

/// Set the dataset label.
///
/// This function stores the dataset label as metadata on the DataFrame.
/// The label will be applied when the DataFrame is written to XPT format.
///
/// Note: Polars DataFrames don't natively support dataset-level metadata,
/// so this function returns the DataFrame unchanged. The label should be
/// set via the `DatasetSpec` when calling `xportrs_write`.
///
/// # Arguments
///
/// * `df` - DataFrame to modify
/// * `label` - Dataset label to apply (max 40 chars for V5, 256 for V8)
///
/// # Returns
///
/// The DataFrame unchanged. Use `DatasetSpec::with_label()` for the actual label.
///
/// # Example
///
/// ```
/// use polars::prelude::*;
/// use xportrs::xportrs_df_label;
///
/// let df = DataFrame::default();
/// let df = xportrs_df_label(df, "Demographics");
/// ```
pub fn xportrs_df_label(df: DataFrame, _label: &str) -> DataFrame {
    // Note: Polars DataFrames don't support dataset-level metadata.
    // The label should be set via DatasetSpec when writing.
    // This function exists for API compatibility with R's xportr_df_label().
    df
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_df_label_returns_unchanged() {
        let df = DataFrame::default();
        let result = xportrs_df_label(df.clone(), "Test Label");
        assert_eq!(result.height(), df.height());
    }
}
