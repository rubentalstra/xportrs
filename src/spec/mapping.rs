//! Column name mapping for flexible metadata input formats.
//!
//! This module provides [`ColumnMapping`] for configuring how `DataFrame`
//! columns are mapped to specification fields. This allows users to use
//! their own column naming conventions when loading specs from `DataFrames`.
//!
//! # Example
//!
//! ```
//! use xportrs::spec::ColumnMapping;
//!
//! // Use default column names
//! let mapping = ColumnMapping::default();
//! assert_eq!(mapping.variable_col, "variable");
//! assert_eq!(mapping.label_col, "label");
//!
//! // Use custom column names
//! let mapping = ColumnMapping::new()
//!     .with_variable_col("Variable Name")
//!     .with_label_col("Variable Label")
//!     .with_type_col("Data Type");
//! ```

/// Column name mapping for metadata specification `DataFrames`.
///
/// This struct defines the mapping between `DataFrame` column names and
/// the specification fields. It allows users to load specifications from
/// `DataFrames` with any column naming convention.
///
/// # Default Column Names
///
/// | Field | Default Column Name |
/// |-------|---------------------|
/// | dataset | "dataset" |
/// | variable | "variable" |
/// | label | "label" |
/// | type | "type" |
/// | length | "length" |
/// | order | "order" |
/// | format | "format" |
/// | informat | "informat" |
/// | origin | "origin" |
/// | core | "core" |
///
/// # Example
///
/// ```
/// use xportrs::spec::ColumnMapping;
///
/// let mapping = ColumnMapping::new()
///     .with_variable_col("VARNAME")
///     .with_label_col("VARLABEL")
///     .with_type_col("VARTYPE");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnMapping {
    /// Column name containing dataset names.
    pub dataset_col: String,
    /// Column name containing variable names.
    pub variable_col: String,
    /// Column name containing variable labels.
    pub label_col: String,
    /// Column name containing variable types ("Num" or "Char").
    pub type_col: String,
    /// Column name containing variable lengths.
    pub length_col: String,
    /// Column name containing variable order/position.
    pub order_col: String,
    /// Column name containing SAS format specifications.
    pub format_col: String,
    /// Column name containing SAS informat specifications.
    pub informat_col: String,
    /// Column name containing variable origin (e.g., "Derived", "CRF").
    pub origin_col: String,
    /// Column name containing CDISC Core designation.
    pub core_col: String,
}

impl Default for ColumnMapping {
    fn default() -> Self {
        Self {
            dataset_col: "dataset".into(),
            variable_col: "variable".into(),
            label_col: "label".into(),
            type_col: "type".into(),
            length_col: "length".into(),
            order_col: "order".into(),
            format_col: "format".into(),
            informat_col: "informat".into(),
            origin_col: "origin".into(),
            core_col: "core".into(),
        }
    }
}

impl ColumnMapping {
    /// Create a new column mapping with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the dataset column name.
    #[must_use]
    pub fn with_dataset_col(mut self, name: impl Into<String>) -> Self {
        self.dataset_col = name.into();
        self
    }

    /// Set the variable column name.
    #[must_use]
    pub fn with_variable_col(mut self, name: impl Into<String>) -> Self {
        self.variable_col = name.into();
        self
    }

    /// Set the label column name.
    #[must_use]
    pub fn with_label_col(mut self, name: impl Into<String>) -> Self {
        self.label_col = name.into();
        self
    }

    /// Set the type column name.
    #[must_use]
    pub fn with_type_col(mut self, name: impl Into<String>) -> Self {
        self.type_col = name.into();
        self
    }

    /// Set the length column name.
    #[must_use]
    pub fn with_length_col(mut self, name: impl Into<String>) -> Self {
        self.length_col = name.into();
        self
    }

    /// Set the order column name.
    #[must_use]
    pub fn with_order_col(mut self, name: impl Into<String>) -> Self {
        self.order_col = name.into();
        self
    }

    /// Set the format column name.
    #[must_use]
    pub fn with_format_col(mut self, name: impl Into<String>) -> Self {
        self.format_col = name.into();
        self
    }

    /// Set the informat column name.
    #[must_use]
    pub fn with_informat_col(mut self, name: impl Into<String>) -> Self {
        self.informat_col = name.into();
        self
    }

    /// Set the origin column name.
    #[must_use]
    pub fn with_origin_col(mut self, name: impl Into<String>) -> Self {
        self.origin_col = name.into();
        self
    }

    /// Set the core column name.
    #[must_use]
    pub fn with_core_col(mut self, name: impl Into<String>) -> Self {
        self.core_col = name.into();
        self
    }

    /// Create a mapping for CDISC-style define.xml column names.
    #[must_use]
    pub fn cdisc_define() -> Self {
        Self {
            dataset_col: "Dataset".into(),
            variable_col: "Variable".into(),
            label_col: "Label".into(),
            type_col: "Type".into(),
            length_col: "Length".into(),
            order_col: "Order".into(),
            format_col: "Format".into(),
            informat_col: "Informat".into(),
            origin_col: "Origin".into(),
            core_col: "Core".into(),
        }
    }

    /// Create a mapping for Pinnacle 21 style column names.
    #[must_use]
    pub fn pinnacle21() -> Self {
        Self {
            dataset_col: "DOMAIN".into(),
            variable_col: "VARNAME".into(),
            label_col: "VARLABEL".into(),
            type_col: "VARTYPE".into(),
            length_col: "LENGTH".into(),
            order_col: "VARNUM".into(),
            format_col: "FORMAT".into(),
            informat_col: "INFORMAT".into(),
            origin_col: "ORIGIN".into(),
            core_col: "CORE".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mapping() {
        let mapping = ColumnMapping::default();
        assert_eq!(mapping.dataset_col, "dataset");
        assert_eq!(mapping.variable_col, "variable");
        assert_eq!(mapping.label_col, "label");
        assert_eq!(mapping.type_col, "type");
        assert_eq!(mapping.length_col, "length");
        assert_eq!(mapping.order_col, "order");
        assert_eq!(mapping.format_col, "format");
    }

    #[test]
    fn test_custom_mapping() {
        let mapping = ColumnMapping::new()
            .with_variable_col("VARNAME")
            .with_label_col("VARLABEL")
            .with_type_col("VARTYPE");

        assert_eq!(mapping.variable_col, "VARNAME");
        assert_eq!(mapping.label_col, "VARLABEL");
        assert_eq!(mapping.type_col, "VARTYPE");
        // Others should still be default
        assert_eq!(mapping.length_col, "length");
    }

    #[test]
    fn test_cdisc_define_mapping() {
        let mapping = ColumnMapping::cdisc_define();
        assert_eq!(mapping.dataset_col, "Dataset");
        assert_eq!(mapping.variable_col, "Variable");
        assert_eq!(mapping.label_col, "Label");
    }

    #[test]
    fn test_pinnacle21_mapping() {
        let mapping = ColumnMapping::pinnacle21();
        assert_eq!(mapping.dataset_col, "DOMAIN");
        assert_eq!(mapping.variable_col, "VARNAME");
        assert_eq!(mapping.label_col, "VARLABEL");
    }
}
