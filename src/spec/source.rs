//! Metadata source trait and implementations.
//!
//! This module provides the [`MetadataSource`] trait for loading specifications
//! from various sources. The primary implementation is [`DataFrameMetadataSource`]
//! which loads specs from Polars DataFrames (requires `polars` feature).
//!
//! # Usage
//!
//! Users typically load their specification data using Polars (CSV, Excel, etc.)
//! and then create a `DataFrameMetadataSource` to parse it into specs:
//!
//! ```ignore
//! use polars::prelude::*;
//! use xportrs::spec::{DataFrameMetadataSource, MetadataSource};
//!
//! // Load spec from CSV via Polars
//! let spec_df = CsvReadOptions::default()
//!     .try_into_reader_with_file_path(Some("specs/var_spec.csv".into()))?
//!     .finish()?;
//!
//! // Create metadata source
//! let source = DataFrameMetadataSource::new(spec_df);
//! let dm_spec = source.load_dataset_spec("DM")?;
//! ```

use crate::error::SpecError;
use crate::spec::DatasetSpec;

/// Trait for loading specifications from various sources.
///
/// Implementations of this trait provide the ability to load dataset
/// specifications from different formats (DataFrames, etc.).
///
/// # Example
///
/// ```ignore
/// use xportrs::spec::{MetadataSource, DataFrameMetadataSource};
///
/// let source = DataFrameMetadataSource::new(df);
/// if source.has_dataset("DM") {
///     let spec = source.load_dataset_spec("DM")?;
///     println!("Loaded spec for {} with {} variables", spec.name, spec.variables.len());
/// }
/// ```
pub trait MetadataSource {
    /// Load a specification for a single dataset.
    ///
    /// # Arguments
    ///
    /// * `dataset_name` - The name of the dataset to load (case-insensitive)
    ///
    /// # Errors
    ///
    /// Returns an error if the dataset is not found or if there's an issue
    /// parsing the specification data.
    fn load_dataset_spec(&self, dataset_name: &str) -> Result<DatasetSpec, SpecError>;

    /// Load specifications for all datasets in the source.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue parsing any specification data.
    fn load_all_specs(&self) -> Result<Vec<DatasetSpec>, SpecError>;

    /// Check if a dataset exists in the source.
    ///
    /// # Arguments
    ///
    /// * `dataset_name` - The name of the dataset to check (case-insensitive)
    fn has_dataset(&self, dataset_name: &str) -> bool;

    /// Get the names of all datasets in the source.
    fn dataset_names(&self) -> Vec<String>;
}

#[cfg(feature = "polars")]
mod polars_source {
    use polars::prelude::*;
    use std::collections::HashSet;

    use crate::error::SpecError;
    use crate::spec::{ColumnMapping, Core, DatasetSpec, VariableSpec};
    use crate::types::{FormatSpec, XptType};

    use super::MetadataSource;

    /// Load specifications from a Polars DataFrame.
    ///
    /// This source expects a DataFrame with rows representing variable specifications.
    /// Each row should contain metadata for one variable, with columns mapped according
    /// to the [`ColumnMapping`] configuration.
    ///
    /// # Required Columns
    ///
    /// - `variable` (or mapped name) - Variable name (required)
    ///
    /// # Optional Columns
    ///
    /// - `dataset` - Dataset name (if absent, a single-dataset source is assumed)
    /// - `label` - Variable label
    /// - `type` - Variable type ("Num"/"Numeric" or "Char"/"Character")
    /// - `length` - Variable length
    /// - `order` - Variable order/position
    /// - `format` - SAS format specification
    /// - `informat` - SAS informat specification
    /// - `origin` - Variable origin (e.g., "Derived", "CRF")
    /// - `core` - CDISC Core designation
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "polars")]
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use polars::prelude::*;
    /// use xportrs::spec::{DataFrameMetadataSource, MetadataSource};
    ///
    /// let df = df! {
    ///     "dataset" => &["DM", "DM", "DM"],
    ///     "variable" => &["USUBJID", "AGE", "SEX"],
    ///     "label" => &["Unique Subject ID", "Age", "Sex"],
    ///     "type" => &["Char", "Num", "Char"],
    ///     "length" => &[20u32, 8, 1],
    ///     "order" => &[1u32, 2, 3],
    /// }?;
    ///
    /// let source = DataFrameMetadataSource::new(df);
    /// let spec = source.load_dataset_spec("DM")?;
    /// assert_eq!(spec.variables.len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    #[derive(Debug, Clone)]
    pub struct DataFrameMetadataSource {
        df: DataFrame,
        mapping: ColumnMapping,
    }

    impl DataFrameMetadataSource {
        /// Create a new metadata source from a DataFrame with default column mapping.
        #[must_use]
        pub fn new(df: DataFrame) -> Self {
            Self {
                df,
                mapping: ColumnMapping::default(),
            }
        }

        /// Use a custom column mapping.
        ///
        /// # Example
        ///
        /// ```
        /// # #[cfg(feature = "polars")]
        /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
        /// use polars::prelude::*;
        /// use xportrs::spec::{DataFrameMetadataSource, ColumnMapping};
        ///
        /// let df = df! {
        ///     "DOMAIN" => &["DM"],
        ///     "VARNAME" => &["USUBJID"],
        ///     "VARLABEL" => &["Unique Subject ID"],
        /// }?;
        ///
        /// let source = DataFrameMetadataSource::new(df)
        ///     .with_mapping(ColumnMapping::pinnacle21());
        /// # Ok(())
        /// # }
        /// ```
        #[must_use]
        pub fn with_mapping(mut self, mapping: ColumnMapping) -> Self {
            self.mapping = mapping;
            self
        }

        /// Get the underlying DataFrame.
        #[must_use]
        pub fn dataframe(&self) -> &DataFrame {
            &self.df
        }

        /// Get the current column mapping.
        #[must_use]
        pub fn mapping(&self) -> &ColumnMapping {
            &self.mapping
        }

        /// Check if a column exists in the DataFrame.
        fn has_column(&self, name: &str) -> bool {
            self.df.get_column_names().iter().any(|c| c.as_str() == name)
        }

        /// Get a string value from a row.
        fn get_string(&self, row: usize, col: &str) -> Option<String> {
            if !self.has_column(col) {
                return None;
            }

            self.df
                .column(col)
                .ok()
                .and_then(|s| s.str().ok())
                .and_then(|ca| ca.get(row))
                .map(|s| s.to_string())
        }

        /// Get an optional u16 value from a row.
        fn get_u16(&self, row: usize, col: &str) -> Option<u16> {
            if !self.has_column(col) {
                return None;
            }

            self.df.column(col).ok().and_then(|s| {
                // Try different numeric types
                if let Ok(ca) = s.i64() {
                    ca.get(row).map(|v| v as u16)
                } else if let Ok(ca) = s.u32() {
                    ca.get(row).map(|v| v as u16)
                } else if let Ok(ca) = s.i32() {
                    ca.get(row).map(|v| v as u16)
                } else if let Ok(ca) = s.f64() {
                    ca.get(row).map(|v| v as u16)
                } else {
                    None
                }
            })
        }

        /// Get an optional usize value from a row.
        fn get_usize(&self, row: usize, col: &str) -> Option<usize> {
            if !self.has_column(col) {
                return None;
            }

            self.df.column(col).ok().and_then(|s| {
                if let Ok(ca) = s.i64() {
                    ca.get(row).map(|v| v as usize)
                } else if let Ok(ca) = s.u32() {
                    ca.get(row).map(|v| v as usize)
                } else if let Ok(ca) = s.i32() {
                    ca.get(row).map(|v| v as usize)
                } else if let Ok(ca) = s.f64() {
                    ca.get(row).map(|v| v as usize)
                } else {
                    None
                }
            })
        }

        /// Parse a type string into XptType.
        fn parse_type(type_str: &str) -> XptType {
            let normalized = type_str.trim().to_lowercase();
            if normalized == "num" || normalized == "numeric" || normalized == "n" {
                XptType::Num
            } else {
                XptType::Char
            }
        }

        /// Parse a Core designation from string.
        fn parse_core(core_str: &str) -> Option<Core> {
            let normalized = core_str.trim().to_lowercase();
            match normalized.as_str() {
                "req" | "required" => Some(Core::Required),
                "exp" | "expected" => Some(Core::Expected),
                "perm" | "permissible" => Some(Core::Permissible),
                _ => None,
            }
        }

        /// Parse a format specification string.
        fn parse_format(format_str: &str) -> Option<FormatSpec> {
            let trimmed = format_str.trim();
            if trimmed.is_empty() {
                return None;
            }
            // Default to width 8 for parsed format strings
            Some(FormatSpec::with_name(trimmed, 8))
        }

        /// Build a VariableSpec from a DataFrame row.
        fn build_variable_spec(&self, row: usize) -> Option<VariableSpec> {
            // Variable name is required
            let name = self.get_string(row, &self.mapping.variable_col)?;

            // Get type and length
            let var_type = self
                .get_string(row, &self.mapping.type_col)
                .map(|s| Self::parse_type(&s))
                .unwrap_or(XptType::Char);

            let length = self.get_u16(row, &self.mapping.length_col);

            // Create base spec
            let mut spec = if var_type.is_numeric() {
                VariableSpec::numeric(&name)
            } else {
                VariableSpec::character(&name, length.unwrap_or(8))
            };

            // Add optional fields
            if let Some(label) = self.get_string(row, &self.mapping.label_col) {
                spec = spec.with_label(label);
            }

            if let Some(order) = self.get_usize(row, &self.mapping.order_col) {
                spec = spec.with_order(order);
            }

            if let Some(format_str) = self.get_string(row, &self.mapping.format_col) {
                if let Some(format) = Self::parse_format(&format_str) {
                    spec = spec.with_format(format);
                }
            }

            if let Some(informat_str) = self.get_string(row, &self.mapping.informat_col) {
                if let Some(informat) = Self::parse_format(&informat_str) {
                    spec = spec.with_informat(informat);
                }
            }

            if let Some(origin) = self.get_string(row, &self.mapping.origin_col) {
                spec = spec.with_origin(origin);
            }

            if let Some(core_str) = self.get_string(row, &self.mapping.core_col) {
                if let Some(core) = Self::parse_core(&core_str) {
                    spec = spec.with_core(core);
                }
            }

            // Update length if it was specified and type is numeric
            if var_type.is_numeric() {
                if let Some(len) = length {
                    spec = spec.with_length(len);
                }
            }

            Some(spec)
        }

        /// Get rows for a specific dataset.
        fn get_dataset_rows(&self, dataset_name: &str) -> Vec<usize> {
            let dataset_name_upper = dataset_name.to_uppercase();

            if !self.has_column(&self.mapping.dataset_col) {
                // No dataset column - return all rows
                return (0..self.df.height()).collect();
            }

            (0..self.df.height())
                .filter(|&row| {
                    self.get_string(row, &self.mapping.dataset_col)
                        .map(|s| s.to_uppercase() == dataset_name_upper)
                        .unwrap_or(false)
                })
                .collect()
        }
    }

    impl MetadataSource for DataFrameMetadataSource {
        fn load_dataset_spec(&self, dataset_name: &str) -> Result<DatasetSpec, SpecError> {
            let rows = self.get_dataset_rows(dataset_name);

            if rows.is_empty() {
                return Err(SpecError::DatasetNotFound {
                    dataset: dataset_name.to_string(),
                });
            }

            let mut spec = DatasetSpec::new(dataset_name);

            for row in rows {
                if let Some(var_spec) = self.build_variable_spec(row) {
                    spec = spec.add_variable(var_spec);
                }
            }

            Ok(spec)
        }

        fn load_all_specs(&self) -> Result<Vec<DatasetSpec>, SpecError> {
            let names = self.dataset_names();

            if names.is_empty() {
                // No dataset column - treat entire DataFrame as single unnamed dataset
                let mut spec = DatasetSpec::new("UNKNOWN");
                for row in 0..self.df.height() {
                    if let Some(var_spec) = self.build_variable_spec(row) {
                        spec = spec.add_variable(var_spec);
                    }
                }
                return Ok(vec![spec]);
            }

            names
                .into_iter()
                .map(|name| self.load_dataset_spec(&name))
                .collect()
        }

        fn has_dataset(&self, dataset_name: &str) -> bool {
            !self.get_dataset_rows(dataset_name).is_empty()
        }

        fn dataset_names(&self) -> Vec<String> {
            if !self.has_column(&self.mapping.dataset_col) {
                return Vec::new();
            }

            let mut names: HashSet<String> = HashSet::new();

            for row in 0..self.df.height() {
                if let Some(name) = self.get_string(row, &self.mapping.dataset_col) {
                    names.insert(name.to_uppercase());
                }
            }

            let mut result: Vec<String> = names.into_iter().collect();
            result.sort();
            result
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_load_single_dataset() {
            let df = df! {
                "dataset" => &["DM", "DM", "DM"],
                "variable" => &["USUBJID", "AGE", "SEX"],
                "label" => &["Unique Subject ID", "Age", "Sex"],
                "type" => &["Char", "Num", "Char"],
                "length" => &[20i64, 8, 1],
                "order" => &[1i64, 2, 3],
            }
            .unwrap();

            let source = DataFrameMetadataSource::new(df);
            let spec = source.load_dataset_spec("DM").unwrap();

            assert_eq!(spec.name, "DM");
            assert_eq!(spec.variables.len(), 3);
            assert_eq!(spec.variables[0].name, "USUBJID");
            assert_eq!(spec.variables[0].label.as_deref(), Some("Unique Subject ID"));
            assert!(spec.variables[0].data_type.is_character());
            assert_eq!(spec.variables[0].length, Some(20));
            assert_eq!(spec.variables[0].order, Some(1));

            assert_eq!(spec.variables[1].name, "AGE");
            assert!(spec.variables[1].data_type.is_numeric());

            assert_eq!(spec.variables[2].name, "SEX");
            assert_eq!(spec.variables[2].length, Some(1));
        }

        #[test]
        fn test_load_multiple_datasets() {
            let df = df! {
                "dataset" => &["DM", "DM", "AE", "AE"],
                "variable" => &["USUBJID", "AGE", "USUBJID", "AETERM"],
                "type" => &["Char", "Num", "Char", "Char"],
                "length" => &[20i64, 8, 20, 200],
            }
            .unwrap();

            let source = DataFrameMetadataSource::new(df);

            assert!(source.has_dataset("DM"));
            assert!(source.has_dataset("AE"));
            assert!(!source.has_dataset("LB"));

            let dm_spec = source.load_dataset_spec("DM").unwrap();
            assert_eq!(dm_spec.variables.len(), 2);

            let ae_spec = source.load_dataset_spec("AE").unwrap();
            assert_eq!(ae_spec.variables.len(), 2);
        }

        #[test]
        fn test_load_all_specs() {
            let df = df! {
                "dataset" => &["DM", "DM", "AE"],
                "variable" => &["USUBJID", "AGE", "AETERM"],
                "type" => &["Char", "Num", "Char"],
                "length" => &[20i64, 8, 200],
            }
            .unwrap();

            let source = DataFrameMetadataSource::new(df);
            let specs = source.load_all_specs().unwrap();

            assert_eq!(specs.len(), 2);
        }

        #[test]
        fn test_dataset_names() {
            let df = df! {
                "dataset" => &["DM", "DM", "AE", "LB"],
                "variable" => &["A", "B", "C", "D"],
            }
            .unwrap();

            let source = DataFrameMetadataSource::new(df);
            let names = source.dataset_names();

            assert_eq!(names.len(), 3);
            assert!(names.contains(&"DM".to_string()));
            assert!(names.contains(&"AE".to_string()));
            assert!(names.contains(&"LB".to_string()));
        }

        #[test]
        fn test_custom_mapping() {
            let df = df! {
                "DOMAIN" => &["DM"],
                "VARNAME" => &["USUBJID"],
                "VARLABEL" => &["Unique Subject ID"],
                "VARTYPE" => &["Char"],
                "LENGTH" => &[20i64],
            }
            .unwrap();

            let source = DataFrameMetadataSource::new(df).with_mapping(ColumnMapping::pinnacle21());

            let spec = source.load_dataset_spec("DM").unwrap();
            assert_eq!(spec.variables.len(), 1);
            assert_eq!(spec.variables[0].name, "USUBJID");
            assert_eq!(
                spec.variables[0].label.as_deref(),
                Some("Unique Subject ID")
            );
        }

        #[test]
        fn test_case_insensitive_dataset_lookup() {
            let df = df! {
                "dataset" => &["DM", "DM"],
                "variable" => &["A", "B"],
            }
            .unwrap();

            let source = DataFrameMetadataSource::new(df);

            assert!(source.has_dataset("DM"));
            assert!(source.has_dataset("dm"));
            assert!(source.has_dataset("Dm"));
        }

        #[test]
        fn test_no_dataset_column() {
            let df = df! {
                "variable" => &["USUBJID", "AGE"],
                "type" => &["Char", "Num"],
                "length" => &[20i64, 8],
            }
            .unwrap();

            let source = DataFrameMetadataSource::new(df);

            // Without dataset column, load_all_specs returns single "UNKNOWN" dataset
            let specs = source.load_all_specs().unwrap();
            assert_eq!(specs.len(), 1);
            assert_eq!(specs[0].name, "UNKNOWN");
            assert_eq!(specs[0].variables.len(), 2);

            // Any dataset name should match all rows
            let spec = source.load_dataset_spec("ANY").unwrap();
            assert_eq!(spec.variables.len(), 2);
        }

        #[test]
        fn test_missing_dataset() {
            let df = df! {
                "dataset" => &["DM"],
                "variable" => &["A"],
            }
            .unwrap();

            let source = DataFrameMetadataSource::new(df);
            let result = source.load_dataset_spec("NONEXISTENT");

            assert!(result.is_err());
            assert!(matches!(result, Err(SpecError::DatasetNotFound { .. })));
        }

        #[test]
        fn test_parse_type() {
            assert!(DataFrameMetadataSource::parse_type("Num").is_numeric());
            assert!(DataFrameMetadataSource::parse_type("numeric").is_numeric());
            assert!(DataFrameMetadataSource::parse_type("N").is_numeric());
            assert!(DataFrameMetadataSource::parse_type("Char").is_character());
            assert!(DataFrameMetadataSource::parse_type("character").is_character());
            assert!(DataFrameMetadataSource::parse_type("C").is_character());
        }

        #[test]
        fn test_parse_core() {
            assert_eq!(
                DataFrameMetadataSource::parse_core("Req"),
                Some(Core::Required)
            );
            assert_eq!(
                DataFrameMetadataSource::parse_core("required"),
                Some(Core::Required)
            );
            assert_eq!(
                DataFrameMetadataSource::parse_core("Exp"),
                Some(Core::Expected)
            );
            assert_eq!(
                DataFrameMetadataSource::parse_core("Perm"),
                Some(Core::Permissible)
            );
            assert_eq!(DataFrameMetadataSource::parse_core("invalid"), None);
        }

        #[test]
        fn test_with_format() {
            let df = df! {
                "dataset" => &["DM"],
                "variable" => &["BRTHDTC"],
                "type" => &["Char"],
                "length" => &[19i64],
                "format" => &["$CHAR19."],
            }
            .unwrap();

            let source = DataFrameMetadataSource::new(df);
            let spec = source.load_dataset_spec("DM").unwrap();

            assert!(spec.variables[0].format.is_some());
        }

        #[test]
        fn test_with_core() {
            let df = df! {
                "dataset" => &["DM", "DM"],
                "variable" => &["USUBJID", "AGEU"],
                "type" => &["Char", "Char"],
                "length" => &[20i64, 10],
                "core" => &["Req", "Perm"],
            }
            .unwrap();

            let source = DataFrameMetadataSource::new(df);
            let spec = source.load_dataset_spec("DM").unwrap();

            assert_eq!(spec.variables[0].core, Some(Core::Required));
            assert_eq!(spec.variables[1].core, Some(Core::Permissible));
        }
    }
}

#[cfg(feature = "polars")]
pub use polars_source::DataFrameMetadataSource;
