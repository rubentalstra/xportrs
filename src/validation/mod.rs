//! Validation framework for XPT datasets.
//!
//! This module provides comprehensive validation for XPT datasets,
//! supporting both V5 and V8 formats with FDA compliance checking.
//!
//! # Architecture
//!
//! The validation framework uses a rule-based architecture:
//! - [`ValidationRule`] trait defines individual validation checks
//! - [`Validator`] orchestrates rule execution and collects results
//! - [`ValidationContext`] provides shared state during validation
//!
//! # Example
//!
//! ```
//! use xportrs::validation::{Validator, ValidationMode};
//! use xportrs::{XptDataset, XptVersion};
//!
//! let dataset = XptDataset::new("DM");
//! let validator = Validator::new(XptVersion::V5)
//!     .with_mode(ValidationMode::FdaCompliant);
//!
//! let result = validator.validate(&dataset);
//! if result.is_valid() {
//!     println!("Dataset is valid!");
//! } else {
//!     for error in &result.errors {
//!         eprintln!("Error: {}", error);
//!     }
//! }
//! ```

mod context;
pub mod rules;
mod severity;

pub use context::ValidationContext;
pub use severity::ActionLevel;

use crate::error::{ValidationError, ValidationResult};
use crate::types::{XptColumn, XptDataset};
use crate::XptVersion;

/// Validation mode determining which rules to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidationMode {
    /// Basic XPT format validation only.
    #[default]
    Basic,
    /// Full FDA compliance validation (stricter).
    FdaCompliant,
    /// Custom validation with specific rules enabled.
    Custom,
}

/// A validation rule that can be applied to datasets or columns.
pub trait ValidationRule: Send + Sync {
    /// The name of this rule for reporting purposes.
    fn name(&self) -> &'static str;

    /// Whether this rule applies in the given mode.
    fn applies_to(&self, mode: ValidationMode) -> bool;

    /// Validate an entire dataset.
    fn validate_dataset(
        &self,
        _dataset: &XptDataset,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        Vec::new()
    }

    /// Validate a single column.
    fn validate_column(
        &self,
        _column: &XptColumn,
        _index: usize,
        _dataset_name: &str,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        Vec::new()
    }
}

/// Dataset validator that applies validation rules and collects errors.
///
/// The validator uses a collect-all-errors pattern, running all applicable
/// rules and returning all validation issues at once.
pub struct Validator {
    version: XptVersion,
    mode: ValidationMode,
    rules: Vec<Box<dyn ValidationRule>>,
}

impl Validator {
    /// Create a new validator for the specified XPT version.
    #[must_use]
    pub fn new(version: XptVersion) -> Self {
        let mut validator = Self {
            version,
            mode: ValidationMode::Basic,
            rules: Vec::new(),
        };
        validator.register_default_rules();
        validator
    }

    /// Create a validator with FDA compliance mode.
    #[must_use]
    pub fn fda_compliant(version: XptVersion) -> Self {
        Self::new(version).with_mode(ValidationMode::FdaCompliant)
    }

    /// Set the validation mode.
    #[must_use]
    pub fn with_mode(mut self, mode: ValidationMode) -> Self {
        self.mode = mode;
        self
    }

    /// Add a custom validation rule.
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Get the XPT version being validated against.
    #[must_use]
    pub fn version(&self) -> XptVersion {
        self.version
    }

    /// Get the current validation mode.
    #[must_use]
    pub fn mode(&self) -> ValidationMode {
        self.mode
    }

    /// Validate a dataset and return all errors/warnings.
    #[must_use]
    pub fn validate(&self, dataset: &XptDataset) -> ValidationResult {
        let ctx = ValidationContext::new(self.version, self.mode);
        let mut result = ValidationResult::new();

        // Run dataset-level validation rules
        for rule in &self.rules {
            if rule.applies_to(self.mode) {
                let errors = rule.validate_dataset(dataset, &ctx);
                for error in errors {
                    result.add(error);
                }
            }
        }

        // Run column-level validation rules
        for (index, column) in dataset.columns.iter().enumerate() {
            for rule in &self.rules {
                if rule.applies_to(self.mode) {
                    let errors = rule.validate_column(column, index, &dataset.name, &ctx);
                    for error in errors {
                        result.add(error);
                    }
                }
            }
        }

        result
    }

    /// Validate and return an error if validation fails.
    ///
    /// This is a convenience method that returns `Ok(())` if validation passes,
    /// or `Err` with all validation errors if it fails.
    pub fn validate_or_error(&self, dataset: &XptDataset) -> Result<(), Vec<ValidationError>> {
        let result = self.validate(dataset);
        result.into_result()
    }

    /// Validate a dataset against a specification.
    ///
    /// This method validates both the standard XPT format rules AND checks
    /// that the dataset conforms to the given specification. It detects:
    /// - Variables in data not in spec
    /// - Variables in spec not in data
    /// - Type mismatches
    /// - Length mismatches
    /// - Order mismatches
    /// - Format mismatches
    /// - Label mismatches
    /// - Dataset metadata mismatches
    ///
    /// # Example
    ///
    /// ```
    /// use xportrs::validation::Validator;
    /// use xportrs::spec::{DatasetSpec, VariableSpec};
    /// use xportrs::{XptDataset, XptColumn, XptVersion};
    ///
    /// let spec = DatasetSpec::new("DM")
    ///     .add_variable(VariableSpec::character("USUBJID", 20));
    ///
    /// let mut dataset = XptDataset::new("DM");
    /// dataset.columns.push(XptColumn::character("USUBJID", 20));
    ///
    /// let validator = Validator::new(XptVersion::V5);
    /// let result = validator.validate_against_spec(&dataset, &spec);
    /// assert!(result.is_valid());
    /// ```
    #[must_use]
    pub fn validate_against_spec(
        &self,
        dataset: &XptDataset,
        spec: &crate::spec::DatasetSpec,
    ) -> ValidationResult {
        self.validate_against_spec_with_config(
            dataset,
            spec,
            rules::SpecConformanceConfig::default(),
        )
    }

    /// Validate a dataset against a specification with custom configuration.
    ///
    /// This allows fine-grained control over which conformance checks are
    /// performed and at what severity level.
    ///
    /// # Example
    ///
    /// ```
    /// use xportrs::validation::{Validator, rules::SpecConformanceConfig};
    /// use xportrs::spec::{DatasetSpec, VariableSpec};
    /// use xportrs::{XptDataset, XptColumn, XptVersion, ActionLevel};
    ///
    /// let spec = DatasetSpec::new("DM")
    ///     .add_variable(VariableSpec::character("USUBJID", 20));
    ///
    /// let mut dataset = XptDataset::new("DM");
    /// dataset.columns.push(XptColumn::character("USUBJID", 20));
    ///
    /// let config = SpecConformanceConfig {
    ///     type_action: ActionLevel::Stop,  // Type mismatch is fatal
    ///     order_action: ActionLevel::None, // Don't check order
    ///     ..Default::default()
    /// };
    ///
    /// let validator = Validator::new(XptVersion::V5);
    /// let result = validator.validate_against_spec_with_config(&dataset, &spec, config);
    /// ```
    #[must_use]
    pub fn validate_against_spec_with_config(
        &self,
        dataset: &XptDataset,
        spec: &crate::spec::DatasetSpec,
        config: rules::SpecConformanceConfig,
    ) -> ValidationResult {
        // First run standard validation
        let mut result = self.validate(dataset);

        // Then run spec conformance rules
        let ctx = ValidationContext::new(self.version, self.mode);
        let spec_rules = config.build_rules(spec);

        // Run dataset-level spec conformance rules
        for rule in &spec_rules {
            let errors = rule.validate_dataset(dataset, &ctx);
            for error in errors {
                result.add(error);
            }
        }

        // Run column-level spec conformance rules
        for (index, column) in dataset.columns.iter().enumerate() {
            for rule in &spec_rules {
                let errors = rule.validate_column(column, index, &dataset.name, &ctx);
                for error in errors {
                    result.add(error);
                }
            }
        }

        result
    }

    /// Validate a dataset against a specification with strict checking.
    ///
    /// Uses `SpecConformanceConfig::strict()` which treats all mismatches as errors.
    #[must_use]
    pub fn validate_against_spec_strict(
        &self,
        dataset: &XptDataset,
        spec: &crate::spec::DatasetSpec,
    ) -> ValidationResult {
        self.validate_against_spec_with_config(
            dataset,
            spec,
            rules::SpecConformanceConfig::strict(),
        )
    }

    /// Register the default set of validation rules.
    fn register_default_rules(&mut self) {
        use rules::{
            DatasetLabelRule, DatasetNameRule, DuplicateVariableRule, FdaAsciiRule, FdaVersionRule,
            FormatNameRule, VariableLabelRule, VariableLengthRule, VariableNameRule,
        };

        // Name validation rules
        self.rules.push(Box::new(DatasetNameRule));
        self.rules.push(Box::new(VariableNameRule));

        // Label validation rules
        self.rules.push(Box::new(DatasetLabelRule));
        self.rules.push(Box::new(VariableLabelRule));

        // Format validation rules
        self.rules.push(Box::new(FormatNameRule));

        // Dataset structure rules
        self.rules.push(Box::new(DuplicateVariableRule));
        self.rules.push(Box::new(VariableLengthRule));

        // FDA-specific rules
        self.rules.push(Box::new(FdaVersionRule));
        self.rules.push(Box::new(FdaAsciiRule));
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new(XptVersion::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::XptColumn;

    #[test]
    fn test_validator_new() {
        let validator = Validator::new(XptVersion::V5);
        assert_eq!(validator.version(), XptVersion::V5);
        assert_eq!(validator.mode(), ValidationMode::Basic);
    }

    #[test]
    fn test_validator_fda_compliant() {
        let validator = Validator::fda_compliant(XptVersion::V5);
        assert_eq!(validator.mode(), ValidationMode::FdaCompliant);
    }

    #[test]
    fn test_validate_empty_dataset() {
        let validator = Validator::new(XptVersion::V5);
        let dataset = XptDataset::new("DM");
        let result = validator.validate(&dataset);
        // Empty dataset should still be valid (no columns is allowed)
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_valid_dataset() {
        let validator = Validator::new(XptVersion::V5);
        let mut dataset = XptDataset::new("DM");
        dataset.columns.push(XptColumn::character("USUBJID", 20));
        dataset.columns.push(XptColumn::numeric("AGE"));

        let result = validator.validate(&dataset);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_invalid_name() {
        let validator = Validator::new(XptVersion::V5);
        let mut dataset = XptDataset::new("TOOLONGNAME"); // > 8 chars for V5
        dataset.columns.push(XptColumn::character("USUBJID", 20));

        let result = validator.validate(&dataset);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_validate_against_spec_passes() {
        use crate::spec::{DatasetSpec, VariableSpec};

        let validator = Validator::new(XptVersion::V5);

        let spec = DatasetSpec::new("DM")
            .with_label("Demographics")
            .add_variable(
                VariableSpec::character("USUBJID", 20)
                    .with_label("Unique Subject Identifier")
                    .with_order(1),
            )
            .add_variable(
                VariableSpec::numeric("AGE")
                    .with_label("Age")
                    .with_order(2),
            );

        let mut dataset = XptDataset::new("DM");
        dataset.label = Some("Demographics".to_string());
        dataset.columns.push(
            XptColumn::character("USUBJID", 20).with_label("Unique Subject Identifier"),
        );
        dataset.columns.push(XptColumn::numeric("AGE").with_label("Age"));

        let result = validator.validate_against_spec(&dataset, &spec);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_against_spec_detects_missing_variable() {
        use crate::spec::{DatasetSpec, VariableSpec};
        use crate::error::ValidationErrorCode;

        let validator = Validator::new(XptVersion::V5);

        let spec = DatasetSpec::new("DM")
            .add_variable(VariableSpec::character("USUBJID", 20))
            .add_variable(VariableSpec::numeric("AGE"))
            .add_variable(VariableSpec::character("SEX", 1)); // Missing in data

        let mut dataset = XptDataset::new("DM");
        dataset.columns.push(XptColumn::character("USUBJID", 20));
        dataset.columns.push(XptColumn::numeric("AGE"));

        let result = validator.validate_against_spec(&dataset, &spec);
        // Should have warning about SEX not in data
        assert!(result.warnings.iter().any(|w| {
            w.code == ValidationErrorCode::VariableNotInData && w.message.contains("SEX")
        }));
    }

    #[test]
    fn test_validate_against_spec_detects_extra_variable() {
        use crate::spec::{DatasetSpec, VariableSpec};
        use crate::error::ValidationErrorCode;

        let validator = Validator::new(XptVersion::V5);

        let spec = DatasetSpec::new("DM")
            .add_variable(VariableSpec::character("USUBJID", 20));

        let mut dataset = XptDataset::new("DM");
        dataset.columns.push(XptColumn::character("USUBJID", 20));
        dataset.columns.push(XptColumn::numeric("EXTRA")); // Not in spec

        let result = validator.validate_against_spec(&dataset, &spec);
        // Should have warning about EXTRA not in spec
        assert!(result.warnings.iter().any(|w| {
            w.code == ValidationErrorCode::VariableNotInSpec && w.message.contains("EXTRA")
        }));
    }

    #[test]
    fn test_validate_against_spec_strict_fails_on_mismatch() {
        use crate::spec::{DatasetSpec, VariableSpec};
        use crate::error::ValidationErrorCode;

        let validator = Validator::new(XptVersion::V5);

        let spec = DatasetSpec::new("DM")
            .add_variable(VariableSpec::numeric("AGE")); // Spec says numeric

        let mut dataset = XptDataset::new("DM");
        dataset.columns.push(XptColumn::character("AGE", 10)); // Data is character

        let result = validator.validate_against_spec_strict(&dataset, &spec);
        // Should have error (not warning) about type mismatch
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.code == ValidationErrorCode::TypeMismatch));
    }

    #[test]
    fn test_validate_against_spec_with_config() {
        use crate::spec::{DatasetSpec, VariableSpec};
        use crate::validation::rules::SpecConformanceConfig;

        let validator = Validator::new(XptVersion::V5);

        let spec = DatasetSpec::new("DM")
            .add_variable(VariableSpec::character("USUBJID", 20).with_order(2))
            .add_variable(VariableSpec::numeric("AGE").with_order(1));

        let mut dataset = XptDataset::new("DM");
        // Wrong order: USUBJID first, AGE second
        dataset.columns.push(XptColumn::character("USUBJID", 20));
        dataset.columns.push(XptColumn::numeric("AGE"));

        // Skip order checking
        let config = SpecConformanceConfig {
            order_action: ActionLevel::None,
            ..Default::default()
        };

        let result = validator.validate_against_spec_with_config(&dataset, &spec, config);
        // Should be valid since we disabled order checking
        assert!(result.is_valid());
    }
}
