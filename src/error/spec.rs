//! Specification loading and parsing errors.
//!
//! [`SpecError`] represents errors that occur when loading or parsing
//! metadata specifications from external sources.

use thiserror::Error;

/// Error type for specification operations.
///
/// These errors occur when loading metadata specifications from
/// DataFrames or other sources.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SpecError {
    /// Failed to load specification from source.
    #[error("failed to load specification: {message}")]
    LoadFailed {
        /// Description of the failure
        message: String,
    },

    /// Required column missing from specification.
    #[error("required column '{column}' missing from specification")]
    MissingColumn {
        /// Column name
        column: String,
    },

    /// Dataset not found in specification.
    #[error("dataset '{dataset}' not found in specification")]
    DatasetNotFound {
        /// Dataset name
        dataset: String,
    },

    /// Duplicate variable in specification.
    #[error("duplicate variable '{variable}' in specification for dataset '{dataset}'")]
    DuplicateVariable {
        /// Dataset name
        dataset: String,
        /// Variable name
        variable: String,
    },

    /// Invalid type value in specification.
    #[error("invalid type '{value}' for variable '{variable}'")]
    InvalidType {
        /// Variable name
        variable: String,
        /// Invalid type value
        value: String,
    },

    /// Invalid length value in specification.
    #[error("invalid length '{value}' for variable '{variable}'")]
    InvalidLength {
        /// Variable name
        variable: String,
        /// Invalid length value
        value: String,
    },

    /// Invalid order value in specification.
    #[error("invalid order '{value}' for variable '{variable}'")]
    InvalidOrder {
        /// Variable name
        variable: String,
        /// Invalid order value
        value: String,
    },

    /// Generic specification error.
    #[error("specification error: {message}")]
    Other {
        /// Error message
        message: String,
    },
}

impl SpecError {
    /// Create a load failed error.
    #[must_use]
    pub fn load_failed(message: impl Into<String>) -> Self {
        Self::LoadFailed {
            message: message.into(),
        }
    }

    /// Create a missing column error.
    #[must_use]
    pub fn missing_column(column: impl Into<String>) -> Self {
        Self::MissingColumn {
            column: column.into(),
        }
    }

    /// Create a dataset not found error.
    #[must_use]
    pub fn dataset_not_found(dataset: impl Into<String>) -> Self {
        Self::DatasetNotFound {
            dataset: dataset.into(),
        }
    }

    /// Create a duplicate variable error.
    #[must_use]
    pub fn duplicate_variable(dataset: impl Into<String>, variable: impl Into<String>) -> Self {
        Self::DuplicateVariable {
            dataset: dataset.into(),
            variable: variable.into(),
        }
    }

    /// Create an invalid type error.
    #[must_use]
    pub fn invalid_type(variable: impl Into<String>, value: impl Into<String>) -> Self {
        Self::InvalidType {
            variable: variable.into(),
            value: value.into(),
        }
    }

    /// Create an invalid length error.
    #[must_use]
    pub fn invalid_length(variable: impl Into<String>, value: impl Into<String>) -> Self {
        Self::InvalidLength {
            variable: variable.into(),
            value: value.into(),
        }
    }

    /// Create an invalid order error.
    #[must_use]
    pub fn invalid_order(variable: impl Into<String>, value: impl Into<String>) -> Self {
        Self::InvalidOrder {
            variable: variable.into(),
            value: value.into(),
        }
    }

    /// Create a generic specification error.
    #[must_use]
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other {
            message: message.into(),
        }
    }
}

/// Result type for specification operations.
pub type SpecResult<T> = std::result::Result<T, SpecError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_error_missing_column() {
        let err = SpecError::missing_column("variable");
        assert!(err.to_string().contains("variable"));
        assert!(err.to_string().contains("missing"));
    }

    #[test]
    fn test_spec_error_dataset_not_found() {
        let err = SpecError::dataset_not_found("DM");
        assert!(err.to_string().contains("DM"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_spec_error_duplicate_variable() {
        let err = SpecError::duplicate_variable("DM", "USUBJID");
        assert!(err.to_string().contains("DM"));
        assert!(err.to_string().contains("USUBJID"));
        assert!(err.to_string().contains("duplicate"));
    }

    #[test]
    fn test_spec_error_invalid_type() {
        let err = SpecError::invalid_type("AGE", "text");
        assert!(err.to_string().contains("AGE"));
        assert!(err.to_string().contains("text"));
    }
}
