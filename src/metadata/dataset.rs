//! Dataset metadata.
//!
//! This module defines metadata for datasets (domain tables).

/// Metadata describing a dataset.
///
/// This struct provides optional metadata that can override or supplement
/// the information in a [`DomainDataset`](crate::DomainDataset).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct DatasetMetadata {
    /// The domain code (e.g., "AE", "DM", "LB").
    pub domain_code: String,

    /// The dataset label (description).
    ///
    /// Limited to 40 bytes in XPT v5.
    pub dataset_label: Option<String>,
}

#[allow(dead_code)]
impl DatasetMetadata {
    /// Creates new dataset metadata with the given domain code.
    #[must_use]
    pub(crate) fn new(domain_code: impl Into<String>) -> Self {
        Self {
            domain_code: domain_code.into(),
            dataset_label: None,
        }
    }

    /// Sets the dataset label.
    #[must_use]
    pub(crate) fn with_label(mut self, label: impl Into<String>) -> Self {
        self.dataset_label = Some(label.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_metadata_builder() {
        let meta = DatasetMetadata::new("AE").with_label("Adverse Events");

        assert_eq!(meta.domain_code, "AE");
        assert_eq!(meta.dataset_label.as_deref(), Some("Adverse Events"));
    }
}
