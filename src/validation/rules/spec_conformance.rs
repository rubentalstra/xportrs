//! Spec conformance validation rules.
//!
//! This module provides validation rules that check whether a dataset
//! conforms to a given specification. These rules are used by
//! `Validator::validate_against_spec()` to detect mismatches between
//! actual data and expected metadata.
//!
//! # Rules
//!
//! - [`VariableInSpecRule`] - Warns when data has variables not in spec
//! - [`VariableInDataRule`] - Warns when spec has variables not in data
//! - [`TypeConformanceRule`] - Checks variable types match spec
//! - [`LengthConformanceRule`] - Checks variable lengths match spec
//! - [`OrderConformanceRule`] - Checks variable order matches spec
//! - [`FormatConformanceRule`] - Checks variable formats match spec
//! - [`LabelConformanceRule`] - Checks variable labels match spec
//! - [`DatasetMetaConformanceRule`] - Checks dataset name and label match spec

use std::collections::HashSet;

use crate::error::{ErrorLocation, ValidationError, ValidationErrorCode};
use crate::spec::DatasetSpec;
use crate::types::{XptColumn, XptDataset};
use crate::validation::{ActionLevel, ValidationContext, ValidationRule};

/// Rule to check that all variables in data exist in the specification.
///
/// When a variable exists in the data but not in the spec, this indicates
/// either a missing spec entry or unexpected data column.
#[derive(Debug)]
pub struct VariableInSpecRule {
    spec: DatasetSpec,
    action: ActionLevel,
}

impl VariableInSpecRule {
    /// Create a new rule with the given spec and action level.
    #[must_use]
    pub fn new(spec: DatasetSpec, action: ActionLevel) -> Self {
        Self { spec, action }
    }
}

impl ValidationRule for VariableInSpecRule {
    fn name(&self) -> &'static str {
        "variable_in_spec"
    }

    fn validate_column(
        &self,
        column: &XptColumn,
        index: usize,
        dataset_name: &str,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        if matches!(self.action, ActionLevel::None) {
            return Vec::new();
        }

        // Check if variable exists in spec
        if self.spec.variable(&column.name).is_none() {
            return vec![ValidationError::new(
                ValidationErrorCode::VariableNotInSpec,
                format!(
                    "Variable '{}' exists in data but not in specification",
                    column.name
                ),
                ErrorLocation::Column {
                    dataset: dataset_name.to_string(),
                    column: column.name.clone(),
                    index,
                },
                self.action.to_severity(),
            )];
        }

        Vec::new()
    }
}

/// Rule to check that all variables in the specification exist in data.
///
/// When a variable is specified but not present in data, this may indicate
/// missing required data or incorrect column naming.
#[derive(Debug)]
pub struct VariableInDataRule {
    spec: DatasetSpec,
    action: ActionLevel,
}

impl VariableInDataRule {
    /// Create a new rule with the given spec and action level.
    #[must_use]
    pub fn new(spec: DatasetSpec, action: ActionLevel) -> Self {
        Self { spec, action }
    }
}

impl ValidationRule for VariableInDataRule {
    fn name(&self) -> &'static str {
        "variable_in_data"
    }

    fn validate_dataset(
        &self,
        dataset: &XptDataset,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        if matches!(self.action, ActionLevel::None) {
            return Vec::new();
        }

        let mut errors = Vec::new();
        let data_columns: HashSet<&str> = dataset.columns.iter().map(|c| c.name.as_str()).collect();

        for var_spec in &self.spec.variables {
            if !data_columns.contains(var_spec.name.as_str()) {
                errors.push(ValidationError::new(
                    ValidationErrorCode::VariableNotInData,
                    format!(
                        "Variable '{}' exists in specification but not in data",
                        var_spec.name
                    ),
                    ErrorLocation::Dataset {
                        name: dataset.name.clone(),
                    },
                    self.action.to_severity(),
                ));
            }
        }

        errors
    }
}

/// Rule to check that variable types match the specification.
#[derive(Debug)]
pub struct TypeConformanceRule {
    spec: DatasetSpec,
    action: ActionLevel,
}

impl TypeConformanceRule {
    /// Create a new rule with the given spec and action level.
    #[must_use]
    pub fn new(spec: DatasetSpec, action: ActionLevel) -> Self {
        Self { spec, action }
    }
}

impl ValidationRule for TypeConformanceRule {
    fn name(&self) -> &'static str {
        "type_conformance"
    }

    fn validate_column(
        &self,
        column: &XptColumn,
        index: usize,
        dataset_name: &str,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        if matches!(self.action, ActionLevel::None) {
            return Vec::new();
        }

        if let Some(var_spec) = self.spec.variable(&column.name) {
            if column.data_type != var_spec.data_type {
                let expected = if var_spec.data_type.is_numeric() {
                    "Numeric"
                } else {
                    "Character"
                };
                let actual = if column.data_type.is_numeric() {
                    "Numeric"
                } else {
                    "Character"
                };

                return vec![ValidationError::new(
                    ValidationErrorCode::TypeMismatch,
                    format!(
                        "Variable '{}' has type {} but specification requires {}",
                        column.name, actual, expected
                    ),
                    ErrorLocation::Column {
                        dataset: dataset_name.to_string(),
                        column: column.name.clone(),
                        index,
                    },
                    self.action.to_severity(),
                )];
            }
        }

        Vec::new()
    }
}

/// Rule to check that variable lengths match the specification.
#[derive(Debug)]
pub struct LengthConformanceRule {
    spec: DatasetSpec,
    action: ActionLevel,
}

impl LengthConformanceRule {
    /// Create a new rule with the given spec and action level.
    #[must_use]
    pub fn new(spec: DatasetSpec, action: ActionLevel) -> Self {
        Self { spec, action }
    }
}

impl ValidationRule for LengthConformanceRule {
    fn name(&self) -> &'static str {
        "length_conformance"
    }

    fn validate_column(
        &self,
        column: &XptColumn,
        index: usize,
        dataset_name: &str,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        if matches!(self.action, ActionLevel::None) {
            return Vec::new();
        }

        if let Some(var_spec) = self.spec.variable(&column.name) {
            if let Some(spec_length) = var_spec.length {
                if column.length != spec_length {
                    return vec![ValidationError::new(
                        ValidationErrorCode::LengthMismatch,
                        format!(
                            "Variable '{}' has length {} but specification requires {}",
                            column.name, column.length, spec_length
                        ),
                        ErrorLocation::Column {
                            dataset: dataset_name.to_string(),
                            column: column.name.clone(),
                            index,
                        },
                        self.action.to_severity(),
                    )];
                }
            }
        }

        Vec::new()
    }
}

/// Rule to check that variable order matches the specification.
#[derive(Debug)]
pub struct OrderConformanceRule {
    spec: DatasetSpec,
    action: ActionLevel,
}

impl OrderConformanceRule {
    /// Create a new rule with the given spec and action level.
    #[must_use]
    pub fn new(spec: DatasetSpec, action: ActionLevel) -> Self {
        Self { spec, action }
    }
}

impl ValidationRule for OrderConformanceRule {
    fn name(&self) -> &'static str {
        "order_conformance"
    }

    fn validate_dataset(
        &self,
        dataset: &XptDataset,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        if matches!(self.action, ActionLevel::None) {
            return Vec::new();
        }

        let mut errors = Vec::new();

        // Build expected order from spec (based on order field or spec position)
        let mut spec_order: Vec<(&str, usize)> = self
            .spec
            .variables
            .iter()
            .enumerate()
            .map(|(idx, v)| (v.name.as_str(), v.order.unwrap_or(idx + 1)))
            .collect();
        spec_order.sort_by_key(|(_, order)| *order);

        // Check each variable's position
        for (expected_pos, (var_name, _)) in spec_order.iter().enumerate() {
            if let Some(actual_pos) = dataset
                .columns
                .iter()
                .position(|c| c.name.as_str() == *var_name)
            {
                if actual_pos != expected_pos {
                    errors.push(ValidationError::new(
                        ValidationErrorCode::OrderMismatch,
                        format!(
                            "Variable '{}' is at position {} but specification expects position {}",
                            var_name,
                            actual_pos + 1,
                            expected_pos + 1
                        ),
                        ErrorLocation::Column {
                            dataset: dataset.name.clone(),
                            column: (*var_name).to_string(),
                            index: actual_pos,
                        },
                        self.action.to_severity(),
                    ));
                }
            }
        }

        errors
    }
}

/// Rule to check that variable formats match the specification.
#[derive(Debug)]
pub struct FormatConformanceRule {
    spec: DatasetSpec,
    action: ActionLevel,
}

impl FormatConformanceRule {
    /// Create a new rule with the given spec and action level.
    #[must_use]
    pub fn new(spec: DatasetSpec, action: ActionLevel) -> Self {
        Self { spec, action }
    }
}

impl ValidationRule for FormatConformanceRule {
    fn name(&self) -> &'static str {
        "format_conformance"
    }

    fn validate_column(
        &self,
        column: &XptColumn,
        index: usize,
        dataset_name: &str,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        if matches!(self.action, ActionLevel::None) {
            return Vec::new();
        }

        if let Some(var_spec) = self.spec.variable(&column.name) {
            if let Some(ref spec_format) = var_spec.format {
                let column_format_str = column.format.as_ref().map(|f| f.to_string());
                let spec_format_str = spec_format.to_string();

                let formats_match = column_format_str
                    .as_ref()
                    .map_or(false, |f| f == &spec_format_str);

                if !formats_match {
                    return vec![ValidationError::new(
                        ValidationErrorCode::FormatMismatch,
                        format!(
                            "Variable '{}' has format {:?} but specification requires {}",
                            column.name, column_format_str, spec_format_str
                        ),
                        ErrorLocation::Column {
                            dataset: dataset_name.to_string(),
                            column: column.name.clone(),
                            index,
                        },
                        self.action.to_severity(),
                    )];
                }
            }
        }

        Vec::new()
    }
}

/// Rule to check that variable labels match the specification.
#[derive(Debug)]
pub struct LabelConformanceRule {
    spec: DatasetSpec,
    action: ActionLevel,
}

impl LabelConformanceRule {
    /// Create a new rule with the given spec and action level.
    #[must_use]
    pub fn new(spec: DatasetSpec, action: ActionLevel) -> Self {
        Self { spec, action }
    }
}

impl ValidationRule for LabelConformanceRule {
    fn name(&self) -> &'static str {
        "label_conformance"
    }

    fn validate_column(
        &self,
        column: &XptColumn,
        index: usize,
        dataset_name: &str,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        if matches!(self.action, ActionLevel::None) {
            return Vec::new();
        }

        if let Some(var_spec) = self.spec.variable(&column.name) {
            if let Some(ref spec_label) = var_spec.label {
                let labels_match = column.label.as_ref().map_or(false, |l| l == spec_label);

                if !labels_match {
                    return vec![ValidationError::new(
                        ValidationErrorCode::LabelMismatch,
                        format!(
                            "Variable '{}' has label {:?} but specification requires '{}'",
                            column.name, column.label, spec_label
                        ),
                        ErrorLocation::Column {
                            dataset: dataset_name.to_string(),
                            column: column.name.clone(),
                            index,
                        },
                        self.action.to_severity(),
                    )];
                }
            }
        }

        Vec::new()
    }
}

/// Rule to check that dataset metadata matches the specification.
///
/// Validates dataset name and label against the spec.
#[derive(Debug)]
pub struct DatasetMetaConformanceRule {
    spec: DatasetSpec,
    action: ActionLevel,
}

impl DatasetMetaConformanceRule {
    /// Create a new rule with the given spec and action level.
    #[must_use]
    pub fn new(spec: DatasetSpec, action: ActionLevel) -> Self {
        Self { spec, action }
    }
}

impl ValidationRule for DatasetMetaConformanceRule {
    fn name(&self) -> &'static str {
        "dataset_meta_conformance"
    }

    fn validate_dataset(
        &self,
        dataset: &XptDataset,
        _ctx: &ValidationContext,
    ) -> Vec<ValidationError> {
        if matches!(self.action, ActionLevel::None) {
            return Vec::new();
        }

        let mut errors = Vec::new();

        // Check dataset name matches
        if dataset.name.to_uppercase() != self.spec.name.to_uppercase() {
            errors.push(ValidationError::new(
                ValidationErrorCode::DatasetNameSpecMismatch,
                format!(
                    "Dataset name '{}' doesn't match specification name '{}'",
                    dataset.name, self.spec.name
                ),
                ErrorLocation::Dataset {
                    name: dataset.name.clone(),
                },
                self.action.to_severity(),
            ));
        }

        // Check dataset label matches (if specified)
        if let Some(ref spec_label) = self.spec.label {
            let labels_match = dataset.label.as_ref().map_or(false, |l| l == spec_label);

            if !labels_match {
                errors.push(ValidationError::new(
                    ValidationErrorCode::DatasetLabelMismatch,
                    format!(
                        "Dataset label {:?} doesn't match specification label '{}'",
                        dataset.label, spec_label
                    ),
                    ErrorLocation::Dataset {
                        name: dataset.name.clone(),
                    },
                    self.action.to_severity(),
                ));
            }
        }

        errors
    }
}

/// Configuration for spec conformance validation.
///
/// This allows setting different action levels for each type of conformance check.
#[derive(Debug, Clone)]
pub struct SpecConformanceConfig {
    /// Action for variables in data but not in spec.
    pub variable_in_spec_action: ActionLevel,
    /// Action for variables in spec but not in data.
    pub variable_in_data_action: ActionLevel,
    /// Action for type mismatches.
    pub type_action: ActionLevel,
    /// Action for length mismatches.
    pub length_action: ActionLevel,
    /// Action for order mismatches.
    pub order_action: ActionLevel,
    /// Action for format mismatches.
    pub format_action: ActionLevel,
    /// Action for label mismatches.
    pub label_action: ActionLevel,
    /// Action for dataset metadata mismatches.
    pub dataset_meta_action: ActionLevel,
}

impl Default for SpecConformanceConfig {
    fn default() -> Self {
        Self {
            variable_in_spec_action: ActionLevel::Warn,
            variable_in_data_action: ActionLevel::Warn,
            type_action: ActionLevel::Warn,
            length_action: ActionLevel::Warn,
            order_action: ActionLevel::Message,
            format_action: ActionLevel::Message,
            label_action: ActionLevel::Message,
            dataset_meta_action: ActionLevel::Warn,
        }
    }
}

impl SpecConformanceConfig {
    /// Create a strict config where all mismatches are errors.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            variable_in_spec_action: ActionLevel::Stop,
            variable_in_data_action: ActionLevel::Stop,
            type_action: ActionLevel::Stop,
            length_action: ActionLevel::Stop,
            order_action: ActionLevel::Stop,
            format_action: ActionLevel::Stop,
            label_action: ActionLevel::Stop,
            dataset_meta_action: ActionLevel::Stop,
        }
    }

    /// Create a lenient config where all mismatches are warnings.
    #[must_use]
    pub fn lenient() -> Self {
        Self::default()
    }

    /// Build all spec conformance rules based on this configuration.
    #[must_use]
    pub fn build_rules(&self, spec: &DatasetSpec) -> Vec<Box<dyn ValidationRule>> {
        let mut rules: Vec<Box<dyn ValidationRule>> = Vec::new();

        if !matches!(self.variable_in_spec_action, ActionLevel::None) {
            rules.push(Box::new(VariableInSpecRule::new(
                spec.clone(),
                self.variable_in_spec_action,
            )));
        }

        if !matches!(self.variable_in_data_action, ActionLevel::None) {
            rules.push(Box::new(VariableInDataRule::new(
                spec.clone(),
                self.variable_in_data_action,
            )));
        }

        if !matches!(self.type_action, ActionLevel::None) {
            rules.push(Box::new(TypeConformanceRule::new(
                spec.clone(),
                self.type_action,
            )));
        }

        if !matches!(self.length_action, ActionLevel::None) {
            rules.push(Box::new(LengthConformanceRule::new(
                spec.clone(),
                self.length_action,
            )));
        }

        if !matches!(self.order_action, ActionLevel::None) {
            rules.push(Box::new(OrderConformanceRule::new(
                spec.clone(),
                self.order_action,
            )));
        }

        if !matches!(self.format_action, ActionLevel::None) {
            rules.push(Box::new(FormatConformanceRule::new(
                spec.clone(),
                self.format_action,
            )));
        }

        if !matches!(self.label_action, ActionLevel::None) {
            rules.push(Box::new(LabelConformanceRule::new(
                spec.clone(),
                self.label_action,
            )));
        }

        if !matches!(self.dataset_meta_action, ActionLevel::None) {
            rules.push(Box::new(DatasetMetaConformanceRule::new(
                spec.clone(),
                self.dataset_meta_action,
            )));
        }

        rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::VariableSpec;
    use crate::types::FormatSpec;

    fn create_test_spec() -> DatasetSpec {
        DatasetSpec::new("DM")
            .with_label("Demographics")
            .add_variable(
                VariableSpec::character("USUBJID", 20)
                    .with_label("Unique Subject Identifier")
                    .with_order(1),
            )
            .add_variable(
                VariableSpec::numeric("AGE")
                    .with_label("Age")
                    .with_format(FormatSpec::best(8))
                    .with_order(2),
            )
            .add_variable(
                VariableSpec::character("SEX", 1)
                    .with_label("Sex")
                    .with_order(3),
            )
    }

    fn create_test_dataset() -> XptDataset {
        let mut dataset = XptDataset::new("DM");
        dataset.label = Some("Demographics".to_string());
        dataset
            .columns
            .push(XptColumn::character("USUBJID", 20).with_label("Unique Subject Identifier"));
        dataset
            .columns
            .push(XptColumn::numeric("AGE").with_label("Age"));
        dataset
            .columns
            .push(XptColumn::character("SEX", 1).with_label("Sex"));
        dataset
    }

    fn make_context() -> ValidationContext {
        ValidationContext::new(crate::XptVersion::V5, ActionLevel::Warn)
    }

    #[test]
    fn test_variable_in_spec_rule_passes() {
        let spec = create_test_spec();
        let dataset = create_test_dataset();
        let rule = VariableInSpecRule::new(spec, ActionLevel::Warn);
        let ctx = make_context();

        for (idx, col) in dataset.columns.iter().enumerate() {
            let errors = rule.validate_column(col, idx, &dataset.name, &ctx);
            assert!(errors.is_empty(), "Expected no errors for {}", col.name);
        }
    }

    #[test]
    fn test_variable_in_spec_rule_fails() {
        let spec = create_test_spec();
        let mut dataset = create_test_dataset();
        dataset
            .columns
            .push(XptColumn::character("EXTRA", 10).with_label("Extra Column"));

        let rule = VariableInSpecRule::new(spec, ActionLevel::Warn);
        let ctx = make_context();

        let errors = rule.validate_column(&dataset.columns[3], 3, &dataset.name, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationErrorCode::VariableNotInSpec);
    }

    #[test]
    fn test_variable_in_data_rule_passes() {
        let spec = create_test_spec();
        let dataset = create_test_dataset();
        let rule = VariableInDataRule::new(spec, ActionLevel::Warn);
        let ctx = make_context();

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_variable_in_data_rule_fails() {
        let mut spec = create_test_spec();
        spec = spec.add_variable(VariableSpec::character("MISSING", 10).with_order(4));

        let dataset = create_test_dataset();
        let rule = VariableInDataRule::new(spec, ActionLevel::Warn);
        let ctx = make_context();

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationErrorCode::VariableNotInData);
    }

    #[test]
    fn test_type_conformance_rule_passes() {
        let spec = create_test_spec();
        let dataset = create_test_dataset();
        let rule = TypeConformanceRule::new(spec, ActionLevel::Warn);
        let ctx = make_context();

        for (idx, col) in dataset.columns.iter().enumerate() {
            let errors = rule.validate_column(col, idx, &dataset.name, &ctx);
            assert!(errors.is_empty());
        }
    }

    #[test]
    fn test_type_conformance_rule_fails() {
        let spec = create_test_spec();
        let mut dataset = create_test_dataset();
        // Replace AGE (numeric) with a character column
        dataset.columns[1] = XptColumn::character("AGE", 10).with_label("Age");

        let rule = TypeConformanceRule::new(spec, ActionLevel::Warn);
        let ctx = make_context();

        let errors = rule.validate_column(&dataset.columns[1], 1, &dataset.name, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationErrorCode::TypeMismatch);
    }

    #[test]
    fn test_length_conformance_rule_fails() {
        let spec = create_test_spec();
        let mut dataset = create_test_dataset();
        // Change USUBJID length from 20 to 30
        dataset.columns[0] =
            XptColumn::character("USUBJID", 30).with_label("Unique Subject Identifier");

        let rule = LengthConformanceRule::new(spec, ActionLevel::Warn);
        let ctx = make_context();

        let errors = rule.validate_column(&dataset.columns[0], 0, &dataset.name, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationErrorCode::LengthMismatch);
    }

    #[test]
    fn test_order_conformance_rule_fails() {
        let spec = create_test_spec();
        let mut dataset = XptDataset::new("DM");
        // Wrong order: SEX, AGE, USUBJID instead of USUBJID, AGE, SEX
        dataset
            .columns
            .push(XptColumn::character("SEX", 1).with_label("Sex"));
        dataset
            .columns
            .push(XptColumn::numeric("AGE").with_label("Age"));
        dataset
            .columns
            .push(XptColumn::character("USUBJID", 20).with_label("Unique Subject Identifier"));

        let rule = OrderConformanceRule::new(spec, ActionLevel::Warn);
        let ctx = make_context();

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(!errors.is_empty());
        assert!(
            errors
                .iter()
                .all(|e| e.code == ValidationErrorCode::OrderMismatch)
        );
    }

    #[test]
    fn test_label_conformance_rule_fails() {
        let spec = create_test_spec();
        let mut dataset = create_test_dataset();
        // Change AGE label
        dataset.columns[1] = XptColumn::numeric("AGE").with_label("Wrong Label");

        let rule = LabelConformanceRule::new(spec, ActionLevel::Warn);
        let ctx = make_context();

        let errors = rule.validate_column(&dataset.columns[1], 1, &dataset.name, &ctx);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ValidationErrorCode::LabelMismatch);
    }

    #[test]
    fn test_dataset_meta_conformance_rule_passes() {
        let spec = create_test_spec();
        let dataset = create_test_dataset();
        let rule = DatasetMetaConformanceRule::new(spec, ActionLevel::Warn);
        let ctx = make_context();

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_dataset_meta_conformance_rule_fails_name() {
        let spec = create_test_spec();
        let mut dataset = create_test_dataset();
        dataset.name = "WRONG".to_string();

        let rule = DatasetMetaConformanceRule::new(spec, ActionLevel::Warn);
        let ctx = make_context();

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(
            errors
                .iter()
                .any(|e| e.code == ValidationErrorCode::DatasetNameSpecMismatch)
        );
    }

    #[test]
    fn test_dataset_meta_conformance_rule_fails_label() {
        let spec = create_test_spec();
        let mut dataset = create_test_dataset();
        dataset.label = Some("Wrong Label".to_string());

        let rule = DatasetMetaConformanceRule::new(spec, ActionLevel::Warn);
        let ctx = make_context();

        let errors = rule.validate_dataset(&dataset, &ctx);
        assert!(
            errors
                .iter()
                .any(|e| e.code == ValidationErrorCode::DatasetLabelMismatch)
        );
    }

    #[test]
    fn test_spec_conformance_config_build_rules() {
        let spec = create_test_spec();
        let config = SpecConformanceConfig::default();
        let rules = config.build_rules(&spec);

        // Should create 8 rules by default
        assert_eq!(rules.len(), 8);
    }

    #[test]
    fn test_spec_conformance_config_strict() {
        let config = SpecConformanceConfig::strict();
        assert_eq!(config.type_action, ActionLevel::Stop);
        assert_eq!(config.length_action, ActionLevel::Stop);
    }

    #[test]
    fn test_action_level_none_skips_validation() {
        let spec = create_test_spec();
        let mut dataset = create_test_dataset();
        dataset.columns[1] = XptColumn::character("AGE", 10); // Wrong type

        // With ActionLevel::None, no errors should be generated
        let rule = TypeConformanceRule::new(spec, ActionLevel::None);
        let ctx = make_context();

        let errors = rule.validate_column(&dataset.columns[1], 1, &dataset.name, &ctx);
        assert!(errors.is_empty());
    }
}
