//! `XportrTransforms` trait for `DataFrame` operations.
//!
//! This module provides the [`XportrTransforms`] trait which adds xportr-style
//! methods to Polars `DataFrames` and `MetadataFrames`.

use std::path::Path;

use polars::prelude::DataFrame;

use crate::config::ActionLevel;
use crate::core::writer::write_xpt_with_options;
use crate::error::TransformError;
use crate::spec::DatasetSpec;
use crate::transform::{
    ApplyFormatConfig, ApplyLabelConfig, ApplyLengthConfig, ApplyOrderConfig, CoerceTypeConfig,
    PipelineReport, XportrConfig, apply_format, apply_label, apply_length, apply_order,
    coerce_type,
};
use crate::types::{XptDataset, XptWriterOptions};

use super::metadata::MetadataFrame;

/// Extension trait providing xportr-style transforms for `DataFrames`.
///
/// This trait adds methods like `xportr_type()`, `xportr_label()`, etc.
/// to Polars `DataFrames` and `MetadataFrames`, enabling R xportr-style workflows.
///
/// # Example
///
/// ```no_run
/// use polars::prelude::*;
/// use xportrs::polars::XportrTransforms;
/// use xportrs::spec::{DatasetSpec, VariableSpec};
/// use xportrs::ActionLevel;
///
/// let spec = DatasetSpec::new("DM")
///     .add_variable(VariableSpec::numeric("AGE").with_label("Age"))
///     .add_variable(VariableSpec::character("SEX", 1).with_label("Sex"));
///
/// let df = df! {
///     "AGE" => &["25", "30"],  // Wrong type - should be numeric
///     "SEX" => &["M", "F"],
/// }.unwrap();
///
/// // Apply transforms
/// let result = df
///     .xportr_metadata(spec.clone())
///     .xportr_type(&spec, ActionLevel::Warn).unwrap()
///     .xportr_label(&spec, ActionLevel::Warn).unwrap()
///     .xportr_df_label("Demographics");
///
/// // Write to XPT
/// // result.xportr_write("dm.xpt", "DM", &spec, false).unwrap();
/// ```
pub trait XportrTransforms {
    /// Apply type coercion to match specification (`xportr_type` equivalent).
    ///
    /// Converts column types to match the spec:
    /// - Character → Numeric (parse as float)
    /// - Numeric → Character (format as string)
    ///
    /// # Arguments
    ///
    /// * `spec` - Dataset specification with expected types
    /// * `action` - Action level for mismatches
    ///
    /// # Errors
    ///
    /// Returns error if `action` is `Stop` and type mismatches are found.
    fn xportr_type(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError>;

    /// Apply variable lengths from specification (`xportr_length` equivalent).
    ///
    /// Sets column lengths from the spec. For character columns, values
    /// may be truncated if they exceed the specified length.
    ///
    /// # Arguments
    ///
    /// * `spec` - Dataset specification with lengths
    /// * `action` - Action level for truncation
    ///
    /// # Errors
    ///
    /// Returns error if `action` is `Stop` and length issues are found.
    fn xportr_length(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError>;

    /// Apply variable labels from specification (`xportr_label` equivalent).
    ///
    /// Sets column labels from the spec. Labels are stored as column
    /// metadata in the resulting `MetadataFrame`.
    ///
    /// # Arguments
    ///
    /// * `spec` - Dataset specification with labels
    /// * `action` - Action level for missing labels
    ///
    /// # Errors
    ///
    /// Returns error if `action` is `Stop` and label issues are found.
    fn xportr_label(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError>;

    /// Apply variable ordering from specification (`xportr_order` equivalent).
    ///
    /// Reorders columns to match the order specified in the spec.
    ///
    /// # Arguments
    ///
    /// * `spec` - Dataset specification with order information
    /// * `action` - Action level for ordering issues
    ///
    /// # Errors
    ///
    /// Returns error if `action` is `Stop` and order issues are found.
    fn xportr_order(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError>;

    /// Apply SAS formats from specification (`xportr_format` equivalent).
    ///
    /// Sets format and informat for columns from the spec.
    ///
    /// # Arguments
    ///
    /// * `spec` - Dataset specification with formats
    /// * `action` - Action level for format issues
    ///
    /// # Errors
    ///
    /// Returns error if `action` is `Stop` and format issues are found.
    fn xportr_format(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError>;

    /// Set the dataset label (`xportr_df_label` equivalent).
    ///
    /// # Arguments
    ///
    /// * `label` - The dataset label
    fn xportr_df_label(self, label: impl Into<String>) -> MetadataFrame;

    /// Attach metadata specification to the `DataFrame`.
    ///
    /// This creates a `MetadataFrame` with the spec attached, enabling
    /// subsequent transform operations to use the spec automatically.
    ///
    /// # Arguments
    ///
    /// * `spec` - The dataset specification to attach
    fn xportr_metadata(self, spec: DatasetSpec) -> MetadataFrame;

    /// Apply full xportr pipeline (xportr equivalent).
    ///
    /// Applies all transforms in the correct order:
    /// 1. Type coercion
    /// 2. Length application
    /// 3. Label application
    /// 4. Order application
    /// 5. Format application
    ///
    /// # Arguments
    ///
    /// * `spec` - Dataset specification
    /// * `config` - Pipeline configuration
    ///
    /// # Errors
    ///
    /// Returns error if any transform fails based on its configured action level.
    fn xportr(
        self,
        spec: &DatasetSpec,
        config: XportrConfig,
    ) -> Result<MetadataFrame, TransformError>;

    /// Write to XPT file with full pipeline (`xportr_write` equivalent).
    ///
    /// Applies the full xportr pipeline and writes the result to an XPT file.
    ///
    /// # Arguments
    ///
    /// * `path` - Output path for the XPT file
    /// * `dataset_name` - Name for the dataset (1-8 chars for V5)
    /// * `spec` - Dataset specification
    /// * `strict_checks` - Whether to use strict validation
    ///
    /// # Returns
    ///
    /// Pipeline report with details of all transformations applied.
    ///
    /// # Errors
    ///
    /// Returns error if transformation or writing fails.
    fn xportr_write(
        self,
        path: impl AsRef<Path>,
        dataset_name: &str,
        spec: &DatasetSpec,
        strict_checks: bool,
    ) -> Result<PipelineReport, TransformError>;
}

impl XportrTransforms for DataFrame {
    fn xportr_type(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError> {
        let dataset = XptDataset::from_dataframe(&self, spec.name.as_str())?;
        let config = CoerceTypeConfig::default().with_action(action);
        let result = coerce_type(dataset, spec, config)?;

        let df = result.dataset.to_dataframe()?;
        let mut mf = MetadataFrame::new(df);
        mf.report_mut().type_conversions = result.conversions;
        mf.report_mut().warnings.extend(result.warnings);

        Ok(mf)
    }

    fn xportr_length(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError> {
        let dataset = XptDataset::from_dataframe(&self, spec.name.as_str())?;
        let config = ApplyLengthConfig::default().with_action(action);
        let result = apply_length(dataset, spec, config)?;

        let df = result.dataset.to_dataframe()?;
        let mut mf = MetadataFrame::new(df);
        mf.report_mut().length_changes = result.changes;
        mf.report_mut().warnings.extend(result.warnings);

        Ok(mf)
    }

    fn xportr_label(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError> {
        let dataset = XptDataset::from_dataframe(&self, spec.name.as_str())?;
        let config = ApplyLabelConfig::default().with_action(action);
        let result = apply_label(dataset, spec, config)?;

        let df = result.dataset.to_dataframe()?;
        let mut mf = MetadataFrame::new(df);
        mf.report_mut().label_changes = result.changes;
        mf.report_mut().warnings.extend(result.warnings);

        Ok(mf)
    }

    fn xportr_order(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError> {
        let dataset = XptDataset::from_dataframe(&self, spec.name.as_str())?;
        let config = ApplyOrderConfig::default().with_action(action);
        let result = apply_order(dataset, spec, config)?;

        let df = result.dataset.to_dataframe()?;
        let mut mf = MetadataFrame::new(df);
        mf.report_mut().order_changes = result.changes;
        mf.report_mut().warnings.extend(result.warnings);

        Ok(mf)
    }

    fn xportr_format(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError> {
        let dataset = XptDataset::from_dataframe(&self, spec.name.as_str())?;
        let config = ApplyFormatConfig::default().with_action(action);
        let result = apply_format(dataset, spec, config)?;

        let df = result.dataset.to_dataframe()?;
        let mut mf = MetadataFrame::new(df);
        mf.report_mut().format_changes = result.changes;
        mf.report_mut().warnings.extend(result.warnings);

        Ok(mf)
    }

    fn xportr_df_label(self, label: impl Into<String>) -> MetadataFrame {
        MetadataFrame::new(self).with_label(label)
    }

    fn xportr_metadata(self, spec: DatasetSpec) -> MetadataFrame {
        MetadataFrame::with_spec(self, spec)
    }

    fn xportr(
        self,
        spec: &DatasetSpec,
        config: XportrConfig,
    ) -> Result<MetadataFrame, TransformError> {
        let dataset = XptDataset::from_dataframe(&self, spec.name.as_str())?;
        let result = crate::transform::xportr(dataset, spec, config)?;

        let df = result.dataset.to_dataframe()?;
        let mut mf = MetadataFrame::new(df);

        // Transfer report data from sub-reports
        mf.report_mut().type_conversions = result.report.type_report.type_conversions;
        mf.report_mut().length_changes = result.report.length_report.length_changes;
        mf.report_mut().label_changes = result.report.label_report.label_changes;
        mf.report_mut().order_changes = result.report.order_report.order_changes;
        mf.report_mut().format_changes = result.report.format_report.format_changes;
        // Merge warnings from all sub-reports
        mf.report_mut()
            .warnings
            .extend(result.report.type_report.warnings);
        mf.report_mut()
            .warnings
            .extend(result.report.length_report.warnings);
        mf.report_mut()
            .warnings
            .extend(result.report.label_report.warnings);
        mf.report_mut()
            .warnings
            .extend(result.report.order_report.warnings);
        mf.report_mut()
            .warnings
            .extend(result.report.format_report.warnings);

        Ok(mf)
    }

    fn xportr_write(
        self,
        path: impl AsRef<Path>,
        dataset_name: &str,
        spec: &DatasetSpec,
        strict_checks: bool,
    ) -> Result<PipelineReport, TransformError> {
        let dataset = XptDataset::from_dataframe(&self, dataset_name)?;

        let config = if strict_checks {
            XportrConfig::fda_strict()
        } else {
            XportrConfig::default()
        };

        let result = crate::transform::xportr(dataset, spec, config)?;

        let options = XptWriterOptions::default().with_version(config.version);
        write_xpt_with_options(path.as_ref(), &result.dataset, &options)?;

        Ok(result.report)
    }
}

impl XportrTransforms for MetadataFrame {
    fn xportr_type(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError> {
        let name = self.dataset_name().unwrap_or(&spec.name).to_string();
        let dataset = XptDataset::from_dataframe(self.df(), &name)?;
        let config = CoerceTypeConfig::default().with_action(action);
        let result = coerce_type(dataset, spec, config)?;

        let df = result.dataset.to_dataframe()?;
        let mut mf = self.with_df(df);
        mf.report_mut().type_conversions.extend(result.conversions);
        mf.report_mut().warnings.extend(result.warnings);

        Ok(mf)
    }

    fn xportr_length(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError> {
        let name = self.dataset_name().unwrap_or(&spec.name).to_string();
        let dataset = XptDataset::from_dataframe(self.df(), &name)?;
        let config = ApplyLengthConfig::default().with_action(action);
        let result = apply_length(dataset, spec, config)?;

        let df = result.dataset.to_dataframe()?;
        let mut mf = self.with_df(df);
        mf.report_mut().length_changes.extend(result.changes);
        mf.report_mut().warnings.extend(result.warnings);

        Ok(mf)
    }

    fn xportr_label(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError> {
        let name = self.dataset_name().unwrap_or(&spec.name).to_string();
        let dataset = XptDataset::from_dataframe(self.df(), &name)?;
        let config = ApplyLabelConfig::default().with_action(action);
        let result = apply_label(dataset, spec, config)?;

        let df = result.dataset.to_dataframe()?;
        let mut mf = self.with_df(df);
        mf.report_mut().label_changes.extend(result.changes);
        mf.report_mut().warnings.extend(result.warnings);

        Ok(mf)
    }

    fn xportr_order(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError> {
        let name = self.dataset_name().unwrap_or(&spec.name).to_string();
        let dataset = XptDataset::from_dataframe(self.df(), &name)?;
        let config = ApplyOrderConfig::default().with_action(action);
        let result = apply_order(dataset, spec, config)?;

        let df = result.dataset.to_dataframe()?;
        let mut mf = self.with_df(df);
        mf.report_mut().order_changes.extend(result.changes);
        mf.report_mut().warnings.extend(result.warnings);

        Ok(mf)
    }

    fn xportr_format(
        self,
        spec: &DatasetSpec,
        action: ActionLevel,
    ) -> Result<MetadataFrame, TransformError> {
        let name = self.dataset_name().unwrap_or(&spec.name).to_string();
        let dataset = XptDataset::from_dataframe(self.df(), &name)?;
        let config = ApplyFormatConfig::default().with_action(action);
        let result = apply_format(dataset, spec, config)?;

        let df = result.dataset.to_dataframe()?;
        let mut mf = self.with_df(df);
        mf.report_mut().format_changes.extend(result.changes);
        mf.report_mut().warnings.extend(result.warnings);

        Ok(mf)
    }

    fn xportr_df_label(self, label: impl Into<String>) -> MetadataFrame {
        self.with_label(label)
    }

    fn xportr_metadata(self, spec: DatasetSpec) -> MetadataFrame {
        self.with_spec_replaced(spec)
    }

    fn xportr(
        self,
        spec: &DatasetSpec,
        config: XportrConfig,
    ) -> Result<MetadataFrame, TransformError> {
        let name = self.dataset_name().unwrap_or(&spec.name).to_string();
        let dataset = XptDataset::from_dataframe(self.df(), &name)?;
        let result = crate::transform::xportr(dataset, spec, config)?;

        let df = result.dataset.to_dataframe()?;
        let mut mf = self.with_df(df);

        // Merge reports from sub-reports
        mf.report_mut()
            .type_conversions
            .extend(result.report.type_report.type_conversions);
        mf.report_mut()
            .length_changes
            .extend(result.report.length_report.length_changes);
        mf.report_mut()
            .label_changes
            .extend(result.report.label_report.label_changes);
        mf.report_mut()
            .order_changes
            .extend(result.report.order_report.order_changes);
        mf.report_mut()
            .format_changes
            .extend(result.report.format_report.format_changes);
        // Merge warnings from all sub-reports
        mf.report_mut()
            .warnings
            .extend(result.report.type_report.warnings);
        mf.report_mut()
            .warnings
            .extend(result.report.length_report.warnings);
        mf.report_mut()
            .warnings
            .extend(result.report.label_report.warnings);
        mf.report_mut()
            .warnings
            .extend(result.report.order_report.warnings);
        mf.report_mut()
            .warnings
            .extend(result.report.format_report.warnings);

        Ok(mf)
    }

    fn xportr_write(
        self,
        path: impl AsRef<Path>,
        dataset_name: &str,
        spec: &DatasetSpec,
        strict_checks: bool,
    ) -> Result<PipelineReport, TransformError> {
        let dataset = XptDataset::from_dataframe(self.df(), dataset_name)?;

        let config = if strict_checks {
            XportrConfig::fda_strict()
        } else {
            XportrConfig::default()
        };

        let result = crate::transform::xportr(dataset, spec, config)?;

        let options = XptWriterOptions::default().with_version(config.version);
        write_xpt_with_options(path.as_ref(), &result.dataset, &options)?;

        Ok(result.report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    use crate::spec::VariableSpec;

    fn create_test_spec() -> DatasetSpec {
        DatasetSpec::new("TEST")
            .add_variable(
                VariableSpec::numeric("AGE")
                    .with_label("Age in Years")
                    .with_order(1),
            )
            .add_variable(
                VariableSpec::character("NAME", 20)
                    .with_label("Subject Name")
                    .with_order(2),
            )
    }

    #[test]
    fn test_xportr_metadata() {
        let df = df! {
            "AGE" => &[25i64, 30],
            "NAME" => &["Alice", "Bob"],
        }
        .unwrap();

        let spec = create_test_spec();
        let mf = df.xportr_metadata(spec);

        assert!(mf.has_spec());
        assert_eq!(mf.dataset_name(), Some("TEST"));
    }

    #[test]
    fn test_xportr_df_label() {
        let df = df! { "X" => &[1, 2, 3] }.unwrap();
        let mf = df.xportr_df_label("Test Label");

        assert_eq!(mf.label(), Some("Test Label"));
    }

    #[test]
    fn test_xportr_type() {
        // Character column that should be numeric
        let df = df! {
            "AGE" => &["25", "30"],
            "NAME" => &["Alice", "Bob"],
        }
        .unwrap();

        let spec = create_test_spec();
        let result = df.xportr_type(&spec, ActionLevel::Warn).unwrap();

        // Check that conversions were recorded
        assert!(!result.report().type_conversions.is_empty());
    }

    #[test]
    fn test_xportr_label() {
        let df = df! {
            "AGE" => &[25.0f64, 30.0],
            "NAME" => &["Alice", "Bob"],
        }
        .unwrap();

        let spec = create_test_spec();
        let result = df.xportr_label(&spec, ActionLevel::Warn).unwrap();

        // Check that label changes were recorded
        assert!(!result.report().label_changes.is_empty());
    }

    #[test]
    fn test_xportr_order() {
        // Columns in wrong order
        let df = df! {
            "NAME" => &["Alice", "Bob"],
            "AGE" => &[25.0f64, 30.0],
        }
        .unwrap();

        let spec = create_test_spec();
        let result = df.xportr_order(&spec, ActionLevel::Warn).unwrap();

        // Check that order changes were recorded
        assert!(!result.report().order_changes.is_empty());

        // Check that columns are now in correct order
        assert_eq!(
            result.column_names(),
            vec!["AGE".to_string(), "NAME".to_string()]
        );
    }

    #[test]
    fn test_xportr_pipeline_chain() {
        let df = df! {
            "NAME" => &["Alice", "Bob"],
            "AGE" => &["25", "30"],  // Wrong type
        }
        .unwrap();

        let spec = create_test_spec();

        let result = df
            .xportr_metadata(spec.clone())
            .xportr_type(&spec, ActionLevel::Warn)
            .unwrap()
            .xportr_label(&spec, ActionLevel::Warn)
            .unwrap()
            .xportr_order(&spec, ActionLevel::Warn)
            .unwrap()
            .xportr_df_label("Test Dataset");

        // All transforms should have been applied
        assert!(!result.report().type_conversions.is_empty());
        assert!(!result.report().label_changes.is_empty());
        assert!(!result.report().order_changes.is_empty());
        assert_eq!(result.label(), Some("Test Dataset"));
    }

    #[test]
    fn test_xportr_full_pipeline() {
        let df = df! {
            "NAME" => &["Alice", "Bob"],
            "AGE" => &[25.0f64, 30.0],
        }
        .unwrap();

        let spec = create_test_spec();
        let config = XportrConfig::default();

        let result = df.xportr(&spec, config).unwrap();

        // Should have processed data
        assert_eq!(result.height(), 2);
    }
}
