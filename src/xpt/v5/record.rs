//! Record-level I/O for XPT v5 files.
//!
//! This module provides [`RecordReader`] and [`RecordWriter`] for handling
//! the 80-byte record structure of XPT files.

use std::io::{self, BufReader, BufWriter, Read, Write};

use super::constants::{PAD_CHAR, RECORD_LEN};

/// A writer that produces 80-byte records.
///
/// All data is accumulated and then written as complete 80-byte blocks,
/// padded with spaces as needed.
pub struct RecordWriter<W: Write> {
    inner: BufWriter<W>,
    buffer: Vec<u8>,
}

impl<W: Write> RecordWriter<W> {
    /// Creates a new record writer.
    pub fn new(writer: W) -> Self {
        Self {
            inner: BufWriter::new(writer),
            buffer: Vec::with_capacity(RECORD_LEN),
        }
    }

    /// Writes raw bytes, accumulating into records.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if writing fails.
    pub fn write_bytes(&mut self, data: &[u8]) -> io::Result<()> {
        for &byte in data {
            self.buffer.push(byte);
            if self.buffer.len() == RECORD_LEN {
                self.flush_buffer()?;
            }
        }
        Ok(())
    }

    /// Writes a complete 80-byte record.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if writing fails.
    ///
    /// # Panics
    ///
    /// Panics if `record.len() != 80`.
    pub fn write_record(&mut self, record: &[u8; RECORD_LEN]) -> io::Result<()> {
        // Flush any partial buffer first
        if !self.buffer.is_empty() {
            self.pad_and_flush()?;
        }
        self.inner.write_all(record)
    }

    /// Writes a string, right-padding with spaces to the specified length.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if writing fails.
    pub fn write_string(&mut self, s: &str, len: usize) -> io::Result<()> {
        let bytes = s.as_bytes();
        let write_len = bytes.len().min(len);
        self.write_bytes(&bytes[..write_len])?;
        // Pad with spaces
        for _ in write_len..len {
            self.write_bytes(&[PAD_CHAR])?;
        }
        Ok(())
    }

    /// Pads the current buffer to 80 bytes and writes it.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if writing fails.
    pub fn pad_and_flush(&mut self) -> io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        // Pad to 80 bytes
        while self.buffer.len() < RECORD_LEN {
            self.buffer.push(PAD_CHAR);
        }
        self.flush_buffer()
    }

    /// Finishes writing, flushing any remaining data.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if flushing fails.
    pub fn finish(mut self) -> io::Result<W> {
        self.pad_and_flush()?;
        self.inner.flush()?;
        Ok(self.inner.into_inner()?)
    }

    /// Returns the current buffer position within the record.
    #[must_use]
    pub fn buffer_position(&self) -> usize {
        self.buffer.len()
    }

    fn flush_buffer(&mut self) -> io::Result<()> {
        self.inner.write_all(&self.buffer)?;
        self.buffer.clear();
        Ok(())
    }
}

/// A reader that processes 80-byte records.
///
/// This reader handles the record structure of XPT files, allowing
/// reading of variable-length data that spans multiple records.
pub struct RecordReader<R: Read> {
    inner: BufReader<R>,
    buffer: [u8; RECORD_LEN],
    position: usize,
    at_eof: bool,
}

impl<R: Read> RecordReader<R> {
    /// Creates a new record reader.
    pub fn new(reader: R) -> Self {
        Self {
            inner: BufReader::new(reader),
            buffer: [0u8; RECORD_LEN],
            position: RECORD_LEN, // Force initial read
            at_eof: false,
        }
    }

    /// Reads exactly `len` bytes, spanning records as needed.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if reading fails or EOF is reached prematurely.
    pub fn read_bytes(&mut self, len: usize) -> io::Result<Vec<u8>> {
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            if self.position >= RECORD_LEN {
                self.read_next_record()?;
            }
            result.push(self.buffer[self.position]);
            self.position += 1;
        }
        Ok(result)
    }

    /// Reads a complete 80-byte record.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if reading fails.
    pub fn read_record(&mut self) -> io::Result<[u8; RECORD_LEN]> {
        // Align to record boundary
        if self.position != 0 && self.position != RECORD_LEN {
            // Skip remaining bytes in current record
            self.position = RECORD_LEN;
        }
        self.read_next_record()?;
        self.position = RECORD_LEN;
        Ok(self.buffer)
    }

    /// Reads a string of the specified length, trimming trailing spaces.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if reading fails.
    pub fn read_string(&mut self, len: usize) -> io::Result<String> {
        let bytes = self.read_bytes(len)?;
        let s = String::from_utf8_lossy(&bytes);
        Ok(s.trim_end().to_string())
    }

    /// Skips to the next record boundary.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if seeking fails.
    pub fn skip_to_record_boundary(&mut self) -> io::Result<()> {
        if self.position != 0 && self.position < RECORD_LEN {
            self.position = RECORD_LEN;
        }
        Ok(())
    }

    /// Skips `n` bytes.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if skipping fails.
    pub fn skip_bytes(&mut self, n: usize) -> io::Result<()> {
        for _ in 0..n {
            if self.position >= RECORD_LEN {
                self.read_next_record()?;
            }
            self.position += 1;
        }
        Ok(())
    }

    /// Returns `true` if at end of file.
    #[must_use]
    pub fn at_eof(&self) -> bool {
        self.at_eof
    }

    fn read_next_record(&mut self) -> io::Result<()> {
        match self.inner.read_exact(&mut self.buffer) {
            Ok(()) => {
                self.position = 0;
                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                self.at_eof = true;
                Err(e)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_record_writer() {
        let mut output = Vec::new();
        {
            let mut writer = RecordWriter::new(&mut output);
            writer.write_string("TEST", 8).unwrap();
            writer.pad_and_flush().unwrap();
        }
        assert_eq!(output.len(), RECORD_LEN);
        assert_eq!(&output[..4], b"TEST");
        assert!(output[4..].iter().all(|&b| b == PAD_CHAR));
    }

    #[test]
    fn test_record_reader() {
        let mut data = [PAD_CHAR; RECORD_LEN];
        data[..4].copy_from_slice(b"TEST");

        let mut reader = RecordReader::new(Cursor::new(data));
        let s = reader.read_string(8).unwrap();
        assert_eq!(s, "TEST");
    }
}
