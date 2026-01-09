//! Optional miette integration for rich error diagnostics.
//!
//! When the `miette` feature is enabled, this module provides implementations
//! of the `miette::Diagnostic` trait for xportrs error types, enabling
//! rich error reports with source spans and suggestions.
//!
//! # Example
//!
//! ```ignore
//! use miette::{IntoDiagnostic, Result};
//! use xportrs::error::TransformError;
//!
//! fn main() -> Result<()> {
//!     // When using miette, errors are automatically formatted with
//!     // rich context, suggestions, and error codes.
//!     let result = some_xportrs_operation()?;
//!     Ok(())
//! }
//! ```

#[cfg(feature = "miette")]
use miette::Diagnostic;

use super::{TransformError, ValidationError, ValidationErrorCode, XptError};

/// Extension trait for getting error codes from xportrs errors.
///
/// This trait is part of the public API for users who want to programmatically
/// handle errors with machine-readable codes and contextual help messages.
#[allow(dead_code)]
pub trait ErrorCodeExt {
    /// Get a string error code for this error.
    fn error_code(&self) -> &'static str;

    /// Get an optional help message for this error.
    fn help_message(&self) -> Option<String>;
}

impl ErrorCodeExt for XptError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::FileNotFound { .. } => "xportrs::file_not_found",
            Self::Io(_) => "xportrs::io",
            Self::InvalidHeader { .. } => "xportrs::invalid_header",
            Self::UnexpectedEof { .. } => "xportrs::unexpected_eof",
            Self::InvalidNamestr { .. } => "xportrs::invalid_namestr",
            Self::FloatConversion { .. } => "xportrs::float_conversion",
            Self::AlignmentError { .. } => "xportrs::alignment",
            Self::InvalidFormat { .. } => "xportrs::invalid_format",
            Self::MissingHeader { .. } => "xportrs::missing_header",
            Self::NumericParse { .. } => "xportrs::numeric_parse",
            Self::ObservationOverflow => "xportrs::observation_overflow",
            Self::TrailingBytes => "xportrs::trailing_bytes",
            Self::RecordOutOfBounds { .. } => "xportrs::record_out_of_bounds",
            Self::InvalidDatasetName { .. } => "xportrs::invalid_dataset_name",
            Self::DatasetNameTooLong { .. } => "xportrs::dataset_name_too_long",
            Self::DatasetLabelTooLong { .. } => "xportrs::dataset_label_too_long",
            Self::InvalidVariableName { .. } => "xportrs::invalid_variable_name",
            Self::VariableNameTooLong { .. } => "xportrs::variable_name_too_long",
            Self::DuplicateVariable { .. } => "xportrs::duplicate_variable",
            Self::ZeroLength { .. } => "xportrs::zero_length",
            Self::VariableLabelTooLong { .. } => "xportrs::variable_label_too_long",
            Self::FormatNameTooLong { .. } => "xportrs::format_name_too_long",
            Self::RowLengthMismatch { .. } => "xportrs::row_length_mismatch",
            Self::Validation(_) => "xportrs::validation",
        }
    }

    fn help_message(&self) -> Option<String> {
        match self {
            Self::FileNotFound { path } => Some(format!(
                "Check that the file '{}' exists and you have read permissions",
                path.display()
            )),
            Self::InvalidHeader { expected } => Some(format!(
                "Expected header type '{}'. Is this a valid XPT file?",
                expected
            )),
            Self::DatasetNameTooLong { name, limit } => Some(format!(
                "Dataset name '{}' has {} characters. Maximum for this version is {}. Consider using V8 format for longer names.",
                name, name.len(), limit
            )),
            Self::VariableNameTooLong { name, limit } => Some(format!(
                "Variable name '{}' has {} characters. Maximum is {}. Consider using V8 format for longer names.",
                name, name.len(), limit
            )),
            Self::DuplicateVariable { name } => Some(format!(
                "Variable '{}' appears more than once. Remove or rename duplicates.",
                name
            )),
            _ => None,
        }
    }
}

impl ErrorCodeExt for TransformError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::TypeCoercion { .. } => "xportrs::type_coercion",
            Self::Truncation { .. } => "xportrs::truncation",
            Self::VariableNotInSpec { .. } => "xportrs::variable_not_in_spec",
            Self::VariableNotInData { .. } => "xportrs::variable_not_in_data",
            Self::LabelTooLong { .. } => "xportrs::label_too_long",
            Self::InvalidFormat { .. } => "xportrs::invalid_format",
            Self::ValidationFailed(_) => "xportrs::validation_failed",
            Self::Io(_) => "xportrs::io",
            Self::Xpt(_) => "xportrs::xpt",
            Self::Other { .. } => "xportrs::transform_other",
        }
    }

    fn help_message(&self) -> Option<String> {
        match self {
            Self::TypeCoercion { variable, message } => Some(format!(
                "Failed to convert variable '{}': {}. Check that the data type in your spec matches the actual data.",
                variable, message
            )),
            Self::Truncation { variable, original_length, max_length, .. } => Some(format!(
                "Value in '{}' was {} characters but max is {}. Consider increasing length in spec or truncating data.",
                variable, original_length, max_length
            )),
            Self::VariableNotInSpec { variable } => Some(format!(
                "Variable '{}' exists in data but not in spec. Add it to your specification or remove from data.",
                variable
            )),
            Self::VariableNotInData { variable } => Some(format!(
                "Variable '{}' is in spec but missing from data. Add it to your data or remove from spec.",
                variable
            )),
            Self::LabelTooLong { variable, max_length, actual_length } => Some(format!(
                "Label for '{}' is {} characters but max is {}. Shorten the label in your spec.",
                variable, actual_length, max_length
            )),
            Self::InvalidFormat { variable, format } => Some(format!(
                "Format '{}' is not valid for variable '{}'. Check SAS format documentation.",
                format, variable
            )),
            _ => None,
        }
    }
}

impl ErrorCodeExt for ValidationError {
    fn error_code(&self) -> &'static str {
        match self.code {
            ValidationErrorCode::EmptyName => "xportrs::validation::empty_name",
            ValidationErrorCode::NameTooLong => "xportrs::validation::name_too_long",
            ValidationErrorCode::InvalidNameCharacter => "xportrs::validation::invalid_name_char",
            ValidationErrorCode::NameStartsWithNumber => "xportrs::validation::name_starts_with_number",
            ValidationErrorCode::NonAsciiName => "xportrs::validation::non_ascii_name",
            ValidationErrorCode::LowercaseName => "xportrs::validation::lowercase_name",
            ValidationErrorCode::LabelTooLong => "xportrs::validation::label_too_long",
            ValidationErrorCode::NonAsciiLabel => "xportrs::validation::non_ascii_label",
            ValidationErrorCode::NonPrintableLabel => "xportrs::validation::non_printable_label",
            ValidationErrorCode::FormatNameTooLong => "xportrs::validation::format_name_too_long",
            ValidationErrorCode::InvalidFormatName => "xportrs::validation::invalid_format_name",
            ValidationErrorCode::InformatNameTooLong => "xportrs::validation::informat_name_too_long",
            ValidationErrorCode::InvalidInformatName => "xportrs::validation::invalid_informat_name",
            ValidationErrorCode::CustomFormat => "xportrs::validation::custom_format",
            ValidationErrorCode::DuplicateColumnName => "xportrs::validation::duplicate_column",
            ValidationErrorCode::ZeroLengthColumn => "xportrs::validation::zero_length_column",
            ValidationErrorCode::ColumnLengthTooLong => "xportrs::validation::column_length_too_long",
            ValidationErrorCode::EmptyDataset => "xportrs::validation::empty_dataset",
            ValidationErrorCode::TooManyColumns => "xportrs::validation::too_many_columns",
            ValidationErrorCode::RowLengthMismatch => "xportrs::validation::row_length_mismatch",
            ValidationErrorCode::CharacterValueTooLong => "xportrs::validation::char_value_too_long",
            ValidationErrorCode::NonAsciiValue => "xportrs::validation::non_ascii_value",
            ValidationErrorCode::WrongVersion => "xportrs::validation::wrong_version",
            ValidationErrorCode::DatasetNameMismatch => "xportrs::validation::dataset_name_mismatch",
            ValidationErrorCode::MultipleDatasets => "xportrs::validation::multiple_datasets",
            ValidationErrorCode::CompressedData => "xportrs::validation::compressed_data",
            ValidationErrorCode::VariableNotInSpec => "xportrs::validation::variable_not_in_spec",
            ValidationErrorCode::VariableNotInData => "xportrs::validation::variable_not_in_data",
            ValidationErrorCode::TypeMismatch => "xportrs::validation::type_mismatch",
            ValidationErrorCode::LengthMismatch => "xportrs::validation::length_mismatch",
            ValidationErrorCode::OrderMismatch => "xportrs::validation::order_mismatch",
            ValidationErrorCode::FormatMismatch => "xportrs::validation::format_mismatch",
            ValidationErrorCode::LabelMismatch => "xportrs::validation::label_mismatch",
            ValidationErrorCode::DatasetNameSpecMismatch => "xportrs::validation::dataset_name_spec_mismatch",
            ValidationErrorCode::DatasetLabelMismatch => "xportrs::validation::dataset_label_mismatch",
        }
    }

    fn help_message(&self) -> Option<String> {
        match self.code {
            ValidationErrorCode::NameTooLong => Some(
                "Consider using XPT V8 format which allows names up to 32 characters.".to_string()
            ),
            ValidationErrorCode::LabelTooLong => Some(
                "Consider using XPT V8 format which allows labels up to 256 characters.".to_string()
            ),
            ValidationErrorCode::LowercaseName => Some(
                "SAS names are case-insensitive but FDA recommends uppercase for consistency.".to_string()
            ),
            ValidationErrorCode::NonAsciiValue => Some(
                "Non-ASCII characters may cause issues with SAS processing. Consider using ASCII equivalents.".to_string()
            ),
            ValidationErrorCode::WrongVersion => Some(
                "FDA requires XPT V5 format. Use XptWriterOptions::with_version(XptVersion::V5).".to_string()
            ),
            _ => None,
        }
    }
}

#[cfg(feature = "miette")]
impl Diagnostic for XptError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(self.error_code()))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.help_message().map(|s| Box::new(s) as Box<dyn std::fmt::Display>)
    }
}

#[cfg(feature = "miette")]
impl Diagnostic for TransformError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(self.error_code()))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.help_message().map(|s| Box::new(s) as Box<dyn std::fmt::Display>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_xpt_error_codes() {
        let err = XptError::FileNotFound { path: PathBuf::from("test.xpt") };
        assert_eq!(err.error_code(), "xportrs::file_not_found");
        assert!(err.help_message().is_some());
    }

    #[test]
    fn test_transform_error_codes() {
        let err = TransformError::TypeCoercion {
            variable: "AGE".to_string(),
            message: "Cannot convert string to number".to_string(),
        };
        assert_eq!(err.error_code(), "xportrs::type_coercion");
        assert!(err.help_message().is_some());
    }
}
