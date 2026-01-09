//! Write plan for xportrs.
//!
//! This module provides the [`XptWritePlan`] and [`FinalizedWritePlan`] types
//! for planning and executing XPT file writes.

use std::path::Path;

use crate::agency::Agency;
use crate::config::Config;
use crate::dataset::DomainDataset;
use crate::error::{Result, XportrsError};
use crate::metadata::{DatasetMetadata, VariableMetadata};
use crate::schema::{SchemaPlan, derive_schema_plan};
use crate::validate::{Issue, IssueCollection, validate_v5_schema};
use crate::xpt::XptVersion;
use crate::xpt::v5::write::XptWriter;

/// A mutable builder for XPT write operations.
///
/// This struct accumulates configuration and validates the dataset before
/// writing. Call [`finalize`](Self::finalize) to create a [`FinalizedWritePlan`].
///
/// # Example
///
/// ```no_run
/// use xportrs::{Xpt, Agency, DomainDataset, Column, ColumnData};
///
/// let dataset = DomainDataset::new(
///     "AE".to_string(),
///     vec![Column::new("AESEQ", ColumnData::I64(vec![Some(1)]))],
/// )?;
///
/// // With agency validation
/// Xpt::writer(dataset)
///     .agency(Agency::FDA)
///     .finalize()?
///     .write_path("ae.xpt")?;
/// # Ok::<(), xportrs::XportrsError>(())
/// ```
#[derive(Debug)]
pub struct XptWritePlan {
    dataset: DomainDataset,
    agency: Option<Agency>,
    config: Config,
    version: XptVersion,
    variable_meta: Option<Vec<VariableMetadata>>,
    dataset_meta: Option<DatasetMetadata>,
}

impl XptWritePlan {
    /// Creates a new write plan for the given dataset.
    #[must_use]
    pub fn new(dataset: DomainDataset) -> Self {
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
    /// use xportrs::{Xpt, Agency, DomainDataset};
    ///
    /// # let dataset = DomainDataset::new("AE".into(), vec![]).unwrap();
    /// Xpt::writer(dataset)
    ///     .agency(Agency::FDA)
    ///     .finalize()?
    ///     .write_path("ae.xpt")?;
    /// # Ok::<(), xportrs::XportrsError>(())
    /// ```
    #[must_use]
    pub fn agency(mut self, agency: Agency) -> Self {
        self.agency = Some(agency);
        self
    }

    /// Sets the configuration.
    #[must_use]
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// Sets the XPT version.
    ///
    /// Note: Currently only XPT v5 is supported. XPT v8 will return
    /// an error during finalization.
    #[must_use]
    pub fn xpt_version(mut self, version: XptVersion) -> Self {
        self.version = version;
        self
    }

    /// Sets variable metadata.
    #[must_use]
    pub fn variable_metadata(mut self, meta: Vec<VariableMetadata>) -> Self {
        self.variable_meta = Some(meta);
        self
    }

    /// Sets dataset metadata.
    #[must_use]
    pub fn dataset_metadata(mut self, meta: DatasetMetadata) -> Self {
        self.dataset_meta = Some(meta);
        self
    }

    /// Finalizes the write plan, performing validation.
    ///
    /// This validates:
    /// 1. XPT v5 structural requirements (always)
    /// 2. Agency-specific requirements (if an agency is set)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - XPT v8 is requested (not yet implemented)
    /// - Strict mode is enabled and validation errors are found
    pub fn finalize(self) -> Result<FinalizedWritePlan> {
        // Check version support
        if !self.version.is_implemented() {
            return Err(XportrsError::UnsupportedVersion {
                version: self.version,
            });
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
            let error_messages: Vec<String> = issues.errors().map(|i| i.message.clone()).collect();
            return Err(XportrsError::validation_failed(error_messages.join("; ")));
        }

        Ok(FinalizedWritePlan {
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
pub struct FinalizedWritePlan {
    dataset: DomainDataset,
    schema: SchemaPlan,
    issues: Vec<Issue>,
    config: Config,
}

impl FinalizedWritePlan {
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
    pub fn schema(&self) -> &SchemaPlan {
        &self.schema
    }

    /// Writes the XPT file to the specified path.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write_path(self, path: impl AsRef<Path>) -> Result<()> {
        let writer = XptWriter::create(path, self.config.write)?;
        writer.write(&self.dataset, &self.schema)?;
        Ok(())
    }

    /// Writes the XPT file to a writer.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
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
        let dataset = DomainDataset::new(
            "AE".into(),
            vec![Column::new(
                "AESEQ",
                ColumnData::F64(vec![Some(1.0), Some(2.0)]),
            )],
        )
        .unwrap();

        let plan = XptWritePlan::new(dataset)
            .xpt_version(XptVersion::V5)
            .finalize();

        assert!(plan.is_ok());
        let finalized = plan.unwrap();
        assert!(!finalized.has_errors());
    }

    #[test]
    fn test_write_plan_with_agency() {
        let dataset = DomainDataset::new(
            "AE".into(),
            vec![Column::new(
                "AESEQ",
                ColumnData::F64(vec![Some(1.0)]),
            )],
        )
        .unwrap();

        let plan = XptWritePlan::new(dataset)
            .agency(Agency::FDA)
            .finalize();

        assert!(plan.is_ok());
    }

    #[test]
    fn test_write_plan_v8_unsupported() {
        let dataset = DomainDataset::new("AE".into(), vec![]).unwrap();

        let plan = XptWritePlan::new(dataset)
            .xpt_version(XptVersion::V8)
            .finalize();

        assert!(plan.is_err());
    }
}
