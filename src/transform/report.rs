//! Transform change tracking and reporting.
//!
//! This module provides types for tracking changes made during transform operations.
//! Each transform function returns a result containing the modified dataset and a
//! report of what changed.

use std::fmt;

/// Record of a type conversion during [`coerce_type`](super::coerce_type).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeConversion {
    /// Variable name that was converted.
    pub variable: String,
    /// Original type description (e.g., "i64", "String").
    pub from_type: String,
    /// Target type description (e.g., "f64", "Char").
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

    /// Set the number of successfully converted values.
    #[must_use]
    pub fn with_converted(mut self, count: usize) -> Self {
        self.values_converted = count;
        self
    }

    /// Set the number of failed conversions.
    #[must_use]
    pub fn with_failed(mut self, count: usize) -> Self {
        self.values_failed = count;
        self
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

/// Record of a label change during [`apply_label`](super::apply_label).
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
            Some(old) => write!(f, "{}: \"{}\" -> \"{}\"", self.variable, old, self.new_label),
            None => write!(f, "{}: (none) -> \"{}\"", self.variable, self.new_label),
        }
    }
}

/// Record of a column order change during [`apply_order`](super::apply_order).
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

/// Record of a length change during [`apply_length`](super::apply_length).
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

    /// Set the number of truncated values.
    #[must_use]
    pub fn with_truncated(mut self, count: usize) -> Self {
        self.truncated_values = count;
        self
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

/// Record of a format change during [`apply_format`](super::apply_format).
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

/// Accumulated report of all changes from transform operations.
///
/// This struct collects changes from multiple transform operations and provides
/// methods to summarize and inspect the changes.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransformReport {
    /// Type conversions performed.
    pub type_conversions: Vec<TypeConversion>,
    /// Label changes applied.
    pub label_changes: Vec<LabelChange>,
    /// Order changes applied.
    pub order_changes: Vec<OrderChange>,
    /// Length changes applied.
    pub length_changes: Vec<LengthChange>,
    /// Format changes applied.
    pub format_changes: Vec<FormatChange>,
    /// Warning messages generated during transforms.
    pub warnings: Vec<String>,
}

impl TransformReport {
    /// Create a new empty report.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any warnings were generated.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Check if any changes were made.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.type_conversions.is_empty()
            || !self.label_changes.is_empty()
            || !self.order_changes.is_empty()
            || !self.length_changes.is_empty()
            || !self.format_changes.is_empty()
    }

    /// Get the total number of changes.
    #[must_use]
    pub fn total_changes(&self) -> usize {
        self.type_conversions.len()
            + self.label_changes.len()
            + self.order_changes.len()
            + self.length_changes.len()
            + self.format_changes.len()
    }

    /// Merge another report into this one.
    pub fn merge(&mut self, other: TransformReport) {
        self.type_conversions.extend(other.type_conversions);
        self.label_changes.extend(other.label_changes);
        self.order_changes.extend(other.order_changes);
        self.length_changes.extend(other.length_changes);
        self.format_changes.extend(other.format_changes);
        self.warnings.extend(other.warnings);
    }

    /// Add a warning message.
    pub fn add_warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    /// Generate a human-readable summary of all changes.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.type_conversions.is_empty() {
            parts.push(format!("{} type conversion(s)", self.type_conversions.len()));
        }
        if !self.label_changes.is_empty() {
            parts.push(format!("{} label change(s)", self.label_changes.len()));
        }
        if !self.order_changes.is_empty() {
            parts.push(format!("{} order change(s)", self.order_changes.len()));
        }
        if !self.length_changes.is_empty() {
            parts.push(format!("{} length change(s)", self.length_changes.len()));
        }
        if !self.format_changes.is_empty() {
            parts.push(format!("{} format change(s)", self.format_changes.len()));
        }
        if !self.warnings.is_empty() {
            parts.push(format!("{} warning(s)", self.warnings.len()));
        }

        if parts.is_empty() {
            "No changes".to_string()
        } else {
            parts.join(", ")
        }
    }
}

impl fmt::Display for TransformReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Transform Report:")?;

        if !self.type_conversions.is_empty() {
            writeln!(f, "  Type Conversions:")?;
            for conv in &self.type_conversions {
                writeln!(f, "    - {conv}")?;
            }
        }

        if !self.label_changes.is_empty() {
            writeln!(f, "  Label Changes:")?;
            for change in &self.label_changes {
                writeln!(f, "    - {change}")?;
            }
        }

        if !self.order_changes.is_empty() {
            writeln!(f, "  Order Changes:")?;
            for change in &self.order_changes {
                writeln!(f, "    - {change}")?;
            }
        }

        if !self.length_changes.is_empty() {
            writeln!(f, "  Length Changes:")?;
            for change in &self.length_changes {
                writeln!(f, "    - {change}")?;
            }
        }

        if !self.format_changes.is_empty() {
            writeln!(f, "  Format Changes:")?;
            for change in &self.format_changes {
                writeln!(f, "    - {change}")?;
            }
        }

        if !self.warnings.is_empty() {
            writeln!(f, "  Warnings:")?;
            for warning in &self.warnings {
                writeln!(f, "    - {warning}")?;
            }
        }

        if !self.has_changes() && !self.has_warnings() {
            writeln!(f, "  No changes")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_conversion() {
        let conv = TypeConversion::new("AGE", "i64", "f64")
            .with_converted(100)
            .with_failed(2);

        assert_eq!(conv.variable, "AGE");
        assert!(conv.has_failures());
        assert!(conv.to_string().contains("100 converted"));
        assert!(conv.to_string().contains("2 failed"));
    }

    #[test]
    fn test_label_change() {
        let change = LabelChange::new("AGE", None, "Age in Years");
        assert!(change.is_new());
        assert!(change.to_string().contains("(none)"));

        let change = LabelChange::new("AGE", Some("Age".into()), "Age in Years");
        assert!(!change.is_new());
    }

    #[test]
    fn test_order_change() {
        let change = OrderChange::new("AGE", 5, 2);
        assert!(change.moved_earlier());
        assert!(!change.moved_later());

        let change = OrderChange::new("SEX", 1, 3);
        assert!(!change.moved_earlier());
        assert!(change.moved_later());
    }

    #[test]
    fn test_length_change() {
        let change = LengthChange::new("NAME", 100, 40).with_truncated(5);
        assert!(change.is_reduction());
        assert!(change.has_truncations());
        assert!(change.to_string().contains("5 values truncated"));
    }

    #[test]
    fn test_format_change() {
        let change = FormatChange::new("DATE", None, "DATE9.");
        assert!(change.is_new());

        let change = FormatChange::new("DATE", Some("DATE7.".into()), "DATE9.");
        assert!(!change.is_new());
    }

    #[test]
    fn test_transform_report() {
        let mut report = TransformReport::new();
        assert!(!report.has_changes());
        assert!(!report.has_warnings());

        report
            .type_conversions
            .push(TypeConversion::new("AGE", "i64", "f64"));
        report.add_warning("Test warning");

        assert!(report.has_changes());
        assert!(report.has_warnings());
        assert_eq!(report.total_changes(), 1);
        assert!(report.summary().contains("1 type conversion"));
        assert!(report.summary().contains("1 warning"));
    }

    #[test]
    fn test_transform_report_merge() {
        let mut report1 = TransformReport::new();
        report1
            .type_conversions
            .push(TypeConversion::new("A", "i64", "f64"));

        let mut report2 = TransformReport::new();
        report2
            .label_changes
            .push(LabelChange::new("B", None, "Label"));

        report1.merge(report2);
        assert_eq!(report1.type_conversions.len(), 1);
        assert_eq!(report1.label_changes.len(), 1);
    }
}
