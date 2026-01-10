//! Public API for xportrs.
//!
//! This module provides the unified [`Xpt`] entry point for all XPT operations.
//!
//! # Examples
//!
//! ## Simple read (most common)
//! ```no_run
//! use xportrs::Xpt;
//!
//! let dataset = Xpt::read("ae.xpt")?;
//! println!("Rows: {}", dataset.nrows);
//! # Ok::<(), xportrs::XportrsError>(())
//! ```
//!
//! ## Read with options
//! ```no_run
//! use xportrs::{Xpt, ReadOptions, TextMode};
//!
//! let dataset = Xpt::reader("ae.xpt")?
//!     .options(ReadOptions::new().with_text_mode(TextMode::Latin1))
//!     .read()?;
//! # Ok::<(), xportrs::XportrsError>(())
//! ```
//!
//! ## Write a dataset
//! ```no_run
//! use xportrs::{Xpt, DomainDataset, Column, ColumnData};
//!
//! let dataset = DomainDataset::new(
//!     "AE".to_string(),
//!     vec![Column::new("AESEQ", ColumnData::I64(vec![Some(1)]))],
//! )?;
//!
//! Xpt::writer(dataset).finalize()?.write_path("ae.xpt")?;
//! # Ok::<(), xportrs::XportrsError>(())
//! ```

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::config::ReadOptions;
use crate::dataset::DomainDataset;
use crate::error::{Result, XportrsError};
use crate::write_plan::XptWritePlan;
use crate::xpt::v5::read::{XptFile, XptReader as V5Reader};

/// Unified entry point for XPT file operations.
///
/// This struct provides static methods for reading and writing XPT files.
/// It's the main interface for the library.
///
/// # Reading
///
/// For simple cases, use [`Xpt::read`]:
/// ```no_run
/// # use xportrs::Xpt;
/// let dataset = Xpt::read("ae.xpt")?;
/// # Ok::<(), xportrs::XportrsError>(())
/// ```
///
/// For more control, use [`Xpt::reader`] to get a builder:
/// ```no_run
/// # use xportrs::Xpt;
/// // Read a specific member
/// let dataset = Xpt::reader("study.xpt")?.read_member("DM")?;
///
/// // Read all members
/// let datasets = Xpt::reader("study.xpt")?.read_all()?;
/// # Ok::<(), xportrs::XportrsError>(())
/// ```
///
/// # Writing
///
/// Use [`Xpt::writer`] to create a write plan:
/// ```no_run
/// # use xportrs::{Xpt, DomainDataset};
/// # let dataset = DomainDataset::new("AE".into(), vec![]).unwrap();
/// Xpt::writer(dataset)
///     .finalize()?
///     .write_path("ae.xpt")?;
/// # Ok::<(), xportrs::XportrsError>(())
/// ```
pub struct Xpt;

impl Xpt {
    /// Reads the first dataset from an XPT file with default options.
    ///
    /// This is the simplest way to read an XPT file. For files with a single
    /// dataset (the common case), this is all you need.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened
    /// - The file is not a valid XPT file
    /// - The file contains no datasets
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xportrs::Xpt;
    ///
    /// let dataset = Xpt::read("ae.xpt")?;
    /// println!("Domain: {}", dataset.domain_code);
    /// println!("Rows: {}", dataset.nrows);
    /// # Ok::<(), xportrs::XportrsError>(())
    /// ```
    #[must_use = "this returns a Result that should be handled"]
    pub fn read(path: impl AsRef<Path>) -> Result<DomainDataset> {
        Self::reader(path)?.read()
    }

    /// Opens an XPT file and returns a reader builder for more control.
    ///
    /// Use this when you need to:
    /// - Set custom read options
    /// - Read a specific member by name
    /// - Read all members from a multi-dataset file
    /// - Inspect file metadata before reading
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or parsed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xportrs::{Xpt, ReadOptions};
    ///
    /// // Read with custom options
    /// let dataset = Xpt::reader("ae.xpt")?
    ///     .options(ReadOptions::default())
    ///     .read()?;
    ///
    /// // Read a specific member
    /// let dm = Xpt::reader("study.xpt")?.read_member("DM")?;
    ///
    /// // Read all members
    /// let all = Xpt::reader("study.xpt")?.read_all()?;
    /// # Ok::<(), xportrs::XportrsError>(())
    /// ```
    #[must_use = "this returns a Result that should be handled"]
    pub fn reader(path: impl AsRef<Path>) -> Result<XptReaderBuilder> {
        let file = File::open(path.as_ref()).map_err(XportrsError::Io)?;
        let reader = V5Reader::new(BufReader::new(file))?;
        Ok(XptReaderBuilder {
            reader,
            options: ReadOptions::default(),
        })
    }

    /// Creates a write plan builder for the given dataset.
    ///
    /// This returns an [`XptWritePlan`] that you can configure before writing.
    /// Call [`finalize()`](XptWritePlan::finalize) to validate and then
    /// [`write_path()`](crate::FinalizedWritePlan::write_path) to write.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xportrs::{Xpt, DomainDataset, Column, ColumnData};
    ///
    /// let dataset = DomainDataset::new(
    ///     "AE".to_string(),
    ///     vec![
    ///         Column::new("USUBJID", ColumnData::String(vec![Some("01-001".into())])),
    ///         Column::new("AESEQ", ColumnData::I64(vec![Some(1)])),
    ///     ],
    /// )?;
    ///
    /// Xpt::writer(dataset)
    ///     .finalize()?
    ///     .write_path("ae.xpt")?;
    /// # Ok::<(), xportrs::XportrsError>(())
    /// ```
    #[must_use]
    pub fn writer(dataset: DomainDataset) -> XptWritePlan {
        XptWritePlan::new(dataset)
    }

    /// Inspects an XPT file without reading all data.
    ///
    /// This is useful for checking what's in an XPT file without loading
    /// all observations into memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or parsed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xportrs::Xpt;
    ///
    /// let info = Xpt::inspect("data.xpt")?;
    /// for name in info.member_names() {
    ///     println!("Member: {}", name);
    /// }
    /// # Ok::<(), xportrs::XportrsError>(())
    /// ```
    #[must_use = "this returns a Result that should be handled"]
    pub fn inspect(path: impl AsRef<Path>) -> Result<XptFile> {
        let file = File::open(path.as_ref()).map_err(XportrsError::Io)?;
        let reader = V5Reader::new(BufReader::new(file))?;
        Ok(reader.file_info().clone())
    }
}

/// Builder for reading XPT files with custom options.
///
/// Created by [`Xpt::reader`]. Allows setting read options and choosing
/// which member(s) to read.
pub struct XptReaderBuilder {
    reader: V5Reader<BufReader<File>>,
    options: ReadOptions,
}

impl std::fmt::Debug for XptReaderBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XptReaderBuilder")
            .field("options", &self.options)
            .finish_non_exhaustive()
    }
}

impl XptReaderBuilder {
    /// Sets custom read options.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xportrs::{Xpt, ReadOptions, TextMode};
    ///
    /// let dataset = Xpt::reader("ae.xpt")?
    ///     .options(ReadOptions::new()
    ///         .with_text_mode(TextMode::Latin1)
    ///         .with_preserve_blanks(false))
    ///     .read()?;
    /// # Ok::<(), xportrs::XportrsError>(())
    /// ```
    #[must_use]
    pub fn options(mut self, options: ReadOptions) -> Self {
        self.options = options;
        self
    }

    /// Returns file metadata without reading observation data.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xportrs::Xpt;
    ///
    /// let reader = Xpt::reader("study.xpt")?;
    /// let info = reader.info();
    /// for name in info.member_names() {
    ///     println!("Member: {}", name);
    /// }
    /// # Ok::<(), xportrs::XportrsError>(())
    /// ```
    #[must_use]
    pub fn info(&self) -> &XptFile {
        self.reader.file_info()
    }

    /// Reads the first dataset from the file.
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails or the file has no members.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xportrs::Xpt;
    ///
    /// let dataset = Xpt::reader("ae.xpt")?.read()?;
    /// # Ok::<(), xportrs::XportrsError>(())
    /// ```
    #[must_use = "this returns a Result that should be handled"]
    pub fn read(mut self) -> Result<DomainDataset> {
        let first_member = self
            .reader
            .file_info()
            .members
            .first()
            .ok_or_else(|| XportrsError::corrupt("XPT file contains no members"))?
            .name
            .clone();

        self.reader.read_member(&first_member, &self.options)
    }

    /// Reads a specific member by name.
    ///
    /// The name matching is case-insensitive.
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails or the member is not found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xportrs::Xpt;
    ///
    /// let dm = Xpt::reader("study.xpt")?.read_member("DM")?;
    /// let ae = Xpt::reader("study.xpt")?.read_member("AE")?;
    /// # Ok::<(), xportrs::XportrsError>(())
    /// ```
    #[must_use = "this returns a Result that should be handled"]
    pub fn read_member(mut self, name: &str) -> Result<DomainDataset> {
        self.reader.read_member(name, &self.options)
    }

    /// Reads all members from the file.
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xportrs::Xpt;
    ///
    /// let datasets = Xpt::reader("study.xpt")?.read_all()?;
    /// for ds in &datasets {
    ///     println!("{}: {} rows", ds.domain_code, ds.nrows);
    /// }
    /// # Ok::<(), xportrs::XportrsError>(())
    /// ```
    #[must_use = "this returns a Result that should be handled"]
    pub fn read_all(mut self) -> Result<Vec<DomainDataset>> {
        self.reader.read_all(&self.options)
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would go here, but they require actual XPT files
}
