//! Transform configuration types.
//!
//! This module provides configuration structures for controlling the behavior
//! of transform operations, including how mismatches between data and
//! specifications are handled.

use crate::validation::ActionLevel;

/// How to handle mismatches between data and specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MismatchAction {
    /// Ignore variables not in spec (keep them as-is).
    #[default]
    Ignore,
    /// Include warning in report for variables not in spec.
    Warn,
    /// Return error for variables not in spec.
    Error,
    /// Remove variables not in spec from output.
    Remove,
}

/// Base configuration for all transform operations.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransformConfig {
    /// Action level for reporting (none/message/warn/stop).
    pub action: ActionLevel,

    /// How to handle variables in data that are not in spec.
    pub variable_not_in_spec: MismatchAction,

    /// How to handle variables in spec that are not in data.
    pub variable_not_in_data: MismatchAction,
}

impl Default for TransformConfig {
    fn default() -> Self {
        Self {
            action: ActionLevel::Warn,
            variable_not_in_spec: MismatchAction::Warn,
            variable_not_in_data: MismatchAction::Warn,
        }
    }
}

impl TransformConfig {
    /// Create a new config with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a strict config that errors on mismatches.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            action: ActionLevel::Stop,
            variable_not_in_spec: MismatchAction::Error,
            variable_not_in_data: MismatchAction::Error,
        }
    }

    /// Create a lenient config that ignores mismatches.
    #[must_use]
    pub fn lenient() -> Self {
        Self {
            action: ActionLevel::Message,
            variable_not_in_spec: MismatchAction::Ignore,
            variable_not_in_data: MismatchAction::Ignore,
        }
    }

    /// Set the action level.
    #[must_use]
    pub fn with_action(mut self, action: ActionLevel) -> Self {
        self.action = action;
        self
    }

    /// Set how to handle variables not in spec.
    #[must_use]
    pub fn with_variable_not_in_spec(mut self, action: MismatchAction) -> Self {
        self.variable_not_in_spec = action;
        self
    }

    /// Set how to handle variables not in data.
    #[must_use]
    pub fn with_variable_not_in_data(mut self, action: MismatchAction) -> Self {
        self.variable_not_in_data = action;
        self
    }

    /// Check if action level should stop on issues.
    #[must_use]
    pub fn should_stop(&self) -> bool {
        self.action == ActionLevel::Stop
    }

    /// Check if we should report messages.
    #[must_use]
    pub fn should_report(&self) -> bool {
        self.action != ActionLevel::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TransformConfig::default();
        assert_eq!(config.action, ActionLevel::Warn);
        assert_eq!(config.variable_not_in_spec, MismatchAction::Warn);
        assert_eq!(config.variable_not_in_data, MismatchAction::Warn);
    }

    #[test]
    fn test_strict_config() {
        let config = TransformConfig::strict();
        assert_eq!(config.action, ActionLevel::Stop);
        assert!(config.should_stop());
    }

    #[test]
    fn test_lenient_config() {
        let config = TransformConfig::lenient();
        assert_eq!(config.action, ActionLevel::Message);
        assert!(!config.should_stop());
    }

    #[test]
    fn test_builder_methods() {
        let config = TransformConfig::new()
            .with_action(ActionLevel::None)
            .with_variable_not_in_spec(MismatchAction::Remove);

        assert_eq!(config.action, ActionLevel::None);
        assert_eq!(config.variable_not_in_spec, MismatchAction::Remove);
        assert!(!config.should_report());
    }
}
