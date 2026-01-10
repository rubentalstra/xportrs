//! Tests for reading XPT v5 files.

use std::path::PathBuf;

use cdisc_metadata::load_standard;
use xportrs::Xpt;

/// Get the path to test data directory.
fn test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

/// Test reading dm.xpt (Demographics domain).
#[test]
fn test_read_dm() {
    let path = test_data_dir().join("dm.xpt");
    let dataset = Xpt::read(&path).expect("Failed to read dm.xpt");

    assert_eq!(dataset.domain_code(), "DM");
    assert!(dataset.ncols() > 0, "DM should have columns");
    assert!(dataset.nrows() > 0, "DM should have rows");

    // DM should have standard identifier variables
    assert!(
        dataset.column("STUDYID").is_some(),
        "DM should have STUDYID"
    );
    assert!(
        dataset.column("USUBJID").is_some(),
        "DM should have USUBJID"
    );
    assert!(dataset.column("DOMAIN").is_some(), "DM should have DOMAIN");
}

/// Test reading lb.xpt (Laboratory domain - large file).
#[test]
fn test_read_lb() {
    let path = test_data_dir().join("lb.xpt");
    let dataset = Xpt::read(&path).expect("Failed to read lb.xpt");

    assert_eq!(dataset.domain_code(), "LB");
    assert!(dataset.ncols() > 0, "LB should have columns");
    assert!(dataset.nrows() > 0, "LB should have rows");

    // LB is a large file, verify it has significant data
    assert!(dataset.nrows() > 100, "LB should have many rows");
}

/// Test reading relrec.xpt (Related Records domain).
#[test]
fn test_read_relrec() {
    let path = test_data_dir().join("relrec.xpt");
    let dataset = Xpt::read(&path).expect("Failed to read relrec.xpt");

    assert_eq!(dataset.domain_code(), "RELREC");
    assert!(dataset.ncols() > 0, "RELREC should have columns");
}

/// Test reading suppdm.xpt (Supplementary DM domain).
#[test]
fn test_read_suppdm() {
    let path = test_data_dir().join("suppdm.xpt");
    let dataset = Xpt::read(&path).expect("Failed to read suppdm.xpt");

    assert_eq!(dataset.domain_code(), "SUPPDM");
    assert!(dataset.ncols() > 0, "SUPPDM should have columns");

    // SUPPQUAL domains should have standard variables
    assert!(
        dataset.column("RDOMAIN").is_some(),
        "SUPPDM should have RDOMAIN"
    );
    assert!(dataset.column("QNAM").is_some(), "SUPPDM should have QNAM");
    assert!(dataset.column("QVAL").is_some(), "SUPPDM should have QVAL");
}

/// Test that DM variables match SDTM metadata expectations.
#[test]
fn test_dm_matches_sdtm_metadata() {
    let xpt_path = test_data_dir().join("dm.xpt");
    let metadata_dir = test_data_dir().join("sdtm/ig/v3.4");

    let dataset = Xpt::read(&xpt_path).expect("Failed to read dm.xpt");
    let standard = load_standard(&metadata_dir).expect("Failed to load SDTM metadata");

    // Get expected DM variables from metadata
    let expected_vars = standard.variables_for_dataset("DM");
    assert!(!expected_vars.is_empty(), "SDTM should define DM variables");

    // Check that required variables are present
    for var in expected_vars
        .iter()
        .filter(|v| v.core.as_deref() == Some("Req"))
    {
        let col = dataset.column(&var.name);
        assert!(
            col.is_some(),
            "DM should have required variable: {}",
            var.name
        );

        // Verify type matches
        if let Some(col) = col {
            if var.var_type.is_numeric() {
                assert!(col.is_numeric(), "Variable {} should be numeric", var.name);
            } else {
                assert!(
                    col.is_character(),
                    "Variable {} should be character",
                    var.name
                );
            }
        }
    }
}

/// Test file inspection without loading full data.
#[test]
fn test_inspect_dm() {
    let path = test_data_dir().join("dm.xpt");
    let info = Xpt::inspect(&path).expect("Failed to inspect dm.xpt");

    assert_eq!(info.members.len(), 1);
    let names: Vec<_> = info.member_names().collect();
    assert!(names.contains(&"DM"));
}

/// Test reading all XPT files in test data directory.
#[test]
fn test_read_all_xpt_files() {
    let xpt_files = ["dm.xpt", "lb.xpt", "relrec.xpt", "suppdm.xpt"];

    for filename in xpt_files {
        let path = test_data_dir().join(filename);
        let result = Xpt::read(&path);
        assert!(
            result.is_ok(),
            "Failed to read {}: {:?}",
            filename,
            result.err()
        );

        let dataset = result.unwrap();
        assert!(dataset.ncols() > 0, "{} should have columns", filename);
    }
}
