//! Domain dataset representation.
//!
//! This module provides the [`Dataset`] struct which represents a CDISC
//! domain dataset (table) in a columnar format.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::error::{Result, Error};

use super::newtypes::{DomainCode, Label};

/// A CDISC domain dataset in columnar format.
///
/// This is the primary data structure for representing tables in xportrs.
/// It uses CDISC SDTM vocabulary where applicable.
///
/// # Invariants
///
/// - All columns must have exactly `nrows` elements.
/// - Domain code should follow CDISC naming conventions (typically 2-8 characters).
///
/// # Example
///
/// ```
/// use xportrs::{Dataset, Column, ColumnData, DomainCode};
///
/// let dataset = Dataset::new(
///     DomainCode::new("AE"),
///     vec![
///         Column::new("USUBJID", ColumnData::String(vec![Some("01-001".into())])),
///         Column::new("AESER", ColumnData::String(vec![Some("Y".into())])),
///     ],
/// ).expect("valid dataset");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Dataset {
    /// The domain code (e.g., "AE", "DM", "LB").
    ///
    /// This is typically 2 characters but can be up to 8 bytes in XPT v5.
    domain_code: DomainCode,

    /// An optional label describing the dataset.
    ///
    /// Limited to 40 bytes in XPT v5.
    dataset_label: Option<Label>,

    /// The columns (variables) in the dataset.
    columns: Vec<Column>,

    /// The number of rows (observations) in the dataset.
    nrows: usize,
}

impl Dataset {
    /// Creates a new domain dataset.
    ///
    /// # Errors
    ///
    /// Returns an error if any column has a different length than the others.
    #[must_use = "this returns a Result that should be handled"]
    pub fn new(domain_code: impl Into<DomainCode>, columns: Vec<Column>) -> Result<Self> {
        let nrows = columns.first().map_or(0, Column::len);

        // Validate all columns have the same length
        for col in &columns {
            if col.len() != nrows {
                return Err(Error::ColumnLengthMismatch {
                    column_name: col.name().to_string(),
                    actual: col.len(),
                    expected: nrows,
                });
            }
        }

        Ok(Self {
            domain_code: domain_code.into(),
            dataset_label: None,
            columns,
            nrows,
        })
    }

    /// Creates a new domain dataset with a label.
    ///
    /// # Errors
    ///
    /// Returns an error if any column has a different length than the others.
    pub fn with_label(
        domain_code: impl Into<DomainCode>,
        dataset_label: Option<impl Into<Label>>,
        columns: Vec<Column>,
    ) -> Result<Self> {
        let mut dataset = Self::new(domain_code, columns)?;
        dataset.dataset_label = dataset_label.map(Into::into);
        Ok(dataset)
    }

    /// Returns the number of columns (variables) in the dataset.
    #[must_use]
    pub fn ncols(&self) -> usize {
        self.columns.len()
    }

    /// Returns `true` if the dataset has no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nrows == 0
    }

    /// Returns the domain code.
    #[must_use]
    pub fn domain_code(&self) -> &str {
        &self.domain_code
    }

    /// Returns the dataset label, if any.
    #[must_use]
    pub fn dataset_label(&self) -> Option<&str> {
        self.dataset_label.as_deref()
    }

    /// Returns a reference to the columns.
    #[must_use]
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// Returns the number of rows (observations) in the dataset.
    #[must_use]
    pub fn nrows(&self) -> usize {
        self.nrows
    }

    /// Returns an iterator over the column names.
    pub fn column_names(&self) -> impl Iterator<Item = &str> {
        self.columns.iter().map(Column::name)
    }

    /// Finds a column by name.
    #[must_use]
    pub fn column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|c| c.name() == name)
    }
}

/// A single column (variable) in a domain dataset.
///
/// Each column has a name, optional role, and typed data.
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    /// The variable name.
    ///
    /// Limited to 8 bytes in XPT v5.
    name: String,

    /// The CDISC variable role, if applicable.
    role: Option<VariableRole>,

    /// The column data.
    data: ColumnData,
}

impl Column {
    /// Creates a new column with the given name and data.
    #[must_use]
    pub fn new(name: impl Into<String>, data: ColumnData) -> Self {
        Self {
            name: name.into(),
            role: None,
            data,
        }
    }

    /// Creates a new column with the given name, role, and data.
    #[must_use]
    pub fn with_role(name: impl Into<String>, role: VariableRole, data: ColumnData) -> Self {
        Self {
            name: name.into(),
            role: Some(role),
            data,
        }
    }

    /// Returns the number of elements in the column.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the column has no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the column name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the column role, if any.
    #[must_use]
    pub fn role(&self) -> Option<VariableRole> {
        self.role
    }

    /// Returns a reference to the column data.
    #[must_use]
    pub fn data(&self) -> &ColumnData {
        &self.data
    }

    /// Returns `true` if the column contains numeric data (for XPT purposes).
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        self.data.is_numeric()
    }

    /// Returns `true` if the column contains character data (for XPT purposes).
    #[must_use]
    pub fn is_character(&self) -> bool {
        self.data.is_character()
    }
}

/// The typed data content of a column.
///
/// XPT v5 only supports two fundamental types: Numeric (8-byte IBM float) and
/// Character (fixed-width byte string). The variants here represent common
/// Rust types that can be converted to these XPT types.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ColumnData {
    /// 64-bit floating-point values.
    ///
    /// Maps directly to XPT Numeric type.
    F64(Vec<Option<f64>>),

    /// 64-bit integer values.
    ///
    /// Converted to XPT Numeric type (as f64) when writing.
    I64(Vec<Option<i64>>),

    /// Boolean values.
    ///
    /// Converted to XPT Numeric type (1.0 or 0.0) when writing.
    Bool(Vec<Option<bool>>),

    /// String values.
    ///
    /// Maps directly to XPT Character type.
    String(Vec<Option<String>>),

    /// Raw byte values.
    ///
    /// Maps to XPT Character type. Strict profiles may forbid this variant.
    Bytes(Vec<Option<Vec<u8>>>),

    /// Date values (without time component).
    ///
    /// Converted to XPT Numeric (SAS date: days since 1960-01-01) when writing,
    /// unless metadata specifies character format.
    Date(Vec<Option<NaiveDate>>),

    /// `DateTime` values.
    ///
    /// Converted to XPT Numeric (SAS datetime: seconds since 1960-01-01 00:00:00)
    /// when writing, unless metadata specifies character format.
    DateTime(Vec<Option<NaiveDateTime>>),

    /// Time values (without date component).
    ///
    /// Converted to XPT Numeric (SAS time: seconds since midnight) when writing,
    /// unless metadata specifies character format.
    Time(Vec<Option<NaiveTime>>),
}

impl ColumnData {
    /// Returns the number of elements in the data.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::F64(v) => v.len(),
            Self::I64(v) => v.len(),
            Self::Bool(v) => v.len(),
            Self::String(v) => v.len(),
            Self::Bytes(v) => v.len(),
            Self::Date(v) => v.len(),
            Self::DateTime(v) => v.len(),
            Self::Time(v) => v.len(),
        }
    }

    /// Returns `true` if the data has no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if this data type maps to XPT Numeric.
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::F64(_)
                | Self::I64(_)
                | Self::Bool(_)
                | Self::Date(_)
                | Self::DateTime(_)
                | Self::Time(_)
        )
    }

    /// Returns `true` if this data type maps to XPT Character.
    #[must_use]
    pub fn is_character(&self) -> bool {
        matches!(self, Self::String(_) | Self::Bytes(_))
    }
}

/// CDISC variable roles.
///
/// These roles are metadata that help classify variables according to CDISC SDTM
/// terminology. They do not affect XPT binary encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum VariableRole {
    /// Identifier variables (e.g., STUDYID, USUBJID, DOMAIN).
    Identifier,
    /// Topic variables (e.g., AEDECOD, LBTESTCD).
    Topic,
    /// Timing variables (e.g., AESTDTC, VISITNUM).
    Timing,
    /// Qualifier variables (e.g., AESER, AESEV).
    Qualifier,
    /// Rule variables (e.g., EPOCH derived from other data).
    Rule,
}

impl VariableRole {
    /// Returns the role as a string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Identifier => "Identifier",
            Self::Topic => "Topic",
            Self::Timing => "Timing",
            Self::Qualifier => "Qualifier",
            Self::Rule => "Rule",
        }
    }
}

impl std::fmt::Display for VariableRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_dataset_new() {
        let cols = vec![
            Column::new("A", ColumnData::F64(vec![Some(1.0), Some(2.0)])),
            Column::new(
                "B",
                ColumnData::String(vec![Some("x".into()), Some("y".into())]),
            ),
        ];
        let ds = Dataset::new("AE", cols).unwrap();
        assert_eq!(ds.nrows(), 2);
        assert_eq!(ds.ncols(), 2);
    }

    #[test]
    fn test_column_length_mismatch() {
        let cols = vec![
            Column::new("A", ColumnData::F64(vec![Some(1.0)])),
            Column::new(
                "B",
                ColumnData::String(vec![Some("x".into()), Some("y".into())]),
            ),
        ];
        let result = Dataset::new("AE", cols);
        assert!(result.is_err());
    }

    #[test]
    fn test_column_data_types() {
        assert!(ColumnData::F64(vec![]).is_numeric());
        assert!(ColumnData::I64(vec![]).is_numeric());
        assert!(ColumnData::Bool(vec![]).is_numeric());
        assert!(ColumnData::Date(vec![]).is_numeric());
        assert!(ColumnData::DateTime(vec![]).is_numeric());
        assert!(ColumnData::Time(vec![]).is_numeric());
        assert!(ColumnData::String(vec![]).is_character());
        assert!(ColumnData::Bytes(vec![]).is_character());
    }
}
