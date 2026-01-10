//! Unified loader for CDISC metadata.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::adam::load_adam;
use crate::error::{Error, Result};
use crate::sdtm::load_sdtm;
use crate::send::load_send;
use crate::types::Standard;

/// Metadata.toml structure.
#[derive(Debug, Deserialize)]
struct MetadataToml {
    standard: StandardInfo,
}

#[derive(Debug, Deserialize)]
struct StandardInfo {
    name: String,
    version: String,
    full_name: Option<String>,
    publishing_set: String,
    effective_date: Option<String>,
}

/// Load a CDISC standard from a directory containing metadata.toml.
///
/// This function automatically detects the standard type (SDTM, SEND, or ADaM)
/// from the metadata.toml file and loads the appropriate CSV files.
///
/// # Example
///
/// ```ignore
/// use cdisc_metadata::load_standard;
/// use std::path::Path;
///
/// let standard = load_standard(Path::new("tests/data/sdtm/ig/v3.4"))?;
/// println!("Loaded {} v{}", standard.name, standard.version);
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - metadata.toml is missing or invalid
/// - Required CSV files are missing
/// - The standard type is unknown
pub fn load_standard(dir: &Path) -> Result<Standard> {
    let metadata_path = dir.join("metadata.toml");

    // Read and parse metadata.toml
    let content = fs::read_to_string(&metadata_path).map_err(|_| Error::MissingFile(metadata_path))?;
    let metadata: MetadataToml = toml::from_str(&content)?;

    // Load the appropriate standard based on publishing_set
    let mut standard = match metadata.standard.publishing_set.to_uppercase().as_str() {
        "SDTM" => load_sdtm(dir)?,
        "SEND" => load_send(dir)?,
        "ADAM" => load_adam(dir)?,
        other => return Err(Error::UnknownStandard(other.to_string())),
    };

    // Populate version info from metadata.toml
    standard.name = metadata.standard.name;
    standard.version = metadata.standard.version;
    if let Some(full_name) = metadata.standard.full_name {
        standard.full_name = full_name;
    }
    standard.publishing_set = metadata.standard.publishing_set;
    standard.effective_date = metadata.standard.effective_date;

    Ok(standard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tests_data_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tests/data")
    }

    #[test]
    fn test_load_sdtm_standard() {
        let dir = tests_data_dir().join("sdtm/ig/v3.4");
        if !dir.exists() {
            return;
        }

        let standard = load_standard(&dir).unwrap();
        assert_eq!(standard.name, "SDTM-IG");
        assert_eq!(standard.version, "3.4");
        assert_eq!(standard.publishing_set, "SDTM");
    }

    #[test]
    fn test_load_send_standard() {
        let dir = tests_data_dir().join("send/ig/v3.1.1");
        if !dir.exists() {
            return;
        }

        let standard = load_standard(&dir).unwrap();
        assert_eq!(standard.name, "SEND-IG");
        assert_eq!(standard.version, "3.1.1");
        assert_eq!(standard.publishing_set, "SEND");
    }

    #[test]
    fn test_load_adam_standard() {
        let dir = tests_data_dir().join("adam/ig/v1.3");
        if !dir.exists() {
            return;
        }

        let standard = load_standard(&dir).unwrap();
        assert_eq!(standard.name, "ADaM-IG");
        assert_eq!(standard.version, "1.3");
        assert_eq!(standard.publishing_set, "ADaM");
    }
}
