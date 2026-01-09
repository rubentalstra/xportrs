//! Fluent spec construction.
//!
//! This module provides a builder pattern for constructing `DatasetSpec`
//! and `VariableSpec` instances with a fluent API.
//!
//! # Example
//!
//! ```
//! use xportrs::spec::{SpecBuilder, XptType};
//!
//! let spec = SpecBuilder::new("DM")
//!     .label("Demographics")
//!     .variable("USUBJID", XptType::Char, |v| {
//!         v.length(20)
//!          .label("Unique Subject Identifier")
//!          .order(1)
//!     })
//!     .variable("AGE", XptType::Num, |v| {
//!         v.label("Age")
//!          .order(2)
//!     })
//!     .build();
//!
//! assert_eq!(spec.name, "DM");
//! assert_eq!(spec.variables.len(), 2);
//! ```

use super::{DatasetSpec, VariableSpec};
use crate::types::{FormatSpec, XptType};

/// Builder for constructing `DatasetSpec` instances with a fluent API.
///
/// This provides a more ergonomic way to build complex specifications
/// compared to chaining `add_variable` calls.
///
/// # Example
///
/// ```
/// use xportrs::spec::{SpecBuilder, XptType};
///
/// let spec = SpecBuilder::new("AE")
///     .label("Adverse Events")
///     .variable("USUBJID", XptType::Char, |v| v.length(20).order(1))
///     .variable("AETERM", XptType::Char, |v| v.length(200).order(2))
///     .variable("AESTDTC", XptType::Char, |v| v.length(19).order(3))
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct SpecBuilder {
    name: String,
    label: Option<String>,
    variables: Vec<VariableSpec>,
    structure: Option<String>,
    class: Option<String>,
    keys: Vec<String>,
}

impl SpecBuilder {
    /// Create a new spec builder for the given dataset name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: None,
            variables: Vec::new(),
            structure: None,
            class: None,
            keys: Vec::new(),
        }
    }

    /// Set the dataset label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the dataset structure.
    #[must_use]
    pub fn structure(mut self, structure: impl Into<String>) -> Self {
        self.structure = Some(structure.into());
        self
    }

    /// Set the dataset class.
    #[must_use]
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    /// Add a key variable.
    #[must_use]
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.keys.push(key.into());
        self
    }

    /// Add multiple key variables.
    #[must_use]
    pub fn keys(mut self, keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.keys.extend(keys.into_iter().map(Into::into));
        self
    }

    /// Add a variable using a closure for configuration.
    ///
    /// The closure receives a `VariableBuilder` that can be used to
    /// configure the variable's properties.
    #[must_use]
    pub fn variable<F>(mut self, name: impl Into<String>, data_type: XptType, config: F) -> Self
    where
        F: FnOnce(VariableBuilder) -> VariableBuilder,
    {
        let builder = VariableBuilder::new(name, data_type);
        let builder = config(builder);
        self.variables.push(builder.build());
        self
    }

    /// Add a numeric variable using a closure for configuration.
    #[must_use]
    pub fn numeric<F>(self, name: impl Into<String>, config: F) -> Self
    where
        F: FnOnce(VariableBuilder) -> VariableBuilder,
    {
        self.variable(name, XptType::Num, config)
    }

    /// Add a character variable using a closure for configuration.
    #[must_use]
    pub fn character<F>(self, name: impl Into<String>, config: F) -> Self
    where
        F: FnOnce(VariableBuilder) -> VariableBuilder,
    {
        self.variable(name, XptType::Char, config)
    }

    /// Add a pre-built `VariableSpec`.
    #[must_use]
    pub fn add_variable(mut self, var: VariableSpec) -> Self {
        self.variables.push(var);
        self
    }

    /// Add multiple pre-built `VariableSpec`s.
    #[must_use]
    pub fn add_variables(mut self, vars: impl IntoIterator<Item = VariableSpec>) -> Self {
        self.variables.extend(vars);
        self
    }

    /// Build the final `DatasetSpec`.
    #[must_use]
    pub fn build(self) -> DatasetSpec {
        DatasetSpec {
            name: self.name,
            label: self.label,
            variables: self.variables,
            structure: self.structure,
            class: self.class,
            keys: self.keys,
        }
    }
}

/// Builder for constructing `VariableSpec` instances.
///
/// This is typically used within the `SpecBuilder::variable` closure.
#[derive(Debug, Clone)]
pub struct VariableBuilder {
    name: String,
    data_type: XptType,
    label: Option<String>,
    length: Option<u16>,
    order: Option<usize>,
    format: Option<FormatSpec>,
    informat: Option<FormatSpec>,
    origin: Option<String>,
}

impl VariableBuilder {
    /// Create a new variable builder.
    #[must_use]
    pub fn new(name: impl Into<String>, data_type: XptType) -> Self {
        Self {
            name: name.into(),
            data_type,
            label: None,
            length: None,
            order: None,
            format: None,
            informat: None,
            origin: None,
        }
    }

    /// Set the variable label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the variable length.
    #[must_use]
    pub fn length(mut self, length: u16) -> Self {
        self.length = Some(length);
        self
    }

    /// Set the variable order.
    #[must_use]
    pub fn order(mut self, order: usize) -> Self {
        self.order = Some(order);
        self
    }

    /// Set the variable format.
    #[must_use]
    pub fn format(mut self, format: FormatSpec) -> Self {
        self.format = Some(format);
        self
    }

    /// Set the variable format from a name and width.
    #[must_use]
    pub fn format_str(mut self, name: impl Into<String>, width: u16) -> Self {
        self.format = Some(FormatSpec::with_name(name, width));
        self
    }

    /// Set the variable informat.
    #[must_use]
    pub fn informat(mut self, informat: FormatSpec) -> Self {
        self.informat = Some(informat);
        self
    }

    /// Set the variable origin.
    #[must_use]
    pub fn origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());
        self
    }

    /// Build the final `VariableSpec`.
    #[must_use]
    pub fn build(self) -> VariableSpec {
        VariableSpec {
            name: self.name,
            label: self.label,
            data_type: self.data_type,
            length: self.length,
            order: self.order,
            format: self.format,
            informat: self.informat,
            origin: self.origin,
            core: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_builder_basic() {
        let spec = SpecBuilder::new("DM")
            .label("Demographics")
            .build();

        assert_eq!(spec.name, "DM");
        assert_eq!(spec.label, Some("Demographics".to_string()));
        assert!(spec.variables.is_empty());
    }

    #[test]
    fn test_spec_builder_with_variables() {
        let spec = SpecBuilder::new("DM")
            .label("Demographics")
            .variable("USUBJID", XptType::Char, |v| {
                v.length(20)
                    .label("Unique Subject Identifier")
                    .order(1)
            })
            .variable("AGE", XptType::Num, |v| {
                v.label("Age").order(2)
            })
            .build();

        assert_eq!(spec.variables.len(), 2);
        assert_eq!(spec.variables[0].name, "USUBJID");
        assert_eq!(spec.variables[0].length, Some(20));
        assert_eq!(spec.variables[1].name, "AGE");
        assert_eq!(spec.variables[1].data_type, XptType::Num);
    }

    #[test]
    fn test_spec_builder_shortcuts() {
        let spec = SpecBuilder::new("AE")
            .numeric("AESEQ", |v| v.label("Sequence Number"))
            .character("AETERM", |v| v.length(200).label("Reported Term"))
            .build();

        assert_eq!(spec.variables.len(), 2);
        assert_eq!(spec.variables[0].data_type, XptType::Num);
        assert_eq!(spec.variables[1].data_type, XptType::Char);
    }

    #[test]
    fn test_spec_builder_keys() {
        let spec = SpecBuilder::new("DM")
            .keys(["STUDYID", "USUBJID"])
            .build();

        assert_eq!(spec.keys.len(), 2);
        assert_eq!(spec.keys[0], "STUDYID");
        assert_eq!(spec.keys[1], "USUBJID");
    }

    #[test]
    fn test_variable_builder() {
        let var = VariableBuilder::new("USUBJID", XptType::Char)
            .length(20)
            .label("Unique Subject Identifier")
            .order(1)
            .origin("CRF")
            .build();

        assert_eq!(var.name, "USUBJID");
        assert_eq!(var.data_type, XptType::Char);
        assert_eq!(var.length, Some(20));
        assert_eq!(var.label, Some("Unique Subject Identifier".to_string()));
        assert_eq!(var.order, Some(1));
        assert_eq!(var.origin, Some("CRF".to_string()));
    }
}
