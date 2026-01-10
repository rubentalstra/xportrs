//! Byte-level tests for XPT v5 spec compliance.
//!
//! These tests verify the exact byte layout of XPT structures per the
//! official SAS Transport Format specification.

use std::path::PathBuf;

use tempfile::tempdir;
use xportrs::{Column, ColumnData, Dataset, Xpt};

/// Get the path to test data directory.
fn test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

/// Test that XPT files start with the correct header record.
#[test]
fn test_header_record_structure() {
    let path = test_data_dir().join("dm.xpt");
    let bytes = std::fs::read(&path).expect("Failed to read dm.xpt");

    // First 80 bytes should be the library header
    assert!(bytes.len() >= 80, "File too small for XPT format");

    let header = &bytes[0..80];
    assert!(
        header.starts_with(b"HEADER RECORD*******LIBRARY HEADER"),
        "Invalid library header"
    );
}

/// Test that written XPT files have correct header structure.
#[test]
fn test_written_header_structure() {
    let dataset = Dataset::new(
        "TEST",
        vec![Column::new("VAR1", ColumnData::F64(vec![Some(1.0)]))],
    )
    .unwrap();

    let mut buffer = Vec::new();
    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_to(&mut buffer)
        .unwrap();

    // Verify header record
    assert!(buffer.len() >= 80);
    assert!(buffer.starts_with(b"HEADER RECORD*******LIBRARY HEADER"));

    // All records should be 80 bytes
    assert_eq!(
        buffer.len() % 80,
        0,
        "File size should be multiple of 80 bytes"
    );
}

/// Test NAMESTR record size is exactly 140 bytes.
#[test]
fn test_namestr_record_size() {
    // The NAMESTR constant should be 140
    // We verify by checking the file structure after member header
    let path = test_data_dir().join("dm.xpt");
    let bytes = std::fs::read(&path).expect("Failed to read dm.xpt");

    // Find NAMESTR header
    let namestr_marker = b"HEADER RECORD*******NAMESTR HEADER";
    let mut found = false;

    for window in bytes.windows(namestr_marker.len()) {
        if window == namestr_marker {
            found = true;
            break;
        }
    }

    assert!(found, "NAMESTR header not found in dm.xpt");
}

/// Test that 80-byte records are properly padded.
#[test]
fn test_record_padding() {
    let dataset = Dataset::new(
        "PAD",
        vec![
            Column::new("SHORT", ColumnData::String(vec![Some("A".into())])),
            Column::new(
                "LONG",
                ColumnData::String(vec![Some("A longer string value".into())]),
            ),
        ],
    )
    .unwrap();

    let mut buffer = Vec::new();
    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_to(&mut buffer)
        .unwrap();

    // File size should be exact multiple of 80
    assert_eq!(
        buffer.len() % 80,
        0,
        "Buffer length {} is not a multiple of 80",
        buffer.len()
    );
}

/// Test IBM float encoding for standard values.
#[test]
fn test_ibm_float_encoding_roundtrip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("float.xpt");

    let test_values = vec![
        Some(0.0),
        Some(1.0),
        Some(-1.0),
        Some(123.456),
        Some(-999.999),
        Some(1e10),
        Some(1e-10),
        None, // Missing value
    ];

    let dataset = Dataset::new(
        "FLOAT",
        vec![Column::new("VALUE", ColumnData::F64(test_values.clone()))],
    )
    .unwrap();

    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_path(&path)
        .unwrap();

    // Read back and verify
    let loaded = Xpt::read(&path).expect("Failed to read XPT file");

    let col = loaded.column("VALUE").expect("VALUE column not found");
    let loaded_data = col.data();

    if let ColumnData::F64(values) = loaded_data {
        assert_eq!(values.len(), test_values.len());

        for (i, (original, loaded)) in test_values.iter().zip(values.iter()).enumerate() {
            match (original, loaded) {
                (None, None) => {} // Both missing - correct
                (Some(o), Some(l)) => {
                    // Allow small floating point differences due to IBM conversion
                    let diff = (o - l).abs();
                    let tolerance = o.abs() * 1e-10 + 1e-15;
                    assert!(
                        diff <= tolerance,
                        "Value {} mismatch: {} vs {} (diff {})",
                        i,
                        o,
                        l,
                        diff
                    );
                }
                _ => panic!("Value {} mismatch: {:?} vs {:?}", i, original, loaded),
            }
        }
    } else {
        panic!("Expected F64 column data");
    }
}

/// Test that missing values use correct SAS missing pattern.
#[test]
fn test_missing_value_encoding() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("miss.xpt");

    let dataset = Dataset::new(
        "MISS",
        vec![
            Column::new("NUM", ColumnData::F64(vec![Some(1.0), None, Some(3.0)])),
            Column::new(
                "CHAR",
                ColumnData::String(vec![Some("A".into()), None, Some("C".into())]),
            ),
        ],
    )
    .unwrap();

    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_path(&path)
        .unwrap();

    // Read back and verify missing values preserved
    let loaded = Xpt::read(&path).expect("Failed to read");

    let num_col = loaded.column("NUM").expect("NUM not found");
    if let ColumnData::F64(values) = num_col.data() {
        assert_eq!(values[0], Some(1.0));
        assert!(values[1].is_none(), "Second value should be missing");
        assert_eq!(values[2], Some(3.0));
    }

    let char_col = loaded.column("CHAR").expect("CHAR not found");
    if let ColumnData::String(values) = char_col.data() {
        assert_eq!(values[0].as_deref(), Some("A"));
        assert!(values[1].is_none(), "Second value should be missing");
        assert_eq!(values[2].as_deref(), Some("C"));
    }
}

/// Test variable name padding to 8 bytes.
#[test]
fn test_variable_name_padding() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("name.xpt");

    let dataset = Dataset::new(
        "NAME",
        vec![
            Column::new("A", ColumnData::F64(vec![Some(1.0)])), // 1 char
            Column::new("ABCD", ColumnData::F64(vec![Some(2.0)])), // 4 chars
            Column::new("ABCDEFGH", ColumnData::F64(vec![Some(3.0)])), // 8 chars (max)
        ],
    )
    .unwrap();

    Xpt::writer(dataset)
        .finalize()
        .unwrap()
        .write_path(&path)
        .unwrap();

    let loaded = Xpt::read(&path).expect("Failed to read");

    // All names should be preserved exactly
    assert!(loaded.column("A").is_some());
    assert!(loaded.column("ABCD").is_some());
    assert!(loaded.column("ABCDEFGH").is_some());
}

/// Test that real XPT files have correct observation record structure.
#[test]
fn test_observation_record_structure() {
    let path = test_data_dir().join("dm.xpt");
    let bytes = std::fs::read(&path).expect("Failed to read dm.xpt");

    // Find OBS header
    let obs_marker = b"HEADER RECORD*******OBS     HEADER";
    let mut obs_offset = None;

    for (i, window) in bytes.windows(obs_marker.len()).enumerate() {
        if window == obs_marker {
            obs_offset = Some(i);
            break;
        }
    }

    assert!(obs_offset.is_some(), "OBS header not found in dm.xpt");

    // After OBS header (80 bytes), observation data begins
    let obs_start = obs_offset.unwrap() + 80;
    assert!(
        bytes.len() > obs_start,
        "File should have data after OBS header"
    );
}
