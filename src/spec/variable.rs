//! Variable-level metadata specification.
//!
//! [`VariableSpec`] defines the expected metadata for a variable including
//! name, type, length, label, format, and ordering information.

use crate::types::{FormatSpec, XptType};

/// CDISC Core classification for variables.
///
/// Indicates whether a variable is required, expected, or permissible
/// in a CDISC-compliant dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Core {
    /// Variable is required (must be present).
    Required,
    /// Variable is expected (should be present if applicable).
    Expected,
    /// Variable is permissible (may be present).
    #[default]
    Permissible,
}

impl Core {
    /// Check if this is a required variable.
    #[must_use]
    pub const fn is_required(&self) -> bool {
        matches!(self, Self::Required)
    }

    /// Parse from string (case-insensitive).
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "REQ" | "REQUIRED" => Some(Self::Required),
            "EXP" | "EXPECTED" => Some(Self::Expected),
            "PERM" | "PERMISSIBLE" => Some(Self::Permissible),
            _ => None,
        }
    }
}

/// Variable-level metadata specification.
///
/// Defines the expected metadata for a single variable in a dataset.
/// This specification is used to transform and validate data before
/// writing to XPT format.
///
/// # Example
///
/// ```
/// use xportrs::spec::VariableSpec;
/// use xportrs::FormatSpec;
///
/// let var = VariableSpec::numeric("AGE")
///     .with_label("Age in Years")
///     .with_order(5)
///     .with_format(FormatSpec::best(8));
///
/// assert_eq!(var.name, "AGE");
/// assert!(var.data_type.is_numeric());
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VariableSpec {
    /// Variable name (uppercase, max 8 chars for V5, 32 for V8).
    pub name: String,

    /// Variable label (max 40 chars for V5, 256 for V8).
    pub label: Option<String>,

    /// Data type (numeric or character).
    pub data_type: XptType,

    /// Length in bytes.
    ///
    /// For numeric: typically 8.
    /// For character: the maximum string length.
    pub length: Option<u16>,

    /// Variable order in the dataset (1-based).
    pub order: Option<usize>,

    /// Output format specification.
    pub format: Option<FormatSpec>,

    /// Input format (informat) specification.
    pub informat: Option<FormatSpec>,

    /// Variable origin (e.g., "Derived", "Collected", "Assigned").
    pub origin: Option<String>,

    /// CDISC Core classification.
    pub core: Option<Core>,
}

impl VariableSpec {
    /// Create a new variable specification with the given name and type.
    ///
    /// The name is automatically converted to uppercase.
    #[must_use]
    pub fn new(name: impl Into<String>, data_type: XptType) -> Self {
        Self {
            name: name.into().trim().to_uppercase(),
            label: None,
            data_type,
            length: None,
            order: None,
            format: None,
            informat: None,
            origin: None,
            core: None,
        }
    }

    /// Create a numeric variable specification.
    ///
    /// Defaults to 8-byte length.
    #[must_use]
    pub fn numeric(name: impl Into<String>) -> Self {
        let mut spec = Self::new(name, XptType::Num);
        spec.length = Some(8);
        spec
    }

    /// Create a character variable specification.
    ///
    /// # Arguments
    /// * `name` - Variable name
    /// * `length` - Maximum character length
    #[must_use]
    pub fn character(name: impl Into<String>, length: u16) -> Self {
        let mut spec = Self::new(name, XptType::Char);
        spec.length = Some(length);
        spec
    }

    /// Set the variable label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        let label_str = label.into();
        self.label = if label_str.is_empty() {
            None
        } else {
            Some(label_str)
        };
        self
    }

    /// Set the variable length.
    #[must_use]
    pub fn with_length(mut self, length: u16) -> Self {
        self.length = Some(length);
        self
    }

    /// Set the variable order (1-based position in dataset).
    #[must_use]
    pub fn with_order(mut self, order: usize) -> Self {
        self.order = Some(order);
        self
    }

    /// Set the output format.
    #[must_use]
    pub fn with_format(mut self, format: FormatSpec) -> Self {
        self.format = Some(format);
        self
    }

    /// Set the input format (informat).
    #[must_use]
    pub fn with_informat(mut self, informat: FormatSpec) -> Self {
        self.informat = Some(informat);
        self
    }

    /// Set the variable origin.
    #[must_use]
    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        let origin_str = origin.into();
        self.origin = if origin_str.is_empty() {
            None
        } else {
            Some(origin_str)
        };
        self
    }

    /// Set the CDISC Core classification.
    #[must_use]
    pub fn with_core(mut self, core: Core) -> Self {
        self.core = Some(core);
        self
    }

    /// Check if this is a numeric variable.
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        self.data_type.is_numeric()
    }

    /// Check if this is a character variable.
    #[must_use]
    pub const fn is_character(&self) -> bool {
        self.data_type.is_character()
    }

    /// Get the effective length (specified length or default for type).
    #[must_use]
    pub fn effective_length(&self) -> u16 {
        self.length.unwrap_or(if self.is_numeric() { 8 } else { 1 })
    }
}

impl Default for VariableSpec {
    fn default() -> Self {
        Self::numeric("VAR")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_spec_numeric() {
        let var = VariableSpec::numeric("age");
        assert_eq!(var.name, "AGE");
        assert!(var.is_numeric());
        assert!(!var.is_character());
        assert_eq!(var.length, Some(8));
    }

    #[test]
    fn test_variable_spec_character() {
        let var = VariableSpec::character("usubjid", 20);
        assert_eq!(var.name, "USUBJID");
        assert!(!var.is_numeric());
        assert!(var.is_character());
        assert_eq!(var.length, Some(20));
    }

    #[test]
    fn test_variable_spec_with_label() {
        let var = VariableSpec::numeric("age").with_label("Age in Years");
        assert_eq!(var.label, Some("Age in Years".to_string()));

        let var = VariableSpec::numeric("age").with_label("");
        assert_eq!(var.label, None);
    }

    #[test]
    fn test_variable_spec_with_order() {
        let var = VariableSpec::numeric("age").with_order(5);
        assert_eq!(var.order, Some(5));
    }

    #[test]
    fn test_variable_spec_with_format() {
        let var = VariableSpec::numeric("date").with_format(FormatSpec::date9());
        assert!(var.format.is_some());
        assert_eq!(var.format.as_ref().unwrap().name(), Some("DATE"));
    }

    #[test]
    fn test_variable_spec_with_core() {
        let var = VariableSpec::numeric("usubjid").with_core(Core::Required);
        assert_eq!(var.core, Some(Core::Required));
        assert!(var.core.as_ref().unwrap().is_required());
    }

    #[test]
    fn test_core_from_str() {
        assert_eq!(Core::from_str("REQ"), Some(Core::Required));
        assert_eq!(Core::from_str("Required"), Some(Core::Required));
        assert_eq!(Core::from_str("EXP"), Some(Core::Expected));
        assert_eq!(Core::from_str("PERM"), Some(Core::Permissible));
        assert_eq!(Core::from_str("unknown"), None);
    }

    #[test]
    fn test_effective_length() {
        let num = VariableSpec::new("X", XptType::Num);
        assert_eq!(num.effective_length(), 8);

        let chr = VariableSpec::new("X", XptType::Char);
        assert_eq!(chr.effective_length(), 1);

        let chr = VariableSpec::character("X", 50);
        assert_eq!(chr.effective_length(), 50);
    }
}
