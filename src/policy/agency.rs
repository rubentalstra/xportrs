//! Agency definitions and policy trait.
//!
//! This module defines the [`Agency`] enum for regulatory bodies and the
//! [`AgencyPolicy`] trait that defines agency-specific constraints.

use std::fmt;

use crate::XptVersion;

use super::rules::FileNamingRules;

/// Regulatory agency identifier.
///
/// Represents the regulatory bodies that have specific requirements
/// for XPT file submissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Agency {
    /// U.S. Food and Drug Administration.
    ///
    /// The FDA requires XPT V5 format for regulatory submissions.
    #[default]
    Fda,

    /// China National Medical Products Administration.
    ///
    /// NMPA has specific requirements including support for bilingual datasets.
    Nmpa,

    /// Japan Pharmaceuticals and Medical Devices Agency.
    ///
    /// PMDA has requirements for Japanese dataset support.
    Pmda,

    /// Custom agency or organization with user-defined rules.
    Custom,
}

impl Agency {
    /// Get the display name for this agency.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Fda => "FDA",
            Self::Nmpa => "NMPA",
            Self::Pmda => "PMDA",
            Self::Custom => "Custom",
        }
    }

    /// Get the full name for this agency.
    #[must_use]
    pub const fn full_name(&self) -> &'static str {
        match self {
            Self::Fda => "U.S. Food and Drug Administration",
            Self::Nmpa => "China National Medical Products Administration",
            Self::Pmda => "Japan Pharmaceuticals and Medical Devices Agency",
            Self::Custom => "Custom Policy",
        }
    }
}

impl fmt::Display for Agency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Trait defining agency-specific policy constraints.
///
/// Implementors of this trait define the rules and limits that apply
/// to XPT files for a specific regulatory agency. These policies are
/// used during validation to ensure compliance.
///
/// # Example
///
/// ```
/// use xportrs::policy::{Agency, AgencyPolicy, FdaPolicy};
/// use xportrs::XptVersion;
///
/// let policy = FdaPolicy::strict();
/// assert_eq!(policy.agency(), Agency::Fda);
/// assert_eq!(policy.required_version(), Some(XptVersion::V5));
/// assert!(policy.require_ascii());
/// ```
pub trait AgencyPolicy: Send + Sync + fmt::Debug {
    /// Get the agency this policy applies to.
    fn agency(&self) -> Agency;

    /// Get the required XPT version, if any.
    ///
    /// Returns `Some(version)` if the agency mandates a specific version,
    /// or `None` if any version is acceptable.
    fn required_version(&self) -> Option<XptVersion>;

    /// Get the maximum allowed file size in bytes, if any.
    ///
    /// Returns `Some(size)` if the agency has a file size limit,
    /// or `None` if there is no limit.
    fn max_file_size(&self) -> Option<u64>;

    /// Get the maximum variable name length.
    fn max_variable_name_length(&self) -> usize;

    /// Get the maximum dataset name length.
    fn max_dataset_name_length(&self) -> usize;

    /// Get the maximum variable label length.
    fn max_variable_label_length(&self) -> usize;

    /// Get the maximum dataset label length.
    fn max_dataset_label_length(&self) -> usize;

    /// Get the maximum format name length.
    fn max_format_name_length(&self) -> usize {
        self.max_variable_name_length()
    }

    /// Get the maximum informat name length.
    fn max_informat_name_length(&self) -> usize {
        self.max_format_name_length()
    }

    /// Whether variable and dataset names must be uppercase.
    fn require_uppercase_names(&self) -> bool;

    /// Whether only ASCII characters are allowed in string data.
    fn require_ascii(&self) -> bool;

    /// Get file naming rules for this agency.
    fn file_naming_rules(&self) -> FileNamingRules;

    /// Whether the policy is in strict mode.
    ///
    /// Strict mode treats warnings as errors.
    fn is_strict(&self) -> bool;

    /// Get a description of this policy.
    fn description(&self) -> String {
        format!(
            "{} Policy ({})",
            self.agency().name(),
            if self.is_strict() {
                "strict"
            } else {
                "lenient"
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agency_names() {
        assert_eq!(Agency::Fda.name(), "FDA");
        assert_eq!(Agency::Nmpa.name(), "NMPA");
        assert_eq!(Agency::Pmda.name(), "PMDA");
        assert_eq!(Agency::Custom.name(), "Custom");
    }

    #[test]
    fn test_agency_full_names() {
        assert!(Agency::Fda.full_name().contains("Food and Drug"));
        assert!(Agency::Nmpa.full_name().contains("China"));
        assert!(Agency::Pmda.full_name().contains("Japan"));
    }

    #[test]
    fn test_agency_display() {
        assert_eq!(format!("{}", Agency::Fda), "FDA");
        assert_eq!(format!("{}", Agency::Nmpa), "NMPA");
    }

    #[test]
    fn test_agency_default() {
        assert_eq!(Agency::default(), Agency::Fda);
    }
}
