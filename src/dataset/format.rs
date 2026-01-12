//! SAS format and informat types for XPT files.
//!
//! This module provides the [`Format`] type for representing SAS formats and informats.
//! Formats control how numeric and character values are displayed, while informats
//! control how raw data is read into SAS variables.
//!
//! # CDISC/FDA Compliance
//!
//! According to SAS TS-140 specification, the NAMESTR record stores format metadata:
//! - `nform`: Format name (8 bytes)
//! - `nfl`: Format length (2 bytes)
//! - `nfd`: Format decimals (2 bytes)
//! - `nfj`: Format justification (2 bytes) - formats only, not informats
//!
//! The same structure applies to informats (`niform`, `nifl`, `nifd`), except
//! informats do not have a justification field.
//!
//! # Format String Syntax
//!
//! SAS format strings follow the pattern: `[name][width].[decimals]`
//!
//! Examples:
//! - `DATE9.` - Date format, 9 characters wide
//! - `8.2` - Numeric format, 8 wide with 2 decimals
//! - `BEST12.` - Best format, 12 characters wide
//! - `$CHAR200.` - Character format, 200 characters wide
//!
//! # Example
//!
//! ```
//! use xportrs::Format;
//!
//! // Parse from format string
//! let date_fmt = Format::parse("DATE9.").unwrap();
//! assert_eq!(date_fmt.name(), "DATE");
//! assert_eq!(date_fmt.length(), 9);
//! assert_eq!(date_fmt.decimals(), 0);
//!
//! // Create numeric format
//! let num_fmt = Format::numeric(8, 2);
//! assert_eq!(num_fmt.length(), 8);
//! assert_eq!(num_fmt.decimals(), 2);
//!
//! // Create character format
//! let char_fmt = Format::character(200);
//! assert_eq!(char_fmt.length(), 200);
//! ```

use std::fmt;
use std::str::FromStr;

/// Error type for format parsing failures.
///
/// This error is returned when a format string cannot be parsed according
/// to SAS format syntax rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatParseError {
    /// The invalid format string that caused the error.
    pub input: String,
    /// A description of what went wrong.
    pub message: String,
}

impl fmt::Display for FormatParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid format '{}': {}", self.input, self.message)
    }
}

impl std::error::Error for FormatParseError {}

/// Text justification for formatted output.
///
/// Controls how values are aligned within their display width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u16)]
pub enum Justification {
    /// Left-justify text (default for character variables).
    Left = 0,
    /// Right-justify text (default for numeric variables).
    #[default]
    Right = 1,
}

impl Justification {
    /// Creates a justification from an XPT `nfj` field value.
    #[must_use]
    pub fn from_nfj(value: i16) -> Self {
        if value == 0 { Self::Left } else { Self::Right }
    }

    /// Returns the XPT `nfj` field value.
    #[must_use]
    pub fn as_nfj(self) -> i16 {
        self as i16
    }
}

/// A SAS format or informat specification.
///
/// Formats control how values are displayed (output), while informats control
/// how raw data is read (input). Both use the same structure internally.
///
/// # Fields in NAMESTR
///
/// When written to XPT files, formats are stored in the NAMESTR record:
/// - `nform`/`niform`: Format/informat name (8 bytes, space-padded)
/// - `nfl`/`nifl`: Length (2-byte big-endian integer)
/// - `nfd`/`nifd`: Decimals (2-byte big-endian integer)
/// - `nfj`: Justification (2-byte big-endian integer) - formats only
///
/// # Example
///
/// ```
/// use xportrs::Format;
///
/// // Parse a date format
/// let fmt = Format::parse("DATE9.").unwrap();
/// assert_eq!(fmt.name(), "DATE");
/// assert_eq!(fmt.length(), 9);
///
/// // Create a numeric format with decimals
/// let num = Format::parse("8.2").unwrap();
/// assert_eq!(num.length(), 8);
/// assert_eq!(num.decimals(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Format {
    /// Format name (e.g., "DATE", "BEST", "CHAR").
    /// Empty string for bare numeric formats like "8.2".
    name: String,
    /// Display width in characters.
    length: u16,
    /// Number of decimal places (numeric formats only).
    decimals: u16,
    /// Text justification (formats only, ignored for informats).
    justification: Justification,
    /// Whether this is a character format (starts with $).
    is_character: bool,
}

impl Default for Format {
    fn default() -> Self {
        Self {
            name: String::new(),
            length: 8,
            decimals: 0,
            justification: Justification::Right,
            is_character: false,
        }
    }
}

impl Format {
    /// Creates a new format with explicit values.
    ///
    /// For most use cases, prefer [`Format::parse`], [`Format::numeric`],
    /// or [`Format::character`].
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        length: u16,
        decimals: u16,
        justification: Justification,
    ) -> Self {
        let name = name.into();
        let is_character = name.starts_with('$') || name.eq_ignore_ascii_case("CHAR");
        Self {
            name,
            length,
            decimals,
            justification,
            is_character,
        }
    }

    /// Creates a numeric format with the given width and decimal places.
    ///
    /// This creates a bare numeric format (no name) like "8.2".
    ///
    /// # Example
    ///
    /// ```
    /// use xportrs::Format;
    ///
    /// let fmt = Format::numeric(8, 2);
    /// assert_eq!(fmt.to_string(), "8.2");
    /// ```
    #[must_use]
    pub fn numeric(length: u16, decimals: u16) -> Self {
        Self {
            name: String::new(),
            length,
            decimals,
            justification: Justification::Right,
            is_character: false,
        }
    }

    /// Creates a character format with the given width.
    ///
    /// This creates a `$CHAR` format like "$CHAR200.".
    ///
    /// # Example
    ///
    /// ```
    /// use xportrs::Format;
    ///
    /// let fmt = Format::character(200);
    /// assert_eq!(fmt.to_string(), "$CHAR200.");
    /// ```
    #[must_use]
    pub fn character(length: u16) -> Self {
        Self {
            name: String::from("$CHAR"),
            length,
            decimals: 0,
            justification: Justification::Left,
            is_character: true,
        }
    }

    /// Creates a format from NAMESTR field values.
    ///
    /// This is used when reading XPT files to reconstruct the Format
    /// from the parsed NAMESTR record fields.
    ///
    /// # Arguments
    ///
    /// * `name` - The format name from `nform` or `niform` (already trimmed)
    /// * `length` - The format length from `nfl` or `nifl`
    /// * `decimals` - The format decimals from `nfd` or `nifd`
    /// * `justification` - The justification from `nfj` (use 0 for informats)
    #[must_use]
    pub fn from_namestr(name: &str, length: i16, decimals: i16, justification: i16) -> Self {
        let is_character = name.starts_with('$') || name.eq_ignore_ascii_case("CHAR");
        Self {
            name: name.to_string(),
            length: length.max(0) as u16,
            decimals: decimals.max(0) as u16,
            justification: Justification::from_nfj(justification),
            is_character,
        }
    }

    /// Parses a SAS format string.
    ///
    /// # Format String Syntax
    ///
    /// Format strings follow the pattern: `[prefix][name][width].[decimals]`
    ///
    /// - Optional `$` prefix for character formats
    /// - Optional format name (e.g., "DATE", "BEST", "CHAR")
    /// - Required width (integer)
    /// - Required `.` separator
    /// - Optional decimals after the dot
    ///
    /// # Examples
    ///
    /// ```
    /// use xportrs::Format;
    ///
    /// // Named formats
    /// assert!(Format::parse("DATE9.").is_ok());
    /// assert!(Format::parse("BEST12.").is_ok());
    /// assert!(Format::parse("$CHAR200.").is_ok());
    ///
    /// // Bare numeric formats
    /// assert!(Format::parse("8.").is_ok());
    /// assert!(Format::parse("8.2").is_ok());
    ///
    /// // Invalid formats
    /// assert!(Format::parse("").is_err());
    /// assert!(Format::parse("DATE").is_err()); // missing width and dot
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`FormatParseError`] if the format string is invalid.
    ///
    /// # Panics
    ///
    /// This function will not panic. The internal `.unwrap()` is safe because
    /// we verify the string contains a '.' before calling `rfind('.')`.
    pub fn parse(s: &str) -> Result<Self, FormatParseError> {
        let s = s.trim();

        if s.is_empty() {
            return Err(FormatParseError {
                input: s.to_string(),
                message: "empty format string".to_string(),
            });
        }

        // Must end with a dot or have decimals after the dot
        if !s.contains('.') {
            return Err(FormatParseError {
                input: s.to_string(),
                message: "format must contain a '.' separator".to_string(),
            });
        }

        // Check for character format prefix
        let (is_char, rest) = if let Some(stripped) = s.strip_prefix('$') {
            (true, stripped)
        } else {
            (false, s)
        };

        // Split at the dot
        let dot_pos = rest.rfind('.').unwrap(); // We know it exists
        let before_dot = &rest[..dot_pos];
        let after_dot = &rest[dot_pos + 1..];

        // Parse decimals (after dot, may be empty)
        let decimals: u16 = if after_dot.is_empty() {
            0
        } else {
            after_dot.parse().map_err(|_| FormatParseError {
                input: s.to_string(),
                message: format!("invalid decimals: '{}'", after_dot),
            })?
        };

        // Parse the part before the dot: [name][width]
        // Find where the name ends and the width begins
        let (name, length) = parse_name_and_width(before_dot, s)?;

        let full_name = if is_char { format!("${}", name) } else { name };

        Ok(Self {
            name: full_name.clone(),
            length,
            decimals,
            justification: if is_char {
                Justification::Left
            } else {
                Justification::Right
            },
            is_character: is_char,
        })
    }

    /// Returns the format name.
    ///
    /// For bare numeric formats like "8.2", this returns an empty string.
    /// For named formats like "DATE9.", this returns "DATE".
    /// For character formats like "$CHAR200.", this returns "$CHAR".
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the format name without the `$` prefix.
    ///
    /// For XPT NAMESTR records, the `$` prefix is implicit for character
    /// formats and should not be written to the 8-byte name field.
    #[must_use]
    pub fn name_without_prefix(&self) -> &str {
        self.name.strip_prefix('$').unwrap_or(&self.name)
    }

    /// Returns the display width in characters.
    #[must_use]
    pub fn length(&self) -> u16 {
        self.length
    }

    /// Returns the number of decimal places.
    #[must_use]
    pub fn decimals(&self) -> u16 {
        self.decimals
    }

    /// Returns the text justification.
    #[must_use]
    pub fn justification(&self) -> Justification {
        self.justification
    }

    /// Returns whether this is a character format.
    #[must_use]
    pub fn is_character(&self) -> bool {
        self.is_character
    }

    /// Sets the justification.
    #[must_use]
    pub fn with_justification(mut self, justification: Justification) -> Self {
        self.justification = justification;
        self
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.name.is_empty() {
            // Bare numeric format: "8.2" or "8."
            if self.decimals > 0 {
                write!(f, "{}.{}", self.length, self.decimals)
            } else {
                write!(f, "{}.", self.length)
            }
        } else {
            // Named format: "DATE9." or "$CHAR200."
            if self.decimals > 0 {
                write!(f, "{}{}.{}", self.name, self.length, self.decimals)
            } else {
                write!(f, "{}{}.", self.name, self.length)
            }
        }
    }
}

impl FromStr for Format {
    type Err = FormatParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Format {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Format {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Format::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// Parses the name and width from the part before the dot.
///
/// Handles cases like:
/// - "8" -> ("", 8)
/// - "DATE9" -> ("DATE", 9)
/// - "BEST12" -> ("BEST", 12)
/// - "CHAR200" -> ("CHAR", 200)
fn parse_name_and_width(
    before_dot: &str,
    original: &str,
) -> Result<(String, u16), FormatParseError> {
    if before_dot.is_empty() {
        return Err(FormatParseError {
            input: original.to_string(),
            message: "missing width before '.'".to_string(),
        });
    }

    // Find where digits start from the end
    let digit_start = before_dot
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_ascii_digit())
        .last()
        .map(|(i, _)| i)
        .unwrap_or(before_dot.len());

    let name_part = &before_dot[..digit_start];
    let width_part = &before_dot[digit_start..];

    if width_part.is_empty() {
        return Err(FormatParseError {
            input: original.to_string(),
            message: "missing width in format".to_string(),
        });
    }

    let width: u16 = width_part.parse().map_err(|_| FormatParseError {
        input: original.to_string(),
        message: format!("invalid width: '{}'", width_part),
    })?;

    Ok((name_part.to_uppercase(), width))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date_format() {
        let fmt = Format::parse("DATE9.").unwrap();
        assert_eq!(fmt.name(), "DATE");
        assert_eq!(fmt.length(), 9);
        assert_eq!(fmt.decimals(), 0);
        assert!(!fmt.is_character());
        assert_eq!(fmt.to_string(), "DATE9.");
    }

    #[test]
    fn test_parse_bare_numeric() {
        let fmt = Format::parse("8.2").unwrap();
        assert_eq!(fmt.name(), "");
        assert_eq!(fmt.length(), 8);
        assert_eq!(fmt.decimals(), 2);
        assert!(!fmt.is_character());
        assert_eq!(fmt.to_string(), "8.2");
    }

    #[test]
    fn test_parse_bare_numeric_no_decimals() {
        let fmt = Format::parse("8.").unwrap();
        assert_eq!(fmt.name(), "");
        assert_eq!(fmt.length(), 8);
        assert_eq!(fmt.decimals(), 0);
        assert_eq!(fmt.to_string(), "8.");
    }

    #[test]
    fn test_parse_best_format() {
        let fmt = Format::parse("BEST12.").unwrap();
        assert_eq!(fmt.name(), "BEST");
        assert_eq!(fmt.length(), 12);
        assert_eq!(fmt.decimals(), 0);
        assert_eq!(fmt.to_string(), "BEST12.");
    }

    #[test]
    fn test_parse_character_format() {
        let fmt = Format::parse("$CHAR200.").unwrap();
        assert_eq!(fmt.name(), "$CHAR");
        assert_eq!(fmt.name_without_prefix(), "CHAR");
        assert_eq!(fmt.length(), 200);
        assert_eq!(fmt.decimals(), 0);
        assert!(fmt.is_character());
        assert_eq!(fmt.justification(), Justification::Left);
        assert_eq!(fmt.to_string(), "$CHAR200.");
    }

    #[test]
    fn test_parse_lowercase_normalized() {
        let fmt = Format::parse("date9.").unwrap();
        assert_eq!(fmt.name(), "DATE");
    }

    #[test]
    fn test_parse_errors() {
        assert!(Format::parse("").is_err());
        assert!(Format::parse("DATE").is_err()); // no dot
        assert!(Format::parse(".2").is_err()); // no width
        assert!(Format::parse("DATE.").is_err()); // no width before dot
    }

    #[test]
    fn test_numeric_constructor() {
        let fmt = Format::numeric(8, 2);
        assert_eq!(fmt.length(), 8);
        assert_eq!(fmt.decimals(), 2);
        assert_eq!(fmt.to_string(), "8.2");
    }

    #[test]
    fn test_character_constructor() {
        let fmt = Format::character(200);
        assert_eq!(fmt.length(), 200);
        assert!(fmt.is_character());
        assert_eq!(fmt.to_string(), "$CHAR200.");
    }

    #[test]
    fn test_from_namestr() {
        let fmt = Format::from_namestr("DATE", 9, 0, 1);
        assert_eq!(fmt.name(), "DATE");
        assert_eq!(fmt.length(), 9);
        assert_eq!(fmt.decimals(), 0);
        assert_eq!(fmt.justification(), Justification::Right);
    }

    #[test]
    fn test_from_namestr_character() {
        let fmt = Format::from_namestr("$CHAR", 200, 0, 0);
        assert!(fmt.is_character());
        assert_eq!(fmt.justification(), Justification::Left);
    }

    #[test]
    fn test_justification() {
        assert_eq!(Justification::from_nfj(0), Justification::Left);
        assert_eq!(Justification::from_nfj(1), Justification::Right);
        assert_eq!(Justification::from_nfj(99), Justification::Right); // non-zero is right

        assert_eq!(Justification::Left.as_nfj(), 0);
        assert_eq!(Justification::Right.as_nfj(), 1);
    }

    #[test]
    fn test_from_str() {
        let fmt: Format = "DATE9.".parse().unwrap();
        assert_eq!(fmt.name(), "DATE");
    }
}
