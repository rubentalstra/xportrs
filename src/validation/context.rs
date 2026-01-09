//! Validation context for passing shared state to rules.
//!
//! The `ValidationContext` carries configuration that rules need during validation,
//! such as the XPT version (which determines limits) and the default action level.

use super::ActionLevel;
use crate::XptVersion;

/// Context passed to validation rules during execution.
///
/// Contains shared state that rules need, such as version limits
/// and the default action level for issues.
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// The XPT version being validated against.
    version: XptVersion,
    /// Default action level for issues.
    default_action: ActionLevel,
}

impl ValidationContext {
    /// Create a new validation context.
    #[must_use]
    pub fn new(version: XptVersion, default_action: ActionLevel) -> Self {
        Self {
            version,
            default_action,
        }
    }

    /// Get the XPT version.
    #[must_use]
    pub fn version(&self) -> XptVersion {
        self.version
    }

    /// Get the default action level.
    #[must_use]
    pub fn default_action(&self) -> ActionLevel {
        self.default_action
    }

    // === Version-specific limits ===

    /// Maximum length for dataset names.
    #[must_use]
    pub fn dataset_name_limit(&self) -> usize {
        self.version.dataset_name_limit()
    }

    /// Maximum length for variable names.
    #[must_use]
    pub fn variable_name_limit(&self) -> usize {
        self.version.variable_name_limit()
    }

    /// Maximum length for variable labels.
    #[must_use]
    pub fn variable_label_limit(&self) -> usize {
        self.version.variable_label_limit()
    }

    /// Maximum length for format names.
    #[must_use]
    pub fn format_name_limit(&self) -> usize {
        self.version.format_name_limit()
    }

    /// Dataset label limit (always 40 for both V5 and V8).
    #[must_use]
    pub fn dataset_label_limit(&self) -> usize {
        40
    }

    /// Check if the action level indicates the issue should be reported.
    #[must_use]
    pub fn should_report(&self) -> bool {
        self.default_action.should_report()
    }

    /// Check if the action level indicates processing should stop on issues.
    #[must_use]
    pub fn should_stop(&self) -> bool {
        self.default_action == ActionLevel::Stop
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new(XptVersion::default(), ActionLevel::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_v5_limits() {
        let ctx = ValidationContext::new(XptVersion::V5, ActionLevel::Warn);
        assert_eq!(ctx.dataset_name_limit(), 8);
        assert_eq!(ctx.variable_name_limit(), 8);
        assert_eq!(ctx.variable_label_limit(), 40);
        assert_eq!(ctx.format_name_limit(), 8);
    }

    #[test]
    fn test_context_v8_limits() {
        let ctx = ValidationContext::new(XptVersion::V8, ActionLevel::Warn);
        assert_eq!(ctx.dataset_name_limit(), 32);
        assert_eq!(ctx.variable_name_limit(), 32);
        assert_eq!(ctx.variable_label_limit(), 256);
        assert_eq!(ctx.format_name_limit(), 32);
    }

    #[test]
    fn test_default_action() {
        let ctx = ValidationContext::new(XptVersion::V5, ActionLevel::Stop);
        assert_eq!(ctx.default_action(), ActionLevel::Stop);
        assert!(ctx.should_stop());
    }

    #[test]
    fn test_should_report() {
        let ctx_warn = ValidationContext::new(XptVersion::V5, ActionLevel::Warn);
        assert!(ctx_warn.should_report());

        let ctx_none = ValidationContext::new(XptVersion::V5, ActionLevel::None);
        assert!(!ctx_none.should_report());
    }
}
