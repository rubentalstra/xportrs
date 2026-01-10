//! Write plan for xportrs.
//!
//! This module provides the [`XptWriterBuilder`] and [`ValidatedWrite`] types
//! for planning and executing XPT file writes.

use std::path::{Path, PathBuf};

use crate::agency::Agency;
use crate::config::Config;
use crate::dataset::Dataset;
use crate::error::{Error, Result};
use crate::metadata::{DatasetMetadata, VariableMetadata};
use crate::schema::{DatasetSchema, derive_schema_plan};
use crate::validate::{Issue, IssueCollection, validate_v5_schema};
use crate::xpt::XptVersion;
use crate::xpt::v5::write::{SplitWriter, XptWriter, estimate_file_size_gb};

/// A mutable builder for XPT write operations.
///
/// This struct accumulates configuration and validates the dataset before
/// writing. Call [`finalize`](Self::finalize) to create a [`ValidatedWrite`].
///
/// # Example
///
/// ```no_run
/// use xportrs::{Xpt, Agency, Dataset, Column, ColumnData};
///
/// let dataset = Dataset::new(
///     "AE".to_string(),
///     vec![Column::new("AESEQ", ColumnData::I64(vec![Some(1)]))],
/// )?;
///
/// // With agency validation
/// let mut builder = Xpt::writer(dataset);
/// builder.agency(Agency::FDA);
/// builder.finalize()?.write_path("ae.xpt")?;
/// # Ok::<(), xportrs::Error>(())
/// ```
#[derive(Debug)]
pub struct XptWriterBuilder {
    dataset: Dataset,
    agency: Option<Agency>,
    config: Config,
    version: XptVersion,
    variable_meta: Option<Vec<VariableMetadata>>,
    dataset_meta: Option<DatasetMetadata>,
}

impl XptWriterBuilder {
    /// Creates a new write plan for the given dataset.
    #[must_use]
    pub fn new(dataset: Dataset) -> Self {
        Self {
            dataset,
            agency: None,
            config: Config::default(),
            version: XptVersion::V5,
            variable_meta: None,
            dataset_meta: None,
        }
    }

    /// Sets the regulatory agency for compliance validation.
    ///
    /// When set, additional validation rules specific to the agency
    /// are applied during finalization.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xportrs::{Xpt, Agency, Dataset};
    ///
    /// # let dataset = Dataset::new("AE", vec![]).unwrap();
    /// let mut builder = Xpt::writer(dataset);
    /// builder.agency(Agency::FDA);
    /// builder.finalize()?.write_path("ae.xpt")?;
    /// # Ok::<(), xportrs::Error>(())
    /// ```
    pub fn agency(&mut self, agency: Agency) -> &mut Self {
        self.agency = Some(agency);
        self
    }

    /// Sets the configuration.
    #[allow(dead_code)]
    pub(crate) fn config(&mut self, config: Config) -> &mut Self {
        self.config = config;
        self
    }

    /// Sets the XPT version.
    ///
    /// Note: Currently only XPT v5 is supported. XPT v8 will return
    /// an error during finalization.
    pub fn xpt_version(&mut self, version: XptVersion) -> &mut Self {
        self.version = version;
        self
    }

    /// Sets variable metadata.
    #[allow(dead_code)]
    pub(crate) fn variable_metadata(&mut self, meta: Vec<VariableMetadata>) -> &mut Self {
        self.variable_meta = Some(meta);
        self
    }

    /// Sets dataset metadata.
    #[allow(dead_code)]
    pub(crate) fn dataset_metadata(&mut self, meta: DatasetMetadata) -> &mut Self {
        self.dataset_meta = Some(meta);
        self
    }

    /// Finalizes the write plan, performing validation.
    ///
    /// This validates:
    /// 1. XPT v5 structural requirements (always)
    /// 2. Agency-specific requirements (if an agency is set)
    ///
    /// When an agency is specified and no `max_size_gb` is configured,
    /// the agency's recommended maximum file size is automatically applied,
    /// enabling automatic file splitting for large datasets.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - XPT v8 is requested (not yet implemented)
    /// - Strict mode is enabled and validation errors are found
    #[must_use = "this returns a Result that should be handled"]
    pub fn finalize(mut self) -> Result<ValidatedWrite> {
        // Check version support
        if !self.version.is_implemented() {
            return Err(Error::UnsupportedVersion {
                version: self.version,
            });
        }

        // Auto-enable file splitting for agency compliance
        if let Some(agency) = self.agency
            && self.config.write.max_size_gb.is_none()
        {
            self.config.write.max_size_gb = Some(agency.max_file_size_gb());
        }

        // Derive schema plan
        let schema = derive_schema_plan(
            &self.dataset,
            self.dataset_meta.as_ref(),
            self.variable_meta.as_deref(),
            self.agency,
            &self.config,
        )?;

        // Validate
        let mut issues = Vec::new();

        // XPT v5 structural checks (always applied)
        issues.extend(validate_v5_schema(&schema));

        // Agency checks (only if agency is set)
        if let Some(agency) = self.agency {
            issues.extend(agency.validate(&schema, None));
        }

        // Check for errors in strict mode
        if self.config.strict_checks && issues.has_errors() {
            let error_messages: Vec<String> = issues.errors().map(ToString::to_string).collect();
            return Err(Error::validation_failed(error_messages.join("; ")));
        }

        Ok(ValidatedWrite {
            dataset: self.dataset,
            schema,
            issues,
            config: self.config,
        })
    }
}

/// An immutable, validated write plan ready for execution.
///
/// This struct contains a validated dataset and schema. Use [`write_path`](Self::write_path)
/// to write the XPT file.
#[derive(Debug)]
pub struct ValidatedWrite {
    dataset: Dataset,
    schema: DatasetSchema,
    issues: Vec<Issue>,
    config: Config,
}

impl ValidatedWrite {
    /// Returns any validation issues found during finalization.
    #[must_use]
    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }

    /// Returns `true` if there are any error-level issues.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.issues.has_errors()
    }

    /// Returns `true` if there are any warning-level issues.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        self.issues.has_warnings()
    }

    /// Returns the finalized schema plan.
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn schema(&self) -> &DatasetSchema {
        &self.schema
    }

    /// Writes the XPT file to the specified path.
    ///
    /// Returns a list of file paths created. If the file was split due to size
    /// limits (configured via `max_size_gb` or automatically when an agency is
    /// specified), multiple paths are returned (e.g., `ae_001.xpt`, `ae_002.xpt`).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xportrs::{Xpt, Agency, Dataset};
    ///
    /// # let dataset = Dataset::new("AE", vec![]).unwrap();
    /// // With FDA agency, files > 5GB are automatically split
    /// let mut builder = Xpt::writer(dataset);
    /// builder.agency(Agency::FDA);
    /// let files = builder.finalize()?.write_path("ae.xpt")?;
    ///
    /// println!("Created {} file(s)", files.len());
    /// // Single file: ["ae.xpt"]
    /// // Split files: ["ae_001.xpt", "ae_002.xpt", ...]
    /// # Ok::<(), xportrs::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    #[must_use = "this returns a Result that should be handled"]
    pub fn write_path(self, path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        let path = path.as_ref();

        // Check if file splitting is needed
        if let Some(max_gb) = self.config.write.max_size_gb {
            let estimated_gb = estimate_file_size_gb(&self.schema, self.dataset.nrows());

            if estimated_gb > max_gb {
                // Use SplitWriter for large files
                let split_writer = SplitWriter::new(path, max_gb, self.config.write);
                return split_writer.write(&self.dataset, &self.schema);
            }
        }

        // Use regular writer for small files (or no size limit set)
        let writer = XptWriter::create(path, self.config.write)?;
        writer.write(&self.dataset, &self.schema)?;
        Ok(vec![path.to_path_buf()])
    }

    /// Writes the XPT file to a writer.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    #[must_use = "this returns a Result that should be handled"]
    pub fn write_to<W: std::io::Write>(self, writer: W) -> Result<()> {
        let xpt_writer = XptWriter::new(writer, self.config.write);
        xpt_writer.write(&self.dataset, &self.schema)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::{Column, ColumnData};

    #[test]
    fn test_write_plan_basic() {
        let dataset = Dataset::new(
            "AE",
            vec![Column::new(
                "AESEQ",
                ColumnData::F64(vec![Some(1.0), Some(2.0)]),
            )],
        )
        .unwrap();

        let mut builder = XptWriterBuilder::new(dataset);
        builder.xpt_version(XptVersion::V5);
        let plan = builder.finalize();

        assert!(plan.is_ok());
        let finalized = plan.unwrap();
        assert!(!finalized.has_errors());
    }

    #[test]
    fn test_write_plan_with_agency() {
        let dataset = Dataset::new(
            "AE",
            vec![Column::new("AESEQ", ColumnData::F64(vec![Some(1.0)]))],
        )
        .unwrap();

        let mut builder = XptWriterBuilder::new(dataset);
        builder.agency(Agency::FDA);
        let plan = builder.finalize();

        assert!(plan.is_ok());
    }

    #[test]
    fn test_write_plan_v8_unsupported() {
        let dataset = Dataset::new("AE", vec![]).unwrap();

        let mut builder = XptWriterBuilder::new(dataset);
        builder.xpt_version(XptVersion::V8);
        let plan = builder.finalize();

        assert!(plan.is_err());
    }
}
