//! Core types for CDISC metadata.

use serde::{Deserialize, Serialize};

/// Variable type: Character or Numeric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VarType {
    /// Character (string) variable.
    Char,
    /// Numeric variable.
    Num,
}

impl VarType {
    /// Returns true if this is a numeric type.
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        matches!(self, VarType::Num)
    }

    /// Returns true if this is a character type.
    #[must_use]
    pub fn is_character(&self) -> bool {
        matches!(self, VarType::Char)
    }
}

/// A variable definition from CDISC metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    /// Variable order within the dataset (1-based).
    pub order: u32,
    /// Variable name (max 8 characters for SDTM/SEND).
    pub name: String,
    /// Variable label (max 40 characters).
    pub label: String,
    /// Variable type: Char or Num.
    pub var_type: VarType,
    /// Dataset/domain name this variable belongs to.
    pub dataset: String,
    /// Variable role (Identifier, Topic, Timing, etc.) - SDTM/SEND only.
    pub role: Option<String>,
    /// Core status (Req, Exp, Perm).
    pub core: Option<String>,
    /// CDISC notes about this variable.
    pub notes: Option<String>,
}

/// A dataset definition from CDISC metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetDef {
    /// Dataset name (domain code for SDTM/SEND).
    pub name: String,
    /// Dataset label/description.
    pub label: String,
    /// Dataset class (Interventions, Events, Findings, etc.).
    pub class: String,
    /// Dataset structure description.
    pub structure: Option<String>,
}

/// A loaded CDISC standard with its metadata.
#[derive(Debug, Clone)]
pub struct Standard {
    /// Standard name (e.g., "SDTM-IG", "SEND-IG", "ADaM-IG").
    pub name: String,
    /// Standard version (e.g., "3.4", "3.1.1", "1.3").
    pub version: String,
    /// Full name of the standard.
    pub full_name: String,
    /// Publishing set (SDTM, SEND, ADaM).
    pub publishing_set: String,
    /// Effective date.
    pub effective_date: Option<String>,
    /// All dataset definitions.
    pub datasets: Vec<DatasetDef>,
    /// All variable definitions.
    pub variables: Vec<Variable>,
}

impl Standard {
    /// Get all variables for a specific dataset/domain.
    #[must_use]
    pub fn variables_for_dataset(&self, dataset_name: &str) -> Vec<&Variable> {
        self.variables
            .iter()
            .filter(|v| v.dataset.eq_ignore_ascii_case(dataset_name))
            .collect()
    }

    /// Get a dataset definition by name.
    #[must_use]
    pub fn dataset(&self, name: &str) -> Option<&DatasetDef> {
        self.datasets
            .iter()
            .find(|d| d.name.eq_ignore_ascii_case(name))
    }

    /// Get all dataset names.
    #[must_use]
    pub fn dataset_names(&self) -> Vec<&str> {
        self.datasets.iter().map(|d| d.name.as_str()).collect()
    }
}
