//! Report types for xportrs transform and write operations.
//!
//! Each `xportrs_*` function returns a report detailing what changes were made.
//! These reports can be used for logging, auditing, or debugging the transformation
//! pipeline.
//!
//! # Report Types
//!
//! | Function | Report Type |
//! |----------|-------------|
//! | `xportrs_type()` | [`TypeReport`] |
//! | `xportrs_length()` | [`LengthReport`] |
//! | `xportrs_label()` | [`LabelReport`] |
//! | `xportrs_format()` | [`FormatReport`] |
//! | `xportrs_order()` | [`OrderReport`] |
//! | `xportrs_write()` | [`WriteReport`] |
//! | `xportrs()` | [`XportrsReport`] |

use std::fmt;
use std::path::PathBuf;

// ============================================================================
// Individual Change Records
// ============================================================================

/// Record of a type conversion during type coercion.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeConversion {
    /// Variable name that was converted.
    pub variable: String,
    /// Original type description.
    pub from_type: String,
    /// Target type description.
    pub to_type: String,
    /// Number of values successfully converted.
    pub values_converted: usize,
    /// Number of values that failed conversion (became missing).
    pub values_failed: usize,
}

impl TypeConversion {
    /// Create a new type conversion record.
    #[must_use]
    pub fn new(
        variable: impl Into<String>,
        from_type: impl Into<String>,
        to_type: impl Into<String>,
    ) -> Self {
        Self {
            variable: variable.into(),
            from_type: from_type.into(),
            to_type: to_type.into(),
            values_converted: 0,
            values_failed: 0,
        }
    }

    /// Check if any values failed conversion.
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.values_failed > 0
    }
}

impl fmt::Display for TypeConversion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} -> {} ({} converted",
            self.variable, self.from_type, self.to_type, self.values_converted
        )?;
        if self.values_failed > 0 {
            write!(f, ", {} failed", self.values_failed)?;
        }
        write!(f, ")")
    }
}

/// Record of a label change.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LabelChange {
    /// Variable name.
    pub variable: String,
    /// Previous label (if any).
    pub old_label: Option<String>,
    /// New label applied.
    pub new_label: String,
}

impl LabelChange {
    /// Create a new label change record.
    #[must_use]
    pub fn new(
        variable: impl Into<String>,
        old_label: Option<String>,
        new_label: impl Into<String>,
    ) -> Self {
        Self {
            variable: variable.into(),
            old_label,
            new_label: new_label.into(),
        }
    }

    /// Check if this was a new label (not a modification).
    #[must_use]
    pub fn is_new(&self) -> bool {
        self.old_label.is_none()
    }
}

impl fmt::Display for LabelChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.old_label {
            Some(old) => write!(
                f,
                "{}: \"{}\" -> \"{}\"",
                self.variable, old, self.new_label
            ),
            None => write!(f, "{}: (none) -> \"{}\"", self.variable, self.new_label),
        }
    }
}

/// Record of a column order change.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrderChange {
    /// Variable name.
    pub variable: String,
    /// Previous position (0-indexed).
    pub old_position: usize,
    /// New position (0-indexed).
    pub new_position: usize,
}

impl OrderChange {
    /// Create a new order change record.
    #[must_use]
    pub fn new(variable: impl Into<String>, old_position: usize, new_position: usize) -> Self {
        Self {
            variable: variable.into(),
            old_position,
            new_position,
        }
    }

    /// Check if the variable moved earlier in the order.
    #[must_use]
    pub fn moved_earlier(&self) -> bool {
        self.new_position < self.old_position
    }

    /// Check if the variable moved later in the order.
    #[must_use]
    pub fn moved_later(&self) -> bool {
        self.new_position > self.old_position
    }
}

impl fmt::Display for OrderChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: position {} -> {}",
            self.variable, self.old_position, self.new_position
        )
    }
}

/// Record of a length change.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LengthChange {
    /// Variable name.
    pub variable: String,
    /// Previous length in bytes.
    pub old_length: u16,
    /// New length in bytes.
    pub new_length: u16,
    /// Number of values that were truncated.
    pub truncated_values: usize,
}

impl LengthChange {
    /// Create a new length change record.
    #[must_use]
    pub fn new(variable: impl Into<String>, old_length: u16, new_length: u16) -> Self {
        Self {
            variable: variable.into(),
            old_length,
            new_length,
            truncated_values: 0,
        }
    }

    /// Check if the length was reduced.
    #[must_use]
    pub fn is_reduction(&self) -> bool {
        self.new_length < self.old_length
    }

    /// Check if any values were truncated.
    #[must_use]
    pub fn has_truncations(&self) -> bool {
        self.truncated_values > 0
    }
}

impl fmt::Display for LengthChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} -> {} bytes",
            self.variable, self.old_length, self.new_length
        )?;
        if self.truncated_values > 0 {
            write!(f, " ({} values truncated)", self.truncated_values)?;
        }
        Ok(())
    }
}

/// Record of a format change.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormatChange {
    /// Variable name.
    pub variable: String,
    /// Previous format (if any).
    pub old_format: Option<String>,
    /// New format applied.
    pub new_format: String,
}

impl FormatChange {
    /// Create a new format change record.
    #[must_use]
    pub fn new(
        variable: impl Into<String>,
        old_format: Option<String>,
        new_format: impl Into<String>,
    ) -> Self {
        Self {
            variable: variable.into(),
            old_format,
            new_format: new_format.into(),
        }
    }

    /// Check if this was a new format (not a modification).
    #[must_use]
    pub fn is_new(&self) -> bool {
        self.old_format.is_none()
    }
}

impl fmt::Display for FormatChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.old_format {
            Some(old) => write!(f, "{}: {} -> {}", self.variable, old, self.new_format),
            None => write!(f, "{}: (none) -> {}", self.variable, self.new_format),
        }
    }
}

// ============================================================================
// Individual Function Reports
// ============================================================================

/// Report from `xportrs_type()` - type coercion operation.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeReport {
    /// Type conversions performed.
    pub conversions: Vec<TypeConversion>,
    /// Warning messages.
    pub warnings: Vec<String>,
}

impl TypeReport {
    /// Create a new empty report.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any changes were made.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.conversions.is_empty()
    }

    /// Check if any warnings were generated.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Check if any conversion failures occurred.
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.conversions.iter().any(TypeConversion::has_failures)
    }
}

impl fmt::Display for TypeReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.conversions.is_empty() {
            writeln!(f, "Type Report: No changes")?;
        } else {
            writeln!(f, "Type Report: {} conversion(s)", self.conversions.len())?;
            for conv in &self.conversions {
                writeln!(f, "  - {conv}")?;
            }
        }
        for warning in &self.warnings {
            writeln!(f, "  Warning: {warning}")?;
        }
        Ok(())
    }
}

/// Report from `xportrs_length()` - length adjustment operation.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LengthReport {
    /// Length changes applied.
    pub changes: Vec<LengthChange>,
    /// Warning messages.
    pub warnings: Vec<String>,
}

impl LengthReport {
    /// Create a new empty report.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any changes were made.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    /// Check if any values were truncated.
    #[must_use]
    pub fn has_truncations(&self) -> bool {
        self.changes.iter().any(LengthChange::has_truncations)
    }
}

impl fmt::Display for LengthReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.changes.is_empty() {
            writeln!(f, "Length Report: No changes")?;
        } else {
            writeln!(f, "Length Report: {} change(s)", self.changes.len())?;
            for change in &self.changes {
                writeln!(f, "  - {change}")?;
            }
        }
        for warning in &self.warnings {
            writeln!(f, "  Warning: {warning}")?;
        }
        Ok(())
    }
}

/// Report from `xportrs_label()` - label assignment operation.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LabelReport {
    /// Label changes applied.
    pub changes: Vec<LabelChange>,
    /// Warning messages.
    pub warnings: Vec<String>,
}

impl LabelReport {
    /// Create a new empty report.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any changes were made.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }
}

impl fmt::Display for LabelReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.changes.is_empty() {
            writeln!(f, "Label Report: No changes")?;
        } else {
            writeln!(f, "Label Report: {} change(s)", self.changes.len())?;
            for change in &self.changes {
                writeln!(f, "  - {change}")?;
            }
        }
        for warning in &self.warnings {
            writeln!(f, "  Warning: {warning}")?;
        }
        Ok(())
    }
}

/// Report from `xportrs_format()` - format assignment operation.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormatReport {
    /// Format changes applied.
    pub changes: Vec<FormatChange>,
    /// Warning messages.
    pub warnings: Vec<String>,
}

impl FormatReport {
    /// Create a new empty report.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any changes were made.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }
}

impl fmt::Display for FormatReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.changes.is_empty() {
            writeln!(f, "Format Report: No changes")?;
        } else {
            writeln!(f, "Format Report: {} change(s)", self.changes.len())?;
            for change in &self.changes {
                writeln!(f, "  - {change}")?;
            }
        }
        for warning in &self.warnings {
            writeln!(f, "  Warning: {warning}")?;
        }
        Ok(())
    }
}

/// Report from `xportrs_order()` - column reordering operation.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrderReport {
    /// Order changes applied.
    pub changes: Vec<OrderChange>,
    /// Variables in spec not found in data.
    pub missing_from_data: Vec<String>,
    /// Variables in data not found in spec.
    pub missing_from_spec: Vec<String>,
    /// Warning messages.
    pub warnings: Vec<String>,
}

impl OrderReport {
    /// Create a new empty report.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any changes were made.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    /// Check if there are any mismatches between spec and data.
    #[must_use]
    pub fn has_mismatches(&self) -> bool {
        !self.missing_from_data.is_empty() || !self.missing_from_spec.is_empty()
    }
}

impl fmt::Display for OrderReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.changes.is_empty() {
            writeln!(f, "Order Report: No changes")?;
        } else {
            writeln!(f, "Order Report: {} change(s)", self.changes.len())?;
            for change in &self.changes {
                writeln!(f, "  - {change}")?;
            }
        }
        if !self.missing_from_data.is_empty() {
            writeln!(
                f,
                "  Missing from data: {}",
                self.missing_from_data.join(", ")
            )?;
        }
        if !self.missing_from_spec.is_empty() {
            writeln!(
                f,
                "  Missing from spec: {}",
                self.missing_from_spec.join(", ")
            )?;
        }
        for warning in &self.warnings {
            writeln!(f, "  Warning: {warning}")?;
        }
        Ok(())
    }
}

/// Report from `xportrs_write()` - file write operation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WriteReport {
    /// Path where the file was written.
    pub path: PathBuf,
    /// Dataset name.
    pub dataset_name: String,
    /// Number of rows written.
    pub rows_written: usize,
    /// Number of columns written.
    pub columns_written: usize,
    /// File size in bytes.
    pub file_size: u64,
    /// Validation warnings (if any).
    pub warnings: Vec<String>,
}

impl WriteReport {
    /// Create a new write report.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, dataset_name: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            dataset_name: dataset_name.into(),
            rows_written: 0,
            columns_written: 0,
            file_size: 0,
            warnings: Vec::new(),
        }
    }
}

impl fmt::Display for WriteReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Write Report:")?;
        writeln!(f, "  Path: {}", self.path.display())?;
        writeln!(f, "  Dataset: {}", self.dataset_name)?;
        writeln!(
            f,
            "  Dimensions: {} rows x {} columns",
            self.rows_written, self.columns_written
        )?;
        writeln!(f, "  File size: {} bytes", self.file_size)?;
        for warning in &self.warnings {
            writeln!(f, "  Warning: {warning}")?;
        }
        Ok(())
    }
}

// ============================================================================
// Combined Pipeline Report
// ============================================================================

/// Combined report from `xportrs()` pipeline - all transforms + write.
///
/// This report aggregates results from all transform operations and the
/// final write operation.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XportrsReport {
    /// Type coercion report (if applied).
    pub type_report: Option<TypeReport>,
    /// Length adjustment report (if applied).
    pub length_report: Option<LengthReport>,
    /// Label assignment report (if applied).
    pub label_report: Option<LabelReport>,
    /// Format assignment report (if applied).
    pub format_report: Option<FormatReport>,
    /// Order change report (if applied).
    pub order_report: Option<OrderReport>,
    /// Write operation report (if file was written).
    pub write_report: Option<WriteReport>,
}

impl XportrsReport {
    /// Create a new empty report.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any transforms made changes.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        self.type_report
            .as_ref()
            .map_or(false, TypeReport::has_changes)
            || self
                .length_report
                .as_ref()
                .map_or(false, LengthReport::has_changes)
            || self
                .label_report
                .as_ref()
                .map_or(false, LabelReport::has_changes)
            || self
                .format_report
                .as_ref()
                .map_or(false, FormatReport::has_changes)
            || self
                .order_report
                .as_ref()
                .map_or(false, OrderReport::has_changes)
    }

    /// Check if any transforms generated warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        self.type_report
            .as_ref()
            .map_or(false, TypeReport::has_warnings)
            || !self
                .length_report
                .as_ref()
                .map_or(true, |r| r.warnings.is_empty())
            || !self
                .label_report
                .as_ref()
                .map_or(true, |r| r.warnings.is_empty())
            || !self
                .format_report
                .as_ref()
                .map_or(true, |r| r.warnings.is_empty())
            || !self
                .order_report
                .as_ref()
                .map_or(true, |r| r.warnings.is_empty())
            || self
                .write_report
                .as_ref()
                .map_or(false, |r| !r.warnings.is_empty())
    }

    /// Generate a summary of all changes.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref r) = self.type_report {
            if r.has_changes() {
                parts.push(format!("{} type conversion(s)", r.conversions.len()));
            }
        }
        if let Some(ref r) = self.length_report {
            if r.has_changes() {
                parts.push(format!("{} length change(s)", r.changes.len()));
            }
        }
        if let Some(ref r) = self.label_report {
            if r.has_changes() {
                parts.push(format!("{} label change(s)", r.changes.len()));
            }
        }
        if let Some(ref r) = self.format_report {
            if r.has_changes() {
                parts.push(format!("{} format change(s)", r.changes.len()));
            }
        }
        if let Some(ref r) = self.order_report {
            if r.has_changes() {
                parts.push(format!("{} order change(s)", r.changes.len()));
            }
        }
        if let Some(ref r) = self.write_report {
            parts.push(format!(
                "wrote {} rows to {}",
                r.rows_written,
                r.path.display()
            ));
        }

        if parts.is_empty() {
            "No changes".to_string()
        } else {
            parts.join(", ")
        }
    }
}

impl fmt::Display for XportrsReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "xportrs Pipeline Report")?;
        writeln!(f, "======================")?;

        if let Some(ref r) = self.type_report {
            write!(f, "{r}")?;
        }
        if let Some(ref r) = self.length_report {
            write!(f, "{r}")?;
        }
        if let Some(ref r) = self.label_report {
            write!(f, "{r}")?;
        }
        if let Some(ref r) = self.format_report {
            write!(f, "{r}")?;
        }
        if let Some(ref r) = self.order_report {
            write!(f, "{r}")?;
        }
        if let Some(ref r) = self.write_report {
            write!(f, "{r}")?;
        }

        if !self.has_changes() {
            writeln!(f, "No changes made")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_conversion() {
        let conv = TypeConversion::new("AGE", "String", "f64");
        assert!(!conv.has_failures());
        assert!(conv.to_string().contains("AGE"));
    }

    #[test]
    fn test_type_report() {
        let mut report = TypeReport::new();
        assert!(!report.has_changes());

        report
            .conversions
            .push(TypeConversion::new("AGE", "i64", "f64"));
        assert!(report.has_changes());
    }

    #[test]
    fn test_xportrs_report_summary() {
        let mut report = XportrsReport::new();
        assert_eq!(report.summary(), "No changes");

        report.type_report = Some(TypeReport {
            conversions: vec![TypeConversion::new("AGE", "i64", "f64")],
            warnings: vec![],
        });

        assert!(report.summary().contains("1 type conversion"));
    }
}
