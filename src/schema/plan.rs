//! Dataset schema structures.
//!
//! This module defines the [`DatasetSchema`] and [`VariableSpec`] types that
//! represent the transport schema for an XPT file.

use crate::dataset::VariableRole;
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

    /// The SAS format string.
    pub format: String,

    /// The SAS informat string.
    pub informat: String,

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
            format: String::new(),
            informat: String::new(),
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
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = format.into();
        self
    }

    /// Sets the informat.
    #[must_use]
    pub fn with_informat(mut self, informat: impl Into<String>) -> Self {
        self.informat = informat.into();
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
