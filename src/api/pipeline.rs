//! All-in-one pipeline function.

use std::path::Path;

use polars::prelude::*;

use crate::config::XportrsConfig;
use crate::error::XptError;
use crate::report::XportrsReport;
use crate::spec::DatasetSpec;

use super::{
    xportrs_format, xportrs_label, xportrs_length, xportrs_order, xportrs_type, xportrs_write,
};

/// All-in-one pipeline: apply transforms and write to XPT file.
///
/// This function applies all enabled transforms (type, length, label, order, format)
/// based on the configuration, then writes the result to an XPT file.
///
/// This is equivalent to R's `xportr()` function - a convenient wrapper that
/// chains all transform operations together.
///
/// # Arguments
///
/// * `df` - DataFrame to transform and write
/// * `path` - Output path for the XPT file
/// * `spec` - Dataset specification with metadata
/// * `config` - Configuration controlling which transforms to apply
///
/// # Returns
///
/// An [`XportrsReport`] containing reports from all applied transforms.
///
/// # Errors
///
/// Returns an error if any transform fails (when using strict action levels)
/// or if the file cannot be written.
///
/// # Example
///
/// ```no_run
/// use xportrs::{xportrs, xportrs_read, XportrsConfig, DatasetSpec};
///
/// let df = xportrs_read("input.xpt").unwrap();
/// let spec = DatasetSpec::new("DM").with_label("Demographics");
/// let config = XportrsConfig::fda();
///
/// let report = xportrs(df, "output.xpt", &spec, config).unwrap();
/// println!("{}", report.summary());
/// ```
pub fn xportrs<P: AsRef<Path>>(
    df: DataFrame,
    path: P,
    spec: &DatasetSpec,
    config: XportrsConfig,
) -> Result<XportrsReport, XptError> {
    let mut report = XportrsReport::new();
    let mut current_df = df;

    // Apply type coercion if enabled
    if config.apply_type {
        let (new_df, type_report) = xportrs_type(current_df, spec, config.type_action)?;
        current_df = new_df;
        report.type_report = Some(type_report);
    }

    // Apply length adjustment if enabled
    if config.apply_length {
        let (new_df, length_report) = xportrs_length(current_df, spec, config.length_action)?;
        current_df = new_df;
        report.length_report = Some(length_report);
    }

    // Apply labels if enabled
    if config.apply_label {
        let (new_df, label_report) = xportrs_label(current_df, spec, config.label_action)?;
        current_df = new_df;
        report.label_report = Some(label_report);
    }

    // Apply ordering if enabled
    if config.apply_order {
        let (new_df, order_report) = xportrs_order(current_df, spec, config.order_action)?;
        current_df = new_df;
        report.order_report = Some(order_report);
    }

    // Apply formats if enabled
    if config.apply_format {
        let (new_df, format_report) = xportrs_format(current_df, spec, config.format_action)?;
        current_df = new_df;
        report.format_report = Some(format_report);
    }

    // Write to file
    let write_report = xportrs_write(path, &current_df, spec, config)?;
    report.write_report = Some(write_report);

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_pipeline_empty_df() {
        let df = DataFrame::default();
        let spec = DatasetSpec::new("TEST");
        let config = XportrsConfig::default();

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.xpt");

        let result = xportrs(df, &path, &spec, config);

        // Should succeed with empty DataFrame
        assert!(result.is_ok());
    }
}
