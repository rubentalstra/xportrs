//! Validation framework for XPT datasets.
//!
//! This module provides comprehensive validation for XPT datasets with
//! support for both basic format validation and agency-specific compliance
//! (FDA, NMPA, PMDA) via the policy system.
//!
//! # Architecture
//!
//! The validation framework uses a rule-based architecture:
//! - [`ValidationRule`] trait defines individual validation checks
//! - [`Validator`] orchestrates rule execution and collects results
//! - [`ValidationContext`] provides shared state during validation
//! - [`crate::policy::AgencyPolicy`] defines agency-specific constraints
//!
//! # Action Levels
//!
//! The `ActionLevel` enum controls how validation issues are reported:
//! - `None` - Don't report the issue
//! - `Message` - Report as informational message
//! - `Warn` - Report as warning (validation passes)
//! - `Stop` - Report as error (validation fails)
//!
//! # Example
//!
//! ```
//! use xportrs::validation::Validator;
//! use xportrs::policy::FdaPolicy;
//! use xportrs::{XptDataset, XptVersion};
//!
//! let dataset = XptDataset::new("DM");
//!
//! // Basic validation
//! let validator = Validator::basic(XptVersion::V5);
//! let result = validator.validate(&dataset);
//!
//! // Policy-based validation (recommended for submissions)
//! let validator = Validator::for_policy(FdaPolicy::strict());
//! let result = validator.validate(&dataset);
//!
//! if result.is_valid() {
//!     println!("Dataset is valid!");
//! }
//! ```

mod context;
pub mod rules;
mod severity;

pub use context::ValidationContext;
pub use severity::ActionLevel;

use std::sync::Arc;

use crate::XptVersion;
use crate::error::{
    ErrorLocation, Severity, ValidationError, ValidationErrorCode, ValidationResult,
};
use crate::policy::AgencyPolicy;
use crate::types::{XptColumn, XptDataset, XptValue};

/// A validation rule that can be applied to datasets or columns.
///
/// Rules implement specific validation checks and return errors/warnings.
pub trait ValidationRule: Send + Sync {
    /// The name of this rule for reporting.
    fn name(&self) -> &'static str;

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
///
/// # Example
///
/// ```
/// use xportrs::validation::{Validator, ActionLevel};
/// use xportrs::policy::FdaPolicy;
/// use xportrs::{XptDataset, XptVersion};
///
/// // Basic validation
/// let validator = Validator::basic(XptVersion::V5);
///
/// // With FDA policy (recommended for submissions)
/// let validator = Validator::for_policy(FdaPolicy::strict());
///
/// // With custom default action level
/// let validator = Validator::basic(XptVersion::V5)
///     .with_default_action(ActionLevel::Stop);
///
/// let dataset = XptDataset::new("DM");
/// let result = validator.validate(&dataset);
/// ```
pub struct Validator {
    version: XptVersion,
    policy: Option<Arc<dyn AgencyPolicy>>,
    rules: Vec<Box<dyn ValidationRule>>,
    default_action: ActionLevel,
}

impl Validator {
    /// Create a basic validator for the specified XPT version.
    ///
    /// Uses default rules without agency-specific constraints.
    #[must_use]
    pub fn basic(version: XptVersion) -> Self {
        let mut validator = Self {
            version,
            policy: None,
            rules: Vec::new(),
            default_action: ActionLevel::Warn,
        };
        validator.register_default_rules();
        validator
    }

    /// Create a validator with an agency policy.
    ///
    /// The policy determines additional constraints like version requirements,
    /// ASCII restrictions, and name length limits.
    #[must_use]
    pub fn for_policy(policy: impl AgencyPolicy + 'static) -> Self {
        let version = policy.required_version().unwrap_or(XptVersion::V5);
        let mut validator = Self {
            version,
            policy: Some(Arc::new(policy)),
            rules: Vec::new(),
            default_action: ActionLevel::Warn,
        };
        validator.register_default_rules();
        validator
    }

    /// Create a validator for FDA compliance.
    ///
    /// Shorthand for `Validator::for_policy(FdaPolicy::strict())`.
    #[must_use]
    pub fn fda() -> Self {
        Self::for_policy(crate::policy::FdaPolicy::strict())
    }

    /// Set the default action level for validation issues.
    ///
    /// This determines how issues without explicit severity are reported.
    #[must_use]
    pub fn with_default_action(mut self, action: ActionLevel) -> Self {
        self.default_action = action;
        self
    }

    /// Override the XPT version (useful for testing).
    #[must_use]
    pub fn with_version(mut self, version: XptVersion) -> Self {
        self.version = version;
        self
    }

    /// Add a custom validation rule.
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Get the XPT version.
    #[must_use]
    pub fn version(&self) -> XptVersion {
        self.version
    }

    /// Get the default action level.
    #[must_use]
    pub fn default_action(&self) -> ActionLevel {
        self.default_action
    }

    /// Get the policy, if set.
    #[must_use]
    pub fn policy(&self) -> Option<&dyn AgencyPolicy> {
        self.policy.as_ref().map(|p| p.as_ref())
    }

    /// Check if a policy is set.
    #[must_use]
    pub fn has_policy(&self) -> bool {
        self.policy.is_some()
    }

    /// Validate a dataset.
    #[must_use]
    pub fn validate(&self, dataset: &XptDataset) -> ValidationResult {
        let ctx = self.create_context();
        let mut result = ValidationResult::new();

        // Run dataset-level validation rules
        for rule in &self.rules {
            let errors = rule.validate_dataset(dataset, &ctx);
            for error in errors {
                result.add(error);
            }
        }

        // Run column-level validation rules
        for (index, column) in dataset.columns.iter().enumerate() {
            for rule in &self.rules {
                let errors = rule.validate_column(column, index, &dataset.name, &ctx);
                for error in errors {
                    result.add(error);
                }
            }
        }

        // Run policy-specific validation
        if let Some(ref policy) = self.policy {
            self.validate_policy(dataset, policy.as_ref(), &mut result);
        }

        result
    }

    /// Validate and return an error if validation fails.
    pub fn validate_or_error(&self, dataset: &XptDataset) -> Result<(), Vec<ValidationError>> {
        self.validate(dataset).into_result()
    }

    /// Validate a dataset against a specification.
    ///
    /// Runs both standard validation AND spec conformance checks.
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
    /// let validator = Validator::basic(XptVersion::V5);
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

    /// Validate against a specification with custom configuration.
    ///
    /// # Example
    ///
    /// ```
    /// use xportrs::validation::{Validator, ActionLevel, rules::SpecConformanceConfig};
    /// use xportrs::spec::{DatasetSpec, VariableSpec};
    /// use xportrs::{XptDataset, XptColumn, XptVersion};
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
    /// let validator = Validator::basic(XptVersion::V5);
    /// let result = validator.validate_against_spec_with_config(&dataset, &spec, config);
    /// ```
    #[must_use]
    pub fn validate_against_spec_with_config(
        &self,
        dataset: &XptDataset,
        spec: &crate::spec::DatasetSpec,
        config: rules::SpecConformanceConfig,
    ) -> ValidationResult {
        let mut result = self.validate(dataset);
        let ctx = self.create_context();
        let spec_rules = config.build_rules(spec);

        // Run spec conformance rules
        for rule in &spec_rules {
            let errors = rule.validate_dataset(dataset, &ctx);
            for error in errors {
                result.add(error);
            }
        }

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

    /// Validate with strict spec checking (all mismatches are errors).
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

    /// Create a validation context.
    fn create_context(&self) -> ValidationContext {
        ValidationContext::new(self.version, self.default_action)
    }

    /// Run policy-specific validation checks.
    fn validate_policy(
        &self,
        dataset: &XptDataset,
        policy: &dyn AgencyPolicy,
        result: &mut ValidationResult,
    ) {
        let severity = if policy.is_strict() {
            Severity::Error
        } else {
            Severity::Warning
        };

        // Check version requirement
        if let Some(required_version) = policy.required_version() {
            if self.version != required_version {
                result.add(ValidationError::new(
                    ValidationErrorCode::WrongVersion,
                    format!(
                        "{} requires XPT {} format, but {} is configured",
                        policy.agency(),
                        required_version,
                        self.version
                    ),
                    ErrorLocation::Dataset {
                        name: dataset.name.clone(),
                    },
                    severity,
                ));
            }
        }

        // Check ASCII requirement for character values
        if policy.require_ascii() {
            self.validate_ascii_values(dataset, severity, result);
        }
    }

    /// Check that all character values are ASCII.
    fn validate_ascii_values(
        &self,
        dataset: &XptDataset,
        severity: Severity,
        result: &mut ValidationResult,
    ) {
        for (row_idx, row) in dataset.rows.iter().enumerate() {
            for (col_idx, value) in row.iter().enumerate() {
                if let XptValue::Char(s) = value {
                    if !s.is_ascii() {
                        let col_name = dataset
                            .columns
                            .get(col_idx)
                            .map(|c| c.name.as_str())
                            .unwrap_or("unknown");

                        result.add(ValidationError::new(
                            ValidationErrorCode::NonAsciiValue,
                            format!(
                                "Non-ASCII character in column '{}' at row {}",
                                col_name, row_idx
                            ),
                            ErrorLocation::Value {
                                dataset: dataset.name.clone(),
                                column: col_name.to_string(),
                                row: row_idx,
                            },
                            severity,
                        ));
                    }
                }
            }
        }
    }

    /// Register default validation rules.
    fn register_default_rules(&mut self) {
        use rules::{
            DatasetLabelRule, DatasetNameRule, DuplicateVariableRule, FormatNameRule,
            VariableLabelRule, VariableLengthRule, VariableNameRule,
        };

        // Name validation
        self.rules.push(Box::new(DatasetNameRule));
        self.rules.push(Box::new(VariableNameRule));

        // Label validation
        self.rules.push(Box::new(DatasetLabelRule));
        self.rules.push(Box::new(VariableLabelRule));

        // Format validation
        self.rules.push(Box::new(FormatNameRule));

        // Structure validation
        self.rules.push(Box::new(DuplicateVariableRule));
        self.rules.push(Box::new(VariableLengthRule));
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::basic(XptVersion::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::FdaPolicy;
    use crate::types::XptColumn;

    #[test]
    fn test_validator_basic() {
        let validator = Validator::basic(XptVersion::V5);
        assert_eq!(validator.version(), XptVersion::V5);
        assert!(validator.policy().is_none());
        assert_eq!(validator.default_action(), ActionLevel::Warn);
    }

    #[test]
    fn test_validator_for_policy() {
        let validator = Validator::for_policy(FdaPolicy::strict());
        assert_eq!(validator.version(), XptVersion::V5); // FDA requires V5
        assert!(validator.policy().is_some());
    }

    #[test]
    fn test_validator_fda() {
        let validator = Validator::fda();
        assert!(validator.policy().is_some());
    }

    #[test]
    fn test_validator_with_default_action() {
        let validator = Validator::basic(XptVersion::V5).with_default_action(ActionLevel::Stop);
        assert_eq!(validator.default_action(), ActionLevel::Stop);
    }

    #[test]
    fn test_validate_empty_dataset() {
        let validator = Validator::basic(XptVersion::V5);
        let dataset = XptDataset::new("DM");
        let result = validator.validate(&dataset);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_valid_dataset() {
        let validator = Validator::basic(XptVersion::V5);
        let mut dataset = XptDataset::new("DM");
        dataset.columns.push(XptColumn::character("USUBJID", 20));
        dataset.columns.push(XptColumn::numeric("AGE"));

        let result = validator.validate(&dataset);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_invalid_name() {
        let validator = Validator::basic(XptVersion::V5);
        let dataset = XptDataset::new("TOOLONGNAME"); // > 8 chars for V5

        let result = validator.validate(&dataset);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_policy_version_check() {
        // FDA requires V5, but we're using V8
        let validator = Validator::for_policy(FdaPolicy::strict()).with_version(XptVersion::V8);
        let dataset = XptDataset::new("DM");

        let result = validator.validate(&dataset);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.code == ValidationErrorCode::WrongVersion)
        );
    }

    #[test]
    fn test_policy_ascii_check() {
        let validator = Validator::for_policy(FdaPolicy::strict());
        let mut dataset = XptDataset::new("DM");
        dataset.columns.push(XptColumn::character("NAME", 20));
        dataset.rows.push(vec![XptValue::character("Hëllo")]); // non-ASCII

        let result = validator.validate(&dataset);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.code == ValidationErrorCode::NonAsciiValue)
        );
    }

    #[test]
    fn test_validate_against_spec_passes() {
        use crate::spec::{DatasetSpec, VariableSpec};

        let validator = Validator::basic(XptVersion::V5);

        let spec = DatasetSpec::new("DM")
            .with_label("Demographics")
            .add_variable(
                VariableSpec::character("USUBJID", 20).with_label("Unique Subject Identifier"),
            )
            .add_variable(VariableSpec::numeric("AGE").with_label("Age"));

        let mut dataset = XptDataset::new("DM");
        dataset.label = Some("Demographics".to_string());
        dataset
            .columns
            .push(XptColumn::character("USUBJID", 20).with_label("Unique Subject Identifier"));
        dataset
            .columns
            .push(XptColumn::numeric("AGE").with_label("Age"));

        let result = validator.validate_against_spec(&dataset, &spec);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_against_spec_detects_missing_variable() {
        use crate::spec::{DatasetSpec, VariableSpec};

        let validator = Validator::basic(XptVersion::V5);

        let spec = DatasetSpec::new("DM")
            .add_variable(VariableSpec::character("USUBJID", 20))
            .add_variable(VariableSpec::character("SEX", 1)); // Missing in data

        let mut dataset = XptDataset::new("DM");
        dataset.columns.push(XptColumn::character("USUBJID", 20));

        let result = validator.validate_against_spec(&dataset, &spec);
        assert!(result.warnings.iter().any(|w| {
            w.code == ValidationErrorCode::VariableNotInData && w.message.contains("SEX")
        }));
    }

    #[test]
    fn test_validate_against_spec_strict_fails_on_mismatch() {
        use crate::spec::{DatasetSpec, VariableSpec};

        let validator = Validator::basic(XptVersion::V5);

        let spec = DatasetSpec::new("DM").add_variable(VariableSpec::numeric("AGE"));

        let mut dataset = XptDataset::new("DM");
        dataset.columns.push(XptColumn::character("AGE", 10)); // Wrong type

        let result = validator.validate_against_spec_strict(&dataset, &spec);
        assert!(!result.is_valid());
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.code == ValidationErrorCode::TypeMismatch)
        );
    }

    #[test]
    fn test_validate_against_spec_with_config() {
        use crate::spec::{DatasetSpec, VariableSpec};

        let validator = Validator::basic(XptVersion::V5);

        let spec = DatasetSpec::new("DM")
            .add_variable(VariableSpec::character("USUBJID", 20).with_order(2))
            .add_variable(VariableSpec::numeric("AGE").with_order(1));

        let mut dataset = XptDataset::new("DM");
        // Wrong order
        dataset.columns.push(XptColumn::character("USUBJID", 20));
        dataset.columns.push(XptColumn::numeric("AGE"));

        // Skip order checking
        let config = rules::SpecConformanceConfig {
            order_action: ActionLevel::None,
            ..Default::default()
        };

        let result = validator.validate_against_spec_with_config(&dataset, &spec, config);
        assert!(result.is_valid());
    }
}
