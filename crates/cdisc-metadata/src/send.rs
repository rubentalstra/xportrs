//! SEND-IG (Standard for Exchange of Nonclinical Data) metadata parser.

use std::path::Path;

use crate::error::{Error, Result};
use crate::types::{DatasetDef, Standard, Variable, VarType};

/// Load SEND-IG metadata from a directory.
///
/// Expects the directory to contain:
/// - `Datasets.csv` - Dataset definitions
/// - `Variables.csv` - Variable definitions
///
/// # Errors
///
/// Returns an error if required files are missing or malformed.
pub fn load_send(dir: &Path) -> Result<Standard> {
    let datasets = load_datasets(dir)?;
    let variables = load_variables(dir)?;

    Ok(Standard {
        name: "SEND-IG".to_string(),
        version: String::new(),
        full_name: "Standard for Exchange of Nonclinical Data Implementation Guide".to_string(),
        publishing_set: "SEND".to_string(),
        effective_date: None,
        datasets,
        variables,
    })
}

/// Load dataset definitions from Datasets.csv.
fn load_datasets(dir: &Path) -> Result<Vec<DatasetDef>> {
    let path = dir.join("Datasets.csv");
    if !path.exists() {
        return Err(Error::MissingFile(path));
    }

    let mut reader = csv::Reader::from_path(&path)?;
    let mut datasets = Vec::new();

    for result in reader.records() {
        let record = result?;

        // CSV columns: Version, Class, Dataset Name, Dataset Label, Structure
        let class = record.get(1).unwrap_or("").to_string();
        let name = record.get(2).unwrap_or("").to_string();
        let label = record.get(3).unwrap_or("").to_string();
        let structure = record.get(4).map(String::from);

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

    for result in reader.records() {
        let record = result?;

        // Same structure as SDTM
        let order: u32 = record.get(1).unwrap_or("0").parse().unwrap_or(0);
        let dataset = record.get(3).unwrap_or("").to_string();
        let name = record.get(4).unwrap_or("").to_string();
        let label = record.get(5).unwrap_or("").to_string();
        let type_str = record.get(6).unwrap_or("");
        let role = record.get(11).map(|s| s.to_string()).filter(|s| !s.is_empty());
        let notes = record.get(12).map(|s| s.to_string()).filter(|s| !s.is_empty());
        let core = record.get(13).map(|s| s.to_string()).filter(|s| !s.is_empty());

        let var_type = if type_str.eq_ignore_ascii_case("Num") {
            VarType::Num
        } else {
            VarType::Char
        };

        if !name.is_empty() {
            variables.push(Variable {
                order,
                name,
                label,
                var_type,
                dataset,
                role,
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
            .join("tests/data/send/ig/v3.1.1")
    }

    #[test]
    fn test_load_send() {
        let dir = test_data_dir();
        if !dir.exists() {
            return;
        }

        let standard = load_send(&dir).unwrap();
        assert_eq!(standard.name, "SEND-IG");
        assert!(!standard.datasets.is_empty());
        assert!(!standard.variables.is_empty());
    }
}
