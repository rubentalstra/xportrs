//! Metadata binding for DataFrames.

use polars::prelude::*;

use crate::spec::DatasetSpec;

/// Bind metadata specification to a DataFrame.
///
/// This function associates a `DatasetSpec` with a DataFrame for use in
/// subsequent transform operations. Since Polars DataFrames don't support
/// custom metadata, this function returns the DataFrame unchanged.
///
/// In practice, you should pass the spec directly to transform functions
/// or use the `xportrs()` pipeline which handles spec binding internally.
///
/// # Arguments
///
/// * `df` - DataFrame to bind metadata to
/// * `spec` - Dataset specification with variable metadata
///
/// # Returns
///
/// The DataFrame unchanged. Pass the spec to transform functions directly.
///
/// # Example
///
/// ```
/// use polars::prelude::*;
/// use xportrs::{xportrs_metadata, DatasetSpec};
///
/// let df = DataFrame::default();
/// let spec = DatasetSpec::new("DM");
/// let df = xportrs_metadata(df, spec);
/// ```
pub fn xportrs_metadata(df: DataFrame, _spec: DatasetSpec) -> DataFrame {
    // Note: Polars DataFrames don't support custom metadata attachment.
    // The spec should be passed directly to transform functions.
    // This function exists for API compatibility with R's xportr_metadata().
    df
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_returns_unchanged() {
        let df = DataFrame::default();
        let spec = DatasetSpec::new("TEST");
        let result = xportrs_metadata(df.clone(), spec);
        assert_eq!(result.height(), df.height());
    }
}
