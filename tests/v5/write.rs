//! Tests for writing XPT v5 files.

use std::path::PathBuf;

use tempfile::tempdir;
use xportrs::{Agency, Column, ColumnData, Dataset, Xpt};

/// Get the path to test data directory.
fn test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

/// Test basic write and read roundtrip.
#[test]
fn test_write_read_roundtrip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("ae.xpt");

    // Create dataset
    let dataset = Dataset::new(
        "AE",
        vec![
            Column::new(
                "USUBJID",
                ColumnData::String(vec![Some("001".into()), Some("002".into())]),
            ),
            Column::new("AESEQ", ColumnData::F64(vec![Some(1.0), Some(2.0)])),
        ],
    )
    .unwrap();

    let original_nrows = dataset.nrows();

    // Write
    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_path(&path)
        .unwrap();

    // Read back
    let loaded = Xpt::read(&path).unwrap();

    assert_eq!(loaded.domain_code(), "AE");
    assert_eq!(loaded.ncols(), 2);
    assert!(!loaded.columns().is_empty());
    assert_eq!(loaded.columns()[0].len(), original_nrows);
}

/// Test FDA agency validation.
#[test]
fn test_fda_agency_validation() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("dm.xpt");

    let dataset = Dataset::new(
        "DM",
        vec![
            Column::new("STUDYID", ColumnData::String(vec![Some("STUDY001".into())])),
            Column::new("AGE", ColumnData::F64(vec![Some(45.0)])),
        ],
    )
    .unwrap();

    let mut builder = Xpt::writer(dataset);
    builder.agency(Agency::FDA);
    let validated = builder.finalize().unwrap();

    // Should have no blocking errors for valid data
    assert!(!validated.has_errors());

    validated.write_path(&path).unwrap();

    let loaded = Xpt::read(&path).unwrap();
    assert_eq!(loaded.domain_code(), "DM");
}

/// Test column length mismatch detection.
#[test]
fn test_column_length_mismatch() {
    let result = Dataset::new(
        "AE",
        vec![
            Column::new("A", ColumnData::F64(vec![Some(1.0), Some(2.0)])),
            Column::new("B", ColumnData::F64(vec![Some(1.0)])), // Wrong length
        ],
    );

    assert!(result.is_err());
}

/// Test From conversions for `ColumnData`.
#[test]
fn test_column_data_from_conversions() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("conv.xpt");

    // Use From conversions
    let dataset = Dataset::new(
        "LB",
        vec![
            Column::new("LBSEQ", vec![1.0f64, 2.0, 3.0].into()),
            Column::new("LBTEST", vec!["HGB", "WBC", "PLT"].into()),
        ],
    )
    .unwrap();

    let original_nrows = dataset.nrows();
    assert_eq!(original_nrows, 3);

    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_path(&path)
        .unwrap();

    let loaded = Xpt::read(&path).unwrap();
    assert_eq!(loaded.columns()[0].len(), original_nrows);
}

/// Test write to in-memory buffer.
#[test]
fn test_write_to_buffer() {
    let dataset = Dataset::new(
        "CM",
        vec![Column::new("CMSEQ", ColumnData::F64(vec![Some(1.0)]))],
    )
    .unwrap();

    let mut buffer = Vec::new();
    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_to(&mut buffer)
        .unwrap();

    // Should contain XPT header
    assert!(!buffer.is_empty());
    assert!(buffer.starts_with(b"HEADER RECORD"));
}

/// Test with missing values.
#[test]
fn test_missing_values() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("vs.xpt");

    let dataset = Dataset::new(
        "VS",
        vec![
            Column::new(
                "USUBJID",
                ColumnData::String(vec![Some("001".into()), None]),
            ),
            Column::new("VSSTRESN", ColumnData::F64(vec![Some(120.0), None])),
        ],
    )
    .unwrap();

    let original_nrows = dataset.nrows();

    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_path(&path)
        .unwrap();

    let loaded = Xpt::read(&path).unwrap();
    assert_eq!(loaded.columns()[0].len(), original_nrows);
}

/// Test multiple columns with various types.
#[test]
fn test_multiple_column_types() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("mixed.xpt");

    let dataset = Dataset::new(
        "AE",
        vec![
            Column::new("USUBJID", ColumnData::String(vec![Some("SUBJ001".into())])),
            Column::new("AESEQ", ColumnData::F64(vec![Some(1.0)])),
            Column::new("AESTDY", ColumnData::F64(vec![Some(15.0)])),
            Column::new("AETERM", ColumnData::String(vec![Some("Headache".into())])),
        ],
    )
    .unwrap();

    let original_nrows = dataset.nrows();

    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_path(&path)
        .unwrap();

    let loaded = Xpt::read(&path).unwrap();
    assert_eq!(loaded.ncols(), 4);
    assert_eq!(loaded.columns()[0].len(), original_nrows);
}

/// Test that all agencies can be used.
#[test]
fn test_all_agencies() {
    let dir = tempdir().unwrap();

    for (agency, name) in [
        (Agency::FDA, "fda.xpt"),
        (Agency::PMDA, "pmda.xpt"),
        (Agency::NMPA, "nmpa.xpt"),
    ] {
        let path = dir.path().join(name);

        let dataset = Dataset::new(
            "AE",
            vec![Column::new("AESEQ", ColumnData::F64(vec![Some(1.0)]))],
        )
        .unwrap();

        let mut builder = Xpt::writer(dataset);
        builder.agency(agency);
        let validated = builder.finalize().unwrap();
        assert!(!validated.has_errors());
        validated.write_path(&path).unwrap();

        // Verify file was created
        assert!(path.exists());
    }
}

/// Test round-trip preserves data from real XPT files.
#[test]
fn test_roundtrip_real_files() {
    let dir = tempdir().unwrap();

    for filename in ["dm.xpt", "relrec.xpt", "suppdm.xpt"] {
        let original_path = test_data_dir().join(filename);
        let output_path = dir.path().join(format!("roundtrip_{}", filename));

        // Read original
        let original = Xpt::read(&original_path).expect("Failed to read original");

        // Write copy
        Xpt::writer(original.clone())
            .finalize()
            .unwrap()
            .write_path(&output_path)
            .unwrap();

        // Read back
        let reloaded = Xpt::read(&output_path).expect("Failed to read roundtrip");

        // Verify structure preserved
        assert_eq!(
            original.domain_code(),
            reloaded.domain_code(),
            "{} domain code mismatch",
            filename
        );
        assert_eq!(
            original.ncols(),
            reloaded.ncols(),
            "{} column count mismatch",
            filename
        );
        assert_eq!(
            original.nrows(),
            reloaded.nrows(),
            "{} row count mismatch",
            filename
        );
    }
}
