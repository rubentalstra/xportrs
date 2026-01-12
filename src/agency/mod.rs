//! Regulatory agency definitions for compliance validation.
//!
//! This module provides the [`Agency`] enum for specifying regulatory
//! requirements when writing XPT files. Each agency has specific [`Rule`] items
//! that produce [`Issue`] items for variable names, labels, and data formatting.
//!
//! # Usage
//!
//! ```no_run
//! use xportrs::{Xpt, Agency, Dataset};
//!
//! # let dataset = Dataset::new("AE", vec![]).unwrap();
//! // Write with FDA validation
//! let mut builder = Xpt::writer(dataset);
//! builder.agency(Agency::FDA);
//! builder.finalize()?.write_path("ae.xpt")?;
//! # Ok::<(), xportrs::Error>(())
//! ```
//!
//! When no agency is specified, only structural XPT v5 validation is applied.

mod rules;

use std::path::Path;

pub use rules::Rule;

use crate::schema::DatasetSchema;
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
/// use xportrs::{Xpt, Agency, Dataset};
///
/// # let dataset = Dataset::new("AE", vec![]).unwrap();
/// let mut builder = Xpt::writer(dataset);
/// builder.agency(Agency::FDA);
/// builder.finalize()?.write_path("ae.xpt")?;
/// # Ok::<(), xportrs::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
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
    /// Requirements:
    /// - XPT v5 format
    /// - Dataset/variable names: ASCII only, max 8 bytes
    /// - Labels: max 40 bytes, **Japanese characters allowed (UTF-8)**
    /// - Character values: max 200 bytes, **Japanese characters allowed**
    /// - Max file size: 5 GB (files should be split if larger)
    ///
    /// Note: Japanese characters are permitted in labels and values where
    /// translation would lose essential information. Data lengths for
    /// Japanese items may differ from ASCII equivalents.
    PMDA,

    /// China National Medical Products Administration.
    ///
    /// Requirements:
    /// - XPT v5 format
    /// - Dataset/variable names: ASCII only, max 8 bytes
    /// - Labels: max 40 bytes, **Chinese characters allowed (UTF-8)**
    /// - Character values: max 200 bytes, **Chinese characters allowed**
    /// - Supported encodings: UTF-8, GB2312, GBK, GB18030, Big5
    /// - Max file size: 5 GB (files should be split if larger)
    ///
    /// Note: Chinese language support is required for labels and certain
    /// data fields. Multi-byte characters count towards byte limits
    /// (1 Chinese character = up to 4 bytes in UTF-8).
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

    /// Returns the default [`XptVersion`] for this agency.
    ///
    /// All major agencies currently require [`XptVersion::V5`].
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
    ///
    /// - FDA: `true` (ASCII only)
    /// - PMDA: `false` (Japanese characters allowed)
    /// - NMPA: `false` (Chinese characters allowed)
    #[must_use]
    pub const fn requires_ascii_labels(self) -> bool {
        match self {
            Self::FDA => true,
            Self::PMDA | Self::NMPA => false,
        }
    }

    /// Returns whether ASCII-only character values are required.
    ///
    /// - FDA: `true` (ASCII only)
    /// - PMDA: `false` (Japanese characters allowed)
    /// - NMPA: `false` (Chinese characters allowed)
    #[must_use]
    pub const fn requires_ascii_values(self) -> bool {
        match self {
            Self::FDA => true,
            Self::PMDA | Self::NMPA => false,
        }
    }

    /// Returns whether dataset name must match the file stem.
    #[must_use]
    pub const fn requires_dataset_name_matches_file_stem(self) -> bool {
        true
    }

    /// Returns the validation [`Rule`] items for this agency.
    ///
    /// Rules are applied in order during validation. Each agency has specific
    /// encoding requirements:
    /// - FDA: ASCII-only for names, labels, and values
    /// - PMDA: ASCII for names, Japanese (UTF-8) allowed in labels/values
    /// - NMPA: ASCII for names, Chinese (UTF-8) allowed in labels/values
    #[must_use]
    pub fn rules(self) -> Vec<Rule> {
        let mut rules = vec![
            // All agencies require ASCII names (SAS XPT format limitation)
            Rule::RequireAsciiNames,
            // Byte length limits
            Rule::DatasetNameMaxBytes(self.max_dataset_name_bytes()),
            Rule::VariableNameMaxBytes(self.max_variable_name_bytes()),
            Rule::LabelMaxBytes(self.max_label_bytes()),
            Rule::CharacterValueMaxBytes(self.max_character_value_bytes()),
            // File naming and pattern rules
            Rule::DatasetNameMatchesFileStem,
            Rule::dataset_name_pattern(r"^[A-Z][A-Z0-9]{0,7}$"),
            Rule::variable_name_pattern(r"^[A-Z_][A-Z0-9_]{0,7}$"),
            Rule::MaxFileSizeGb(self.max_file_size_gb()),
        ];

        // Agency-specific encoding rules for labels and values
        if self.requires_ascii_labels() {
            rules.push(Rule::RequireAsciiLabels);
        }
        if self.requires_ascii_values() {
            rules.push(Rule::RequireAsciiCharacterValues);
        }

        rules
    }

    /// Validates a schema plan against this agency's requirements.
    ///
    /// Applies all [`Rule`] items for this agency and returns any [`Issue`] items found.
    #[must_use]
    pub(crate) fn validate(self, plan: &DatasetSchema, file_path: Option<&Path>) -> Vec<Issue> {
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
    use crate::schema::plan::VariableSpec;
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
        let mut plan = DatasetSchema::new("AE");
        plan.variables = vec![
            VariableSpec::numeric("AESEQ"),
            VariableSpec::character("USUBJID", 20),
        ];
        plan.recalculate_positions();

        let issues = Agency::FDA.validate(&plan, None);
        assert!(!issues.iter().any(|i| i.severity() == Severity::Error));
    }

    #[test]
    fn test_agency_validation_non_ascii() {
        let mut plan = DatasetSchema::new("AÉ");
        plan.variables = vec![VariableSpec::numeric("AESEQ")];
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

    #[test]
    fn test_agency_specific_ascii_requirements() {
        // FDA requires ASCII for everything
        assert!(Agency::FDA.requires_ascii_names());
        assert!(Agency::FDA.requires_ascii_labels());
        assert!(Agency::FDA.requires_ascii_values());

        // PMDA allows non-ASCII in labels/values
        assert!(Agency::PMDA.requires_ascii_names());
        assert!(!Agency::PMDA.requires_ascii_labels());
        assert!(!Agency::PMDA.requires_ascii_values());

        // NMPA allows non-ASCII in labels/values
        assert!(Agency::NMPA.requires_ascii_names());
        assert!(!Agency::NMPA.requires_ascii_labels());
        assert!(!Agency::NMPA.requires_ascii_values());
    }

    #[test]
    fn test_agency_rules_differ() {
        let fda_rules = Agency::FDA.rules();
        let pmda_rules = Agency::PMDA.rules();

        // FDA should have RequireAsciiLabels
        assert!(
            fda_rules
                .iter()
                .any(|r| matches!(r, Rule::RequireAsciiLabels))
        );

        // PMDA should NOT have RequireAsciiLabels
        assert!(
            !pmda_rules
                .iter()
                .any(|r| matches!(r, Rule::RequireAsciiLabels))
        );
    }

    #[test]
    fn test_pmda_allows_japanese_labels() {
        let mut plan = DatasetSchema::new("AE");
        plan.dataset_label = Some("有害事象".to_string()); // Japanese
        plan.variables = vec![VariableSpec::numeric("AESEQ")];
        plan.recalculate_positions();

        let issues = Agency::PMDA.validate(&plan, None);
        // Should NOT have any errors (warnings are OK)
        assert!(!issues.iter().any(|i| i.severity() == Severity::Error));
    }

    #[test]
    fn test_nmpa_allows_chinese_labels() {
        let mut plan = DatasetSchema::new("AE");
        plan.dataset_label = Some("不良事件".to_string()); // Chinese
        plan.variables = vec![VariableSpec::numeric("AESEQ")];
        plan.recalculate_positions();

        let issues = Agency::NMPA.validate(&plan, None);
        // Should NOT have any errors (warnings are OK)
        assert!(!issues.iter().any(|i| i.severity() == Severity::Error));
    }

    #[test]
    fn test_fda_rejects_non_ascii_labels() {
        let mut plan = DatasetSchema::new("AE");
        plan.dataset_label = Some("Événements".to_string()); // French accented
        plan.variables = vec![VariableSpec::numeric("AESEQ")];
        plan.recalculate_positions();

        let issues = Agency::FDA.validate(&plan, None);
        // Should have NonAsciiDatasetLabel error
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, Issue::NonAsciiDatasetLabel { .. }))
        );
    }

    #[test]
    fn test_multibyte_label_warning() {
        let mut plan = DatasetSchema::new("AE");
        // Japanese label using 36 bytes (90% of 40-byte limit)
        // 12 Japanese characters * 3 bytes each = 36 bytes
        plan.dataset_label = Some("有害事象データセット詳細".to_string());
        plan.variables = vec![VariableSpec::numeric("AESEQ")];
        plan.recalculate_positions();

        let issues = Agency::PMDA.validate(&plan, None);
        // Should have a warning about approaching byte limit
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, Issue::MultiByteLabelNearLimit { .. }))
        );
    }
}
