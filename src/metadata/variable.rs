//! Variable metadata.
//!
//! This module defines metadata for individual variables (columns) in a dataset.

use crate::dataset::VariableRole;

/// Metadata describing a single variable.
///
/// This struct provides optional metadata that guides XPT file generation.
/// When provided, it takes precedence over inferred values from the data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct VariableMetadata {
    /// The domain code this variable belongs to.
    pub domain_code: String,

    /// The variable name.
    pub variable_name: String,

    /// The desired XPT type (Numeric or Character).
    ///
    /// If not specified, the type is inferred from the column data.
    pub xpt_type: Option<XptVarType>,

    /// The byte length for character variables.
    ///
    /// For numeric variables, this is always 8 in XPT v5.
    /// For character variables, this specifies the fixed-width length.
    pub length: Option<usize>,

    /// The variable label (description).
    ///
    /// Limited to 40 bytes in XPT v5.
    pub label: Option<String>,

    /// The SAS format name (e.g., "DATE9.", "8.2").
    ///
    /// Stored in the NAMESTR record.
    pub format: Option<String>,

    /// The ordering key for variable position.
    ///
    /// Variables are sorted by this value when generating the XPT file.
    pub order: Option<i32>,

    /// The CDISC variable role.
    pub role: Option<VariableRole>,
}

#[allow(dead_code)]
impl VariableMetadata {
    /// Creates new variable metadata with required fields.
    #[must_use]
    pub(crate) fn new(domain_code: impl Into<String>, variable_name: impl Into<String>) -> Self {
        Self {
            domain_code: domain_code.into(),
            variable_name: variable_name.into(),
            ..Default::default()
        }
    }

    /// Sets the XPT type.
    #[must_use]
    pub(crate) fn with_xpt_type(mut self, xpt_type: XptVarType) -> Self {
        self.xpt_type = Some(xpt_type);
        self
    }

    /// Sets the byte length.
    #[must_use]
    pub(crate) fn with_length(mut self, length: usize) -> Self {
        self.length = Some(length);
        self
    }

    /// Sets the label.
    #[must_use]
    pub(crate) fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the format.
    #[must_use]
    pub(crate) fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Sets the ordering key.
    #[must_use]
    pub(crate) fn with_order(mut self, order: i32) -> Self {
        self.order = Some(order);
        self
    }

    /// Sets the variable role.
    #[must_use]
    pub(crate) fn with_role(mut self, role: VariableRole) -> Self {
        self.role = Some(role);
        self
    }
}

/// The XPT variable type.
///
/// XPT v5 only supports two fundamental types: Numeric and Character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum XptVarType {
    /// Numeric type (8-byte IBM floating-point in XPT v5).
    #[default]
    Numeric,
    /// Character type (fixed-width byte string).
    Character,
}

impl XptVarType {
    /// Returns `true` if this is the numeric type.
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        matches!(self, Self::Numeric)
    }

    /// Returns `true` if this is the character type.
    #[must_use]
    pub const fn is_character(&self) -> bool {
        matches!(self, Self::Character)
    }
}

impl std::fmt::Display for XptVarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Numeric => write!(f, "Numeric"),
            Self::Character => write!(f, "Character"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_metadata_builder() {
        let meta = VariableMetadata::new("AE", "AESER")
            .with_xpt_type(XptVarType::Character)
            .with_length(1)
            .with_label("Serious Event")
            .with_order(5);

        assert_eq!(meta.domain_code, "AE");
        assert_eq!(meta.variable_name, "AESER");
        assert_eq!(meta.xpt_type, Some(XptVarType::Character));
        assert_eq!(meta.length, Some(1));
        assert_eq!(meta.label.as_deref(), Some("Serious Event"));
        assert_eq!(meta.order, Some(5));
    }
}
