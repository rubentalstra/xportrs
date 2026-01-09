//! Validation issue types.
//!
//! This module defines the [`Issue`] struct and related types for representing
//! validation problems.

use std::path::PathBuf;

/// A validation issue found during XPT generation or reading.
#[derive(Debug, Clone)]
pub struct Issue {
    /// The severity of the issue.
    pub severity: Severity,

    /// A unique code identifying the type of issue.
    pub code: &'static str,

    /// A human-readable description of the issue.
    pub message: String,

    /// The target of the issue (dataset, variable, or file).
    pub target: Option<Target>,
}

impl Issue {
    /// Creates a new issue.
    #[must_use]
    pub fn new(severity: Severity, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity,
            code,
            message: message.into(),
            target: None,
        }
    }

    /// Creates a new error issue.
    #[must_use]
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, code, message)
    }

    /// Creates a new warning issue.
    #[must_use]
    pub fn warning(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, code, message)
    }

    /// Creates a new info issue.
    #[must_use]
    pub fn info(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(Severity::Info, code, message)
    }

    /// Sets the target dataset.
    #[must_use]
    pub fn with_dataset(mut self, name: impl Into<String>) -> Self {
        self.target = Some(Target::Dataset(name.into()));
        self
    }

    /// Sets the target variable.
    #[must_use]
    pub fn with_variable(mut self, name: impl Into<String>) -> Self {
        self.target = Some(Target::Variable(name.into()));
        self
    }

    /// Sets the target file.
    #[must_use]
    pub fn with_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.target = Some(Target::File(path.into()));
        self
    }

    /// Returns `true` if this is an error.
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self.severity, Severity::Error)
    }

    /// Returns `true` if this is a warning.
    #[must_use]
    pub const fn is_warning(&self) -> bool {
        matches!(self.severity, Severity::Warning)
    }
}

impl std::fmt::Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.severity, self.code, self.message)?;
        if let Some(ref target) = self.target {
            write!(f, " ({})", target)?;
        }
        Ok(())
    }
}

/// The severity level of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Severity {
    /// Informational message (does not block generation).
    Info,
    /// Warning (does not block generation, but indicates potential issues).
    Warning,
    /// Error (blocks generation in strict mode).
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// The target of a validation issue.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Target {
    /// A dataset by name.
    Dataset(String),
    /// A variable by name.
    Variable(String),
    /// A file by path.
    File(PathBuf),
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dataset(name) => write!(f, "dataset: {}", name),
            Self::Variable(name) => write!(f, "variable: {}", name),
            Self::File(path) => write!(f, "file: {}", path.display()),
        }
    }
}

/// Extension trait for working with collections of issues.
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
        let issue =
            Issue::error("XPT001", "Variable name too long").with_variable("TOOLONGVARIABLENAME");

        let display = format!("{}", issue);
        assert!(display.contains("ERROR"));
        assert!(display.contains("XPT001"));
        assert!(display.contains("Variable name too long"));
        assert!(display.contains("TOOLONGVARIABLENAME"));
    }

    #[test]
    fn test_issue_collection() {
        let issues = vec![
            Issue::error("E001", "error"),
            Issue::warning("W001", "warning"),
            Issue::info("I001", "info"),
        ];

        assert!(issues.has_errors());
        assert!(issues.has_warnings());
        assert_eq!(issues.errors().count(), 1);
        assert_eq!(issues.warnings().count(), 1);
    }
}
