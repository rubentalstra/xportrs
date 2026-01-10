//! Validation issue types.
//!
//! This module defines the [`Issue`] enum for representing validation problems.
//! Each variant is a specific issue type with its own data.

use std::fmt;
use std::path::PathBuf;

/// A validation issue found during XPT generation or reading.
///
/// Each variant represents a specific type of issue with relevant context data.
/// The issue's code, severity, and message are derived from the variant.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Issue {
    // =========================================================================
    // XPT v5 Structural Issues
    // =========================================================================
    /// Dataset name exceeds maximum byte length.
    DatasetNameTooLong {
        /// The dataset name.
        dataset: String,
        /// Maximum allowed bytes.
        max: usize,
        /// Actual byte length.
        actual: usize,
    },

    /// Dataset label exceeds maximum byte length.
    DatasetLabelTooLong {
        /// The dataset name.
        dataset: String,
        /// Maximum allowed bytes.
        max: usize,
        /// Actual byte length.
        actual: usize,
    },

    /// Variable name exceeds maximum byte length.
    VariableNameTooLong {
        /// The variable name.
        variable: String,
        /// Maximum allowed bytes.
        max: usize,
        /// Actual byte length.
        actual: usize,
    },

    /// Variable label exceeds maximum byte length.
    VariableLabelTooLong {
        /// The variable name.
        variable: String,
        /// Maximum allowed bytes.
        max: usize,
        /// Actual byte length.
        actual: usize,
    },

    /// Numeric variable has incorrect length (must be 8).
    NumericWrongLength {
        /// The variable name.
        variable: String,
        /// Expected length (8).
        expected: usize,
        /// Actual length.
        actual: usize,
    },

    /// Character variable length is below minimum.
    CharacterLengthTooShort {
        /// The variable name.
        variable: String,
        /// Minimum allowed length.
        min: usize,
        /// Actual length.
        actual: usize,
    },

    /// Character variable length exceeds maximum.
    CharacterLengthTooLong {
        /// The variable name.
        variable: String,
        /// Maximum allowed length.
        max: usize,
        /// Actual length.
        actual: usize,
    },

    /// Row length is inconsistent with sum of variable lengths.
    RowLenInconsistent {
        /// Recorded row length.
        recorded: usize,
        /// Computed row length.
        computed: usize,
    },

    // =========================================================================
    // Agency-Specific Issues
    // =========================================================================
    /// Dataset name does not match required pattern.
    DatasetNamePatternMismatch {
        /// The dataset name.
        dataset: String,
        /// Agency name.
        agency: &'static str,
        /// Required pattern.
        pattern: String,
    },

    /// Variable name does not match required pattern.
    VariableNamePatternMismatch {
        /// The variable name.
        variable: String,
        /// Agency name.
        agency: &'static str,
        /// Required pattern.
        pattern: String,
    },

    /// Dataset name does not match file stem.
    DatasetNameFileStemMismatch {
        /// The dataset name.
        dataset: String,
        /// The file stem.
        stem: String,
    },

    /// Dataset name contains non-ASCII characters.
    NonAsciiDatasetName {
        /// The dataset name.
        dataset: String,
    },

    /// Variable name contains non-ASCII characters.
    NonAsciiVariableName {
        /// The variable name.
        variable: String,
    },

    /// Dataset label contains non-ASCII characters.
    NonAsciiDatasetLabel {
        /// The dataset name.
        dataset: String,
    },

    /// Variable label contains non-ASCII characters.
    NonAsciiVariableLabel {
        /// The variable name.
        variable: String,
    },

    /// Dataset name exceeds agency byte limit.
    AgencyDatasetNameTooLong {
        /// The dataset name.
        dataset: String,
        /// Maximum allowed bytes.
        max: usize,
        /// Actual byte length.
        actual: usize,
    },

    /// Variable name exceeds agency byte limit.
    AgencyVariableNameTooLong {
        /// The variable name.
        variable: String,
        /// Maximum allowed bytes.
        max: usize,
        /// Actual byte length.
        actual: usize,
    },

    /// Label exceeds agency byte limit.
    AgencyLabelTooLong {
        /// The name (dataset or variable).
        name: String,
        /// Whether this is a dataset (true) or variable (false).
        is_dataset: bool,
        /// Maximum allowed bytes.
        max: usize,
        /// Actual byte length.
        actual: usize,
    },

    /// Character value length exceeds agency policy limit.
    CharacterValueLengthExceeded {
        /// The variable name.
        variable: String,
        /// The variable's length.
        length: usize,
        /// Agency name.
        agency: &'static str,
        /// Maximum policy limit.
        max: usize,
    },
}

impl Issue {
    /// Returns the severity of this issue.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        match self {
            // Warnings
            Self::CharacterValueLengthExceeded { .. } => Severity::Warning,
            // Everything else is an error
            _ => Severity::Error,
        }
    }

    /// Returns the target of this issue (dataset, variable, or none).
    #[must_use]
    pub(crate) fn target(&self) -> Option<Target> {
        match self {
            // Dataset targets
            Self::DatasetNameTooLong { dataset, .. }
            | Self::DatasetLabelTooLong { dataset, .. }
            | Self::DatasetNamePatternMismatch { dataset, .. }
            | Self::DatasetNameFileStemMismatch { dataset, .. }
            | Self::NonAsciiDatasetName { dataset }
            | Self::NonAsciiDatasetLabel { dataset }
            | Self::AgencyDatasetNameTooLong { dataset, .. } => {
                Some(Target::Dataset(dataset.clone()))
            }

            // Variable targets
            Self::VariableNameTooLong { variable, .. }
            | Self::VariableLabelTooLong { variable, .. }
            | Self::NumericWrongLength { variable, .. }
            | Self::CharacterLengthTooShort { variable, .. }
            | Self::CharacterLengthTooLong { variable, .. }
            | Self::VariableNamePatternMismatch { variable, .. }
            | Self::NonAsciiVariableName { variable }
            | Self::NonAsciiVariableLabel { variable }
            | Self::AgencyVariableNameTooLong { variable, .. }
            | Self::CharacterValueLengthExceeded { variable, .. } => {
                Some(Target::Variable(variable.clone()))
            }

            // Special case for label (can be either)
            Self::AgencyLabelTooLong {
                name, is_dataset, ..
            } => {
                if *is_dataset {
                    Some(Target::Dataset(name.clone()))
                } else {
                    Some(Target::Variable(name.clone()))
                }
            }

            // No target
            Self::RowLenInconsistent { .. } => None,
        }
    }

    /// Returns `true` if this is an error.
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self.severity(), Severity::Error)
    }

    /// Returns `true` if this is a warning.
    #[must_use]
    pub const fn is_warning(&self) -> bool {
        matches!(self.severity(), Severity::Warning)
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format: [SEVERITY] message (target)
        write!(f, "[{}] ", self.severity())?;

        // Write the message based on variant
        match self {
            // Dataset name too long (both structural and agency)
            Self::DatasetNameTooLong {
                dataset,
                max,
                actual,
            }
            | Self::AgencyDatasetNameTooLong {
                dataset,
                max,
                actual,
            } => {
                write!(
                    f,
                    "dataset name '{}' exceeds {} bytes (has {} bytes)",
                    dataset, max, actual
                )?;
            }

            // Variable name too long (both structural and agency)
            Self::VariableNameTooLong {
                variable,
                max,
                actual,
            }
            | Self::AgencyVariableNameTooLong {
                variable,
                max,
                actual,
            } => {
                write!(
                    f,
                    "variable name '{}' exceeds {} bytes (has {} bytes)",
                    variable, max, actual
                )?;
            }

            Self::DatasetLabelTooLong { max, actual, .. } => {
                write!(
                    f,
                    "dataset label exceeds {} bytes (has {} bytes)",
                    max, actual
                )?;
            }

            Self::VariableLabelTooLong { max, actual, .. } => {
                write!(
                    f,
                    "variable label exceeds {} bytes (has {} bytes)",
                    max, actual
                )?;
            }

            Self::NumericWrongLength {
                variable,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "numeric variable '{}' must have length {} (has {})",
                    variable, expected, actual
                )?;
            }

            Self::CharacterLengthTooShort {
                variable,
                min,
                actual,
            } => {
                write!(
                    f,
                    "character variable '{}' must have length >= {} (has {})",
                    variable, min, actual
                )?;
            }

            Self::CharacterLengthTooLong {
                variable,
                max,
                actual,
            } => {
                write!(
                    f,
                    "character variable '{}' must have length <= {} (has {})",
                    variable, max, actual
                )?;
            }

            Self::RowLenInconsistent { recorded, computed } => {
                write!(
                    f,
                    "row_len inconsistency: recorded {} but computed {}",
                    recorded, computed
                )?;
            }

            Self::DatasetNamePatternMismatch {
                dataset,
                agency,
                pattern,
            } => {
                write!(
                    f,
                    "dataset name '{}' does not match {} required pattern '{}'",
                    dataset, agency, pattern
                )?;
            }

            Self::VariableNamePatternMismatch {
                variable,
                agency,
                pattern,
            } => {
                write!(
                    f,
                    "variable name '{}' does not match {} required pattern '{}'",
                    variable, agency, pattern
                )?;
            }

            Self::DatasetNameFileStemMismatch { dataset, stem } => {
                write!(
                    f,
                    "dataset name '{}' does not match file stem '{}'",
                    dataset, stem
                )?;
            }

            Self::NonAsciiDatasetName { dataset } => {
                write!(
                    f,
                    "dataset name '{}' contains non-ASCII characters",
                    dataset
                )?;
            }

            Self::NonAsciiVariableName { variable } => {
                write!(
                    f,
                    "variable name '{}' contains non-ASCII characters",
                    variable
                )?;
            }

            Self::NonAsciiDatasetLabel { .. } => {
                write!(f, "dataset label contains non-ASCII characters")?;
            }

            Self::NonAsciiVariableLabel { variable } => {
                write!(
                    f,
                    "variable '{}' label contains non-ASCII characters",
                    variable
                )?;
            }

            Self::AgencyLabelTooLong {
                name,
                is_dataset,
                max,
                actual,
            } => {
                let kind = if *is_dataset { "dataset" } else { "variable" };
                write!(
                    f,
                    "{} '{}' label exceeds {} bytes (has {} bytes)",
                    kind, name, max, actual
                )?;
            }
            Self::CharacterValueLengthExceeded {
                variable,
                length,
                agency,
                max,
            } => {
                write!(
                    f,
                    "character variable '{}' length {} exceeds {} policy limit of {} bytes",
                    variable, length, agency, max
                )?;
            }
        }

        // Append target if present
        if let Some(ref target) = self.target() {
            write!(f, " ({})", target)?;
        }

        Ok(())
    }
}

/// The severity level of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Severity {
    /// Informational message (does not block generation).
    Info,
    /// Warning (does not block generation, but indicates potential issues).
    Warning,
    /// Error (blocks generation in strict mode).
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// The target of a validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(dead_code)]
pub enum Target {
    /// A dataset by name.
    Dataset(String),
    /// A variable by name.
    Variable(String),
    /// A file by path.
    File(PathBuf),
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dataset(name) => write!(f, "dataset: {}", name),
            Self::Variable(name) => write!(f, "variable: {}", name),
            Self::File(path) => write!(f, "file: {}", path.display()),
        }
    }
}

/// Extension trait for working with collections of issues.
#[allow(dead_code)]
pub trait IssueCollection {
    /// Returns `true` if there are any errors.
    fn has_errors(&self) -> bool;

    /// Returns `true` if there are any warnings.
    fn has_warnings(&self) -> bool;

    /// Returns an iterator over error issues.
    fn errors(&self) -> impl Iterator<Item = &Issue>;

    /// Returns an iterator over warning issues.
    fn warnings(&self) -> impl Iterator<Item = &Issue>;
}

impl IssueCollection for [Issue] {
    fn has_errors(&self) -> bool {
        self.iter().any(Issue::is_error)
    }

    fn has_warnings(&self) -> bool {
        self.iter().any(Issue::is_warning)
    }

    fn errors(&self) -> impl Iterator<Item = &Issue> {
        self.iter().filter(|i| i.is_error())
    }

    fn warnings(&self) -> impl Iterator<Item = &Issue> {
        self.iter().filter(|i| i.is_warning())
    }
}

impl IssueCollection for Vec<Issue> {
    fn has_errors(&self) -> bool {
        self.as_slice().has_errors()
    }

    fn has_warnings(&self) -> bool {
        self.as_slice().has_warnings()
    }

    fn errors(&self) -> impl Iterator<Item = &Issue> {
        self.as_slice().errors()
    }

    fn warnings(&self) -> impl Iterator<Item = &Issue> {
        self.as_slice().warnings()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_display() {
        let issue = Issue::VariableNameTooLong {
            variable: "TOOLONGVARIABLENAME".into(),
            max: 8,
            actual: 19,
        };

        let display = format!("{}", issue);
        assert!(display.contains("ERROR"));
        assert!(display.contains("TOOLONGVARIABLENAME"));
        assert!(display.contains("exceeds 8 bytes"));
    }

    #[test]
    fn test_issue_collection() {
        let issues = vec![
            Issue::DatasetNameTooLong {
                dataset: "TOOLONG".into(),
                max: 8,
                actual: 10,
            },
            Issue::CharacterValueLengthExceeded {
                variable: "VAR".into(),
                length: 300,
                agency: "FDA",
                max: 200,
            },
        ];

        assert!(issues.has_errors());
        assert!(issues.has_warnings());
        assert_eq!(issues.errors().count(), 1);
        assert_eq!(issues.warnings().count(), 1);
    }
}
