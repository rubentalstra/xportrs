//! Public API functions for xportrs.
//!
//! This module provides the high-level public API functions for reading
//! and writing XPT files.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::config::ReadOptions;
use crate::dataset::DomainDataset;
use crate::error::{Result, XportrsError};
use crate::write_plan::XptWritePlan;
use crate::xpt::XptVersion;
use crate::xpt::v5::read::{XptFile, XptReader};

/// Inspects an XPT file and returns metadata without reading all data.
///
/// This is useful for quickly checking what's in an XPT file without
/// loading all observations into memory.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or parsed.
///
/// # Example
///
/// ```no_run
/// use xportrs::inspect_xpt;
///
/// let info = inspect_xpt("data.xpt")?;
/// for name in info.member_names() {
///     println!("Member: {}", name);
/// }
/// # Ok::<(), xportrs::XportrsError>(())
/// ```
pub fn inspect_xpt(path: impl AsRef<Path>) -> Result<XptFile> {
    let file = File::open(path.as_ref()).map_err(XportrsError::Io)?;
    let reader = XptReader::new(BufReader::new(file))?;
    Ok(reader.file_info().clone())
}

/// Reads the first member from an XPT file.
///
/// This is a convenience function for files that contain a single dataset.
///
/// # Errors
///
/// Returns an error if the file cannot be read or contains no members.
///
/// # Example
///
/// ```no_run
/// use xportrs::{read_xpt, ReadOptions};
///
/// let dataset = read_xpt("ae.xpt", &ReadOptions::default())?;
/// println!("Domain: {}", dataset.domain_code);
/// println!("Rows: {}", dataset.nrows);
/// # Ok::<(), xportrs::XportrsError>(())
/// ```
pub fn read_xpt(path: impl AsRef<Path>, options: &ReadOptions) -> Result<DomainDataset> {
    let file = File::open(path.as_ref()).map_err(XportrsError::Io)?;
    let mut reader = XptReader::new(BufReader::new(file))?;

    let first_member = reader
        .file_info()
        .members
        .first()
        .ok_or_else(|| XportrsError::corrupt("XPT file contains no members"))?
        .name
        .clone();

    reader.read_member(&first_member, options)
}

/// Reads a specific member from an XPT file by domain code.
///
/// The domain code matching is case-insensitive.
///
/// # Errors
///
/// Returns an error if the file cannot be read or the member is not found.
///
/// # Example
///
/// ```no_run
/// use xportrs::{read_xpt_member, ReadOptions};
///
/// let dataset = read_xpt_member("study.xpt", "DM", &ReadOptions::default())?;
/// println!("Domain: {}", dataset.domain_code);
/// # Ok::<(), xportrs::XportrsError>(())
/// ```
pub fn read_xpt_member(
    path: impl AsRef<Path>,
    domain_code: &str,
    options: &ReadOptions,
) -> Result<DomainDataset> {
    let file = File::open(path.as_ref()).map_err(XportrsError::Io)?;
    let mut reader = XptReader::new(BufReader::new(file))?;
    reader.read_member(domain_code, options)
}

/// Reads all members from an XPT file.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
///
/// # Example
///
/// ```no_run
/// use xportrs::{read_xpt_all, ReadOptions};
///
/// let datasets = read_xpt_all("study.xpt", &ReadOptions::default())?;
/// for ds in &datasets {
///     println!("Domain: {} ({} rows)", ds.domain_code, ds.nrows);
/// }
/// # Ok::<(), xportrs::XportrsError>(())
/// ```
pub fn read_xpt_all(path: impl AsRef<Path>, options: &ReadOptions) -> Result<Vec<DomainDataset>> {
    let file = File::open(path.as_ref()).map_err(XportrsError::Io)?;
    let mut reader = XptReader::new(BufReader::new(file))?;
    reader.read_all(options)
}

/// Writes a dataset to an XPT v5 file with default settings.
///
/// This is a convenience function for simple use cases. For more control,
/// use [`XptWritePlan`].
///
/// # Errors
///
/// Returns an error if writing fails.
///
/// # Example
///
/// ```no_run
/// use xportrs::{write_xpt_v5, DomainDataset, Column, ColumnData};
///
/// let dataset = DomainDataset::new(
///     "AE".to_string(),
///     vec![
///         Column::new("USUBJID", ColumnData::String(vec![Some("01-001".into())])),
///         Column::new("AESEQ", ColumnData::I64(vec![Some(1)])),
///     ],
/// )?;
///
/// write_xpt_v5(dataset, "ae.xpt")?;
/// # Ok::<(), xportrs::XportrsError>(())
/// ```
pub fn write_xpt_v5(dataset: DomainDataset, out_path: impl AsRef<Path>) -> Result<()> {
    XptWritePlan::new(dataset)
        .xpt_version(XptVersion::V5)
        .finalize()?
        .write_path(out_path)
}

#[cfg(test)]
mod tests {
    // Integration tests would go here, but they require actual XPT files
}
