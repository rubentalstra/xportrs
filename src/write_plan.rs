//! Write plan for xportrs.
//!
//! This module provides the [`XptWritePlan`] and [`FinalizedWritePlan`] types
//! for planning and executing XPT file writes.

use std::path::Path;

use crate::config::Config;
use crate::dataset::DomainDataset;
use crate::error::{Result, XportrsError};
use crate::metadata::{DatasetMetadata, VariableMetadata};
use crate::profile::ComplianceProfile;
use crate::schema::{SchemaPlan, derive_schema_plan};
use crate::validate::{Issue, IssueCollection, validate_profile, validate_v5_schema};
use crate::xpt::XptVersion;
use crate::xpt::v5::write::XptWriter;

/// A mutable builder for XPT write operations.
///
/// This struct accumulates configuration and validates the dataset before
/// writing. Call [`finalize`](Self::finalize) to create a [`FinalizedWritePlan`].
#[derive(Debug)]
pub struct XptWritePlan {
    dataset: DomainDataset,
    profile: Option<ComplianceProfile>,
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
            profile: None,
            config: Config::default(),
            version: XptVersion::V5,
            variable_meta: None,
            dataset_meta: None,
        }
    }

    /// Sets the compliance profile.
    #[must_use]
    pub fn profile(mut self, profile: ComplianceProfile) -> Self {
        self.profile = Some(profile);
        self
    }

    /// Sets the configuration.
    #[must_use]
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// Sets the XPT version.
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
    /// # Errors
    ///
    /// Returns an error if the plan cannot be finalized (e.g., V8 is requested
    /// but not implemented).
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
            self.profile.as_ref(),
            &self.config,
        )?;

        // Validate
        let mut issues = Vec::new();

        // XPT v5 structural checks
        issues.extend(validate_v5_schema(&schema));

        // Profile checks
        if let Some(ref profile) = self.profile {
            issues.extend(validate_profile(&schema, profile, None));
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
    fn test_write_plan_v8_unsupported() {
        let dataset = DomainDataset::new("AE".into(), vec![]).unwrap();

        let plan = XptWritePlan::new(dataset)
            .xpt_version(XptVersion::V8)
            .finalize();

        assert!(plan.is_err());
    }
}
