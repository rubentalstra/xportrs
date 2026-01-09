//! Dataset label application transform (`xportr_df_label` equivalent).
//!
//! Sets the dataset label from either a string value or from a specification.

use crate::types::XptDataset;

/// Apply a label to the dataset.
///
/// This is equivalent to R's `xportr_df_label()` function. It sets the dataset
/// label to the specified value.
///
/// Unlike other transforms, this function does not require a specification -
/// it directly applies the provided label to the dataset.
///
/// # Arguments
///
/// * `dataset` - The dataset to transform
/// * `label` - The label to apply
///
/// # Returns
///
/// The dataset with the updated label.
///
/// # Example
///
/// ```
/// use xportrs::XptDataset;
/// use xportrs::transform::apply_df_label;
///
/// let dataset = XptDataset::new("DM");
/// let result = apply_df_label(dataset, "Demographics");
/// assert_eq!(result.label, Some("Demographics".to_string()));
/// ```
#[must_use]
pub fn apply_df_label(mut dataset: XptDataset, label: impl Into<String>) -> XptDataset {
    let label_str = label.into();
    dataset.label = if label_str.is_empty() {
        None
    } else {
        Some(label_str)
    };
    dataset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_df_label_basic() {
        let dataset = XptDataset::new("DM");
        let result = apply_df_label(dataset, "Demographics");

        assert_eq!(result.label, Some("Demographics".to_string()));
    }

    #[test]
    fn test_apply_df_label_replace_existing() {
        let dataset = XptDataset::new("DM").with_label("Old Label");
        let result = apply_df_label(dataset, "New Label");

        assert_eq!(result.label, Some("New Label".to_string()));
    }

    #[test]
    fn test_apply_df_label_empty_clears_label() {
        let dataset = XptDataset::new("DM").with_label("Old Label");
        let result = apply_df_label(dataset, "");

        assert_eq!(result.label, None);
    }

    #[test]
    fn test_apply_df_label_from_string() {
        let dataset = XptDataset::new("DM");
        let label = String::from("Demographics");
        let result = apply_df_label(dataset, label);

        assert_eq!(result.label, Some("Demographics".to_string()));
    }

    #[test]
    fn test_apply_df_label_preserves_other_fields() {
        let mut dataset = XptDataset::new("DM");
        dataset.dataset_type = Some("DATA".to_string());

        let result = apply_df_label(dataset, "Demographics");

        assert_eq!(result.name, "DM");
        assert_eq!(result.dataset_type, Some("DATA".to_string()));
        assert_eq!(result.label, Some("Demographics".to_string()));
    }
}
