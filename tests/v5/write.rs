//! Tests for writing XPT v5 files.

use std::path::PathBuf;

use tempfile::tempdir;
use xportrs::{Agency, Column, ColumnData, Dataset, Format, Xpt};

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

/// Test metadata (labels, formats) roundtrip.
#[test]
fn test_metadata_roundtrip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("ae_metadata.xpt");

    // Create dataset with full CDISC metadata
    let dataset = Dataset::with_label("AE", "Adverse Events", vec![
        Column::new("STUDYID", ColumnData::String(vec![Some("STUDY123".into())]))
            .with_label("Study Identifier")
            .with_format(Format::character(20))
            .with_length(20), // explicit length override
        Column::new("USUBJID", ColumnData::String(vec![Some("001-001".into())]))
            .with_label("Unique Subject Identifier")
            .with_format(Format::character(40))
            .with_length(40), // explicit length override
        Column::new("AESEQ", ColumnData::F64(vec![Some(1.0)]))
            .with_label("Sequence Number")
            .with_format(Format::numeric(8, 0)),
        Column::new("AESTDTC", ColumnData::F64(vec![Some(21185.0)]))
            .with_label("Start Date/Time")
            .with_format_str("DATE9.")
            .unwrap(),
    ])
    .unwrap();

    // Verify original dataset has metadata
    assert_eq!(dataset.dataset_label().unwrap(), "Adverse Events");
    assert_eq!(dataset.columns()[0].label().unwrap().to_string(), "Study Identifier");
    assert!(dataset.columns()[0].format().is_some());

    // Write
    Xpt::writer(dataset.clone())
        .finalize()
        .unwrap()
        .write_path(&path)
        .unwrap();

    // Read back
    let loaded = Xpt::read(&path).unwrap();

    // Verify dataset label preserved
    assert_eq!(loaded.dataset_label().unwrap(), "Adverse Events");

    // Verify variable labels preserved
    assert_eq!(loaded.columns()[0].label().unwrap().to_string(), "Study Identifier");
    assert_eq!(loaded.columns()[1].label().unwrap().to_string(), "Unique Subject Identifier");
    assert_eq!(loaded.columns()[2].label().unwrap().to_string(), "Sequence Number");
    assert_eq!(loaded.columns()[3].label().unwrap().to_string(), "Start Date/Time");

    // Verify formats preserved
    let format0 = loaded.columns()[0].format().expect("STUDYID should have format");
    assert_eq!(format0.name_without_prefix(), "CHAR");
    assert_eq!(format0.length(), 20);

    // Note: Numeric formats with bare width (like "8.") are not written to XPT
    // as they have no format name. Only named formats like DATE9. are preserved.
    // The AESEQ column used Format::numeric(8, 0) which creates a bare format.

    let format3 = loaded.columns()[3].format().expect("AESTDTC should have format");
    assert_eq!(format3.name_without_prefix(), "DATE");
    assert_eq!(format3.length(), 9);

    // Verify explicit length preserved for character variables
    assert_eq!(loaded.columns()[0].explicit_length(), Some(20));
    assert_eq!(loaded.columns()[1].explicit_length(), Some(40));
}

/// Test that missing labels generate warnings.
#[test]
fn test_missing_labels_warning() {
    let dataset = Dataset::new(
        "AE",
        vec![
            Column::new("USUBJID", ColumnData::String(vec![Some("001".into())])),
            Column::new("AESEQ", ColumnData::F64(vec![Some(1.0)])),
        ],
    )
    .unwrap();

    // Without labels, validation should produce warnings
    let validated = Xpt::writer(dataset).finalize().unwrap();

    // Should have warnings for missing labels (but no errors)
    assert!(validated.has_warnings());
    assert!(!validated.has_errors());

    // Check that the issues include missing label warnings
    let issues = validated.issues();
    assert!(
        issues.iter().any(|i| format!("{}", i).contains("missing a label")),
        "Expected missing label warnings, got: {:?}",
        issues
    );
}

/// Test Format parsing.
#[test]
fn test_format_parsing() {
    // Date format
    let date_format = Format::parse("DATE9.").unwrap();
    assert_eq!(date_format.name_without_prefix(), "DATE");
    assert_eq!(date_format.length(), 9);
    assert_eq!(date_format.decimals(), 0);

    // Numeric format with decimals
    let num_format = Format::parse("8.2").unwrap();
    assert_eq!(num_format.name(), "");
    assert_eq!(num_format.length(), 8);
    assert_eq!(num_format.decimals(), 2);

    // Character format
    let char_format = Format::parse("$CHAR200.").unwrap();
    assert_eq!(char_format.name_without_prefix(), "CHAR");
    assert_eq!(char_format.length(), 200);
    assert!(char_format.is_character());

    // BEST format
    let best_format = Format::parse("BEST12.").unwrap();
    assert_eq!(best_format.name_without_prefix(), "BEST");
    assert_eq!(best_format.length(), 12);
}

/// Test Format constructors.
#[test]
fn test_format_constructors() {
    // Numeric constructor
    let num = Format::numeric(8, 2);
    assert_eq!(num.length(), 8);
    assert_eq!(num.decimals(), 2);
    assert!(!num.is_character());

    // Character constructor
    let char_fmt = Format::character(200);
    assert_eq!(char_fmt.name_without_prefix(), "CHAR");
    assert_eq!(char_fmt.length(), 200);
    assert!(char_fmt.is_character());
}
