//! Main XPT v5 reader.
//!
//! This module provides the high-level [`XptReader`] for reading XPT v5 files.

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use crate::config::ReadOptions;
use crate::dataset::{Column, ColumnData, DomainDataset};
use crate::error::{Result, XportrsError};

use super::obs::ObservationReader;
use super::parse::{XptMemberInfo, parse_header};

/// Information about an XPT file.
///
/// This struct provides metadata about the file without reading all data.
#[derive(Debug, Clone)]
pub struct XptFile {
    /// The members (datasets) in the file.
    pub members: Vec<XptMemberInfo>,
    /// The library label (if present).
    pub library_label: Option<String>,
    /// File creation timestamp (if present).
    pub created: Option<String>,
    /// File modification timestamp (if present).
    pub modified: Option<String>,
}

impl XptFile {
    /// Returns the member names.
    pub fn member_names(&self) -> impl Iterator<Item = &str> {
        self.members.iter().map(|m| m.name.as_str())
    }

    /// Finds a member by name (case-insensitive).
    #[must_use]
    pub fn find_member(&self, name: &str) -> Option<&XptMemberInfo> {
        self.members
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(name))
    }
}

/// Reader for XPT v5 files.
///
/// This struct handles reading and parsing XPT v5 format files.
pub struct XptReader<R: Read + Seek> {
    reader: BufReader<R>,
    file_info: XptFile,
}

impl<R: Read + Seek> XptReader<R> {
    /// Creates a new XPT reader from a reader.
    ///
    /// # Errors
    ///
    /// Returns an error if the file header cannot be parsed.
    pub fn new(reader: R) -> Result<Self> {
        let mut buf_reader = BufReader::new(reader);
        let file_info = parse_header(&mut buf_reader)?;

        Ok(Self {
            reader: buf_reader,
            file_info,
        })
    }

    /// Returns file information.
    #[must_use]
    pub fn file_info(&self) -> &XptFile {
        &self.file_info
    }

    /// Reads a specific member by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the member is not found or cannot be read.
    pub(crate) fn read_member(&mut self, name: &str, options: &ReadOptions) -> Result<DomainDataset> {
        let member = self
            .file_info
            .find_member(name)
            .ok_or_else(|| XportrsError::MemberNotFound {
                domain_code: name.to_string(),
            })?
            .clone();

        self.read_member_data(&member, options)
    }

    /// Reads all members in the file.
    ///
    /// # Errors
    ///
    /// Returns an error if any member cannot be read.
    pub(crate) fn read_all(&mut self, options: &ReadOptions) -> Result<Vec<DomainDataset>> {
        let members: Vec<_> = self.file_info.members.clone();
        let mut datasets = Vec::with_capacity(members.len());

        for member in members {
            let ds = self.read_member_data(&member, options)?;
            datasets.push(ds);
        }

        Ok(datasets)
    }

    /// Reads data for a specific member.
    fn read_member_data(
        &mut self,
        member: &XptMemberInfo,
        options: &ReadOptions,
    ) -> Result<DomainDataset> {
        // Seek to the observation data
        self.reader
            .seek(SeekFrom::Start(member.obs_offset))
            .map_err(XportrsError::Io)?;

        // Create observation reader
        let mut obs_reader = ObservationReader::new(&mut self.reader, &member.variables, options)?;

        // Read observations
        let row_limit = options.row_limit.unwrap_or(usize::MAX);
        let mut rows_read = 0;

        // Initialize column data vectors
        let mut columns: Vec<ColumnData> = member
            .variables
            .iter()
            .map(|v| {
                if v.xpt_type().is_numeric() {
                    ColumnData::F64(Vec::new())
                } else {
                    ColumnData::String(Vec::new())
                }
            })
            .collect();

        // Read rows
        while rows_read < row_limit {
            match obs_reader.read_observation()? {
                Some(row) => {
                    for (i, value) in row.into_iter().enumerate() {
                        match (&mut columns[i], value) {
                            (ColumnData::F64(vec), ObsValue::Numeric(v)) => vec.push(v),
                            (ColumnData::String(vec), ObsValue::Character(v)) => vec.push(v),
                            _ => {
                                return Err(XportrsError::corrupt(
                                    "type mismatch in observation data",
                                ));
                            }
                        }
                    }
                    rows_read += 1;
                }
                None => break,
            }
        }

        // Build columns
        let cols: Vec<Column> = member
            .variables
            .iter()
            .zip(columns)
            .map(|(var, data)| Column {
                name: var.nname.clone(),
                role: None,
                data,
            })
            .collect();

        DomainDataset::with_label(member.name.clone(), member.label.clone(), cols)
    }
}

impl XptReader<BufReader<File>> {
    /// Opens an XPT file from a path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or parsed.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path.as_ref()).map_err(XportrsError::Io)?;
        Self::new(BufReader::new(file))
    }
}

/// A value from an observation row.
#[derive(Debug, Clone)]
pub enum ObsValue {
    /// A numeric value.
    Numeric(Option<f64>),
    /// A character value.
    Character(Option<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xpt_file_find_member() {
        let file = XptFile {
            members: vec![XptMemberInfo {
                name: "AE".into(),
                label: Some("Adverse Events".into()),
                variables: vec![],
                obs_offset: 0,
                obs_count: 0,
                row_len: 0,
            }],
            library_label: None,
            created: None,
            modified: None,
        };

        assert!(file.find_member("AE").is_some());
        assert!(file.find_member("ae").is_some());
        assert!(file.find_member("DM").is_none());
    }
}
