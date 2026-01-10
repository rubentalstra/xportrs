//! ADaM-IG (Analysis Data Model) metadata parser.

use std::path::Path;

use crate::error::{Error, Result};
use crate::types::{DatasetDef, Standard, VarType, Variable};

/// Load ADaM-IG metadata from a directory.
///
/// Expects the directory to contain:
/// - `DataStructures.csv` - Data structure definitions (note: different filename!)
/// - `Variables.csv` - Variable definitions
///
/// # Errors
///
/// Returns an error if required files are missing or malformed.
pub fn load_adam(dir: &Path) -> Result<Standard> {
    let datasets = load_data_structures(dir)?;
    let variables = load_variables(dir)?;

    Ok(Standard {
        name: "ADaM-IG".to_string(),
        version: String::new(),
        full_name: "Analysis Data Model Implementation Guide".to_string(),
        publishing_set: "ADaM".to_string(),
        effective_date: None,
        datasets,
        variables,
    })
}

/// Load data structure definitions from DataStructures.csv.
fn load_data_structures(dir: &Path) -> Result<Vec<DatasetDef>> {
    let path = dir.join("DataStructures.csv");
    if !path.exists() {
        return Err(Error::MissingFile(path));
    }

    let mut reader = csv::Reader::from_path(&path)?;
    let mut datasets = Vec::new();

    for result in reader.records() {
        let record = result?;

        // CSV columns: Version, Data Structure Name, Data Structure Description, Class, Subclass, CDISC Notes
        let name = record.get(1).unwrap_or("").to_string();
        let label = record.get(2).unwrap_or("").to_string();
        let class = record.get(3).unwrap_or("").to_string();
        let subclass = record.get(4).unwrap_or("");

        // Include subclass in structure if present
        let structure = if subclass.is_empty() {
            None
        } else {
            Some(subclass.to_string())
        };

        if !name.is_empty() {
            datasets.push(DatasetDef {
                name,
                label,
                class,
                structure,
            });
        }
    }

    Ok(datasets)
}

/// Load variable definitions from Variables.csv.
fn load_variables(dir: &Path) -> Result<Vec<Variable>> {
    let path = dir.join("Variables.csv");
    if !path.exists() {
        return Err(Error::MissingFile(path));
    }

    let mut reader = csv::Reader::from_path(&path)?;
    let mut variables = Vec::new();
    let mut order_counter: u32 = 0;

    for result in reader.records() {
        let record = result?;

        // ADaM CSV columns: Version, Data Structure Name, Variable Set, Variable Name,
        //                   Variable Label, Type, Codelist codes, Codelist values,
        //                   Described Value Domains, Value List Value, Core, CDISC Notes
        let dataset = record.get(1).unwrap_or("").to_string();
        // Variable Set is at index 2 (used as role for ADaM)
        let variable_set = record
            .get(2)
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());
        let name = record.get(3).unwrap_or("").to_string();
        let label = record.get(4).unwrap_or("").to_string();
        let type_str = record.get(5).unwrap_or("");
        let core = record
            .get(10)
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());
        let notes = record
            .get(11)
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());

        let var_type = if type_str.eq_ignore_ascii_case("Num") {
            VarType::Num
        } else {
            VarType::Char
        };

        if !name.is_empty() {
            order_counter += 1;
            variables.push(Variable {
                order: order_counter,
                name,
                label,
                var_type,
                dataset,
                role: variable_set, // ADaM uses Variable Set instead of Role
                core,
                notes,
            });
        }
    }

    Ok(variables)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_data_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tests/data/adam/ig/v1.3")
    }

    #[test]
    fn test_load_adam() {
        let dir = test_data_dir();
        if !dir.exists() {
            return;
        }

        let standard = load_adam(&dir).unwrap();
        assert_eq!(standard.name, "ADaM-IG");
        assert!(!standard.datasets.is_empty());
        assert!(!standard.variables.is_empty());
    }

    #[test]
    fn test_adam_data_structures() {
        let dir = test_data_dir();
        if !dir.exists() {
            return;
        }

        let standard = load_adam(&dir).unwrap();

        // ADaM should have ADSL, BDS, TTE data structures
        let adsl = standard.dataset("ADSL");
        assert!(adsl.is_some());

        let bds = standard.dataset("BDS");
        assert!(bds.is_some());
    }
}
