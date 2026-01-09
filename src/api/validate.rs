//! Validation operations.

use polars::prelude::*;

use crate::error::{ErrorLocation, ValidationError, ValidationErrorCode, ValidationResult};
use crate::policy::AgencyPolicy;
use crate::types::XptVersion;

/// Validate a DataFrame against an agency policy.
///
/// This function validates the DataFrame structure and contents against
/// the rules defined by the given agency policy (FDA, NMPA, PMDA, or custom).
///
/// # Arguments
///
/// * `df` - DataFrame to validate
/// * `policy` - Agency policy defining validation rules
///
/// # Returns
///
/// A `ValidationResult` containing any validation errors found.
///
/// # Example
///
/// ```no_run
/// use xportrs::{xportrs_validate, xportrs_read};
/// use xportrs::policy::FdaPolicy;
///
/// let df = xportrs_read("data.xpt").unwrap();
/// let result = xportrs_validate(&df, &FdaPolicy::strict());
///
/// if result.is_valid() {
///     println!("DataFrame is FDA-compliant");
/// } else {
///     for error in result.errors.iter() {
///         println!("Validation error: {}", error);
///     }
/// }
/// ```
pub fn xportrs_validate(df: &DataFrame, policy: &dyn AgencyPolicy) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Get required version from policy
    let version = policy.required_version().unwrap_or(XptVersion::V5);

    // Validate column names
    for (idx, col) in df.get_columns().iter().enumerate() {
        let name = col.name().as_str();

        // Check name length
        if name.len() > version.variable_name_limit() {
            result.add_error(ValidationError::error(
                ValidationErrorCode::NameTooLong,
                format!(
                    "Variable name '{}' exceeds {} character limit",
                    name,
                    version.variable_name_limit()
                ),
                ErrorLocation::Column {
                    dataset: String::new(),
                    column: name.to_string(),
                    index: idx,
                },
            ));
        }

        // Check name is valid SAS name
        if !is_valid_sas_name(name) {
            result.add_error(ValidationError::error(
                ValidationErrorCode::InvalidNameCharacter,
                format!("Invalid SAS variable name: '{}'", name),
                ErrorLocation::Column {
                    dataset: String::new(),
                    column: name.to_string(),
                    index: idx,
                },
            ));
        }

        // Check for ASCII
        if !name.is_ascii() {
            result.add_error(ValidationError::error(
                ValidationErrorCode::NonAsciiName,
                format!("Variable name '{}' contains non-ASCII characters", name),
                ErrorLocation::Column {
                    dataset: String::new(),
                    column: name.to_string(),
                    index: idx,
                },
            ));
        }
    }

    // Check for duplicate column names
    let mut seen_names = std::collections::HashSet::new();
    for (idx, col) in df.get_columns().iter().enumerate() {
        let name_upper = col.name().as_str().to_uppercase();
        if !seen_names.insert(name_upper.clone()) {
            result.add_error(ValidationError::error(
                ValidationErrorCode::DuplicateColumnName,
                format!("Duplicate variable name: '{}'", col.name().as_str()),
                ErrorLocation::Column {
                    dataset: String::new(),
                    column: col.name().as_str().to_string(),
                    index: idx,
                },
            ));
        }
    }

    // Validate string column contents for ASCII
    for col in df.get_columns() {
        if col.dtype() == &DataType::String {
            let series = col.as_materialized_series();
            if let Ok(ca) = series.str() {
                for (row_idx, opt) in ca.into_iter().enumerate() {
                    if let Some(s) = opt {
                        if !s.is_ascii() {
                            result.add_warning(ValidationError::warning(
                                ValidationErrorCode::NonAsciiValue,
                                format!(
                                    "Non-ASCII character in column '{}' at row {}",
                                    col.name().as_str(),
                                    row_idx + 1
                                ),
                                ErrorLocation::Value {
                                    dataset: String::new(),
                                    column: col.name().as_str().to_string(),
                                    row: row_idx,
                                },
                            ));
                            break; // Only report first non-ASCII per column
                        }
                    }
                }
            }
        }
    }

    result
}

/// Check if a name is a valid SAS variable name.
fn is_valid_sas_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    // First character must be letter or underscore
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }

    // Remaining characters must be alphanumeric or underscore
    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_sas_names() {
        assert!(is_valid_sas_name("AGE"));
        assert!(is_valid_sas_name("_TEMP"));
        assert!(is_valid_sas_name("VAR123"));
        assert!(is_valid_sas_name("A"));
    }

    #[test]
    fn test_invalid_sas_names() {
        assert!(!is_valid_sas_name(""));
        assert!(!is_valid_sas_name("123VAR"));
        assert!(!is_valid_sas_name("VAR-NAME"));
        assert!(!is_valid_sas_name("VAR NAME"));
    }
}
