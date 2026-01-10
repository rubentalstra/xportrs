//! Configuration structures.
//!
//! This module defines the configuration options used throughout xportrs.

use chrono::{DateTime, Utc};

/// Main configuration for xportrs operations.
///
/// This struct controls the behavior of reading and writing operations,
/// including strictness levels and verbosity.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Config {
    /// Whether to use strict checks (errors abort writes).
    pub strict_checks: bool,

    /// Whether to allow safe automatic fixes (e.g., name normalization).
    pub auto_fix: bool,

    /// Verbosity level for logging/diagnostics.
    pub verbosity: Verbosity,

    /// Options specific to writing operations.
    pub write: WriteOptions,

    /// Options specific to reading operations.
    pub read: ReadOptions,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            strict_checks: true,
            auto_fix: false,
            verbosity: Verbosity::Warn,
            write: WriteOptions::default(),
            read: ReadOptions::default(),
        }
    }
}

#[allow(dead_code)]
impl Config {
    /// Creates a new configuration with default settings.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Sets strict checks mode.
    #[must_use]
    pub(crate) fn with_strict_checks(mut self, strict: bool) -> Self {
        self.strict_checks = strict;
        self
    }

    /// Sets auto-fix mode.
    #[must_use]
    pub(crate) fn with_auto_fix(mut self, auto_fix: bool) -> Self {
        self.auto_fix = auto_fix;
        self
    }

    /// Sets the verbosity level.
    #[must_use]
    pub(crate) fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }
}

/// Verbosity level for diagnostics and logging.
///
/// # Example
///
/// ```
/// use xportrs::Verbosity;
///
/// // Warn is the default verbosity level
/// let level = Verbosity::default();
/// match level {
///     Verbosity::None => println!("Silent mode"),
///     Verbosity::Info => println!("Info messages enabled"),
///     Verbosity::Warn => println!("Warnings enabled"),
///     Verbosity::Error => println!("Errors only"),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Verbosity {
    /// No output.
    None,
    /// Informational messages only.
    Info,
    /// Warnings and above (default).
    #[default]
    Warn,
    /// Errors only.
    Error,
}

/// Options for writing XPT files.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct WriteOptions {
    /// Maximum file size in GB before splitting.
    ///
    /// If set, large datasets will be split into multiple files.
    pub max_size_gb: Option<f64>,

    /// Creation timestamp to write into the file header.
    ///
    /// If not set, the current time will be used.
    pub created: Option<DateTime<Utc>>,

    /// Modification timestamp to write into the file header.
    ///
    /// If not set, the current time will be used.
    pub modified: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
impl WriteOptions {
    /// Creates new write options with default settings.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum file size for splitting.
    #[must_use]
    pub(crate) fn with_max_size_gb(mut self, max_size: f64) -> Self {
        self.max_size_gb = Some(max_size);
        self
    }

    /// Sets the creation timestamp.
    #[must_use]
    pub(crate) fn with_created(mut self, created: DateTime<Utc>) -> Self {
        self.created = Some(created);
        self
    }

    /// Sets the modification timestamp.
    #[must_use]
    pub(crate) fn with_modified(mut self, modified: DateTime<Utc>) -> Self {
        self.modified = Some(modified);
        self
    }
}

/// Options for reading XPT files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReadOptions {
    /// Text decoding mode for character variables.
    pub text_mode: TextMode,

    /// Maximum number of rows to read.
    ///
    /// If `None`, all rows are read.
    pub row_limit: Option<usize>,

    /// Whether to preserve trailing blanks in character values.
    ///
    /// By default, trailing spaces are trimmed.
    pub preserve_blanks: bool,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            text_mode: TextMode::LossyUtf8,
            row_limit: None,
            preserve_blanks: false,
        }
    }
}

#[allow(dead_code)]
impl ReadOptions {
    /// Creates new read options with default settings.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Sets the text decoding mode.
    #[must_use]
    pub(crate) fn with_text_mode(mut self, mode: TextMode) -> Self {
        self.text_mode = mode;
        self
    }

    /// Sets a row limit.
    #[must_use]
    pub(crate) fn with_row_limit(mut self, limit: usize) -> Self {
        self.row_limit = Some(limit);
        self
    }

    /// Sets whether to preserve trailing blanks.
    #[must_use]
    pub(crate) fn with_preserve_blanks(mut self, preserve: bool) -> Self {
        self.preserve_blanks = preserve;
        self
    }
}

/// Text decoding mode for character variables.
///
/// XPT files can contain character data in various encodings.
/// This enum controls how that data is decoded into Rust strings.
///
/// # Variants
///
/// - [`TextMode::StrictUtf8`] - Errors on invalid UTF-8 sequences
/// - [`TextMode::LossyUtf8`] - Replaces invalid sequences with U+FFFD (default)
/// - [`TextMode::Latin1`] - Interprets bytes as ISO-8859-1 code points
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TextMode {
    /// Strict UTF-8: invalid sequences cause an error.
    StrictUtf8,

    /// Lossy UTF-8: invalid sequences are replaced with the replacement character (default).
    #[default]
    LossyUtf8,

    /// Latin-1 (ISO-8859-1): bytes are interpreted as Latin-1 code points.
    Latin1,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert!(config.strict_checks);
        assert!(!config.auto_fix);
        assert_eq!(config.verbosity, Verbosity::Warn);
    }

    #[test]
    fn test_read_options_builder() {
        let opts = ReadOptions::new()
            .with_text_mode(TextMode::StrictUtf8)
            .with_row_limit(100)
            .with_preserve_blanks(true);

        assert_eq!(opts.text_mode, TextMode::StrictUtf8);
        assert_eq!(opts.row_limit, Some(100));
        assert!(opts.preserve_blanks);
    }
}
