//! NAMESTR record handling for XPT v5.
//!
//! This module handles the 140-byte NAMESTR records that describe each
//! variable in an XPT v5 file.

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

use crate::error::{Result, Error};
use crate::metadata::XptVarType;
use crate::schema::VariableSpec;

use super::constants::NAMESTR_LEN;

/// A parsed NAMESTR record.
///
/// This struct holds the metadata for a single variable as stored in the
/// XPT v5 file format.
#[derive(Debug, Clone)]
pub struct NamestrV5 {
    /// Variable type: 1 = numeric, 2 = character.
    pub ntype: i16,
    /// Hash value (unused in v5).
    pub nhfun: i16,
    /// Variable length in bytes.
    pub nlng: i16,
    /// Variable number (1-based).
    pub nvar0: i16,
    /// Variable name (8 bytes max).
    pub nname: String,
    /// Variable label (40 bytes max).
    pub nlabel: String,
    /// Format name (8 bytes max).
    pub nform: String,
    /// Format length.
    pub nfl: i16,
    /// Format decimals.
    pub nfd: i16,
    /// Format justification.
    pub nfj: i16,
    /// Unused fields.
    pub nfill: [u8; 2],
    /// Informat name (8 bytes max).
    pub niform: String,
    /// Informat length.
    pub nifl: i16,
    /// Informat decimals.
    pub nifd: i16,
    /// Position in observation (0-based).
    pub npos: i64,
    /// Remaining unused bytes.
    pub rest: [u8; 48],
}

impl Default for NamestrV5 {
    fn default() -> Self {
        Self {
            ntype: 1,
            nhfun: 0,
            nlng: 8,
            nvar0: 1,
            nname: String::new(),
            nlabel: String::new(),
            nform: String::new(),
            nfl: 0,
            nfd: 0,
            nfj: 0,
            nfill: [0; 2],
            niform: String::new(),
            nifl: 0,
            nifd: 0,
            npos: 0,
            rest: [0; 48],
        }
    }
}

impl NamestrV5 {
    /// Returns the XPT variable type.
    #[must_use]
    pub fn xpt_type(&self) -> XptVarType {
        if self.ntype == 2 {
            XptVarType::Character
        } else {
            XptVarType::Numeric
        }
    }

    /// Returns the variable length.
    #[must_use]
    pub fn length(&self) -> usize {
        self.nlng as usize
    }

    /// Returns the position in the observation record.
    #[must_use]
    pub fn position(&self) -> usize {
        self.npos as usize
    }
}

/// Packs a [`PlannedVariable`] into a 140-byte NAMESTR record.
///
/// # Errors
///
/// Returns an error if packing fails (should not happen with valid input).
pub(crate) fn pack_namestr(var: &VariableSpec, var_num: usize) -> Result<[u8; NAMESTR_LEN]> {
    let mut buf = [0u8; NAMESTR_LEN];
    let mut cursor = Cursor::new(&mut buf[..]);

    // ntype: 1 = numeric, 2 = character
    let ntype: i16 = if var.xpt_type.is_numeric() { 1 } else { 2 };
    cursor
        .write_i16::<BigEndian>(ntype)
        .map_err(Error::Io)?;

    // nhfun: hash (unused)
    cursor.write_i16::<BigEndian>(0).map_err(Error::Io)?;

    // nlng: length
    cursor
        .write_i16::<BigEndian>(var.length as i16)
        .map_err(Error::Io)?;

    // nvar0: variable number (1-based)
    cursor
        .write_i16::<BigEndian>((var_num + 1) as i16)
        .map_err(Error::Io)?;

    // nname: variable name (8 bytes, space-padded)
    let name_bytes = pad_string(&var.name, 8);
    cursor
        .get_mut()
        .get_mut(8..16)
        .ok_or_else(|| Error::corrupt("buffer too small"))?
        .copy_from_slice(&name_bytes);

    // nlabel: label (40 bytes, space-padded)
    let label_bytes = pad_string(&var.label, 40);
    cursor
        .get_mut()
        .get_mut(16..56)
        .ok_or_else(|| Error::corrupt("buffer too small"))?
        .copy_from_slice(&label_bytes);

    // nform: format name (8 bytes, space-padded)
    let format_bytes = pad_string(&var.format, 8);
    cursor
        .get_mut()
        .get_mut(56..64)
        .ok_or_else(|| Error::corrupt("buffer too small"))?
        .copy_from_slice(&format_bytes);

    // nfl, nfd, nfj: format length, decimals, justification
    cursor.set_position(64);
    cursor.write_i16::<BigEndian>(0).map_err(Error::Io)?; // nfl
    cursor.write_i16::<BigEndian>(0).map_err(Error::Io)?; // nfd
    cursor.write_i16::<BigEndian>(0).map_err(Error::Io)?; // nfj

    // nfill: 2 bytes unused
    cursor.set_position(72);

    // niform: informat name (8 bytes, space-padded)
    let informat_bytes = pad_string(&var.informat, 8);
    cursor
        .get_mut()
        .get_mut(72..80)
        .ok_or_else(|| Error::corrupt("buffer too small"))?
        .copy_from_slice(&informat_bytes);

    // nifl, nifd: informat length, decimals
    cursor.set_position(80);
    cursor.write_i16::<BigEndian>(0).map_err(Error::Io)?; // nifl
    cursor.write_i16::<BigEndian>(0).map_err(Error::Io)?; // nifd

    // npos: position (8 bytes, big-endian)
    cursor.set_position(84);
    cursor
        .write_i64::<BigEndian>(var.position as i64)
        .map_err(Error::Io)?;

    // rest: 48 bytes unused (already zeroed)

    Ok(buf)
}

/// Unpacks a 140-byte NAMESTR record into a [`NamestrV5`].
///
/// # Errors
///
/// Returns an error if the record is malformed.
pub fn unpack_namestr(data: &[u8; NAMESTR_LEN]) -> Result<NamestrV5> {
    let mut cursor = Cursor::new(data);

    let ntype = cursor.read_i16::<BigEndian>().map_err(Error::Io)?;
    let nhfun = cursor.read_i16::<BigEndian>().map_err(Error::Io)?;
    let nlng = cursor.read_i16::<BigEndian>().map_err(Error::Io)?;
    let nvar0 = cursor.read_i16::<BigEndian>().map_err(Error::Io)?;

    let nname = String::from_utf8_lossy(&data[8..16]).trim_end().to_string();
    let nlabel = String::from_utf8_lossy(&data[16..56])
        .trim_end()
        .to_string();
    let nform = String::from_utf8_lossy(&data[56..64])
        .trim_end()
        .to_string();

    cursor.set_position(64);
    let nfl = cursor.read_i16::<BigEndian>().map_err(Error::Io)?;
    let nfd = cursor.read_i16::<BigEndian>().map_err(Error::Io)?;
    let nfj = cursor.read_i16::<BigEndian>().map_err(Error::Io)?;

    let mut nfill = [0u8; 2];
    nfill.copy_from_slice(&data[70..72]);

    let niform = String::from_utf8_lossy(&data[72..80])
        .trim_end()
        .to_string();

    cursor.set_position(80);
    let nifl = cursor.read_i16::<BigEndian>().map_err(Error::Io)?;
    let nifd = cursor.read_i16::<BigEndian>().map_err(Error::Io)?;

    cursor.set_position(84);
    let npos = cursor.read_i64::<BigEndian>().map_err(Error::Io)?;

    let mut rest = [0u8; 48];
    rest.copy_from_slice(&data[92..140]);

    Ok(NamestrV5 {
        ntype,
        nhfun,
        nlng,
        nvar0,
        nname,
        nlabel,
        nform,
        nfl,
        nfd,
        nfj,
        nfill,
        niform,
        nifl,
        nifd,
        npos,
        rest,
    })
}

/// Pads a string with spaces to the specified length.
fn pad_string(s: &str, len: usize) -> Vec<u8> {
    let mut bytes = s.as_bytes().to_vec();
    bytes.truncate(len);
    bytes.resize(len, b' ');
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack_roundtrip() {
        let var = VariableSpec::numeric("AESEQ")
            .with_label("Sequence Number")
            .with_format("8.")
            .with_source_index(0);

        let packed = pack_namestr(&var, 0).unwrap();
        let unpacked = unpack_namestr(&packed).unwrap();

        assert_eq!(unpacked.nname, "AESEQ");
        assert_eq!(unpacked.nlabel, "Sequence Number");
        assert_eq!(unpacked.ntype, 1); // numeric
        assert_eq!(unpacked.nlng, 8);
    }

    #[test]
    fn test_character_variable() {
        let var = VariableSpec::character("USUBJID", 20);

        let packed = pack_namestr(&var, 1).unwrap();
        let unpacked = unpack_namestr(&packed).unwrap();

        assert_eq!(unpacked.ntype, 2); // character
        assert_eq!(unpacked.nlng, 20);
    }
}
