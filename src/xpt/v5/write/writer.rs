//! Main XPT v5 writer.
//!
//! This module provides the [`XptWriter`] for writing XPT v5 files.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use chrono::Utc;

use crate::config::WriteOptions;
use crate::dataset::{ColumnData, Dataset};
use crate::error::{Error, Result};
use crate::schema::DatasetSchema;
use crate::xpt::v5::constants::{
    LIBRARY_HEADER, MEMBER_HEADER, MEMBER_HEADER_DATA, NAMESTR_HEADER, OBS_HEADER, PAD_CHAR,
    RECORD_LEN,
};
use crate::xpt::v5::encoding::encode_ibm_float;
use crate::xpt::v5::namestr::pack_namestr;
use crate::xpt::v5::record::RecordWriter;
use crate::xpt::v5::timestamp::{
    format_sas_timestamp, sas_days_since_1960, sas_seconds_since_1960, sas_seconds_since_midnight,
};

/// Writer for XPT v5 files.
pub struct XptWriter<W: Write> {
    writer: RecordWriter<W>,
    options: WriteOptions,
}

impl<W: Write> XptWriter<W> {
    /// Creates a new XPT writer.
    pub(crate) fn new(writer: W, options: WriteOptions) -> Self {
        Self {
            writer: RecordWriter::new(writer),
            options,
        }
    }

    /// Writes a complete XPT file with one dataset.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub(crate) fn write(mut self, dataset: &Dataset, plan: &DatasetSchema) -> Result<W> {
        self.write_library_header()?;
        self.write_member(dataset, plan)?;
        self.writer.finish().map_err(Error::Io)
    }

    /// Writes the library header section.
    fn write_library_header(&mut self) -> Result<()> {
        // Record 1: Library header marker
        self.writer
            .write_record(LIBRARY_HEADER)
            .map_err(Error::Io)?;

        // Record 2: SAS identifier and timestamps
        let now = Utc::now();
        let created = self.options.created.unwrap_or(now);
        let modified = self.options.modified.unwrap_or(now);

        let created_str = format_sas_timestamp(created);
        let modified_str = format_sas_timestamp(modified);

        // Record 2: First real header
        // Per SAS spec: created timestamp at bytes 64-79
        let mut rec2 = [PAD_CHAR; RECORD_LEN];
        rec2[..24].copy_from_slice(b"SAS     SAS     SASLIB  ");
        rec2[24..32].copy_from_slice(b"9.4     "); // SAS version
        // Bytes 32-39: OS name (leave as spaces)
        // Bytes 40-63: blanks (leave as spaces)
        rec2[64..80].copy_from_slice(created_str.as_bytes());

        self.writer.write_record(&rec2).map_err(Error::Io)?;

        // Record 3: Second real header
        // Per SAS spec: modified timestamp at bytes 0-15
        let mut rec3 = [PAD_CHAR; RECORD_LEN];
        rec3[..16].copy_from_slice(modified_str.as_bytes());
        // Rest is blanks

        self.writer.write_record(&rec3).map_err(Error::Io)?;

        Ok(())
    }

    /// Writes a single member (dataset).
    fn write_member(&mut self, dataset: &Dataset, plan: &DatasetSchema) -> Result<()> {
        self.write_member_header(plan)?;
        self.write_namestr_section(plan)?;
        self.write_observations(dataset, plan)?;
        Ok(())
    }

    /// Writes the member header section.
    fn write_member_header(&mut self, plan: &DatasetSchema) -> Result<()> {
        let now = Utc::now();
        let created = self.options.created.unwrap_or(now);
        let modified = self.options.modified.unwrap_or(now);
        let created_str = format_sas_timestamp(created);
        let modified_str = format_sas_timestamp(modified);

        // Record 1: Member header marker
        self.writer.write_record(MEMBER_HEADER).map_err(Error::Io)?;

        // Record 2: DSCRPTR header marker
        self.writer
            .write_record(MEMBER_HEADER_DATA)
            .map_err(Error::Io)?;

        // Record 3: Member descriptor data 1
        // Format per SAS spec:
        // [0..8]: "SAS     "
        // [8..16]: dataset name
        // [16..24]: "SASDATA "
        // [24..32]: SAS version
        // [32..40]: OS name
        // [40..64]: blanks
        // [64..80]: created timestamp (16 chars)
        let mut rec1 = [PAD_CHAR; RECORD_LEN];
        rec1[..8].copy_from_slice(b"SAS     ");
        rec1[8..16].copy_from_slice(pad_string(&plan.domain_code, 8).as_slice());
        rec1[16..24].copy_from_slice(b"SASDATA ");
        rec1[24..32].copy_from_slice(b"9.4     "); // SAS version
        // [32..40] OS name - leave as spaces
        // [40..64] blanks - leave as spaces
        rec1[64..80].copy_from_slice(created_str.as_bytes());
        self.writer.write_record(&rec1).map_err(Error::Io)?;

        // Record 4: Member descriptor data 2
        // Format per SAS spec:
        // [0..16]: modified timestamp
        // [16..32]: blanks
        // [32..72]: dataset label (40 bytes)
        // [72..80]: dataset type (8 bytes, usually blanks)
        let mut rec2 = [PAD_CHAR; RECORD_LEN];
        rec2[..16].copy_from_slice(modified_str.as_bytes());
        if let Some(ref label) = plan.dataset_label {
            let label_bytes = pad_string(label, 40);
            rec2[32..72].copy_from_slice(&label_bytes);
        }
        self.writer.write_record(&rec2).map_err(Error::Io)?;

        Ok(())
    }

    /// Writes the NAMESTR section.
    fn write_namestr_section(&mut self, plan: &DatasetSchema) -> Result<()> {
        let nvars = plan.variables.len();

        // NAMESTR header record
        // Per SAS spec: nvars is a 4-digit field at bytes 54-57
        let mut header = [PAD_CHAR; RECORD_LEN];
        header[..54].copy_from_slice(NAMESTR_HEADER);
        let nvars_str = format!("{:04}", nvars);
        header[54..58].copy_from_slice(nvars_str.as_bytes());
        header[58..78].copy_from_slice(b"00000000000000000000");
        header[78..80].copy_from_slice(b"  ");

        self.writer.write_record(&header).map_err(Error::Io)?;

        // Write NAMESTR records for each variable
        for (i, var) in plan.variables.iter().enumerate() {
            let namestr = pack_namestr(var, i)?;
            self.writer.write_bytes(&namestr).map_err(Error::Io)?;
        }

        // Pad to record boundary
        self.writer.pad_and_flush().map_err(Error::Io)?;

        // OBS header record
        self.writer.write_record(OBS_HEADER).map_err(Error::Io)?;

        Ok(())
    }

    /// Writes observation data.
    fn write_observations(&mut self, dataset: &Dataset, plan: &DatasetSchema) -> Result<()> {
        for row_idx in 0..dataset.nrows() {
            for var in &plan.variables {
                let col = dataset.column(&var.name).ok_or_else(|| {
                    Error::invalid_schema(format!("column '{}' not found in dataset", var.name))
                })?;

                if var.xpt_type.is_numeric() {
                    let value = get_numeric_value(col.data(), row_idx)?;
                    let bytes = encode_ibm_float(value);
                    self.writer.write_bytes(&bytes).map_err(Error::Io)?;
                } else {
                    let value = get_character_value(col.data(), row_idx)?;
                    let bytes = pad_string(&value.unwrap_or_default(), var.length);
                    self.writer.write_bytes(&bytes).map_err(Error::Io)?;
                }
            }
        }

        // Pad final record
        self.writer.pad_and_flush().map_err(Error::Io)?;

        Ok(())
    }
}

impl XptWriter<BufWriter<File>> {
    /// Creates a writer for a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created.
    pub(crate) fn create(path: impl AsRef<Path>, options: WriteOptions) -> Result<Self> {
        let file = File::create(path.as_ref()).map_err(Error::Io)?;
        Ok(Self::new(BufWriter::new(file), options))
    }
}

/// Gets a numeric value from column data at the given row index.
fn get_numeric_value(data: &ColumnData, row: usize) -> Result<Option<f64>> {
    match data {
        ColumnData::F64(v) => Ok(v.get(row).copied().flatten()),
        ColumnData::I64(v) => Ok(v.get(row).copied().flatten().map(|i| i as f64)),
        ColumnData::Bool(v) => Ok(v
            .get(row)
            .copied()
            .flatten()
            .map(|b| if b { 1.0 } else { 0.0 })),
        ColumnData::Date(v) => Ok(v
            .get(row)
            .copied()
            .flatten()
            .map(|d| sas_days_since_1960(d) as f64)),
        ColumnData::DateTime(v) => Ok(v
            .get(row)
            .copied()
            .flatten()
            .map(|dt| sas_seconds_since_1960(dt) as f64)),
        ColumnData::Time(v) => Ok(v
            .get(row)
            .copied()
            .flatten()
            .map(|t| sas_seconds_since_midnight(t) as f64)),
        _ => Err(Error::invalid_schema("expected numeric column data type")),
    }
}

/// Gets a character value from column data at the given row index.
fn get_character_value(data: &ColumnData, row: usize) -> Result<Option<String>> {
    match data {
        ColumnData::String(v) => Ok(v.get(row).cloned().flatten()),
        ColumnData::Bytes(v) => Ok(v
            .get(row)
            .cloned()
            .flatten()
            .map(|b| String::from_utf8_lossy(&b).into_owned())),
        // Allow temporal types as character if metadata requested it
        ColumnData::Date(v) => Ok(v
            .get(row)
            .copied()
            .flatten()
            .map(|d| d.format("%Y-%m-%d").to_string())),
        ColumnData::DateTime(v) => Ok(v
            .get(row)
            .copied()
            .flatten()
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string())),
        ColumnData::Time(v) => Ok(v
            .get(row)
            .copied()
            .flatten()
            .map(|t| t.format("%H:%M:%S").to_string())),
        _ => Err(Error::invalid_schema("expected character column data type")),
    }
}

/// Pads a string with spaces to the specified length.
fn pad_string(s: &str, len: usize) -> Vec<u8> {
    let mut bytes = s.as_bytes().to_vec();
    bytes.truncate(len);
    bytes.resize(len, PAD_CHAR);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::Column;
    use crate::schema::plan::VariableSpec;
    use std::io::Cursor;

    #[test]
    fn test_write_empty_dataset() {
        let dataset = Dataset::new("AE", vec![]).unwrap();
        let mut plan = DatasetSchema::new("AE");
        plan.recalculate_positions();

        let output = Vec::new();
        let writer = XptWriter::new(Cursor::new(output), WriteOptions::default());
        let result = writer.write(&dataset, &plan);

        assert!(result.is_ok());
    }

    #[test]
    fn test_write_simple_dataset() {
        let dataset = Dataset::new(
            "AE",
            vec![Column::new(
                "AESEQ",
                ColumnData::F64(vec![Some(1.0), Some(2.0)]),
            )],
        )
        .unwrap();

        let mut plan = DatasetSchema::new("AE");
        plan.variables = vec![VariableSpec::numeric("AESEQ")];
        plan.recalculate_positions();

        let output = Vec::new();
        let writer = XptWriter::new(Cursor::new(output), WriteOptions::default());
        let result = writer.write(&dataset, &plan);

        assert!(result.is_ok());
    }
}
