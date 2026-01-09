//! Text encoding utilities for XPT v5.
//!
//! This module handles encoding and decoding of character data in XPT v5 files.

use crate::config::TextMode;

/// Decodes bytes to a string using the specified text mode.
///
/// # Arguments
///
/// * `bytes` - The raw bytes from the XPT file
/// * `mode` - The text decoding mode to use
/// * `trim_spaces` - Whether to trim trailing spaces
///
/// # Errors
///
/// Returns an error in strict UTF-8 mode if the bytes are not valid UTF-8.
pub fn decode_text(
    bytes: &[u8],
    mode: TextMode,
    trim_spaces: bool,
) -> Result<String, std::string::FromUtf8Error> {
    let s = match mode {
        TextMode::StrictUtf8 => String::from_utf8(bytes.to_vec())?,
        TextMode::LossyUtf8 => String::from_utf8_lossy(bytes).into_owned(),
        TextMode::Latin1 => bytes.iter().map(|&b| b as char).collect(),
    };

    Ok(if trim_spaces {
        s.trim_end().to_string()
    } else {
        s
    })
}

/// Encodes a string to bytes for XPT v5, padding to the specified length.
///
/// # Arguments
///
/// * `s` - The string to encode
/// * `length` - The fixed width to pad to
/// * `require_ascii` - If true, validates that the string contains only ASCII
///
/// # Errors
///
/// Returns an error if `require_ascii` is true and the string contains non-ASCII.
pub fn encode_text(
    s: Option<&str>,
    length: usize,
    require_ascii: bool,
) -> Result<Vec<u8>, TextEncodingError> {
    let bytes = match s {
        Some(s) => {
            if require_ascii && !s.is_ascii() {
                return Err(TextEncodingError::NonAscii(s.to_string()));
            }
            let mut b = s.as_bytes().to_vec();
            b.truncate(length);
            b
        }
        None => Vec::new(),
    };

    // Pad with spaces to the required length
    let mut result = bytes;
    result.resize(length, b' ');
    Ok(result)
}

/// Error type for text encoding operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum TextEncodingError {
    /// The text contains non-ASCII characters when ASCII was required.
    #[error("non-ASCII character in text: {0}")]
    NonAscii(String),
}

/// Validates that a string contains only ASCII characters suitable for XPT.
///
/// Returns `true` if the string is valid.
#[must_use]
pub fn is_valid_xpt_string(s: &str) -> bool {
    s.bytes().all(|b| b >= 0x20 && b < 0x7F)
}

/// Truncates a string to a maximum byte length, respecting UTF-8 boundaries.
#[must_use]
pub fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }

    // Find the last valid UTF-8 boundary
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }

    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_utf8() {
        let bytes = b"Hello   ";
        let s = decode_text(bytes, TextMode::LossyUtf8, true).unwrap();
        assert_eq!(s, "Hello");
    }

    #[test]
    fn test_decode_preserve_spaces() {
        let bytes = b"Hello   ";
        let s = decode_text(bytes, TextMode::LossyUtf8, false).unwrap();
        assert_eq!(s, "Hello   ");
    }

    #[test]
    fn test_decode_latin1() {
        // Latin-1 character (e with acute accent)
        let bytes = [0xE9]; // é in Latin-1
        let s = decode_text(&bytes, TextMode::Latin1, false).unwrap();
        assert_eq!(s, "é");
    }

    #[test]
    fn test_encode_ascii() {
        let result = encode_text(Some("Test"), 8, true).unwrap();
        assert_eq!(result, b"Test    ");
    }

    #[test]
    fn test_encode_non_ascii_rejected() {
        let result = encode_text(Some("Tëst"), 8, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_none() {
        let result = encode_text(None, 8, false).unwrap();
        assert_eq!(result, b"        ");
    }

    #[test]
    fn test_truncate_utf8() {
        assert_eq!(truncate_utf8("hello", 10), "hello");
        assert_eq!(truncate_utf8("hello", 3), "hel");
        // UTF-8 boundary test: "héllo" has 'é' as 2 bytes
        assert_eq!(truncate_utf8("héllo", 2), "h");
    }
}
