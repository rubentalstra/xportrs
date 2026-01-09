//! XPT v5 constants and marker records.
//!
//! This module defines the magic strings and marker records used in XPT v5 files.

/// The length of a single record in bytes.
pub const RECORD_LEN: usize = 80;

/// First library header record.
///
/// Format: "HEADER RECORD*******LIBRARY HEADER RECORD!!!!!!!000000000000000000000000000000  "
pub const LIBRARY_HEADER: &[u8; RECORD_LEN] =
    b"HEADER RECORD*******LIBRARY HEADER RECORD!!!!!!!000000000000000000000000000000  ";

/// First header record marker (SAS identifier).
///
/// Format: "SAS     SAS     SASLIB  " followed by version info and timestamp.
pub const HEADER_RECORD_1: &[u8; 24] = b"SAS     SAS     SASLIB  ";

/// Second header record marker.
///
/// Format: Typically contains the dataset modified timestamp.
pub const HEADER_RECORD_2: &[u8; 16] = b"                ";

/// Member header record.
///
/// Format: "HEADER RECORD*******MEMBER  HEADER RECORD!!!!!!!000000000000000001600000000140  "
pub const MEMBER_HEADER: &[u8; RECORD_LEN] =
    b"HEADER RECORD*******MEMBER  HEADER RECORD!!!!!!!000000000000000001600000000140  ";

/// Member header data record marker.
///
/// Format: "HEADER RECORD*******DSCRPTR HEADER RECORD!!!!!!!000000000000000000000000000000  "
pub const MEMBER_HEADER_DATA: &[u8; RECORD_LEN] =
    b"HEADER RECORD*******DSCRPTR HEADER RECORD!!!!!!!000000000000000000000000000000  ";

/// NAMESTR header record template.
///
/// The last digits represent the number of variables (must be filled in).
/// Format: "HEADER RECORD*******NAMESTR HEADER RECORD!!!!!!!000000NNNNNN00000000000000000000  "
pub const NAMESTR_HEADER: &[u8; 54] = b"HEADER RECORD*******NAMESTR HEADER RECORD!!!!!!!000000";

/// Observation header record.
///
/// Format: "HEADER RECORD*******OBS     HEADER RECORD!!!!!!!000000000000000000000000000000  "
pub const OBS_HEADER: &[u8; RECORD_LEN] =
    b"HEADER RECORD*******OBS     HEADER RECORD!!!!!!!000000000000000000000000000000  ";

/// NAMESTR record length in bytes.
pub const NAMESTR_LEN: usize = 140;

/// Pad character (ASCII space).
pub const PAD_CHAR: u8 = 0x20;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_lengths() {
        assert_eq!(LIBRARY_HEADER.len(), RECORD_LEN);
        assert_eq!(MEMBER_HEADER.len(), RECORD_LEN);
        assert_eq!(MEMBER_HEADER_DATA.len(), RECORD_LEN);
        assert_eq!(OBS_HEADER.len(), RECORD_LEN);
    }
}
