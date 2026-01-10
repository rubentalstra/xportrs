//! XPT v5 header parsing.
//!
//! This module handles parsing the header sections of XPT v5 files.

use std::io::{Read, Seek};

use crate::error::{Error, Result};
use crate::xpt::v5::constants::{
    LIBRARY_HEADER, MEMBER_HEADER, NAMESTR_HEADER, NAMESTR_LEN, OBS_HEADER, RECORD_LEN,
};
use crate::xpt::v5::namestr::{NamestrV5, unpack_namestr};

use super::reader::XptInfo;

/// Information about a member (dataset) in the XPT file.
#[derive(Debug, Clone)]
pub struct XptMemberInfo {
    /// The dataset name (domain code).
    pub name: String,
    /// The dataset label.
    pub label: Option<String>,
    /// Variable definitions.
    pub variables: Vec<NamestrV5>,
    /// File offset to the observation data.
    pub obs_offset: u64,
    /// Number of observations (if known).
    pub obs_count: usize,
    /// Row length in bytes.
    pub row_len: usize,
}

/// Parses the XPT file header and returns file information.
///
/// # Errors
///
/// Returns an error if the file is not a valid XPT v5 file.
pub fn parse_header<R: Read + Seek>(reader: &mut R) -> Result<XptInfo> {
    // Read and validate library header
    let mut header_buf = [0u8; RECORD_LEN];
    reader.read_exact(&mut header_buf).map_err(Error::Io)?;

    if &header_buf != LIBRARY_HEADER {
        return Err(Error::corrupt(
            "invalid library header - not an XPT v5 file",
        ));
    }

    // Read first real header record (contains SAS identifier)
    // Per SAS spec: created timestamp is at bytes 64-79
    reader.read_exact(&mut header_buf).map_err(Error::Io)?;
    let created = extract_timestamp(&header_buf, 64, 80);

    // Read second header record
    // Per SAS spec: modified timestamp is at bytes 0-15
    reader.read_exact(&mut header_buf).map_err(Error::Io)?;
    let modified = extract_timestamp(&header_buf, 0, 16);

    // Parse members
    let mut members = Vec::new();

    loop {
        // Try to read the next header
        match reader.read_exact(&mut header_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Error::Io(e)),
        }

        // Check if this is a member header
        if &header_buf == MEMBER_HEADER {
            let member = parse_member(reader)?;
            members.push(member);
        } else if header_buf.iter().all(|&b| b == 0x20 || b == 0) {
            // End of file (padding or EOF)
            break;
        } else {
            // Unknown record type - might be end of file
            break;
        }
    }

    Ok(XptInfo {
        members,
        library_label: None,
        created,
        modified,
    })
}

/// Parses a single member (dataset) from the file.
fn parse_member<R: Read + Seek>(reader: &mut R) -> Result<XptMemberInfo> {
    let mut buf = [0u8; RECORD_LEN];

    // Read and verify DSCRPTR header
    reader.read_exact(&mut buf).map_err(Error::Io)?;
    if !buf.starts_with(b"HEADER RECORD*******DSCRPTR") {
        return Err(Error::corrupt("expected DSCRPTR header"));
    }

    // Read member descriptor data record 1 (contains dataset name)
    reader.read_exact(&mut buf).map_err(Error::Io)?;
    let name = String::from_utf8_lossy(&buf[8..16]).trim().to_string();

    // Read member descriptor data record 2 (contains label)
    reader.read_exact(&mut buf).map_err(Error::Io)?;
    let label = {
        let l = String::from_utf8_lossy(&buf[32..72]).trim().to_string();
        if l.is_empty() { None } else { Some(l) }
    };

    // Read NAMESTR header
    reader.read_exact(&mut buf).map_err(Error::Io)?;

    if !buf.starts_with(&NAMESTR_HEADER[..]) {
        return Err(Error::corrupt("expected NAMESTR header"));
    }

    // Parse number of variables from NAMESTR header
    // Per SAS spec: nvars is a 4-digit field at bytes 54-57 (right-aligned with leading zeros)
    let nvars_str = String::from_utf8_lossy(&buf[54..58]).trim().to_string();
    let nvars: usize = nvars_str
        .parse()
        .map_err(|_| Error::corrupt(format!("invalid variable count: {}", nvars_str)))?;

    // Read NAMESTR records
    let mut variables = Vec::with_capacity(nvars);
    let namestr_total_bytes = nvars * NAMESTR_LEN;
    let namestr_records = namestr_total_bytes.div_ceil(RECORD_LEN);

    let mut namestr_data = vec![0u8; namestr_records * RECORD_LEN];
    reader.read_exact(&mut namestr_data).map_err(Error::Io)?;

    for i in 0..nvars {
        let start = i * NAMESTR_LEN;
        let end = start + NAMESTR_LEN;
        if end > namestr_data.len() {
            return Err(Error::corrupt("NAMESTR data truncated"));
        }

        let mut namestr_buf = [0u8; NAMESTR_LEN];
        namestr_buf.copy_from_slice(&namestr_data[start..end]);
        let namestr = unpack_namestr(&namestr_buf)?;
        variables.push(namestr);
    }

    // Calculate row length
    let row_len: usize = variables.iter().map(NamestrV5::length).sum();

    // Read OBS header
    reader.read_exact(&mut buf).map_err(Error::Io)?;

    if !buf.starts_with(&OBS_HEADER[..54]) {
        return Err(Error::corrupt("expected OBS header"));
    }

    // Record the offset to observation data
    let obs_offset = reader.stream_position().map_err(Error::Io)?;

    // Note: obs_count is set to 0 here because the actual observation count
    // is determined during reading by detecting padding rows (all 0x20 bytes).
    // XPT v5 doesn't store an explicit observation count, and files are padded
    // to 80-byte record boundaries with spaces.
    let obs_count = 0;

    Ok(XptMemberInfo {
        name,
        label,
        variables,
        obs_offset,
        obs_count,
        row_len,
    })
}

/// Extracts a timestamp string from a header buffer.
fn extract_timestamp(buf: &[u8], start: usize, end: usize) -> Option<String> {
    if end > buf.len() {
        return None;
    }
    let s = String::from_utf8_lossy(&buf[start..end]).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_timestamp() {
        let mut buf = [b' '; 80];
        buf[32..48].copy_from_slice(b"15JUN24:14:30:45");
        let ts = extract_timestamp(&buf, 32, 48);
        assert_eq!(ts, Some("15JUN24:14:30:45".to_string()));
    }

    #[test]
    fn test_parse_dm_xpt_header() {
        let path = std::path::Path::new("tests/data/dm.xpt");
        if !path.exists() {
            return; // Skip if test file not available
        }

        let file = std::fs::File::open(path).expect("Failed to open dm.xpt");
        let mut reader = std::io::BufReader::new(file);

        let info = parse_header(&mut reader).expect("parse_header failed");

        // Verify parsing results
        assert_eq!(info.members.len(), 1);
        assert_eq!(info.members[0].name, "DM");
        assert_eq!(info.members[0].label, Some("Demographics".to_string()));
        assert_eq!(info.members[0].variables.len(), 26);

        // Verify timestamps were parsed
        assert!(info.created.is_some());
        assert!(info.modified.is_some());
    }
}
