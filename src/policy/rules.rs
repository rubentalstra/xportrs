//! Policy rules and constraints.
//!
//! This module defines rules for file naming, encoding, and other
//! agency-specific constraints.

use std::fmt;
use std::path::Path;

/// File naming rules for XPT files.
///
/// Different agencies have different requirements for how XPT files
/// should be named. This struct captures those requirements.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FileNamingRules {
    /// Maximum filename length (excluding extension).
    pub max_filename_length: usize,

    /// Required file extension (lowercase, without dot).
    pub required_extension: String,

    /// Whether filenames must be lowercase.
    pub require_lowercase: bool,

    /// Whether filenames must be uppercase.
    pub require_uppercase: bool,

    /// Whether filenames must match dataset names.
    pub match_dataset_name: bool,

    /// Allowed characters in filename (regex pattern).
    pub allowed_chars_pattern: String,

    /// Whether spaces are allowed in filenames.
    pub allow_spaces: bool,

    /// Whether underscores are allowed in filenames.
    pub allow_underscores: bool,

    /// Whether hyphens are allowed in filenames.
    pub allow_hyphens: bool,
}

impl Default for FileNamingRules {
    fn default() -> Self {
        Self {
            max_filename_length: 8,
            required_extension: "xpt".to_string(),
            require_lowercase: true,
            require_uppercase: false,
            match_dataset_name: true,
            allowed_chars_pattern: r"^[a-z][a-z0-9]*$".to_string(),
            allow_spaces: false,
            allow_underscores: false,
            allow_hyphens: false,
        }
    }
}

impl FileNamingRules {
    /// Create FDA-compliant file naming rules.
    #[must_use]
    pub fn fda() -> Self {
        Self {
            max_filename_length: 8,
            required_extension: "xpt".to_string(),
            require_lowercase: true,
            require_uppercase: false,
            match_dataset_name: true,
            allowed_chars_pattern: r"^[a-z][a-z0-9]*$".to_string(),
            allow_spaces: false,
            allow_underscores: false,
            allow_hyphens: false,
        }
    }

    /// Create NMPA-compliant file naming rules.
    #[must_use]
    pub fn nmpa() -> Self {
        Self {
            max_filename_length: 8,
            required_extension: "xpt".to_string(),
            require_lowercase: true,
            require_uppercase: false,
            match_dataset_name: true,
            allowed_chars_pattern: r"^[a-z][a-z0-9]*$".to_string(),
            allow_spaces: false,
            allow_underscores: false,
            allow_hyphens: false,
        }
    }

    /// Create PMDA-compliant file naming rules.
    #[must_use]
    pub fn pmda() -> Self {
        Self {
            max_filename_length: 8,
            required_extension: "xpt".to_string(),
            require_lowercase: true,
            require_uppercase: false,
            match_dataset_name: true,
            allowed_chars_pattern: r"^[a-z][a-z0-9]*$".to_string(),
            allow_spaces: false,
            allow_underscores: false,
            allow_hyphens: false,
        }
    }

    /// Create permissive file naming rules (for V8 extended format).
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            max_filename_length: 32,
            required_extension: "xpt".to_string(),
            require_lowercase: false,
            require_uppercase: false,
            match_dataset_name: false,
            allowed_chars_pattern: r"^[a-zA-Z][a-zA-Z0-9_]*$".to_string(),
            allow_spaces: false,
            allow_underscores: true,
            allow_hyphens: false,
        }
    }

    /// Validate a filename against these rules.
    ///
    /// Returns a list of validation issues found.
    #[must_use]
    pub fn validate(&self, filename: &str) -> Vec<FileNamingIssue> {
        let mut issues = Vec::new();

        // Get the stem (filename without extension)
        let path = Path::new(filename);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        // Check extension
        if !extension.eq_ignore_ascii_case(&self.required_extension) {
            issues.push(FileNamingIssue::WrongExtension {
                expected: self.required_extension.clone(),
                found: extension.to_string(),
            });
        }

        // Check length
        if stem.len() > self.max_filename_length {
            issues.push(FileNamingIssue::TooLong {
                max: self.max_filename_length,
                actual: stem.len(),
            });
        }

        // Check case requirements
        if self.require_lowercase && stem.chars().any(|c| c.is_ascii_uppercase()) {
            issues.push(FileNamingIssue::NotLowercase);
        }
        if self.require_uppercase && stem.chars().any(|c| c.is_ascii_lowercase()) {
            issues.push(FileNamingIssue::NotUppercase);
        }

        // Check for spaces
        if !self.allow_spaces && stem.contains(' ') {
            issues.push(FileNamingIssue::ContainsSpaces);
        }

        // Check for underscores
        if !self.allow_underscores && stem.contains('_') {
            issues.push(FileNamingIssue::ContainsUnderscores);
        }

        // Check for hyphens
        if !self.allow_hyphens && stem.contains('-') {
            issues.push(FileNamingIssue::ContainsHyphens);
        }

        // Check for empty filename
        if stem.is_empty() {
            issues.push(FileNamingIssue::Empty);
        }

        // Check that filename starts with a letter
        if let Some(first) = stem.chars().next() {
            if !first.is_ascii_alphabetic() {
                issues.push(FileNamingIssue::DoesNotStartWithLetter);
            }
        }

        issues
    }

    /// Check if a filename is valid according to these rules.
    #[must_use]
    pub fn is_valid(&self, filename: &str) -> bool {
        self.validate(filename).is_empty()
    }
}

/// Issue found during file naming validation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum FileNamingIssue {
    /// Filename has wrong extension.
    WrongExtension {
        /// Expected extension
        expected: String,
        /// Found extension
        found: String,
    },

    /// Filename is too long.
    TooLong {
        /// Maximum allowed length
        max: usize,
        /// Actual length
        actual: usize,
    },

    /// Filename is not lowercase as required.
    NotLowercase,

    /// Filename is not uppercase as required.
    NotUppercase,

    /// Filename contains spaces.
    ContainsSpaces,

    /// Filename contains underscores.
    ContainsUnderscores,

    /// Filename contains hyphens.
    ContainsHyphens,

    /// Filename is empty.
    Empty,

    /// Filename does not start with a letter.
    DoesNotStartWithLetter,

    /// Filename does not match dataset name.
    DoesNotMatchDataset {
        /// Expected dataset name
        dataset: String,
    },
}

impl fmt::Display for FileNamingIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongExtension { expected, found } => {
                write!(
                    f,
                    "wrong extension: expected '.{expected}', found '.{found}'"
                )
            }
            Self::TooLong { max, actual } => {
                write!(f, "filename too long: {actual} chars (max {max})")
            }
            Self::NotLowercase => write!(f, "filename must be lowercase"),
            Self::NotUppercase => write!(f, "filename must be uppercase"),
            Self::ContainsSpaces => write!(f, "filename contains spaces"),
            Self::ContainsUnderscores => write!(f, "filename contains underscores"),
            Self::ContainsHyphens => write!(f, "filename contains hyphens"),
            Self::Empty => write!(f, "filename is empty"),
            Self::DoesNotStartWithLetter => write!(f, "filename must start with a letter"),
            Self::DoesNotMatchDataset { dataset } => {
                write!(f, "filename does not match dataset name '{dataset}'")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fda_rules_valid_filename() {
        let rules = FileNamingRules::fda();
        assert!(rules.is_valid("dm.xpt"));
        assert!(rules.is_valid("ae.xpt"));
        assert!(rules.is_valid("subject1.xpt"));
    }

    #[test]
    fn test_fda_rules_invalid_extension() {
        let rules = FileNamingRules::fda();
        let issues = rules.validate("dm.csv");
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, FileNamingIssue::WrongExtension { .. }))
        );
    }

    #[test]
    fn test_fda_rules_too_long() {
        let rules = FileNamingRules::fda();
        let issues = rules.validate("verylongname.xpt");
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, FileNamingIssue::TooLong { .. }))
        );
    }

    #[test]
    fn test_fda_rules_uppercase() {
        let rules = FileNamingRules::fda();
        let issues = rules.validate("DM.xpt");
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, FileNamingIssue::NotLowercase))
        );
    }

    #[test]
    fn test_fda_rules_spaces() {
        let rules = FileNamingRules::fda();
        let issues = rules.validate("d m.xpt");
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, FileNamingIssue::ContainsSpaces))
        );
    }

    #[test]
    fn test_fda_rules_starts_with_number() {
        let rules = FileNamingRules::fda();
        let issues = rules.validate("1dm.xpt");
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, FileNamingIssue::DoesNotStartWithLetter))
        );
    }

    #[test]
    fn test_permissive_rules() {
        let rules = FileNamingRules::permissive();
        assert!(rules.is_valid("DM.xpt"));
        assert!(rules.is_valid("dm.xpt"));
        assert!(rules.is_valid("dm_final.xpt"));
        assert!(rules.is_valid("veryLongDatasetName.xpt"));
    }

    #[test]
    fn test_default_rules() {
        let rules = FileNamingRules::default();
        assert_eq!(rules.max_filename_length, 8);
        assert_eq!(rules.required_extension, "xpt");
        assert!(rules.require_lowercase);
    }
}
