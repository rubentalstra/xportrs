//! Regulatory agency definitions for compliance validation.
//!
//! This module provides the [`Agency`] enum for specifying regulatory
//! requirements when writing XPT files. Each agency has specific validation
//! rules for variable names, labels, and data formatting.
//!
//! # Usage
//!
//! ```no_run
//! use xportrs::{Xpt, Agency, DomainDataset};
//!
//! # let dataset = DomainDataset::new("AE".into(), vec![]).unwrap();
//! // Write with FDA validation
//! Xpt::writer(dataset)
//!     .agency(Agency::FDA)
//!     .finalize()?
//!     .write_path("ae.xpt")?;
//! # Ok::<(), xportrs::XportrsError>(())
//! ```
//!
//! When no agency is specified, only structural XPT v5 validation is applied.

mod rules;

use std::path::Path;

pub use rules::Rule;

use crate::schema::SchemaPlan;
use crate::validate::Issue;
use crate::xpt::XptVersion;

/// Regulatory agency for XPT file compliance validation.
///
/// Each agency has specific requirements for clinical trial data submissions.
/// When specified, additional validation rules are applied beyond the basic
/// XPT v5 structural requirements.
///
/// # Agencies
///
/// - [`Agency::FDA`] - U.S. Food and Drug Administration
/// - [`Agency::PMDA`] - Japan Pharmaceuticals and Medical Devices Agency
/// - [`Agency::NMPA`] - China National Medical Products Administration
///
/// # Example
///
/// ```no_run
/// use xportrs::{Xpt, Agency, DomainDataset};
///
/// # let dataset = DomainDataset::new("AE".into(), vec![]).unwrap();
/// Xpt::writer(dataset)
///     .agency(Agency::FDA)
///     .finalize()?
///     .write_path("ae.xpt")?;
/// # Ok::<(), xportrs::XportrsError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Agency {
    /// U.S. Food and Drug Administration.
    ///
    /// Requirements:
    /// - XPT v5 format
    /// - ASCII-only names, labels, and character values
    /// - Dataset/variable names: max 8 bytes, uppercase alphanumeric
    /// - Labels: max 40 bytes
    /// - Character values: max 200 bytes (policy recommendation)
    /// - Max file size: 5 GB (files should be split if larger)
    FDA,

    /// Japan Pharmaceuticals and Medical Devices Agency.
    ///
    /// Requirements are largely aligned with FDA:
    /// - XPT v5 format
    /// - ASCII-only names, labels, and character values
    /// - Dataset/variable names: max 8 bytes
    /// - Labels: max 40 bytes
    /// - Character values: max 200 bytes
    PMDA,

    /// China National Medical Products Administration.
    ///
    /// Requirements are largely aligned with FDA:
    /// - XPT v5 format
    /// - ASCII-only names, labels, and character values
    /// - Dataset/variable names: max 8 bytes
    /// - Labels: max 40 bytes
    /// - Character values: max 200 bytes
    NMPA,
}

impl Agency {
    /// Returns the agency name as a string.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::FDA => "FDA",
            Self::PMDA => "PMDA",
            Self::NMPA => "NMPA",
        }
    }

    /// Returns the default XPT version for this agency.
    ///
    /// All major agencies currently require XPT v5.
    #[must_use]
    pub const fn xpt_version(self) -> XptVersion {
        XptVersion::V5
    }

    /// Returns the maximum dataset name length in bytes.
    #[must_use]
    pub const fn max_dataset_name_bytes(self) -> usize {
        8
    }

    /// Returns the maximum variable name length in bytes.
    #[must_use]
    pub const fn max_variable_name_bytes(self) -> usize {
        8
    }

    /// Returns the maximum label length in bytes.
    #[must_use]
    pub const fn max_label_bytes(self) -> usize {
        40
    }

    /// Returns the maximum character value length in bytes (policy limit).
    ///
    /// Note: XPT v5 structurally supports up to 200 bytes, but agencies
    /// may recommend shorter lengths for practical reasons.
    #[must_use]
    pub const fn max_character_value_bytes(self) -> usize {
        200
    }

    /// Returns the maximum file size in GB before splitting is recommended.
    ///
    /// All major agencies recommend splitting files larger than 5 GB.
    #[must_use]
    pub const fn max_file_size_gb(self) -> f64 {
        5.0
    }

    /// Returns whether ASCII-only names are required.
    #[must_use]
    pub const fn requires_ascii_names(self) -> bool {
        true
    }

    /// Returns whether ASCII-only labels are required.
    #[must_use]
    pub const fn requires_ascii_labels(self) -> bool {
        true
    }

    /// Returns whether ASCII-only character values are required.
    #[must_use]
    pub const fn requires_ascii_values(self) -> bool {
        true
    }

    /// Returns whether dataset name must match the file stem.
    #[must_use]
    pub const fn requires_dataset_name_matches_file_stem(self) -> bool {
        true
    }

    /// Returns the validation rules for this agency.
    ///
    /// Rules are applied in order during validation.
    #[must_use]
    pub fn rules(self) -> Vec<Rule> {
        // All agencies currently share the same rules (aligned with FDA)
        vec![
            Rule::RequireAsciiNames,
            Rule::RequireAsciiLabels,
            Rule::RequireAsciiCharacterValues,
            Rule::DatasetNameMaxBytes(self.max_dataset_name_bytes()),
            Rule::VariableNameMaxBytes(self.max_variable_name_bytes()),
            Rule::LabelMaxBytes(self.max_label_bytes()),
            Rule::CharacterValueMaxBytes(self.max_character_value_bytes()),
            Rule::DatasetNameMatchesFileStem,
            Rule::dataset_name_pattern(r"^[A-Z][A-Z0-9]{0,7}$"),
            Rule::variable_name_pattern(r"^[A-Z_][A-Z0-9_]{0,7}$"),
            Rule::MaxFileSizeGb(self.max_file_size_gb()),
        ]
    }

    /// Validates a schema plan against this agency's requirements.
    ///
    /// Applies all rules for this agency and returns any issues found.
    #[must_use]
    pub fn validate(self, plan: &SchemaPlan, file_path: Option<&Path>) -> Vec<Issue> {
        let mut issues = Vec::new();
        let agency_name = self.name();

        for rule in self.rules() {
            issues.extend(rule.validate(plan, file_path, agency_name));
        }

        issues
    }
}

impl std::fmt::Display for Agency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::plan::PlannedVariable;
    use crate::validate::Severity;

    #[test]
    fn test_agency_properties() {
        assert_eq!(Agency::FDA.name(), "FDA");
        assert_eq!(Agency::FDA.max_dataset_name_bytes(), 8);
        assert_eq!(Agency::FDA.max_variable_name_bytes(), 8);
        assert_eq!(Agency::FDA.max_label_bytes(), 40);
        assert_eq!(Agency::FDA.max_character_value_bytes(), 200);
    }

    #[test]
    fn test_agency_rules() {
        let rules = Agency::FDA.rules();
        assert!(!rules.is_empty());

        // Check that key rules are present
        assert!(rules.iter().any(|r| matches!(r, Rule::RequireAsciiNames)));
        assert!(rules.iter().any(|r| matches!(r, Rule::LabelMaxBytes(40))));
    }

    #[test]
    fn test_agency_validation_valid() {
        let mut plan = SchemaPlan::new("AE".into());
        plan.variables = vec![
            PlannedVariable::numeric("AESEQ"),
            PlannedVariable::character("USUBJID", 20),
        ];
        plan.recalculate_positions();

        let issues = Agency::FDA.validate(&plan, None);
        assert!(!issues.iter().any(|i| i.severity() == Severity::Error));
    }

    #[test]
    fn test_agency_validation_non_ascii() {
        let mut plan = SchemaPlan::new("AÉ".into());
        plan.variables = vec![PlannedVariable::numeric("AESEQ")];
        plan.recalculate_positions();

        let issues = Agency::FDA.validate(&plan, None);
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, Issue::NonAsciiDatasetName { .. }))
        );
    }

    #[test]
    fn test_agency_display() {
        assert_eq!(format!("{}", Agency::FDA), "FDA");
        assert_eq!(format!("{}", Agency::PMDA), "PMDA");
        assert_eq!(format!("{}", Agency::NMPA), "NMPA");
    }
}
