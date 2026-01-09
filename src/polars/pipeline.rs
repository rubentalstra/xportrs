//! DataFrame pipeline utilities.
//!
//! This module provides helper functions for working with DataFrames
//! in xportr-style pipelines.

use std::path::Path;

use polars::prelude::DataFrame;

use crate::core::writer::write_xpt_with_options;
use crate::error::TransformError;
use crate::spec::DatasetSpec;
use crate::transform::{PipelineReport, XportrConfig};
use crate::types::{XptDataset, XptWriterOptions};

/// Write a DataFrame to an XPT file with full xportr pipeline.
///
/// This is a convenience function that applies the full xportr pipeline
/// and writes the result to an XPT file.
///
/// # Arguments
///
/// * `path` - Output path for the XPT file
/// * `df` - The DataFrame to transform and write
/// * `spec` - Dataset specification
/// * `config` - Pipeline configuration
///
/// # Returns
///
/// Pipeline report with details of all transformations.
///
/// # Example
///
/// ```no_run
/// use polars::prelude::*;
/// use xportrs::spec::{DatasetSpec, VariableSpec};
/// use xportrs::transform::XportrConfig;
/// use xportrs::polars::write_df_with_pipeline;
///
/// let df = df! {
///     "USUBJID" => &["001", "002"],
///     "AGE" => &[25i64, 30],
/// }.unwrap();
///
/// let spec = DatasetSpec::new("DM")
///     .add_variable(VariableSpec::character("USUBJID", 20))
///     .add_variable(VariableSpec::numeric("AGE"));
///
/// let report = write_df_with_pipeline(
///     "dm.xpt",
///     &df,
///     &spec,
///     XportrConfig::default(),
/// ).unwrap();
/// ```
pub fn write_df_with_pipeline(
    path: impl AsRef<Path>,
    df: &DataFrame,
    spec: &DatasetSpec,
    config: XportrConfig,
) -> Result<PipelineReport, TransformError> {
    let dataset = XptDataset::from_dataframe(df, &spec.name)?;
    let result = crate::transform::xportr(dataset, spec, config.clone())?;

    let options = XptWriterOptions::default().with_version(config.version);
    write_xpt_with_options(path.as_ref(), &result.dataset, &options)?;

    Ok(result.report)
}

/// Write a DataFrame to an XPT file with FDA-compliant settings.
///
/// This is a convenience function that applies strict FDA-compliant
/// settings and writes to XPT V5 format.
///
/// # Arguments
///
/// * `path` - Output path for the XPT file
/// * `df` - The DataFrame to transform and write
/// * `spec` - Dataset specification
///
/// # Returns
///
/// Pipeline report with details of all transformations.
///
/// # Example
///
/// ```no_run
/// use polars::prelude::*;
/// use xportrs::spec::{DatasetSpec, VariableSpec};
/// use xportrs::polars::write_df_fda_compliant;
///
/// let df = df! {
///     "USUBJID" => &["001", "002"],
///     "AGE" => &[25i64, 30],
/// }.unwrap();
///
/// let spec = DatasetSpec::new("DM")
///     .add_variable(VariableSpec::character("USUBJID", 20))
///     .add_variable(VariableSpec::numeric("AGE"));
///
/// let report = write_df_fda_compliant("dm.xpt", &df, &spec).unwrap();
/// ```
pub fn write_df_fda_compliant(
    path: impl AsRef<Path>,
    df: &DataFrame,
    spec: &DatasetSpec,
) -> Result<PipelineReport, TransformError> {
    write_df_with_pipeline(path, df, spec, XportrConfig::fda_strict())
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use tempfile::tempdir;

    use crate::spec::VariableSpec;

    #[test]
    fn test_write_df_with_pipeline() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.xpt");

        let df = df! {
            "AGE" => &[25.0f64, 30.0],
            "NAME" => &["Alice", "Bob"],
        }
        .unwrap();

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::numeric("AGE"))
            .add_variable(VariableSpec::character("NAME", 20));

        let report = write_df_with_pipeline(&path, &df, &spec, XportrConfig::default()).unwrap();

        assert!(path.exists());
        assert!(report.is_valid());
    }

    #[test]
    fn test_write_df_fda_compliant() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("dm.xpt");

        let df = df! {
            "USUBJID" => &["001", "002"],
            "AGE" => &[25.0f64, 30.0],
        }
        .unwrap();

        let spec = DatasetSpec::new("DM")
            .add_variable(VariableSpec::character("USUBJID", 10))
            .add_variable(VariableSpec::numeric("AGE"));

        let report = write_df_fda_compliant(&path, &df, &spec).unwrap();

        assert!(path.exists());
        assert!(report.is_valid());
    }
}
