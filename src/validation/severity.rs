//! Action levels for transform operations.
//!
//! [`ActionLevel`] controls how transform operations respond to issues,
//! similar to R xportr's messaging system.

use std::fmt;
use std::str::FromStr;

use crate::error::Severity;

/// Action level for transform operations.
///
/// Controls how the system responds when a transform encounters an issue
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
    pub const fn to_severity(self) -> Severity {
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
    ///
    /// # Errors
    ///
    /// Returns `Err(())` if the string doesn't match any known action level.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_level_default() {
        assert_eq!(ActionLevel::default(), ActionLevel::Message);
    }

    #[test]
    fn test_should_continue() {
        assert!(ActionLevel::None.should_continue());
        assert!(ActionLevel::Message.should_continue());
        assert!(ActionLevel::Warn.should_continue());
        assert!(!ActionLevel::Stop.should_continue());
    }

    #[test]
    fn test_should_report() {
        assert!(!ActionLevel::None.should_report());
        assert!(ActionLevel::Message.should_report());
        assert!(ActionLevel::Warn.should_report());
        assert!(ActionLevel::Stop.should_report());
    }

    #[test]
    fn test_level_checks() {
        assert!(ActionLevel::None.is_silent());
        assert!(ActionLevel::Message.is_message());
        assert!(ActionLevel::Warn.is_warning());
        assert!(ActionLevel::Stop.is_error());
    }

    #[test]
    fn test_from_str() {
        assert_eq!("none".parse::<ActionLevel>(), Ok(ActionLevel::None));
        assert_eq!("NONE".parse::<ActionLevel>(), Ok(ActionLevel::None));
        assert_eq!("message".parse::<ActionLevel>(), Ok(ActionLevel::Message));
        assert_eq!("MSG".parse::<ActionLevel>(), Ok(ActionLevel::Message));
        assert_eq!("warn".parse::<ActionLevel>(), Ok(ActionLevel::Warn));
        assert_eq!("Warning".parse::<ActionLevel>(), Ok(ActionLevel::Warn));
        assert_eq!("stop".parse::<ActionLevel>(), Ok(ActionLevel::Stop));
        assert_eq!("error".parse::<ActionLevel>(), Ok(ActionLevel::Stop));
        assert_eq!("unknown".parse::<ActionLevel>(), Err(()));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", ActionLevel::None), "none");
        assert_eq!(format!("{}", ActionLevel::Message), "message");
        assert_eq!(format!("{}", ActionLevel::Warn), "warn");
        assert_eq!(format!("{}", ActionLevel::Stop), "stop");
    }

    #[test]
    fn test_to_severity() {
        use crate::error::Severity;

        assert_eq!(ActionLevel::Stop.to_severity(), Severity::Error);
        assert_eq!(ActionLevel::Warn.to_severity(), Severity::Warning);
        assert_eq!(ActionLevel::Message.to_severity(), Severity::Message);
        assert_eq!(ActionLevel::None.to_severity(), Severity::Message);
    }
}
