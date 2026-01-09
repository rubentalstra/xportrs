//! Full xportr pipeline (xportr and xportr_write equivalents).
//!
//! This module provides the complete xportr-style pipeline that applies all
//! metadata transforms in sequence and writes the result to an XPT file.

use std::path::Path;

use crate::XptVersion;
use crate::core::writer::write_xpt_with_options;
use crate::error::TransformError;
use crate::spec::DatasetSpec;
use crate::types::{XptDataset, XptWriterOptions};
use crate::validation::ActionLevel;

use super::apply_df_label::apply_df_label;
use super::apply_format::{ApplyFormatConfig, apply_format};
use super::apply_label::{ApplyLabelConfig, apply_label};
use super::apply_length::{ApplyLengthConfig, apply_length};
use super::apply_order::{ApplyOrderConfig, apply_order};
use super::coerce_type::{CoerceTypeConfig, coerce_type};
use super::report::TransformReport;

/// Configuration for the full xportr pipeline.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XportrConfig {
    /// Action level for type coercion issues.
    pub type_action: ActionLevel,

    /// Action level for label application issues.
    pub label_action: ActionLevel,

    /// Action level for length application issues.
    pub length_action: ActionLevel,

    /// Action level for order application issues.
    pub order_action: ActionLevel,

    /// Action level for format application issues.
    pub format_action: ActionLevel,

    /// Whether to enforce strict checks (treat warnings as errors).
    pub strict_checks: bool,

    /// Whether to apply type coercion.
    pub apply_type: bool,

    /// Whether to apply length.
    pub apply_length: bool,

    /// Whether to apply labels.
    pub apply_label: bool,

    /// Whether to apply ordering.
    pub apply_order: bool,

    /// Whether to apply formats.
    pub apply_format: bool,

    /// XPT version to use for writing.
    pub version: XptVersion,
}

impl Default for XportrConfig {
    fn default() -> Self {
        Self {
            type_action: ActionLevel::Warn,
            label_action: ActionLevel::Warn,
            length_action: ActionLevel::Warn,
            order_action: ActionLevel::Message,
            format_action: ActionLevel::Message,
            strict_checks: false,
            apply_type: true,
            apply_length: true,
            apply_label: true,
            apply_order: true,
            apply_format: true,
            version: XptVersion::V5,
        }
    }
}

impl XportrConfig {
    /// Create a new config with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a strict FDA-compliant configuration.
    ///
    /// All transforms are applied with Stop action level.
    #[must_use]
    pub fn fda_strict() -> Self {
        Self {
            type_action: ActionLevel::Stop,
            label_action: ActionLevel::Stop,
            length_action: ActionLevel::Stop,
            order_action: ActionLevel::Stop,
            format_action: ActionLevel::Stop,
            strict_checks: true,
            apply_type: true,
            apply_length: true,
            apply_label: true,
            apply_order: true,
            apply_format: true,
            version: XptVersion::V5,
        }
    }

    /// Create a lenient FDA configuration.
    ///
    /// All transforms are applied with Warn action level.
    #[must_use]
    pub fn fda_lenient() -> Self {
        Self {
            type_action: ActionLevel::Warn,
            label_action: ActionLevel::Warn,
            length_action: ActionLevel::Warn,
            order_action: ActionLevel::Warn,
            format_action: ActionLevel::Warn,
            strict_checks: false,
            apply_type: true,
            apply_length: true,
            apply_label: true,
            apply_order: true,
            apply_format: true,
            version: XptVersion::V5,
        }
    }

    /// Set the XPT version for output.
    #[must_use]
    pub fn with_version(mut self, version: XptVersion) -> Self {
        self.version = version;
        self
    }

    /// Set whether to apply type coercion.
    #[must_use]
    pub fn with_type(mut self, apply: bool) -> Self {
        self.apply_type = apply;
        self
    }

    /// Set whether to apply length.
    #[must_use]
    pub fn with_length(mut self, apply: bool) -> Self {
        self.apply_length = apply;
        self
    }

    /// Set whether to apply labels.
    #[must_use]
    pub fn with_label(mut self, apply: bool) -> Self {
        self.apply_label = apply;
        self
    }

    /// Set whether to apply ordering.
    #[must_use]
    pub fn with_order(mut self, apply: bool) -> Self {
        self.apply_order = apply;
        self
    }

    /// Set whether to apply formats.
    #[must_use]
    pub fn with_format(mut self, apply: bool) -> Self {
        self.apply_format = apply;
        self
    }

    /// Set strict checks mode.
    #[must_use]
    pub fn with_strict_checks(mut self, strict: bool) -> Self {
        self.strict_checks = strict;
        self
    }
}

/// Result of the full xportr pipeline.
#[derive(Debug)]
pub struct XportrResult {
    /// The transformed dataset.
    pub dataset: XptDataset,
    /// Comprehensive report of all changes.
    pub report: PipelineReport,
}

/// Comprehensive report of all pipeline changes.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PipelineReport {
    /// Report from type coercion transform.
    pub type_report: TransformReport,
    /// Report from length application transform.
    pub length_report: TransformReport,
    /// Report from label application transform.
    pub label_report: TransformReport,
    /// Report from order application transform.
    pub order_report: TransformReport,
    /// Report from format application transform.
    pub format_report: TransformReport,
    /// Whether the dataset label was changed.
    pub dataset_label_changed: bool,
    /// Old dataset label (if changed).
    pub old_dataset_label: Option<String>,
    /// New dataset label (if changed).
    pub new_dataset_label: Option<String>,
    /// All errors encountered during the pipeline.
    pub errors: Vec<String>,
}

impl PipelineReport {
    /// Create a new empty report.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the pipeline completed without blocking errors.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if any warnings were generated.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        self.type_report.has_warnings()
            || self.length_report.has_warnings()
            || self.label_report.has_warnings()
            || self.order_report.has_warnings()
            || self.format_report.has_warnings()
    }

    /// Check if any changes were made.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        self.type_report.has_changes()
            || self.length_report.has_changes()
            || self.label_report.has_changes()
            || self.order_report.has_changes()
            || self.format_report.has_changes()
            || self.dataset_label_changed
    }

    /// Get the total number of changes.
    #[must_use]
    pub fn total_changes(&self) -> usize {
        let mut total = self.type_report.total_changes()
            + self.length_report.total_changes()
            + self.label_report.total_changes()
            + self.order_report.total_changes()
            + self.format_report.total_changes();
        if self.dataset_label_changed {
            total += 1;
        }
        total
    }

    /// Generate a human-readable summary.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.type_report.type_conversions.is_empty() {
            parts.push(format!(
                "{} type conversion(s)",
                self.type_report.type_conversions.len()
            ));
        }
        if !self.length_report.length_changes.is_empty() {
            parts.push(format!(
                "{} length change(s)",
                self.length_report.length_changes.len()
            ));
        }
        if !self.label_report.label_changes.is_empty() {
            parts.push(format!(
                "{} label change(s)",
                self.label_report.label_changes.len()
            ));
        }
        if !self.order_report.order_changes.is_empty() {
            parts.push(format!(
                "{} order change(s)",
                self.order_report.order_changes.len()
            ));
        }
        if !self.format_report.format_changes.is_empty() {
            parts.push(format!(
                "{} format change(s)",
                self.format_report.format_changes.len()
            ));
        }
        if self.dataset_label_changed {
            parts.push("dataset label changed".to_string());
        }

        let total_warnings = self.type_report.warnings.len()
            + self.length_report.warnings.len()
            + self.label_report.warnings.len()
            + self.order_report.warnings.len()
            + self.format_report.warnings.len();
        if total_warnings > 0 {
            parts.push(format!("{total_warnings} warning(s)"));
        }

        if !self.errors.is_empty() {
            parts.push(format!("{} error(s)", self.errors.len()));
        }

        if parts.is_empty() {
            "No changes".to_string()
        } else {
            parts.join(", ")
        }
    }

    /// Get all warnings from all transforms.
    #[must_use]
    pub fn all_warnings(&self) -> Vec<&str> {
        let mut warnings = Vec::new();
        for w in &self.type_report.warnings {
            warnings.push(w.as_str());
        }
        for w in &self.length_report.warnings {
            warnings.push(w.as_str());
        }
        for w in &self.label_report.warnings {
            warnings.push(w.as_str());
        }
        for w in &self.order_report.warnings {
            warnings.push(w.as_str());
        }
        for w in &self.format_report.warnings {
            warnings.push(w.as_str());
        }
        warnings
    }
}

/// Apply the full xportr pipeline to a dataset.
///
/// This is equivalent to R's `xportr()` function. It applies all transforms
/// in the correct order:
///
/// 1. Type coercion (`xportr_type`)
/// 2. Length application (`xportr_length`)
/// 3. Label application (`xportr_label`)
/// 4. Order application (`xportr_order`)
/// 5. Format application (`xportr_format`)
/// 6. Dataset label (`xportr_df_label`)
///
/// # Arguments
///
/// * `dataset` - The dataset to transform
/// * `spec` - The specification to apply
/// * `config` - Pipeline configuration
///
/// # Errors
///
/// Returns error if any transform fails with `ActionLevel::Stop` configured.
///
/// # Example
///
/// ```
/// use xportrs::{XptDataset, XptColumn, XptValue, DatasetSpec, VariableSpec};
/// use xportrs::transform::{xportr, XportrConfig};
///
/// let mut dataset = XptDataset::with_columns("DM", vec![
///     XptColumn::character("AGE", 8),
///     XptColumn::character("SEX", 1),
/// ]);
/// dataset.add_row(vec![XptValue::character("25"), XptValue::character("M")]);
///
/// let spec = DatasetSpec::new("DM")
///     .with_label("Demographics")
///     .add_variable(VariableSpec::numeric("AGE").with_label("Age").with_order(1))
///     .add_variable(VariableSpec::character("SEX", 1).with_label("Sex").with_order(2));
///
/// let result = xportr(dataset, &spec, XportrConfig::default()).unwrap();
/// assert!(result.dataset.columns[0].is_numeric()); // Type coerced
/// assert_eq!(result.dataset.label, Some("Demographics".to_string()));
/// ```
pub fn xportr(
    dataset: XptDataset,
    spec: &DatasetSpec,
    config: XportrConfig,
) -> Result<XportrResult, TransformError> {
    let mut report = PipelineReport::new();
    let mut current_dataset = dataset;

    // 1. Type coercion
    if config.apply_type {
        let type_config = CoerceTypeConfig::new().with_action(config.type_action);
        let result = coerce_type(current_dataset, spec, type_config)?;
        current_dataset = result.dataset;
        report.type_report.type_conversions = result.conversions;
        report.type_report.warnings = result.warnings;
    }

    // 2. Length application
    if config.apply_length {
        let length_config = ApplyLengthConfig::new().with_action(config.length_action);
        let result = apply_length(current_dataset, spec, length_config)?;
        current_dataset = result.dataset;
        report.length_report.length_changes = result.changes;
        report.length_report.warnings = result.warnings;
    }

    // 3. Label application
    if config.apply_label {
        let label_config = ApplyLabelConfig::new().with_action(config.label_action);
        let result = apply_label(current_dataset, spec, label_config)?;
        current_dataset = result.dataset;
        report.label_report.label_changes = result.changes;
        report.label_report.warnings = result.warnings;
    }

    // 4. Order application
    if config.apply_order {
        let order_config = ApplyOrderConfig::new().with_action(config.order_action);
        let result = apply_order(current_dataset, spec, order_config)?;
        current_dataset = result.dataset;
        report.order_report.order_changes = result.changes;
        report.order_report.warnings = result.warnings;
    }

    // 5. Format application
    if config.apply_format {
        let format_config = ApplyFormatConfig::new().with_action(config.format_action);
        let result = apply_format(current_dataset, spec, format_config)?;
        current_dataset = result.dataset;
        report.format_report.format_changes = result.changes;
        report.format_report.warnings = result.warnings;
    }

    // 6. Dataset label
    if let Some(label) = &spec.label {
        report.old_dataset_label = current_dataset.label.clone();
        if current_dataset.label.as_ref() != Some(label) {
            current_dataset = apply_df_label(current_dataset, label);
            report.dataset_label_changed = true;
            report.new_dataset_label = Some(label.clone());
        }
    }

    Ok(XportrResult {
        dataset: current_dataset,
        report,
    })
}

/// Apply the full xportr pipeline and write to an XPT file.
///
/// This is equivalent to R's `xportr_write()` function. It applies the full
/// xportr pipeline and then writes the result to a file.
///
/// # Arguments
///
/// * `dataset` - The dataset to transform
/// * `spec` - The specification to apply
/// * `path` - Path to write the XPT file
/// * `config` - Pipeline configuration
///
/// # Errors
///
/// Returns error if any transform fails or if file writing fails.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use xportrs::{XptDataset, XptColumn, XptValue, DatasetSpec, VariableSpec};
/// use xportrs::transform::{xportr_write, XportrConfig};
///
/// let mut dataset = XptDataset::with_columns("DM", vec![
///     XptColumn::numeric("AGE"),
/// ]);
/// dataset.add_row(vec![XptValue::numeric(25.0)]);
///
/// let spec = DatasetSpec::new("DM")
///     .with_label("Demographics")
///     .add_variable(VariableSpec::numeric("AGE").with_label("Age"));
///
/// let report = xportr_write(dataset, &spec, Path::new("dm.xpt"), XportrConfig::default()).unwrap();
/// println!("Changes: {}", report.summary());
/// ```
pub fn xportr_write(
    dataset: XptDataset,
    spec: &DatasetSpec,
    path: impl AsRef<Path>,
    config: XportrConfig,
) -> Result<PipelineReport, TransformError> {
    let version = config.version;
    let result = xportr(dataset, spec, config)?;

    let options = XptWriterOptions::new().with_version(version);
    write_xpt_with_options(path.as_ref(), &result.dataset, &options)?;

    Ok(result.report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::VariableSpec;
    use crate::types::{FormatSpec, XptColumn, XptValue};

    #[test]
    fn test_xportr_config_default() {
        let config = XportrConfig::default();
        assert_eq!(config.type_action, ActionLevel::Warn);
        assert!(!config.strict_checks);
        assert!(config.apply_type);
        assert_eq!(config.version, XptVersion::V5);
    }

    #[test]
    fn test_xportr_config_fda_strict() {
        let config = XportrConfig::fda_strict();
        assert_eq!(config.type_action, ActionLevel::Stop);
        assert!(config.strict_checks);
    }

    #[test]
    fn test_xportr_config_fda_lenient() {
        let config = XportrConfig::fda_lenient();
        assert_eq!(config.type_action, ActionLevel::Warn);
        assert!(!config.strict_checks);
    }

    #[test]
    fn test_xportr_full_pipeline() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![
                XptColumn::character("AGE", 8),    // Wrong type
                XptColumn::character("NAME", 100), // Wrong length
            ],
        );
        dataset.add_row(vec![XptValue::character("25"), XptValue::character("John")]);

        let spec = DatasetSpec::new("TEST")
            .with_label("Test Dataset")
            .add_variable(
                VariableSpec::numeric("AGE")
                    .with_label("Age in Years")
                    .with_format(FormatSpec::best(8))
                    .with_order(1),
            )
            .add_variable(
                VariableSpec::character("NAME", 20)
                    .with_label("Subject Name")
                    .with_order(2),
            );

        let result = xportr(dataset, &spec, XportrConfig::default()).unwrap();

        // Type was coerced
        assert!(result.dataset.columns[0].is_numeric());
        assert_eq!(result.dataset.rows[0][0].as_f64(), Some(25.0));

        // Length was applied
        assert_eq!(result.dataset.columns[1].length, 20);

        // Labels were applied
        assert_eq!(
            result.dataset.columns[0].label,
            Some("Age in Years".to_string())
        );
        assert_eq!(
            result.dataset.columns[1].label,
            Some("Subject Name".to_string())
        );

        // Format was applied
        assert_eq!(result.dataset.columns[0].format, Some("BEST".to_string()));

        // Dataset label was applied
        assert_eq!(result.dataset.label, Some("Test Dataset".to_string()));

        // Report has changes
        assert!(result.report.has_changes());
        assert!(!result.report.type_report.type_conversions.is_empty());
        assert!(!result.report.length_report.length_changes.is_empty());
        assert!(!result.report.label_report.label_changes.is_empty());
        assert!(!result.report.format_report.format_changes.is_empty());
        assert!(result.report.dataset_label_changed);
    }

    #[test]
    fn test_xportr_no_changes() {
        let mut dataset = XptDataset::with_columns(
            "TEST",
            vec![XptColumn::numeric("AGE").with_label("Age in Years")],
        );
        dataset.add_row(vec![XptValue::numeric(25.0)]);

        let spec = DatasetSpec::new("TEST")
            .add_variable(VariableSpec::numeric("AGE").with_label("Age in Years"));

        let result = xportr(dataset, &spec, XportrConfig::default()).unwrap();

        // No type changes (already numeric)
        assert!(result.report.type_report.type_conversions.is_empty());
        // No label changes (already has same label)
        assert!(result.report.label_report.label_changes.is_empty());
    }

    #[test]
    fn test_xportr_selective_transforms() {
        let mut dataset = XptDataset::with_columns("TEST", vec![XptColumn::character("AGE", 8)]);
        dataset.add_row(vec![XptValue::character("25")]);

        let spec = DatasetSpec::new("TEST")
            .with_label("Test")
            .add_variable(VariableSpec::numeric("AGE").with_label("Age"));

        // Disable type coercion
        let config = XportrConfig::default().with_type(false);
        let result = xportr(dataset, &spec, config).unwrap();

        // Should still be character (type not coerced)
        assert!(result.dataset.columns[0].is_character());
        // But label should still be applied
        assert_eq!(result.dataset.columns[0].label, Some("Age".to_string()));
    }

    #[test]
    fn test_pipeline_report() {
        let report = PipelineReport::new();
        assert!(report.is_valid());
        assert!(!report.has_warnings());
        assert!(!report.has_changes());
        assert_eq!(report.summary(), "No changes");
    }

    #[test]
    fn test_pipeline_report_with_changes() {
        let mut report = PipelineReport::new();
        report
            .type_report
            .type_conversions
            .push(super::super::report::TypeConversion::new(
                "AGE", "Char", "Num",
            ));
        report.type_report.warnings.push("Test warning".into());
        report.dataset_label_changed = true;

        assert!(report.is_valid());
        assert!(report.has_warnings());
        assert!(report.has_changes());
        assert_eq!(report.total_changes(), 2); // 1 type + 1 label

        let summary = report.summary();
        assert!(summary.contains("1 type conversion"));
        assert!(summary.contains("dataset label changed"));
        assert!(summary.contains("1 warning"));
    }
}
