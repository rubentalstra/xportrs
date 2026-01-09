//! Transform operation errors.
//!
//! [`TransformError`] represents errors that occur during metadata-driven
//! transform operations (type coercion, length application, etc.).

use thiserror::Error;

use crate::error::ValidationError;

/// Error type for transform operations.
///
/// These errors occur during the application of metadata specifications
/// to data, such as type coercion, length truncation, or format application.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TransformError {
    /// Type coercion failed for a variable.
    #[error("type coercion failed for variable '{variable}': {message}")]
    TypeCoercion {
        /// Variable name
        variable: String,
        /// Description of the coercion failure
        message: String,
    },

    /// Value was truncated during length application.
    #[error(
        "value truncated in variable '{variable}' at row {row}: \
         {original_length} chars > {max_length} max"
    )]
    Truncation {
        /// Variable name
        variable: String,
        /// Row index (0-based)
        row: usize,
        /// Original string length
        original_length: usize,
        /// Maximum allowed length
        max_length: usize,
    },

    /// Variable in data not found in specification.
    #[error("variable '{variable}' not found in specification")]
    VariableNotInSpec {
        /// Variable name
        variable: String,
    },

    /// Variable in specification not found in data.
    #[error("variable '{variable}' in specification not found in data")]
    VariableNotInData {
        /// Variable name
        variable: String,
    },

    /// Label exceeds maximum length.
    #[error("label for variable '{variable}' exceeds {max_length} character limit")]
    LabelTooLong {
        /// Variable name
        variable: String,
        /// Maximum allowed length
        max_length: usize,
        /// Actual label length
        actual_length: usize,
    },

    /// Format name is invalid.
    #[error("invalid format '{format}' for variable '{variable}'")]
    InvalidFormat {
        /// Variable name
        variable: String,
        /// Format string
        format: String,
    },

    /// Validation failed during transform.
    #[error("validation failed")]
    ValidationFailed(Vec<ValidationError>),

    /// File I/O error during transform.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// XPT format error during transform.
    #[error("XPT error: {0}")]
    Xpt(#[from] crate::XptError),

    /// Generic transform error.
    #[error("transform error: {message}")]
    Other {
        /// Error message
        message: String,
    },
}

impl TransformError {
    /// Create a type coercion error.
    #[must_use]
    pub fn type_coercion(variable: impl Into<String>, message: impl Into<String>) -> Self {
        Self::TypeCoercion {
            variable: variable.into(),
            message: message.into(),
        }
    }

    /// Create a truncation error.
    #[must_use]
    pub fn truncation(
        variable: impl Into<String>,
        row: usize,
        original_length: usize,
        max_length: usize,
    ) -> Self {
        Self::Truncation {
            variable: variable.into(),
            row,
            original_length,
            max_length,
        }
    }

    /// Create a "variable not in spec" error.
    #[must_use]
    pub fn variable_not_in_spec(variable: impl Into<String>) -> Self {
        Self::VariableNotInSpec {
            variable: variable.into(),
        }
    }

    /// Create a "variable not in data" error.
    #[must_use]
    pub fn variable_not_in_data(variable: impl Into<String>) -> Self {
        Self::VariableNotInData {
            variable: variable.into(),
        }
    }

    /// Create a label too long error.
    #[must_use]
    pub fn label_too_long(
        variable: impl Into<String>,
        max_length: usize,
        actual_length: usize,
    ) -> Self {
        Self::LabelTooLong {
            variable: variable.into(),
            max_length,
            actual_length,
        }
    }

    /// Create an invalid format error.
    #[must_use]
    pub fn invalid_format(variable: impl Into<String>, format: impl Into<String>) -> Self {
        Self::InvalidFormat {
            variable: variable.into(),
            format: format.into(),
        }
    }

    /// Create a validation failed error.
    #[must_use]
    pub fn validation_failed(errors: Vec<ValidationError>) -> Self {
        Self::ValidationFailed(errors)
    }

    /// Create a generic transform error.
    #[must_use]
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other {
            message: message.into(),
        }
    }
}

impl From<Vec<ValidationError>> for TransformError {
    fn from(errors: Vec<ValidationError>) -> Self {
        Self::ValidationFailed(errors)
    }
}

/// Result type for transform operations.
pub type TransformResult<T> = std::result::Result<T, TransformError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_error_type_coercion() {
        let err = TransformError::type_coercion("AGE", "cannot convert string to numeric");
        assert!(matches!(err, TransformError::TypeCoercion { .. }));
        assert!(err.to_string().contains("AGE"));
    }

    #[test]
    fn test_transform_error_truncation() {
        let err = TransformError::truncation("NAME", 5, 100, 40);
        assert!(matches!(err, TransformError::Truncation { .. }));
        assert!(err.to_string().contains("NAME"));
        assert!(err.to_string().contains("row 5"));
    }

    #[test]
    fn test_transform_error_variable_not_in_spec() {
        let err = TransformError::variable_not_in_spec("UNKNOWN");
        assert!(err.to_string().contains("UNKNOWN"));
        assert!(err.to_string().contains("not found in specification"));
    }

    #[test]
    fn test_transform_error_display() {
        let err = TransformError::label_too_long("VAR", 40, 100);
        let msg = err.to_string();
        assert!(msg.contains("VAR"));
        assert!(msg.contains("40"));
    }
}
