//! Action levels for transform operations.
//!
//! [`ActionLevel`] controls how transform operations respond to issues,
//! similar to R xportr's messaging system.

// Re-export ActionLevel from config to unify the types
pub use crate::config::ActionLevel;

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
