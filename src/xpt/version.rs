//! XPT version definitions.
//!
//! This module defines the [`XptVersion`] enum for selecting the XPT format version.

/// The XPT format version.
///
/// Currently, only v5 is fully implemented. V8 is API-ready but not yet implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum XptVersion {
    /// XPT Version 5 (SAS Transport format).
    ///
    /// This is the traditional format used for CDISC submissions.
    /// - 8-byte variable names
    /// - 40-byte labels
    /// - IBM floating-point encoding
    #[default]
    V5,

    /// XPT Version 8 (extended format).
    ///
    /// **Not yet implemented.** This version supports:
    /// - Longer variable names (up to 32 bytes)
    /// - Longer labels (up to 256 bytes)
    /// - Different header structure
    V8,
}

impl XptVersion {
    /// Returns `true` if this is version 5.
    #[must_use]
    pub const fn is_v5(&self) -> bool {
        matches!(self, Self::V5)
    }

    /// Returns `true` if this is version 8.
    #[must_use]
    pub const fn is_v8(&self) -> bool {
        matches!(self, Self::V8)
    }

    /// Returns the maximum variable name length for this version.
    #[must_use]
    pub const fn max_variable_name_len(&self) -> usize {
        match self {
            Self::V5 => 8,
            Self::V8 => 32,
        }
    }

    /// Returns the maximum label length for this version.
    #[must_use]
    pub const fn max_label_len(&self) -> usize {
        match self {
            Self::V5 => 40,
            Self::V8 => 256,
        }
    }

    /// Returns `true` if this version is implemented.
    #[must_use]
    pub const fn is_implemented(&self) -> bool {
        matches!(self, Self::V5)
    }
}

impl std::fmt::Display for XptVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V5 => write!(f, "v5"),
            Self::V8 => write!(f, "v8"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_properties() {
        assert!(XptVersion::V5.is_v5());
        assert!(!XptVersion::V5.is_v8());
        assert!(XptVersion::V5.is_implemented());

        assert!(!XptVersion::V8.is_v5());
        assert!(XptVersion::V8.is_v8());
        assert!(!XptVersion::V8.is_implemented());
    }

    #[test]
    fn test_version_limits() {
        assert_eq!(XptVersion::V5.max_variable_name_len(), 8);
        assert_eq!(XptVersion::V5.max_label_len(), 40);
        assert_eq!(XptVersion::V8.max_variable_name_len(), 32);
        assert_eq!(XptVersion::V8.max_label_len(), 256);
    }
}
