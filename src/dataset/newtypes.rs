//! Type-safe newtypes for domain-specific strings.
//!
//! This module provides newtypes that add type safety to common string fields
//! in the XPT/CDISC domain model.

use std::fmt;
use std::ops::Deref;

/// A domain code identifying a CDISC dataset.
///
/// Domain codes follow CDISC SDTM conventions and are typically 2-8 characters
/// (e.g., "AE", "DM", "LB", "SUPPAE").
///
/// # Example
///
/// ```
/// use xportrs::DomainCode;
///
/// let code = DomainCode::new("AE");
/// assert_eq!(code.as_str(), "AE");
///
/// // DomainCode derefs to &str for ergonomic use
/// assert!(code.len() == 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DomainCode(String);

impl DomainCode {
    /// Creates a new domain code.
    ///
    /// No validation is performed at construction time. Validation against
    /// XPT v5 or agency rules occurs during write plan finalization.
    #[must_use]
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    /// Returns the domain code as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the domain code and returns the inner string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for DomainCode {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for DomainCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DomainCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for DomainCode {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for DomainCode {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DomainCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DomainCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(DomainCode)
    }
}

/// A label for a dataset or variable.
///
/// Labels provide human-readable descriptions and are limited to 40 bytes
/// in XPT v5 format. Longer labels will be truncated during writing.
///
/// # Example
///
/// ```
/// use xportrs::Label;
///
/// let label = Label::new("Adverse Events");
/// assert_eq!(label.as_str(), "Adverse Events");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label(String);

impl Label {
    /// Creates a new label.
    ///
    /// No validation is performed at construction time. Validation against
    /// XPT v5 or agency rules occurs during write plan finalization.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self(label.into())
    }

    /// Returns the label as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the label and returns the inner string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for Label {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Label {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Label {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Label {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Label {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Label {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_code() {
        let code = DomainCode::new("AE");
        assert_eq!(code.as_str(), "AE");
        assert_eq!(code.len(), 2);
        assert_eq!(format!("{}", code), "AE");

        // From conversions
        let code2: DomainCode = "DM".into();
        assert_eq!(code2.as_str(), "DM");

        let code3: DomainCode = String::from("LB").into();
        assert_eq!(code3.as_str(), "LB");
    }

    #[test]
    fn test_label() {
        let label = Label::new("Adverse Events");
        assert_eq!(label.as_str(), "Adverse Events");
        assert_eq!(format!("{}", label), "Adverse Events");

        // From conversions
        let label2: Label = "Demographics".into();
        assert_eq!(label2.as_str(), "Demographics");
    }

    #[test]
    fn test_domain_code_equality() {
        let code1 = DomainCode::new("AE");
        let code2 = DomainCode::new("AE");
        let code3 = DomainCode::new("DM");

        assert_eq!(code1, code2);
        assert_ne!(code1, code3);
    }

    #[test]
    fn test_label_into_inner() {
        let label = Label::new("Test Label");
        let inner = label.into_inner();
        assert_eq!(inner, "Test Label");
    }
}
