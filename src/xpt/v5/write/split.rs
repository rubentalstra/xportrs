//! File splitting for XPT v5.
//!
//! This module provides functionality to split large datasets into
//! multiple XPT files.

use std::path::{Path, PathBuf};

use crate::config::WriteOptions;
use crate::dataset::{Column, ColumnData, Dataset};
use crate::error::{Error, Result};
use crate::schema::DatasetSchema;

use super::size::max_rows_for_size;
use super::writer::XptWriter;

/// Writer that automatically splits output into multiple files.
pub struct SplitWriter {
    base_path: PathBuf,
    max_size_bytes: usize,
    options: WriteOptions,
}

impl SplitWriter {
    /// Creates a new split writer.
    ///
    /// # Arguments
    ///
    /// * `base_path` - Base path for output files (e.g., "output/ae.xpt")
    /// * `max_size_gb` - Maximum file size in GB
    /// * `options` - Write options
    pub(crate) fn new(
        base_path: impl AsRef<Path>,
        max_size_gb: f64,
        options: WriteOptions,
    ) -> Self {
        let max_size_bytes = (max_size_gb * 1024.0 * 1024.0 * 1024.0) as usize;
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            max_size_bytes,
            options,
        }
    }

    /// Writes a dataset, splitting into multiple files if necessary.
    ///
    /// Returns the list of file paths that were written.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub(crate) fn write(self, dataset: &Dataset, plan: &DatasetSchema) -> Result<Vec<PathBuf>> {
        let max_rows = max_rows_for_size(plan, self.max_size_bytes);

        let max_rows = match max_rows {
            Some(r) if r > 0 => r,
            _ => {
                return Err(Error::invalid_schema(
                    "dataset schema is too large for the specified file size limit",
                ));
            }
        };

        if dataset.nrows() <= max_rows {
            // No splitting needed
            let writer = XptWriter::create(&self.base_path, self.options)?;
            writer.write(dataset, plan)?;
            return Ok(vec![self.base_path]);
        }

        // Split into multiple files
        let mut written_files = Vec::new();
        let mut start_row = 0;
        let mut file_num = 1;

        while start_row < dataset.nrows() {
            let end_row = (start_row + max_rows).min(dataset.nrows());

            // Create subset dataset
            let subset = slice_dataset(dataset, start_row, end_row)?;

            // Generate file path
            let file_path = self.numbered_path(file_num);

            // Write subset
            let writer = XptWriter::create(&file_path, self.options.clone())?;
            writer.write(&subset, plan)?;

            written_files.push(file_path);
            start_row = end_row;
            file_num += 1;
        }

        Ok(written_files)
    }

    /// Generates a numbered file path.
    fn numbered_path(&self, num: usize) -> PathBuf {
        let stem = self
            .base_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("data");
        let ext = self
            .base_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("xpt");
        let parent = self.base_path.parent().unwrap_or(Path::new("."));

        parent.join(format!("{}_{:03}.{}", stem, num, ext))
    }
}

/// Creates a slice of a dataset (row subset).
fn slice_dataset(dataset: &Dataset, start: usize, end: usize) -> Result<Dataset> {
    let columns: Vec<Column> = dataset
        .columns()
        .iter()
        .map(|col| {
            let data = slice_column_data(col.data(), start, end);
            if let Some(role) = col.role() {
                Column::with_role(col.name(), role, data)
            } else {
                Column::new(col.name(), data)
            }
        })
        .collect();

    let mut result = Dataset::new(dataset.domain_code().to_string(), columns)?;
    if let Some(label) = dataset.dataset_label() {
        result.set_label(label);
    }
    Ok(result)
}

/// Slices column data.
fn slice_column_data(data: &ColumnData, start: usize, end: usize) -> ColumnData {
    match data {
        ColumnData::F64(v) => ColumnData::F64(v[start..end].to_vec()),
        ColumnData::I64(v) => ColumnData::I64(v[start..end].to_vec()),
        ColumnData::Bool(v) => ColumnData::Bool(v[start..end].to_vec()),
        ColumnData::String(v) => ColumnData::String(v[start..end].to_vec()),
        ColumnData::Bytes(v) => ColumnData::Bytes(v[start..end].to_vec()),
        ColumnData::Date(v) => ColumnData::Date(v[start..end].to_vec()),
        ColumnData::DateTime(v) => ColumnData::DateTime(v[start..end].to_vec()),
        ColumnData::Time(v) => ColumnData::Time(v[start..end].to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numbered_path() {
        let writer = SplitWriter::new("/tmp/test.xpt", 1.0, WriteOptions::default());

        assert_eq!(writer.numbered_path(1), PathBuf::from("/tmp/test_001.xpt"));
        assert_eq!(writer.numbered_path(42), PathBuf::from("/tmp/test_042.xpt"));
    }

    #[test]
    fn test_slice_column_data() {
        let data = ColumnData::F64(vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0)]);
        let sliced = slice_column_data(&data, 1, 3);

        match sliced {
            ColumnData::F64(v) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0], Some(2.0));
                assert_eq!(v[1], Some(3.0));
            }
            _ => panic!("unexpected type"),
        }
    }
}
