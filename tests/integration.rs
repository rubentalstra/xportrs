//! Integration tests for xportrs.
//!
//! These tests verify the full read/write cycle and various edge cases.

use tempfile::tempdir;
use xportrs::{Agency, Column, ColumnData, Dataset, Xpt};

#[test]
fn test_roundtrip_simple() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.xpt");

    // Create a simple dataset
    let original = Dataset::new(
        "AE",
        vec![
            Column::new(
                "USUBJID",
                ColumnData::String(vec![
                    Some("001".into()),
                    Some("002".into()),
                    Some("003".into()),
                ]),
            ),
            Column::new("AESEQ", ColumnData::I64(vec![Some(1), Some(2), Some(1)])),
            Column::new(
                "AESTDY",
                ColumnData::F64(vec![Some(10.0), Some(15.5), Some(20.0)]),
            ),
        ],
    )
    .unwrap();

    // Write the dataset
    Xpt::writer(original.clone())
        .finalize()
        .unwrap()
        .write_path(&path)
        .unwrap();

    // Read it back
    let loaded = Xpt::read(&path).unwrap();

    // Verify
    assert_eq!(loaded.domain_code(), "AE");
    assert_eq!(loaded.nrows(), 3);
    assert_eq!(loaded.ncols(), 3);
}

#[test]
fn test_roundtrip_with_fda_agency() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("dm.xpt");

    let dataset = Dataset::new(
        "DM",
        vec![
            Column::new(
                "STUDYID",
                ColumnData::String(vec![Some("STUDY001".into()), Some("STUDY001".into())]),
            ),
            Column::new(
                "USUBJID",
                ColumnData::String(vec![Some("001".into()), Some("002".into())]),
            ),
            Column::new("AGE", ColumnData::I64(vec![Some(45), Some(62)])),
        ],
    )
    .unwrap();

    // Write with FDA agency
    let validated = Xpt::writer(dataset).agency(Agency::FDA).finalize().unwrap();

    // Should have no errors
    assert!(!validated.has_errors());

    validated.write_path(&path).unwrap();

    // Read back
    let loaded = Xpt::read(&path).unwrap();
    assert_eq!(loaded.domain_code(), "DM");
    assert_eq!(loaded.nrows(), 2);
}

#[test]
fn test_empty_dataset() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("empty.xpt");

    // Create an empty dataset (no rows)
    let dataset = Dataset::new("TE", vec![]).unwrap();

    assert_eq!(dataset.nrows(), 0);
    assert_eq!(dataset.ncols(), 0);

    // Write and read back
    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_path(&path)
        .unwrap();

    let loaded = Xpt::read(&path).unwrap();
    assert_eq!(loaded.nrows(), 0);
    assert_eq!(loaded.ncols(), 0);
}

#[test]
fn test_missing_values() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("missing.xpt");

    // Create dataset with missing values
    let dataset = Dataset::new(
        "VS",
        vec![
            Column::new(
                "USUBJID",
                ColumnData::String(vec![Some("001".into()), None, Some("003".into())]),
            ),
            Column::new(
                "VSSTRESN",
                ColumnData::F64(vec![Some(120.0), Some(130.0), None]),
            ),
        ],
    )
    .unwrap();

    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_path(&path)
        .unwrap();

    let loaded = Xpt::read(&path).unwrap();
    assert_eq!(loaded.nrows(), 3);
}

#[test]
fn test_from_conversions() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("conv.xpt");

    // Use From conversions for ColumnData
    let dataset = Dataset::new(
        "LB",
        vec![
            Column::new("LBSEQ", vec![1i64, 2, 3].into()),
            Column::new("LBSTRESN", vec![1.5, 2.5, 3.5].into()),
            Column::new("LBTEST", vec!["HGB", "WBC", "PLT"].into()),
        ],
    )
    .unwrap();

    assert_eq!(dataset.nrows(), 3);
    assert_eq!(dataset.ncols(), 3);

    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_path(&path)
        .unwrap();

    let loaded = Xpt::read(&path).unwrap();
    assert_eq!(loaded.nrows(), 3);
}

#[test]
fn test_inspect() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("inspect.xpt");

    // Create and write a dataset
    let dataset = Dataset::new(
        "EX",
        vec![
            Column::new("USUBJID", vec!["001", "002"].into()),
            Column::new("EXSEQ", vec![1i64, 1].into()),
        ],
    )
    .unwrap();

    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_path(&path)
        .unwrap();

    // Inspect without loading data
    let info = Xpt::inspect(&path).unwrap();

    assert_eq!(info.members.len(), 1);
    let member_names: Vec<_> = info.member_names().collect();
    assert!(member_names.contains(&"EX"));
}

#[test]
fn test_column_length_mismatch_error() {
    // This should fail because columns have different lengths
    let result = Dataset::new(
        "AE",
        vec![
            Column::new("A", ColumnData::I64(vec![Some(1), Some(2)])),
            Column::new("B", ColumnData::I64(vec![Some(1)])), // Different length
        ],
    );

    assert!(result.is_err());
}

#[test]
fn test_write_to_writer() {
    let dataset = Dataset::new("CM", vec![Column::new("CMSEQ", vec![1i64, 2, 3].into())]).unwrap();

    let mut buffer = Vec::new();
    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_to(&mut buffer)
        .unwrap();

    // Buffer should contain XPT data
    assert!(!buffer.is_empty());

    // Should start with XPT header
    assert!(buffer.starts_with(b"HEADER RECORD"));
}
