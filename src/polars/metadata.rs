//! MetadataFrame - DataFrame wrapper with attached specification.
//!
//! This module provides [`MetadataFrame`], a wrapper around Polars DataFrame
//! that carries metadata specification through transform pipelines. This enables
//! xportr-style workflows where metadata is attached and tracked.

use polars::prelude::DataFrame;

use crate::spec::DatasetSpec;
use crate::transform::TransformReport;

/// DataFrame wrapper that carries metadata specification through pipeline.
///
/// `MetadataFrame` wraps a Polars DataFrame along with an optional dataset
/// specification and accumulated transform report. It provides an explicit
/// API (no `Deref` magic) for clarity about when you're accessing the
/// DataFrame vs metadata.
///
/// # Example
///
/// ```
/// use polars::prelude::*;
/// use xportrs::polars::MetadataFrame;
/// use xportrs::spec::{DatasetSpec, VariableSpec};
///
/// let df = df! {
///     "AGE" => &[25i64, 30],
///     "SEX" => &["M", "F"],
/// }.unwrap();
///
/// // Create without spec
/// let mf = MetadataFrame::new(df.clone());
/// assert!(mf.spec().is_none());
///
/// // Create with spec
/// let spec = DatasetSpec::new("DM")
///     .add_variable(VariableSpec::numeric("AGE"));
/// let mf = MetadataFrame::with_spec(df, spec);
/// assert!(mf.spec().is_some());
///
/// // Access DataFrame explicitly
/// assert_eq!(mf.df().height(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct MetadataFrame {
    /// The underlying DataFrame.
    df: DataFrame,
    /// Optional dataset specification.
    spec: Option<DatasetSpec>,
    /// Dataset label.
    dataset_label: Option<String>,
    /// Accumulated transform report.
    report: TransformReport,
}

impl MetadataFrame {
    /// Create a new MetadataFrame from a DataFrame without a spec.
    ///
    /// # Arguments
    ///
    /// * `df` - The DataFrame to wrap
    ///
    /// # Example
    ///
    /// ```
    /// use polars::prelude::*;
    /// use xportrs::polars::MetadataFrame;
    ///
    /// let df = df! { "X" => &[1, 2, 3] }.unwrap();
    /// let mf = MetadataFrame::new(df);
    /// assert!(mf.spec().is_none());
    /// ```
    #[must_use]
    pub fn new(df: DataFrame) -> Self {
        Self {
            df,
            spec: None,
            dataset_label: None,
            report: TransformReport::new(),
        }
    }

    /// Create a MetadataFrame with an attached specification.
    ///
    /// # Arguments
    ///
    /// * `df` - The DataFrame to wrap
    /// * `spec` - The dataset specification
    ///
    /// # Example
    ///
    /// ```
    /// use polars::prelude::*;
    /// use xportrs::polars::MetadataFrame;
    /// use xportrs::spec::DatasetSpec;
    ///
    /// let df = df! { "X" => &[1, 2, 3] }.unwrap();
    /// let spec = DatasetSpec::new("TEST");
    /// let mf = MetadataFrame::with_spec(df, spec);
    /// assert!(mf.spec().is_some());
    /// ```
    #[must_use]
    pub fn with_spec(df: DataFrame, spec: DatasetSpec) -> Self {
        Self {
            df,
            spec: Some(spec),
            dataset_label: None,
            report: TransformReport::new(),
        }
    }

    /// Access the underlying DataFrame (immutable).
    ///
    /// Use this to perform Polars operations on the data.
    ///
    /// # Example
    ///
    /// ```
    /// use polars::prelude::*;
    /// use xportrs::polars::MetadataFrame;
    ///
    /// let df = df! { "X" => &[1, 2, 3] }.unwrap();
    /// let mf = MetadataFrame::new(df);
    /// assert_eq!(mf.df().height(), 3);
    /// assert_eq!(mf.df().width(), 1);
    /// ```
    #[must_use]
    pub fn df(&self) -> &DataFrame {
        &self.df
    }

    /// Access the underlying DataFrame (mutable).
    ///
    /// Use this to modify the DataFrame in place.
    #[must_use]
    pub fn df_mut(&mut self) -> &mut DataFrame {
        &mut self.df
    }

    /// Take ownership of the DataFrame, discarding metadata.
    ///
    /// # Example
    ///
    /// ```
    /// use polars::prelude::*;
    /// use xportrs::polars::MetadataFrame;
    ///
    /// let df = df! { "X" => &[1, 2, 3] }.unwrap();
    /// let mf = MetadataFrame::new(df);
    /// let df: DataFrame = mf.into_df();
    /// assert_eq!(df.height(), 3);
    /// ```
    #[must_use]
    pub fn into_df(self) -> DataFrame {
        self.df
    }

    /// Get the attached specification, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use polars::prelude::*;
    /// use xportrs::polars::MetadataFrame;
    /// use xportrs::spec::DatasetSpec;
    ///
    /// let df = df! { "X" => &[1, 2, 3] }.unwrap();
    /// let spec = DatasetSpec::new("TEST").with_label("Test Dataset");
    /// let mf = MetadataFrame::with_spec(df, spec);
    ///
    /// let spec_ref = mf.spec().unwrap();
    /// assert_eq!(spec_ref.name, "TEST");
    /// ```
    #[must_use]
    pub fn spec(&self) -> Option<&DatasetSpec> {
        self.spec.as_ref()
    }

    /// Take the specification, removing it from the MetadataFrame.
    #[must_use]
    pub fn take_spec(&mut self) -> Option<DatasetSpec> {
        self.spec.take()
    }

    /// Set or replace the specification.
    pub fn set_spec(&mut self, spec: DatasetSpec) {
        self.spec = Some(spec);
    }

    /// Get the accumulated transform report.
    ///
    /// The report contains records of all transformations applied,
    /// including type conversions, label changes, etc.
    #[must_use]
    pub fn report(&self) -> &TransformReport {
        &self.report
    }

    /// Get a mutable reference to the transform report.
    #[must_use]
    pub fn report_mut(&mut self) -> &mut TransformReport {
        &mut self.report
    }

    /// Get the dataset label.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.dataset_label.as_deref()
    }

    /// Set the dataset label.
    ///
    /// # Example
    ///
    /// ```
    /// use polars::prelude::*;
    /// use xportrs::polars::MetadataFrame;
    ///
    /// let df = df! { "X" => &[1, 2, 3] }.unwrap();
    /// let mf = MetadataFrame::new(df).with_label("Demographics");
    /// assert_eq!(mf.label(), Some("Demographics"));
    /// ```
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.dataset_label = Some(label.into());
        self
    }

    /// Set the dataset label (mutating version).
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.dataset_label = Some(label.into());
    }

    /// Clear the dataset label.
    pub fn clear_label(&mut self) {
        self.dataset_label = None;
    }

    /// Get the name from the spec, if available.
    #[must_use]
    pub fn dataset_name(&self) -> Option<&str> {
        self.spec.as_ref().map(|s| s.name.as_str())
    }

    /// Check if a spec is attached.
    #[must_use]
    pub fn has_spec(&self) -> bool {
        self.spec.is_some()
    }

    /// Get column names from the DataFrame.
    #[must_use]
    pub fn column_names(&self) -> Vec<String> {
        self.df
            .get_column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get the number of rows.
    #[must_use]
    pub fn height(&self) -> usize {
        self.df.height()
    }

    /// Get the number of columns.
    #[must_use]
    pub fn width(&self) -> usize {
        self.df.width()
    }

    /// Create a new MetadataFrame with a fresh report but same data and spec.
    #[must_use]
    pub fn with_fresh_report(mut self) -> Self {
        self.report = TransformReport::new();
        self
    }

    /// Replace the DataFrame, keeping metadata.
    #[must_use]
    pub fn with_df(mut self, df: DataFrame) -> Self {
        self.df = df;
        self
    }

    /// Replace the spec, keeping other metadata.
    #[must_use]
    pub fn with_spec_replaced(mut self, spec: DatasetSpec) -> Self {
        self.spec = Some(spec);
        self
    }
}

impl From<DataFrame> for MetadataFrame {
    fn from(df: DataFrame) -> Self {
        Self::new(df)
    }
}

impl From<MetadataFrame> for DataFrame {
    fn from(mf: MetadataFrame) -> Self {
        mf.df
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    use crate::spec::VariableSpec;

    #[test]
    fn test_metadata_frame_new() {
        let df = df! { "X" => &[1, 2, 3] }.unwrap();
        let mf = MetadataFrame::new(df);

        assert!(mf.spec().is_none());
        assert!(mf.label().is_none());
        assert_eq!(mf.height(), 3);
        assert_eq!(mf.width(), 1);
    }

    #[test]
    fn test_metadata_frame_with_spec() {
        let df = df! { "X" => &[1, 2, 3] }.unwrap();
        let spec = DatasetSpec::new("TEST")
            .with_label("Test Dataset")
            .add_variable(VariableSpec::numeric("X"));

        let mf = MetadataFrame::with_spec(df, spec);

        assert!(mf.has_spec());
        assert_eq!(mf.dataset_name(), Some("TEST"));
        assert_eq!(mf.spec().unwrap().label.as_deref(), Some("Test Dataset"));
    }

    #[test]
    fn test_metadata_frame_with_label() {
        let df = df! { "X" => &[1, 2, 3] }.unwrap();
        let mf = MetadataFrame::new(df).with_label("Demographics");

        assert_eq!(mf.label(), Some("Demographics"));
    }

    #[test]
    fn test_metadata_frame_df_access() {
        let df = df! {
            "A" => &[1, 2],
            "B" => &["x", "y"],
        }
        .unwrap();
        let mf = MetadataFrame::new(df);

        assert_eq!(mf.df().height(), 2);
        assert_eq!(mf.df().width(), 2);
        assert_eq!(mf.column_names(), vec!["A".to_string(), "B".to_string()]);
    }

    #[test]
    fn test_metadata_frame_into_df() {
        let df = df! { "X" => &[1, 2, 3] }.unwrap();
        let spec = DatasetSpec::new("TEST");
        let mf = MetadataFrame::with_spec(df, spec).with_label("Label");

        let recovered: DataFrame = mf.into_df();
        assert_eq!(recovered.height(), 3);
    }

    #[test]
    fn test_metadata_frame_from_dataframe() {
        let df = df! { "X" => &[1, 2, 3] }.unwrap();
        let mf: MetadataFrame = df.into();

        assert_eq!(mf.height(), 3);
        assert!(mf.spec().is_none());
    }

    #[test]
    fn test_metadata_frame_report() {
        let df = df! { "X" => &[1, 2, 3] }.unwrap();
        let mf = MetadataFrame::new(df);

        // Fresh report should be empty
        assert!(mf.report().type_conversions.is_empty());
        assert!(mf.report().label_changes.is_empty());
    }

    #[test]
    fn test_metadata_frame_mutability() {
        let df = df! { "X" => &[1, 2, 3] }.unwrap();
        let mut mf = MetadataFrame::new(df);

        // Set spec
        mf.set_spec(DatasetSpec::new("UPDATED"));
        assert_eq!(mf.dataset_name(), Some("UPDATED"));

        // Set label
        mf.set_label("New Label");
        assert_eq!(mf.label(), Some("New Label"));

        // Clear label
        mf.clear_label();
        assert!(mf.label().is_none());
    }

    #[test]
    fn test_metadata_frame_clone() {
        let df = df! { "X" => &[1, 2, 3] }.unwrap();
        let spec = DatasetSpec::new("TEST");
        let mf1 = MetadataFrame::with_spec(df, spec).with_label("Label");

        let mf2 = mf1.clone();

        assert_eq!(mf1.height(), mf2.height());
        assert_eq!(mf1.dataset_name(), mf2.dataset_name());
        assert_eq!(mf1.label(), mf2.label());
    }
}
