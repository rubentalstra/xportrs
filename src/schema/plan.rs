//! Dataset schema structures.
//!
//! This module defines the [`DatasetSchema`] and [`VariableSpec`] types that
//! represent the transport schema for an XPT file.

use crate::dataset::{Format, VariableRole};
use crate::metadata::XptVarType;

/// A planned schema for XPT file generation.
///
/// This represents the finalized byte layout for a dataset, including
/// variable positions, lengths, and metadata.
#[derive(Debug, Clone)]
pub(crate) struct DatasetSchema {
    /// The domain code (dataset name).
    pub domain_code: String,

    /// The dataset label.
    pub dataset_label: Option<String>,

    /// The planned variables in order.
    pub variables: Vec<VariableSpec>,

    /// The total row length in bytes.
    pub row_len: usize,
}

#[allow(dead_code)]
impl DatasetSchema {
    /// Creates a new schema plan.
    #[must_use]
    pub fn new(domain_code: impl Into<String>) -> Self {
        Self {
            domain_code: domain_code.into(),
            dataset_label: None,
            variables: Vec::new(),
            row_len: 0,
        }
    }

    /// Sets the dataset label.
    #[must_use]
    pub fn with_label(mut self, label: Option<String>) -> Self {
        self.dataset_label = label;
        self
    }

    /// Returns the number of variables.
    #[must_use]
    pub fn num_variables(&self) -> usize {
        self.variables.len()
    }

    /// Recalculates positions and row length from variable lengths.
    pub fn recalculate_positions(&mut self) {
        let mut position = 0;
        for var in &mut self.variables {
            var.position = position;
            position += var.length;
        }
        self.row_len = position;
    }

    /// Returns an iterator over numeric variables.
    pub fn numeric_variables(&self) -> impl Iterator<Item = &VariableSpec> {
        self.variables.iter().filter(|v| v.xpt_type.is_numeric())
    }

    /// Returns an iterator over character variables.
    pub fn character_variables(&self) -> impl Iterator<Item = &VariableSpec> {
        self.variables.iter().filter(|v| v.xpt_type.is_character())
    }
}

/// A planned variable in the schema.
///
/// This represents a single variable with all information needed for
/// XPT file generation.
#[derive(Debug, Clone)]
pub(crate) struct VariableSpec {
    /// The variable name (max 8 bytes in v5).
    pub name: String,

    /// The XPT type (Numeric or Character).
    pub xpt_type: XptVarType,

    /// The byte length (8 for numeric, variable for character).
    pub length: usize,

    /// The variable label (max 40 bytes in v5).
    pub label: String,

    /// The SAS format (controls display output).
    ///
    /// Contains format name, length, decimals, and justification.
    pub format: Option<Format>,

    /// The SAS informat (controls data input).
    ///
    /// Contains informat name, length, and decimals.
    /// Note: Informats do not have justification.
    pub informat: Option<Format>,

    /// The byte position within the observation record.
    pub position: usize,

    /// The CDISC variable role.
    pub role: Option<VariableRole>,

    /// The original column index in the dataset.
    pub source_index: usize,
}

#[allow(dead_code)]
impl VariableSpec {
    /// Creates a new planned variable.
    #[must_use]
    pub fn new(name: String, xpt_type: XptVarType, length: usize) -> Self {
        Self {
            name,
            xpt_type,
            length,
            label: String::new(),
            format: None,
            informat: None,
            position: 0,
            role: None,
            source_index: 0,
        }
    }

    /// Creates a new numeric variable.
    #[must_use]
    pub fn numeric(name: impl Into<String>) -> Self {
        Self::new(name.into(), XptVarType::Numeric, 8)
    }

    /// Creates a new character variable with the specified length.
    #[must_use]
    pub fn character(name: impl Into<String>, length: usize) -> Self {
        Self::new(name.into(), XptVarType::Character, length)
    }

    /// Sets the label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the format.
    #[must_use]
    pub fn with_format(mut self, format: Format) -> Self {
        self.format = Some(format);
        self
    }

    /// Sets the informat.
    #[must_use]
    pub fn with_informat(mut self, informat: Format) -> Self {
        self.informat = Some(informat);
        self
    }

    /// Sets the role.
    #[must_use]
    pub fn with_role(mut self, role: VariableRole) -> Self {
        self.role = Some(role);
        self
    }

    /// Sets the source index.
    #[must_use]
    pub fn with_source_index(mut self, index: usize) -> Self {
        self.source_index = index;
        self
    }

    /// Returns the format name for NAMESTR, or empty string if no format.
    #[must_use]
    pub fn format_name(&self) -> &str {
        self.format
            .as_ref()
            .map(|f| f.name_without_prefix())
            .unwrap_or("")
    }

    /// Returns the format length for NAMESTR, or 0 if no format.
    #[must_use]
    pub fn format_length(&self) -> u16 {
        self.format.as_ref().map(|f| f.length()).unwrap_or(0)
    }

    /// Returns the format decimals for NAMESTR, or 0 if no format.
    #[must_use]
    pub fn format_decimals(&self) -> u16 {
        self.format.as_ref().map(|f| f.decimals()).unwrap_or(0)
    }

    /// Returns the format justification for NAMESTR, or 0 if no format.
    #[must_use]
    pub fn format_justification(&self) -> i16 {
        self.format
            .as_ref()
            .map(|f| f.justification().as_nfj())
            .unwrap_or(0)
    }

    /// Returns the informat name for NAMESTR, or empty string if no informat.
    #[must_use]
    pub fn informat_name(&self) -> &str {
        self.informat
            .as_ref()
            .map(|f| f.name_without_prefix())
            .unwrap_or("")
    }

    /// Returns the informat length for NAMESTR, or 0 if no informat.
    #[must_use]
    pub fn informat_length(&self) -> u16 {
        self.informat.as_ref().map(|f| f.length()).unwrap_or(0)
    }

    /// Returns the informat decimals for NAMESTR, or 0 if no informat.
    #[must_use]
    pub fn informat_decimals(&self) -> u16 {
        self.informat.as_ref().map(|f| f.decimals()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_plan_positions() {
        let mut plan = DatasetSchema::new("AE");
        plan.variables = vec![
            VariableSpec::numeric("AESEQ"),
            VariableSpec::character("USUBJID", 20),
            VariableSpec::numeric("AESTDY"),
        ];
        plan.recalculate_positions();

        assert_eq!(plan.variables[0].position, 0);
        assert_eq!(plan.variables[1].position, 8);
        assert_eq!(plan.variables[2].position, 28);
        assert_eq!(plan.row_len, 36);
    }
}
