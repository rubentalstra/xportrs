//! Tests for error handling.

use cdisc_metadata::{load_adam, load_sdtm, load_send, load_standard, Error};
use std::path::Path;

// =============================================================================
// Error Type Tests
// =============================================================================

#[test]
fn error_display_missing_file() {
    let path = Path::new("/nonexistent/path");
    let err = Error::MissingFile(path.to_path_buf());
    let display = format!("{err}");

    assert!(display.contains("missing required file"));
    assert!(display.contains("/nonexistent/path"));
}

#[test]
fn error_display_invalid_format() {
    let err = Error::InvalidFormat("test message".to_string());
    let display = format!("{err}");

    assert!(display.contains("invalid metadata format"));
    assert!(display.contains("test message"));
}

#[test]
fn error_display_unknown_standard() {
    let err = Error::UnknownStandard("UNKNOWN".to_string());
    let display = format!("{err}");

    assert!(display.contains("unknown standard"));
    assert!(display.contains("UNKNOWN"));
}

#[test]
fn error_debug_format() {
    let err = Error::InvalidFormat("test".to_string());
    let debug = format!("{err:?}");

    assert!(debug.contains("InvalidFormat"));
}

// =============================================================================
// Error Handling in Loaders Tests
// =============================================================================

#[test]
fn load_standard_nonexistent_path_returns_error() {
    let result = load_standard(Path::new("/nonexistent/path"));
    assert!(result.is_err());
}

#[test]
fn load_sdtm_nonexistent_path_returns_error() {
    let result = load_sdtm(Path::new("/nonexistent/path"));
    assert!(result.is_err());
}

#[test]
fn load_send_nonexistent_path_returns_error() {
    let result = load_send(Path::new("/nonexistent/path"));
    assert!(result.is_err());
}

#[test]
fn load_adam_nonexistent_path_returns_error() {
    let result = load_adam(Path::new("/nonexistent/path"));
    assert!(result.is_err());
}

#[test]
fn load_standard_empty_directory_returns_error() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let result = load_standard(temp_dir.path());

    assert!(result.is_err());
}

#[test]
fn load_sdtm_missing_datasets_csv_returns_error() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create only Variables.csv
    std::fs::write(
        temp_dir.path().join("Variables.csv"),
        "Version,Variable Order,Class,Dataset Name,Variable Name,Variable Label,Type\n",
    )
    .unwrap();

    let result = load_sdtm(temp_dir.path());
    assert!(result.is_err());
}

#[test]
fn load_sdtm_missing_variables_csv_returns_error() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create only Datasets.csv
    std::fs::write(
        temp_dir.path().join("Datasets.csv"),
        "Version,Class,Dataset Name,Dataset Label,Structure\n",
    )
    .unwrap();

    let result = load_sdtm(temp_dir.path());
    assert!(result.is_err());
}

#[test]
fn load_adam_missing_datastructures_csv_returns_error() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create only Variables.csv
    std::fs::write(
        temp_dir.path().join("Variables.csv"),
        "Version,Data Structure Name,Variable Set,Variable Name,Variable Label,Type\n",
    )
    .unwrap();

    let result = load_adam(temp_dir.path());
    assert!(result.is_err());
}

// =============================================================================
// Result Type Tests
// =============================================================================

#[test]
fn result_ok_can_be_unwrapped() {
    let result: cdisc_metadata::Result<i32> = Ok(42);
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn result_err_can_be_checked() {
    let result: cdisc_metadata::Result<i32> = Err(Error::InvalidFormat("test".to_string()));
    assert!(result.is_err());
}
