//! Observation reading for XPT v5.
//!
//! This module handles reading and decoding observation (row) data.

use std::io::Read;

use crate::config::ReadOptions;
use crate::error::{Result, XportrsError};
use crate::xpt::v5::constants::RECORD_LEN;
use crate::xpt::v5::encoding::{decode_ibm_float, decode_text};
use crate::xpt::v5::namestr::NamestrV5;

use super::reader::ObsValue;

/// Reader for observation data.
pub struct ObservationReader<'a, R: Read> {
    reader: &'a mut R,
    variables: Vec<NamestrV5>,
    row_len: usize,
    options: ReadOptions,
    buffer: Vec<u8>,
    buffer_pos: usize,
    buffer_len: usize,
    at_eof: bool,
}

impl<'a, R: Read> ObservationReader<'a, R> {
    /// Creates a new observation reader.
    ///
    /// # Errors
    ///
    /// Returns an error if the reader cannot be initialized.
    pub fn new(reader: &'a mut R, variables: &[NamestrV5], options: &ReadOptions) -> Result<Self> {
        let row_len: usize = variables.iter().map(|v| v.length()).sum();

        Ok(Self {
            reader,
            variables: variables.to_vec(),
            row_len,
            options: options.clone(),
            buffer: vec![0u8; RECORD_LEN * 10], // Buffer multiple records
            buffer_pos: 0,
            buffer_len: 0,
            at_eof: false,
        })
    }

    /// Reads a single observation (row).
    ///
    /// Returns `None` when all observations have been read.
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails.
    pub fn read_observation(&mut self) -> Result<Option<Vec<ObsValue>>> {
        if self.at_eof {
            return Ok(None);
        }

        // Read enough data for one row
        let row_data = match self.read_row_bytes()? {
            Some(data) => data,
            None => return Ok(None),
        };

        // Decode each variable
        let mut values = Vec::with_capacity(self.variables.len());

        for var in &self.variables {
            let start = var.position();
            let end = start + var.length();

            if end > row_data.len() {
                return Err(XportrsError::corrupt(format!(
                    "observation data truncated: expected {} bytes, got {}",
                    end,
                    row_data.len()
                )));
            }

            let var_data = &row_data[start..end];

            let value = if var.xpt_type().is_numeric() {
                // Decode numeric value
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(var_data);
                let f = decode_ibm_float(&bytes);
                ObsValue::Numeric(f)
            } else {
                // Decode character value
                let trim = !self.options.preserve_blanks;
                match decode_text(var_data, self.options.text_mode, trim) {
                    Ok(s) => ObsValue::Character(if s.is_empty() { None } else { Some(s) }),
                    Err(_) => {
                        // Fall back to lossy decoding
                        let s = String::from_utf8_lossy(var_data);
                        let s = if trim {
                            s.trim_end().to_string()
                        } else {
                            s.into_owned()
                        };
                        ObsValue::Character(if s.is_empty() { None } else { Some(s) })
                    }
                }
            };

            values.push(value);
        }

        Ok(Some(values))
    }

    /// Reads bytes for a single row, handling record boundaries.
    fn read_row_bytes(&mut self) -> Result<Option<Vec<u8>>> {
        let mut row_data = Vec::with_capacity(self.row_len);

        while row_data.len() < self.row_len {
            // Refill buffer if needed
            if self.buffer_pos >= self.buffer_len {
                if !self.refill_buffer()? {
                    // EOF reached
                    if row_data.is_empty() {
                        return Ok(None);
                    }
                    // Partial row - likely end of file
                    self.at_eof = true;
                    return Ok(None);
                }
            }

            // Copy available bytes
            let available = self.buffer_len - self.buffer_pos;
            let needed = self.row_len - row_data.len();
            let copy_len = available.min(needed);

            row_data.extend_from_slice(&self.buffer[self.buffer_pos..self.buffer_pos + copy_len]);
            self.buffer_pos += copy_len;
        }

        Ok(Some(row_data))
    }

    /// Refills the internal buffer from the reader.
    fn refill_buffer(&mut self) -> Result<bool> {
        match self.reader.read(&mut self.buffer) {
            Ok(0) => {
                self.at_eof = true;
                Ok(false)
            }
            Ok(n) => {
                self.buffer_len = n;
                self.buffer_pos = 0;
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                self.at_eof = true;
                Ok(false)
            }
            Err(e) => Err(XportrsError::Io(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn make_namestr(name: &str, ntype: i16, length: i16, position: i64) -> NamestrV5 {
        NamestrV5 {
            ntype,
            nhfun: 0,
            nlng: length,
            nvar0: 1,
            nname: name.to_string(),
            nlabel: String::new(),
            nform: String::new(),
            nfl: 0,
            nfd: 0,
            nfj: 0,
            nfill: [0; 2],
            niform: String::new(),
            nifl: 0,
            nifd: 0,
            npos: position,
            rest: [0; 48],
        }
    }

    #[test]
    fn test_observation_reader_creation() {
        let data = vec![0u8; 80];
        let mut cursor = Cursor::new(data);
        let vars = vec![make_namestr("TEST", 1, 8, 0)];
        let options = ReadOptions::default();

        let reader = ObservationReader::new(&mut cursor, &vars, &options);
        assert!(reader.is_ok());
    }
}
