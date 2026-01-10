//! CDISC metadata parser for SDTM, SEND, and ADaM standards.
//!
//! This crate provides parsers for CDISC Implementation Guide metadata files:
//! - SDTM-IG (Study Data Tabulation Model)
//! - SEND-IG (Standard for Exchange of Nonclinical Data)
//! - ADaM-IG (Analysis Data Model)
//!
//! # Bundled Standards
//!
//! This crate includes metadata for commonly used standard versions:
//! - SDTM-IG v3.4
//! - SEND-IG v3.1.1
//! - ADaM-IG v1.3
//!
//! # Example
//!
//! ```
//! use cdisc_metadata::{sdtm_ig_v3_4, Standard};
//!
//! let standard = sdtm_ig_v3_4().expect("Failed to load SDTM-IG v3.4");
//! assert_eq!(standard.name, "SDTM-IG");
//! assert_eq!(standard.version, "3.4");
//!
//! // Get variables for a specific domain
//! let dm_vars = standard.variables_for_dataset("DM");
//! println!("DM has {} variables", dm_vars.len());
//! ```

use std::path::{Path, PathBuf};

mod adam;
mod error;
mod loader;
mod sdtm;
mod send;
mod types;

pub use error::{Error, Result};
pub use loader::load_standard;
pub use types::{DatasetDef, Standard, VarType, Variable};

// Re-export standard-specific loaders for direct access
pub use adam::load_adam;
pub use sdtm::load_sdtm;
pub use send::load_send;

/// Returns the path to the bundled CDISC metadata directory.
///
/// This directory contains SDTM, SEND, and ADaM metadata files.
#[must_use]
pub fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

/// Returns the path to a specific bundled standard.
///
/// # Arguments
///
/// * `standard` - One of "sdtm", "send", or "adam"
/// * `ig_version` - The implementation guide version (e.g., "v3.4")
#[must_use]
pub fn standard_path(standard: &str, ig_version: &str) -> PathBuf {
    data_dir().join(standard).join("ig").join(ig_version)
}

/// Loads the bundled SDTM-IG v3.4 standard.
///
/// # Errors
///
/// Returns an error if the bundled metadata files cannot be parsed.
///
/// # Example
///
/// ```
/// use cdisc_metadata::sdtm_ig_v3_4;
///
/// let standard = sdtm_ig_v3_4().unwrap();
/// assert_eq!(standard.name, "SDTM-IG");
/// ```
pub fn sdtm_ig_v3_4() -> Result<Standard> {
    load_standard(&standard_path("sdtm", "v3.4"))
}

/// Loads the bundled SEND-IG v3.1.1 standard.
///
/// # Errors
///
/// Returns an error if the bundled metadata files cannot be parsed.
///
/// # Example
///
/// ```
/// use cdisc_metadata::send_ig_v3_1_1;
///
/// let standard = send_ig_v3_1_1().unwrap();
/// assert_eq!(standard.name, "SEND-IG");
/// ```
pub fn send_ig_v3_1_1() -> Result<Standard> {
    load_standard(&standard_path("send", "v3.1.1"))
}

/// Loads the bundled ADaM-IG v1.3 standard.
///
/// # Errors
///
/// Returns an error if the bundled metadata files cannot be parsed.
///
/// # Example
///
/// ```
/// use cdisc_metadata::adam_ig_v1_3;
///
/// let standard = adam_ig_v1_3().unwrap();
/// assert_eq!(standard.name, "ADaM-IG");
/// ```
pub fn adam_ig_v1_3() -> Result<Standard> {
    load_standard(&standard_path("adam", "v1.3"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_dir_exists() {
        assert!(data_dir().exists(), "data directory should exist");
    }

    #[test]
    fn test_standard_paths_exist() {
        assert!(standard_path("sdtm", "v3.4").exists());
        assert!(standard_path("send", "v3.1.1").exists());
        assert!(standard_path("adam", "v1.3").exists());
    }

    #[test]
    fn test_load_bundled_sdtm() {
        let standard = sdtm_ig_v3_4().expect("Failed to load SDTM-IG v3.4");
        assert_eq!(standard.name, "SDTM-IG");
        assert_eq!(standard.version, "3.4");
        assert!(!standard.variables.is_empty());
        assert!(!standard.datasets.is_empty());
    }

    #[test]
    fn test_load_bundled_send() {
        let standard = send_ig_v3_1_1().expect("Failed to load SEND-IG v3.1.1");
        assert_eq!(standard.name, "SEND-IG");
        assert_eq!(standard.version, "3.1.1");
        assert!(!standard.variables.is_empty());
    }

    #[test]
    fn test_load_bundled_adam() {
        let standard = adam_ig_v1_3().expect("Failed to load ADaM-IG v1.3");
        assert_eq!(standard.name, "ADaM-IG");
        assert_eq!(standard.version, "1.3");
        assert!(!standard.variables.is_empty());
    }
}
