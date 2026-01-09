//! Dataset-level metadata specification.
//!
//! [`DatasetSpec`] defines the expected metadata for a complete dataset
//! including name, label, and variable specifications.

use super::VariableSpec;

/// Dataset-level metadata specification.
///
/// Defines the expected metadata for a complete dataset including
/// the dataset name, label, and all variable specifications.
///
/// # Example
///
/// ```
/// use xportrs::spec::{DatasetSpec, VariableSpec};
///
/// let spec = DatasetSpec::new("DM")
///     .with_label("Demographics")
///     .add_variable(VariableSpec::character("USUBJID", 20))
///     .add_variable(VariableSpec::numeric("AGE"));
///
/// assert_eq!(spec.name, "DM");
/// assert_eq!(spec.variables.len(), 2);
/// assert!(spec.variable("USUBJID").is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DatasetSpec {
    /// Dataset name (uppercase, max 8 chars for V5, 32 for V8).
    pub name: String,

    /// Dataset label (max 40 chars for V5, 256 for V8).
    pub label: Option<String>,

    /// Variable specifications in order.
    pub variables: Vec<VariableSpec>,

    /// Dataset structure (e.g., "One record per subject").
    pub structure: Option<String>,

    /// Dataset class (e.g., "SPECIAL PURPOSE", "FINDINGS").
    pub class: Option<String>,

    /// Key variables for sorting/uniqueness.
    pub keys: Vec<String>,
}

impl DatasetSpec {
    /// Create a new dataset specification with the given name.
    ///
    /// The name is automatically converted to uppercase.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into().trim().to_uppercase(),
            label: None,
            variables: Vec::new(),
            structure: None,
            class: None,
            keys: Vec::new(),
        }
    }

    /// Set the dataset label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        let label_str = label.into();
        self.label = if label_str.is_empty() {
            None
        } else {
            Some(label_str)
        };
        self
    }

    /// Set multiple variables at once, replacing any existing.
    #[must_use]
    pub fn with_variables(mut self, vars: Vec<VariableSpec>) -> Self {
        self.variables = vars;
        self
    }

    /// Add a single variable specification.
    #[must_use]
    pub fn add_variable(mut self, var: VariableSpec) -> Self {
        self.variables.push(var);
        self
    }

    /// Set the dataset structure description.
    #[must_use]
    pub fn with_structure(mut self, structure: impl Into<String>) -> Self {
        let s = structure.into();
        self.structure = if s.is_empty() { None } else { Some(s) };
        self
    }

    /// Set the dataset class.
    #[must_use]
    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        let c = class.into();
        self.class = if c.is_empty() { None } else { Some(c) };
        self
    }

    /// Set the key variables.
    #[must_use]
    pub fn with_keys(mut self, keys: Vec<String>) -> Self {
        self.keys = keys.into_iter().map(|k| k.trim().to_uppercase()).collect();
        self
    }

    /// Add a key variable.
    #[must_use]
    pub fn add_key(mut self, key: impl Into<String>) -> Self {
        self.keys.push(key.into().trim().to_uppercase());
        self
    }

    /// Get a variable specification by name (case-insensitive).
    #[must_use]
    pub fn variable(&self, name: &str) -> Option<&VariableSpec> {
        let name_upper = name.trim().to_uppercase();
        self.variables.iter().find(|v| v.name == name_upper)
    }

    /// Get a mutable reference to a variable specification by name.
    #[must_use]
    pub fn variable_mut(&mut self, name: &str) -> Option<&mut VariableSpec> {
        let name_upper = name.trim().to_uppercase();
        self.variables.iter_mut().find(|v| v.name == name_upper)
    }

    /// Check if a variable exists in this specification.
    #[must_use]
    pub fn has_variable(&self, name: &str) -> bool {
        self.variable(name).is_some()
    }

    /// Get the number of variables.
    #[must_use]
    pub fn num_variables(&self) -> usize {
        self.variables.len()
    }

    /// Check if this specification is empty (no variables).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// Get variable names in order.
    #[must_use]
    pub fn variable_names(&self) -> Vec<&str> {
        self.variables.iter().map(|v| v.name.as_str()).collect()
    }

    /// Sort variables by their order field.
    ///
    /// Variables without an order are placed at the end in their current order.
    pub fn sort_by_order(&mut self) {
        self.variables.sort_by(|a, b| {
            match (a.order, b.order) {
                (Some(oa), Some(ob)) => oa.cmp(&ob),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
    }

    /// Get variables sorted by their order field (non-mutating).
    #[must_use]
    pub fn variables_by_order(&self) -> Vec<&VariableSpec> {
        let mut vars: Vec<_> = self.variables.iter().collect();
        vars.sort_by(|a, b| {
            match (a.order, b.order) {
                (Some(oa), Some(ob)) => oa.cmp(&ob),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
        vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_spec_new() {
        let spec = DatasetSpec::new("dm");
        assert_eq!(spec.name, "DM");
        assert!(spec.label.is_none());
        assert!(spec.variables.is_empty());
    }

    #[test]
    fn test_dataset_spec_with_label() {
        let spec = DatasetSpec::new("DM").with_label("Demographics");
        assert_eq!(spec.label, Some("Demographics".to_string()));

        let spec = DatasetSpec::new("DM").with_label("");
        assert_eq!(spec.label, None);
    }

    #[test]
    fn test_dataset_spec_add_variable() {
        let spec = DatasetSpec::new("DM")
            .add_variable(VariableSpec::character("USUBJID", 20))
            .add_variable(VariableSpec::numeric("AGE"));

        assert_eq!(spec.num_variables(), 2);
        assert!(spec.has_variable("USUBJID"));
        assert!(spec.has_variable("usubjid")); // case-insensitive
        assert!(spec.has_variable("AGE"));
        assert!(!spec.has_variable("SEX"));
    }

    #[test]
    fn test_dataset_spec_with_variables() {
        let vars = vec![
            VariableSpec::character("USUBJID", 20),
            VariableSpec::numeric("AGE"),
        ];
        let spec = DatasetSpec::new("DM").with_variables(vars);
        assert_eq!(spec.num_variables(), 2);
    }

    #[test]
    fn test_dataset_spec_variable_lookup() {
        let spec = DatasetSpec::new("DM")
            .add_variable(VariableSpec::character("USUBJID", 20).with_label("Subject ID"))
            .add_variable(VariableSpec::numeric("AGE").with_label("Age"));

        let var = spec.variable("USUBJID").unwrap();
        assert_eq!(var.label, Some("Subject ID".to_string()));

        let var = spec.variable("age").unwrap();
        assert_eq!(var.label, Some("Age".to_string()));

        assert!(spec.variable("NONEXISTENT").is_none());
    }

    #[test]
    fn test_dataset_spec_variable_names() {
        let spec = DatasetSpec::new("DM")
            .add_variable(VariableSpec::character("USUBJID", 20))
            .add_variable(VariableSpec::numeric("AGE"));

        let names = spec.variable_names();
        assert_eq!(names, vec!["USUBJID", "AGE"]);
    }

    #[test]
    fn test_dataset_spec_with_keys() {
        let spec = DatasetSpec::new("DM")
            .with_keys(vec!["STUDYID".to_string(), "usubjid".to_string()]);

        assert_eq!(spec.keys, vec!["STUDYID", "USUBJID"]);
    }

    #[test]
    fn test_dataset_spec_sort_by_order() {
        let mut spec = DatasetSpec::new("DM")
            .add_variable(VariableSpec::numeric("AGE").with_order(2))
            .add_variable(VariableSpec::character("USUBJID", 20).with_order(1))
            .add_variable(VariableSpec::character("SEX", 1)); // no order

        spec.sort_by_order();

        assert_eq!(spec.variables[0].name, "USUBJID");
        assert_eq!(spec.variables[1].name, "AGE");
        assert_eq!(spec.variables[2].name, "SEX");
    }

    #[test]
    fn test_dataset_spec_variables_by_order() {
        let spec = DatasetSpec::new("DM")
            .add_variable(VariableSpec::numeric("AGE").with_order(2))
            .add_variable(VariableSpec::character("USUBJID", 20).with_order(1));

        let ordered = spec.variables_by_order();
        assert_eq!(ordered[0].name, "USUBJID");
        assert_eq!(ordered[1].name, "AGE");
    }

    #[test]
    fn test_dataset_spec_structure_and_class() {
        let spec = DatasetSpec::new("DM")
            .with_structure("One record per subject")
            .with_class("SPECIAL PURPOSE");

        assert_eq!(spec.structure, Some("One record per subject".to_string()));
        assert_eq!(spec.class, Some("SPECIAL PURPOSE".to_string()));
    }
}
