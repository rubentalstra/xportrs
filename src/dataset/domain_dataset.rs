//! Domain dataset representation.
//!
//! This module provides the [`Dataset`] struct which represents a CDISC
//! domain dataset (table) in a columnar format.

use std::fmt;
use std::ops::Index;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::error::{Error, Result};

use super::format::{Format, FormatParseError};
use super::iter::{ColumnNames, IntoIter, Iter, IterMut};
use super::newtypes::{DomainCode, Label, VariableName};

/// A CDISC domain dataset in columnar format.
///
/// This is the primary data structure for representing tables in xportrs.
/// Contains a [`DomainCode`], optional [`Label`], and a collection of [`Column`] items.
///
/// # Invariants
///
/// - All [`Column`] items must have exactly `nrows` elements.
/// - [`DomainCode`] should follow CDISC naming conventions (typically 2-8 characters).
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
    /// Returns [`Error::ColumnLengthMismatch`] if any [`Column`] has a different length than the others.
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
    /// Returns [`Error::ColumnLengthMismatch`] if any [`Column`] has a different length than the others.
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

    /// Returns the domain code as a string slice.
    #[must_use]
    pub fn domain_code(&self) -> &str {
        self.domain_code.as_str()
    }

    /// Returns the dataset label, if any.
    #[must_use]
    pub fn dataset_label(&self) -> Option<&str> {
        self.dataset_label.as_ref().map(Label::as_str)
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

    /// Returns an [`Iter`] over references to the columns.
    #[must_use]
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(&self.columns)
    }

    /// Returns an [`IterMut`] over the columns.
    #[must_use]
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut::new(&mut self.columns)
    }

    /// Returns a [`ColumnNames`] iterator over the column names.
    #[must_use]
    pub fn column_names(&self) -> ColumnNames<'_> {
        ColumnNames::new(&self.columns)
    }

    /// Finds a [`Column`] by name.
    #[must_use]
    pub fn column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|c| c.name() == name)
    }
}

impl IntoIterator for Dataset {
    type Item = Column;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.columns)
    }
}

impl<'a> IntoIterator for &'a Dataset {
    type Item = &'a Column;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Dataset {
    type Item = &'a mut Column;
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl Index<usize> for Dataset {
    type Output = Column;

    fn index(&self, index: usize) -> &Self::Output {
        &self.columns[index]
    }
}

impl Index<&str> for Dataset {
    type Output = Column;

    /// Indexes the dataset by column name.
    ///
    /// # Panics
    ///
    /// Panics if no column with the given name exists.
    fn index(&self, name: &str) -> &Self::Output {
        self.columns
            .iter()
            .find(|c| c.name() == name)
            .unwrap_or_else(|| panic!("no column named '{}'", name))
    }
}

impl Extend<Column> for Dataset {
    /// Extends the dataset with columns from an iterator.
    ///
    /// # Panics
    ///
    /// Panics if any column has a different length than the existing dataset.
    /// If the dataset is empty (nrows == 0), the first column's length becomes
    /// the new nrows.
    fn extend<T: IntoIterator<Item = Column>>(&mut self, iter: T) {
        for col in iter {
            if self.nrows == 0 && self.columns.is_empty() {
                // Empty dataset - take length from first column
                self.nrows = col.len();
            } else if col.len() != self.nrows {
                panic!(
                    "column '{}' has length {} but dataset has {} rows",
                    col.name(),
                    col.len(),
                    self.nrows
                );
            }
            self.columns.push(col);
        }
    }
}

impl fmt::Display for Dataset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{} rows × {} cols]",
            self.domain_code.as_str(),
            self.nrows,
            self.columns.len()
        )
    }
}

/// A single column (variable) in a domain dataset.
///
/// Each column has a [`VariableName`], optional [`VariableRole`], and typed [`ColumnData`].
/// Columns can also have metadata: label, format, informat, and explicit length.
///
/// # CDISC/FDA Compliance
///
/// For FDA regulatory submissions, variable labels are **recommended** to help
/// reviewers understand the data. Labels are limited to 40 bytes in XPT v5.
///
/// Formats control how numeric values are displayed (e.g., `DATE9.`, `8.2`).
/// While FDA recommends avoiding custom SAS formats, standard formats should
/// still have their metadata correctly populated.
///
/// # Example
///
/// ```
/// use xportrs::{Column, ColumnData, VariableRole, Format};
///
/// // Create a simple string column
/// let col = Column::new("USUBJID", ColumnData::String(vec![
///     Some("01-001".into()),
///     Some("01-002".into()),
/// ]));
/// println!("{}", col);  // Prints: USUBJID (String)
///
/// // Create a column with full CDISC metadata
/// let col = Column::new("AESTDTC", ColumnData::String(vec![Some("2024-01-15".into())]))
///     .with_label("Start Date/Time of Adverse Event")
///     .with_format(Format::character(19));
///
/// // Create a column with a CDISC role
/// let col = Column::with_role(
///     "AESEQ",
///     VariableRole::Identifier,
///     ColumnData::I64(vec![Some(1), Some(2)]),
/// );
/// if let Some(role) = col.role() {
///     println!("{} is an {} variable", col.name(), role);
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    /// The variable name.
    ///
    /// Limited to 8 bytes in XPT v5.
    name: VariableName,

    /// The CDISC [`VariableRole`], if applicable.
    role: Option<VariableRole>,

    /// The [`ColumnData`] containing the typed values.
    data: ColumnData,

    /// The variable label (max 40 bytes in XPT v5).
    ///
    /// Labels are recommended for FDA submissions to help reviewers
    /// understand the data. Missing labels will trigger a validation warning.
    label: Option<Label>,

    /// The SAS format (controls display output).
    ///
    /// Examples: `DATE9.`, `8.2`, `$CHAR200.`
    format: Option<Format>,

    /// The SAS informat (controls data input).
    ///
    /// Typically only relevant when reading data from other sources.
    informat: Option<Format>,

    /// Explicit character length override.
    ///
    /// If not set, length is derived from the maximum value length in the data.
    /// Only applicable to character columns.
    length: Option<usize>,
}

impl Column {
    /// Creates a new column with the given name and data.
    #[must_use]
    pub fn new(name: impl Into<VariableName>, data: ColumnData) -> Self {
        Self {
            name: name.into(),
            role: None,
            data,
            label: None,
            format: None,
            informat: None,
            length: None,
        }
    }

    /// Creates a new column with the given name, role, and data.
    #[must_use]
    pub fn with_role(name: impl Into<VariableName>, role: VariableRole, data: ColumnData) -> Self {
        Self {
            name: name.into(),
            role: Some(role),
            data,
            label: None,
            format: None,
            informat: None,
            length: None,
        }
    }

    /// Sets the variable label.
    ///
    /// Labels are limited to 40 bytes in XPT v5 and are **recommended** for
    /// FDA regulatory submissions.
    ///
    /// # Example
    ///
    /// ```
    /// use xportrs::{Column, ColumnData};
    ///
    /// let col = Column::new("USUBJID", ColumnData::String(vec![Some("01-001".into())]))
    ///     .with_label("Unique Subject Identifier");
    ///
    /// assert_eq!(col.label(), Some("Unique Subject Identifier"));
    /// ```
    #[must_use]
    pub fn with_label(mut self, label: impl Into<Label>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the SAS format.
    ///
    /// Formats control how values are displayed. Common formats include:
    /// - `DATE9.` - Display as date (e.g., "15JAN2024")
    /// - `8.2` - Numeric with 8 width and 2 decimals
    /// - `$CHAR200.` - Character with 200 width
    ///
    /// # Example
    ///
    /// ```
    /// use xportrs::{Column, ColumnData, Format};
    ///
    /// let col = Column::new("AESTDT", ColumnData::F64(vec![Some(23391.0)]))
    ///     .with_format(Format::parse("DATE9.").unwrap());
    ///
    /// assert!(col.format().is_some());
    /// ```
    #[must_use]
    pub fn with_format(mut self, format: Format) -> Self {
        self.format = Some(format);
        self
    }

    /// Sets the SAS format from a format string.
    ///
    /// This is a convenience method that parses the format string.
    ///
    /// # Errors
    ///
    /// Returns [`FormatParseError`] if the format string is invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use xportrs::{Column, ColumnData};
    ///
    /// let col = Column::new("AESTDT", ColumnData::F64(vec![Some(23391.0)]))
    ///     .with_format_str("DATE9.")?;
    ///
    /// assert!(col.format().is_some());
    /// # Ok::<(), xportrs::FormatParseError>(())
    /// ```
    pub fn with_format_str(mut self, format: &str) -> std::result::Result<Self, FormatParseError> {
        self.format = Some(Format::parse(format)?);
        Ok(self)
    }

    /// Sets the SAS informat.
    ///
    /// Informats control how data is read from external sources.
    /// Typically only relevant when reading data.
    #[must_use]
    pub fn with_informat(mut self, informat: Format) -> Self {
        self.informat = Some(informat);
        self
    }

    /// Sets an explicit character length.
    ///
    /// This overrides the default behavior of deriving length from the data.
    /// Only applicable to character columns; ignored for numeric columns.
    ///
    /// # Example
    ///
    /// ```
    /// use xportrs::{Column, ColumnData};
    ///
    /// // Reserve 200 bytes even though data is shorter
    /// let col = Column::new("AETERM", ColumnData::String(vec![Some("HEADACHE".into())]))
    ///     .with_length(200);
    ///
    /// assert_eq!(col.explicit_length(), Some(200));
    /// ```
    #[must_use]
    pub fn with_length(mut self, length: usize) -> Self {
        self.length = Some(length);
        self
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
        self.name.as_str()
    }

    /// Returns the [`VariableRole`], if any.
    #[must_use]
    pub fn role(&self) -> Option<VariableRole> {
        self.role
    }

    /// Returns the variable label, if any.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_ref().map(Label::as_str)
    }

    /// Returns the SAS format, if any.
    #[must_use]
    pub fn format(&self) -> Option<&Format> {
        self.format.as_ref()
    }

    /// Returns the SAS informat, if any.
    #[must_use]
    pub fn informat(&self) -> Option<&Format> {
        self.informat.as_ref()
    }

    /// Returns the explicit character length, if any.
    ///
    /// This is the length set via [`Column::with_length`], not the
    /// automatically derived length from the data.
    #[must_use]
    pub fn explicit_length(&self) -> Option<usize> {
        self.length
    }

    /// Returns a reference to the [`ColumnData`].
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

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = match &self.data {
            ColumnData::F64(_) => "F64",
            ColumnData::I64(_) => "I64",
            ColumnData::Bool(_) => "Bool",
            ColumnData::String(_) => "String",
            ColumnData::Bytes(_) => "Bytes",
            ColumnData::Date(_) => "Date",
            ColumnData::DateTime(_) => "DateTime",
            ColumnData::Time(_) => "Time",
        };
        write!(f, "{} ({})", self.name.as_str(), type_name)
    }
}

/// The typed data content of a column.
///
/// XPT v5 only supports two fundamental types: Numeric (8-byte IBM float) and
/// Character (fixed-width byte string). The variants here represent common
/// Rust types that can be converted to these XPT types.
///
/// # Example
///
/// ```
/// use xportrs::ColumnData;
///
/// // Numeric types - None represents missing values
/// let ages = ColumnData::F64(vec![Some(25.0), Some(30.5), None]);
/// println!("{}", ages);  // Prints: F64(3)
///
/// // Convenience: create from plain vectors (all values present)
/// let data: ColumnData = vec![1.0, 2.0, 3.0].into();
/// let data: ColumnData = vec!["Alice", "Bob", "Carol"].into();
///
/// // Check type category
/// if data.is_character() {
///     println!("Character column with {} values", data.len());
/// }
/// ```
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

impl fmt::Display for ColumnData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (type_name, len) = match self {
            Self::F64(v) => ("F64", v.len()),
            Self::I64(v) => ("I64", v.len()),
            Self::Bool(v) => ("Bool", v.len()),
            Self::String(v) => ("String", v.len()),
            Self::Bytes(v) => ("Bytes", v.len()),
            Self::Date(v) => ("Date", v.len()),
            Self::DateTime(v) => ("DateTime", v.len()),
            Self::Time(v) => ("Time", v.len()),
        };
        write!(f, "{}({})", type_name, len)
    }
}

// Convenience From implementations for ColumnData.
// These allow creating columns without wrapping values in Option.

impl From<Vec<f64>> for ColumnData {
    fn from(values: Vec<f64>) -> Self {
        Self::F64(values.into_iter().map(Some).collect())
    }
}

impl From<Vec<i64>> for ColumnData {
    fn from(values: Vec<i64>) -> Self {
        Self::I64(values.into_iter().map(Some).collect())
    }
}

impl From<Vec<i32>> for ColumnData {
    fn from(values: Vec<i32>) -> Self {
        Self::I64(values.into_iter().map(|v| Some(i64::from(v))).collect())
    }
}

impl From<Vec<bool>> for ColumnData {
    fn from(values: Vec<bool>) -> Self {
        Self::Bool(values.into_iter().map(Some).collect())
    }
}

impl From<Vec<String>> for ColumnData {
    fn from(values: Vec<String>) -> Self {
        Self::String(values.into_iter().map(Some).collect())
    }
}

impl From<Vec<&str>> for ColumnData {
    fn from(values: Vec<&str>) -> Self {
        Self::String(values.into_iter().map(|s| Some(s.to_string())).collect())
    }
}

impl From<Vec<NaiveDate>> for ColumnData {
    fn from(values: Vec<NaiveDate>) -> Self {
        Self::Date(values.into_iter().map(Some).collect())
    }
}

impl From<Vec<NaiveDateTime>> for ColumnData {
    fn from(values: Vec<NaiveDateTime>) -> Self {
        Self::DateTime(values.into_iter().map(Some).collect())
    }
}

impl From<Vec<NaiveTime>> for ColumnData {
    fn from(values: Vec<NaiveTime>) -> Self {
        Self::Time(values.into_iter().map(Some).collect())
    }
}

/// CDISC variable roles.
///
/// These roles are metadata that help classify variables according to CDISC SDTM
/// terminology. They do not affect XPT binary encoding.
///
/// # Roles
///
/// - [`VariableRole::Identifier`] - Uniquely identifies observations (e.g., STUDYID, USUBJID)
/// - [`VariableRole::Topic`] - The focus of the observation (e.g., AEDECOD, LBTESTCD)
/// - [`VariableRole::Timing`] - When the observation occurred (e.g., AESTDTC, VISITNUM)
/// - [`VariableRole::Qualifier`] - Additional context (e.g., AESER, AESEV)
/// - [`VariableRole::Rule`] - Derived or algorithmic values (e.g., EPOCH)
///
/// # Example
///
/// ```
/// use xportrs::{Column, ColumnData, VariableRole};
///
/// // Assign roles to clinical data variables
/// let usubjid = Column::with_role(
///     "USUBJID",
///     VariableRole::Identifier,
///     ColumnData::String(vec![Some("01-001".into())]),
/// );
///
/// let aedecod = Column::with_role(
///     "AEDECOD",
///     VariableRole::Topic,
///     ColumnData::String(vec![Some("HEADACHE".into())]),
/// );
///
/// println!("{} role: {}", usubjid.name(), usubjid.role().unwrap());
/// ```
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

    #[test]
    fn test_column_data_from_conversions() {
        // Test From<Vec<f64>>
        let data: ColumnData = vec![1.0, 2.0, 3.0].into();
        assert_eq!(data.len(), 3);
        assert!(data.is_numeric());

        // Test From<Vec<i64>>
        let data: ColumnData = vec![1i64, 2, 3].into();
        assert_eq!(data.len(), 3);
        assert!(data.is_numeric());

        // Test From<Vec<i32>>
        let data: ColumnData = vec![1i32, 2, 3].into();
        assert_eq!(data.len(), 3);
        assert!(data.is_numeric());

        // Test From<Vec<bool>>
        let data: ColumnData = vec![true, false, true].into();
        assert_eq!(data.len(), 3);
        assert!(data.is_numeric());

        // Test From<Vec<String>>
        let data: ColumnData = vec!["a".to_string(), "b".to_string()].into();
        assert_eq!(data.len(), 2);
        assert!(data.is_character());

        // Test From<Vec<&str>>
        let data: ColumnData = vec!["a", "b", "c"].into();
        assert_eq!(data.len(), 3);
        assert!(data.is_character());
    }
}
