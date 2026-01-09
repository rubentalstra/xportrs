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

/// End-of-file marker (optional).
pub const EOF_MARKER: &[u8; RECORD_LEN] = &[0x20; RECORD_LEN];

/// SAS missing value patterns in IBM float format.
pub mod missing {
    /// Standard missing value (.).
    pub const MISSING: [u8; 8] = [0x2E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    /// Special missing A.
    pub const MISSING_A: [u8; 8] = [0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    /// Special missing B.
    pub const MISSING_B: [u8; 8] = [0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    // ... and so on for C-Z and _
}

/// NAMESTR record length in bytes.
pub const NAMESTR_LEN: usize = 140;

/// Maximum number of NAMESTRs per 80-byte record block.
pub const NAMESTRS_PER_BLOCK: usize = 0; // NAMESTRs are 140 bytes, so they span multiple records

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
