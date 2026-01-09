//! Unified configuration for xportrs operations.
//!
//! This module provides [`XportrsConfig`], a single configuration struct that
//! controls all aspects of xportrs operations including reading, writing,
//! transforms, and validation.
//!
//! The design is inspired by R's xportr package which uses global options
//! with function-level overrides.
//!
//! # Example
//!
//! ```
//! use xportrs::{XportrsConfig, ActionLevel, XptVersion};
//!
//! // Default config (V5, warn for all)
//! let config = XportrsConfig::default();
//!
//! // FDA-compliant config (V5, strict validation)
//! let config = XportrsConfig::fda();
//!
//! // Custom config with builder pattern
//! let config = XportrsConfig::default()
//!     .with_version(XptVersion::V5)
//!     .with_action(ActionLevel::Warn);
//! ```

use std::fmt;
use std::str::FromStr;

use chrono::NaiveDateTime;

// Re-export version from types (will be moved to internal later)
pub use crate::types::XptVersion;

/// Action level for transform and validation operations.
///
/// Controls how the system responds when an operation encounters an issue
/// (e.g., type mismatch, missing variable, truncation). This is equivalent
/// to R xportr's `.msg_type` parameter.
///
/// # Example
///
/// ```
/// use xportrs::ActionLevel;
///
/// let level = ActionLevel::Warn;
/// assert!(level.should_continue());
/// assert!(level.should_report());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActionLevel {
    /// Don't report the issue at all.
    ///
    /// The issue is silently ignored. Use sparingly.
    None,

    /// Report as informational message.
    ///
    /// The issue is logged but processing continues normally.
    /// This is the default for most operations.
    #[default]
    Message,

    /// Report as warning.
    ///
    /// The issue is logged as a warning and processing continues.
    /// Use when the issue may need attention but isn't blocking.
    Warn,

    /// Halt with error.
    ///
    /// The operation stops and returns an error.
    /// Use for issues that must be resolved before proceeding.
    Stop,
}

impl ActionLevel {
    /// Check if processing should continue at this level.
    ///
    /// Returns `true` for None, Message, and Warn.
    /// Returns `false` for Stop.
    #[must_use]
    pub const fn should_continue(&self) -> bool {
        !matches!(self, Self::Stop)
    }

    /// Check if this level should be reported (logged or returned).
    ///
    /// Returns `false` for None.
    /// Returns `true` for Message, Warn, and Stop.
    #[must_use]
    pub const fn should_report(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Check if this level is an error (Stop).
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Stop)
    }

    /// Check if this level is a warning (Warn).
    #[must_use]
    pub const fn is_warning(&self) -> bool {
        matches!(self, Self::Warn)
    }

    /// Check if this level is informational (Message).
    #[must_use]
    pub const fn is_message(&self) -> bool {
        matches!(self, Self::Message)
    }

    /// Check if this level is silent (None).
    #[must_use]
    pub const fn is_silent(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Convert to [`Severity`] for validation errors.
    ///
    /// Maps action levels to severity levels:
    /// - `Stop` → `Error`
    /// - `Warn` → `Warning`
    /// - `Message` → `Message`
    /// - `None` → `Message` (fallback, should check `should_report()` first)
    ///
    /// # Example
    ///
    /// ```
    /// use xportrs::{ActionLevel, Severity};
    ///
    /// assert_eq!(ActionLevel::Stop.to_severity(), Severity::Error);
    /// assert_eq!(ActionLevel::Warn.to_severity(), Severity::Warning);
    /// assert_eq!(ActionLevel::Message.to_severity(), Severity::Message);
    /// ```
    #[must_use]
    pub const fn to_severity(self) -> crate::error::Severity {
        use crate::error::Severity;
        match self {
            Self::Stop => Severity::Error,
            Self::Warn => Severity::Warning,
            Self::Message | Self::None => Severity::Message,
        }
    }
}

impl FromStr for ActionLevel {
    type Err = ();

    /// Parse from string (case-insensitive).
    ///
    /// Accepts: "none", "message", "msg", "warn", "warning", "stop", "error"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" | "silent" => Ok(Self::None),
            "message" | "msg" | "info" => Ok(Self::Message),
            "warn" | "warning" => Ok(Self::Warn),
            "stop" | "error" | "err" => Ok(Self::Stop),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ActionLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Message => write!(f, "message"),
            Self::Warn => write!(f, "warn"),
            Self::Stop => write!(f, "stop"),
        }
    }
}

/// Unified configuration for all xportrs operations.
///
/// This struct combines configuration for reading, writing, transforms,
/// and validation into a single, consistent interface. It follows the
/// R xportr package's design of having sensible defaults with easy
/// customization.
///
/// # Presets
///
/// - [`XportrsConfig::default()`] - V5 format, warn for all issues
/// - [`XportrsConfig::fda()`] - FDA-compliant, strict validation
/// - [`XportrsConfig::nmpa()`] - NMPA-compliant (China), strict validation
/// - [`XportrsConfig::pmda()`] - PMDA-compliant (Japan), strict validation
///
/// # Example
///
/// ```
/// use xportrs::{XportrsConfig, ActionLevel, XptVersion};
///
/// let config = XportrsConfig::default()
///     .with_version(XptVersion::V5)
///     .with_type_action(ActionLevel::Stop)
///     .with_length_action(ActionLevel::Warn);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XportrsConfig {
    // === XPT Format Options ===
    /// XPT format version (default: V5 for FDA compliance).
    pub version: XptVersion,

    /// Trim trailing spaces from character values when reading (default: true).
    pub trim_strings: bool,

    // === Transform Action Levels ===
    /// Action level for type mismatches (xportrs_type).
    pub type_action: ActionLevel,

    /// Action level for length mismatches (xportrs_length).
    pub length_action: ActionLevel,

    /// Action level for label mismatches (xportrs_label).
    pub label_action: ActionLevel,

    /// Action level for order mismatches (xportrs_order).
    pub order_action: ActionLevel,

    /// Action level for format mismatches (xportrs_format).
    pub format_action: ActionLevel,

    // === Transform Toggles (for xportrs() pipeline) ===
    /// Apply type coercion in pipeline (default: true).
    pub apply_type: bool,

    /// Apply length adjustment in pipeline (default: true).
    pub apply_length: bool,

    /// Apply label assignment in pipeline (default: true).
    pub apply_label: bool,

    /// Apply column ordering in pipeline (default: true).
    pub apply_order: bool,

    /// Apply format assignment in pipeline (default: true).
    pub apply_format: bool,

    // === Writer Options ===
    /// SAS version string (max 8 chars, default: "9.4").
    pub sas_version: String,

    /// Operating system name (max 8 chars, default: "RUST").
    pub os_name: String,

    /// File creation datetime (default: current time when writing).
    pub created: Option<NaiveDateTime>,

    /// File modification datetime (default: same as created).
    pub modified: Option<NaiveDateTime>,
}

impl Default for XportrsConfig {
    fn default() -> Self {
        Self {
            // Format options
            version: XptVersion::V5,
            trim_strings: true,

            // Transform action levels (default: warn)
            type_action: ActionLevel::Warn,
            length_action: ActionLevel::Warn,
            label_action: ActionLevel::Warn,
            order_action: ActionLevel::Warn,
            format_action: ActionLevel::Warn,

            // Transform toggles (default: all enabled)
            apply_type: true,
            apply_length: true,
            apply_label: true,
            apply_order: true,
            apply_format: true,

            // Writer options
            sas_version: "9.4".to_string(),
            os_name: "RUST".to_string(),
            created: None,
            modified: None,
        }
    }
}

impl XportrsConfig {
    /// Create a new configuration with default settings.
    ///
    /// Defaults to V5 format with warn-level action for all transforms.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an FDA-compliant configuration with strict validation.
    ///
    /// - Uses V5 format (required by FDA)
    /// - Stops on any validation issue
    ///
    /// Use this when preparing files for regulatory submission.
    #[must_use]
    pub fn fda() -> Self {
        Self {
            version: XptVersion::V5,
            type_action: ActionLevel::Stop,
            length_action: ActionLevel::Stop,
            label_action: ActionLevel::Stop,
            order_action: ActionLevel::Stop,
            format_action: ActionLevel::Stop,
            ..Self::default()
        }
    }

    /// Create an NMPA-compliant configuration (China).
    ///
    /// - Uses V5 format
    /// - Strict validation for regulatory submission
    #[must_use]
    pub fn nmpa() -> Self {
        Self {
            version: XptVersion::V5,
            type_action: ActionLevel::Stop,
            length_action: ActionLevel::Stop,
            label_action: ActionLevel::Stop,
            order_action: ActionLevel::Warn,
            format_action: ActionLevel::Warn,
            ..Self::default()
        }
    }

    /// Create a PMDA-compliant configuration (Japan).
    ///
    /// - Uses V5 format
    /// - Strict validation for regulatory submission
    #[must_use]
    pub fn pmda() -> Self {
        Self {
            version: XptVersion::V5,
            type_action: ActionLevel::Stop,
            length_action: ActionLevel::Stop,
            label_action: ActionLevel::Warn,
            order_action: ActionLevel::Warn,
            format_action: ActionLevel::Warn,
            ..Self::default()
        }
    }

    // === Version Options ===

    /// Set the XPT format version.
    #[must_use]
    pub fn with_version(mut self, version: XptVersion) -> Self {
        self.version = version;
        self
    }

    /// Use V8 format (extended names and labels).
    ///
    /// Note: V8 is not accepted for FDA submissions.
    #[must_use]
    pub fn v8(mut self) -> Self {
        self.version = XptVersion::V8;
        self
    }

    // === Global Action Level ===

    /// Set the action level for all transforms.
    ///
    /// This sets type_action, length_action, label_action, order_action,
    /// and format_action to the same value.
    #[must_use]
    pub fn with_action(mut self, action: ActionLevel) -> Self {
        self.type_action = action;
        self.length_action = action;
        self.label_action = action;
        self.order_action = action;
        self.format_action = action;
        self
    }

    // === Individual Action Levels ===

    /// Set the action level for type coercion.
    #[must_use]
    pub fn with_type_action(mut self, action: ActionLevel) -> Self {
        self.type_action = action;
        self
    }

    /// Set the action level for length adjustment.
    #[must_use]
    pub fn with_length_action(mut self, action: ActionLevel) -> Self {
        self.length_action = action;
        self
    }

    /// Set the action level for label assignment.
    #[must_use]
    pub fn with_label_action(mut self, action: ActionLevel) -> Self {
        self.label_action = action;
        self
    }

    /// Set the action level for column ordering.
    #[must_use]
    pub fn with_order_action(mut self, action: ActionLevel) -> Self {
        self.order_action = action;
        self
    }

    /// Set the action level for format assignment.
    #[must_use]
    pub fn with_format_action(mut self, action: ActionLevel) -> Self {
        self.format_action = action;
        self
    }

    // === Transform Toggles ===

    /// Enable or disable type coercion in the pipeline.
    #[must_use]
    pub fn with_apply_type(mut self, apply: bool) -> Self {
        self.apply_type = apply;
        self
    }

    /// Enable or disable length adjustment in the pipeline.
    #[must_use]
    pub fn with_apply_length(mut self, apply: bool) -> Self {
        self.apply_length = apply;
        self
    }

    /// Enable or disable label assignment in the pipeline.
    #[must_use]
    pub fn with_apply_label(mut self, apply: bool) -> Self {
        self.apply_label = apply;
        self
    }

    /// Enable or disable column ordering in the pipeline.
    #[must_use]
    pub fn with_apply_order(mut self, apply: bool) -> Self {
        self.apply_order = apply;
        self
    }

    /// Enable or disable format assignment in the pipeline.
    #[must_use]
    pub fn with_apply_format(mut self, apply: bool) -> Self {
        self.apply_format = apply;
        self
    }

    // === Writer Options ===

    /// Set the SAS version string (max 8 characters).
    #[must_use]
    pub fn with_sas_version(mut self, version: impl Into<String>) -> Self {
        let mut v = version.into();
        v.truncate(8);
        self.sas_version = v;
        self
    }

    /// Set the operating system name (max 8 characters).
    #[must_use]
    pub fn with_os_name(mut self, os: impl Into<String>) -> Self {
        let mut o = os.into();
        o.truncate(8);
        self.os_name = o;
        self
    }

    /// Set the file creation datetime.
    #[must_use]
    pub fn with_created(mut self, datetime: NaiveDateTime) -> Self {
        self.created = Some(datetime);
        self
    }

    /// Set the file modification datetime.
    #[must_use]
    pub fn with_modified(mut self, datetime: NaiveDateTime) -> Self {
        self.modified = Some(datetime);
        self
    }

    /// Set whether to trim trailing spaces from character values.
    #[must_use]
    pub fn with_trim_strings(mut self, trim: bool) -> Self {
        self.trim_strings = trim;
        self
    }

    // === Helper Methods ===

    /// Get the created datetime (current time if not set).
    #[must_use]
    pub fn get_created(&self) -> NaiveDateTime {
        self.created
            .unwrap_or_else(|| chrono::Local::now().naive_local())
    }

    /// Get the modified datetime (created time if not set).
    #[must_use]
    pub fn get_modified(&self) -> NaiveDateTime {
        self.modified.unwrap_or_else(|| self.get_created())
    }

    /// Check if this configuration is FDA-compliant.
    ///
    /// Returns `true` if using V5 format.
    #[must_use]
    pub fn is_fda_compliant(&self) -> bool {
        self.version.is_fda_compliant()
    }

    /// Check if any action level is set to Stop.
    #[must_use]
    pub fn has_strict_validation(&self) -> bool {
        self.type_action.is_error()
            || self.length_action.is_error()
            || self.label_action.is_error()
            || self.order_action.is_error()
            || self.format_action.is_error()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = XportrsConfig::default();
        assert_eq!(config.version, XptVersion::V5);
        assert_eq!(config.type_action, ActionLevel::Warn);
        assert!(config.apply_type);
        assert!(config.is_fda_compliant());
    }

    #[test]
    fn test_fda_config() {
        let config = XportrsConfig::fda();
        assert_eq!(config.version, XptVersion::V5);
        assert_eq!(config.type_action, ActionLevel::Stop);
        assert!(config.has_strict_validation());
    }

    #[test]
    fn test_builder_pattern() {
        let config = XportrsConfig::default()
            .with_version(XptVersion::V8)
            .with_type_action(ActionLevel::Stop)
            .with_apply_type(false);

        assert_eq!(config.version, XptVersion::V8);
        assert_eq!(config.type_action, ActionLevel::Stop);
        assert!(!config.apply_type);
    }

    #[test]
    fn test_with_action_sets_all() {
        let config = XportrsConfig::default().with_action(ActionLevel::Stop);

        assert_eq!(config.type_action, ActionLevel::Stop);
        assert_eq!(config.length_action, ActionLevel::Stop);
        assert_eq!(config.label_action, ActionLevel::Stop);
        assert_eq!(config.order_action, ActionLevel::Stop);
        assert_eq!(config.format_action, ActionLevel::Stop);
    }

    #[test]
    fn test_action_level_default() {
        assert_eq!(ActionLevel::default(), ActionLevel::Message);
    }

    #[test]
    fn test_action_level_should_continue() {
        assert!(ActionLevel::None.should_continue());
        assert!(ActionLevel::Message.should_continue());
        assert!(ActionLevel::Warn.should_continue());
        assert!(!ActionLevel::Stop.should_continue());
    }

    #[test]
    fn test_action_level_should_report() {
        assert!(!ActionLevel::None.should_report());
        assert!(ActionLevel::Message.should_report());
        assert!(ActionLevel::Warn.should_report());
        assert!(ActionLevel::Stop.should_report());
    }

    #[test]
    fn test_action_level_from_str() {
        assert_eq!("none".parse::<ActionLevel>(), Ok(ActionLevel::None));
        assert_eq!("warn".parse::<ActionLevel>(), Ok(ActionLevel::Warn));
        assert_eq!("stop".parse::<ActionLevel>(), Ok(ActionLevel::Stop));
        assert_eq!("error".parse::<ActionLevel>(), Ok(ActionLevel::Stop));
        assert!("invalid".parse::<ActionLevel>().is_err());
    }

    #[test]
    fn test_sas_version_truncation() {
        let config = XportrsConfig::default().with_sas_version("9.4.0.1.2.3");
        assert_eq!(config.sas_version.len(), 8);
    }
}
