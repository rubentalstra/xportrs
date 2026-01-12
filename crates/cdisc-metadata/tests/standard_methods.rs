//! Tests for Standard struct methods.

use cdisc_metadata::{adam_ig_v1_3, sdtm_ig_v3_4, send_ig_v3_1_1};

// =============================================================================
// variables_for_dataset() Tests
// =============================================================================

#[test]
fn variables_for_dataset_returns_correct_variables() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let dm_vars = standard.variables_for_dataset("DM");

    assert!(!dm_vars.is_empty(), "DM should have variables");

    // All returned variables should belong to DM
    for var in &dm_vars {
        assert_eq!(
            var.dataset.to_uppercase(),
            "DM",
            "Variable {} should belong to DM",
            var.name
        );
    }
}

#[test]
fn variables_for_dataset_case_insensitive() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");

    let upper = standard.variables_for_dataset("DM");
    let lower = standard.variables_for_dataset("dm");
    let mixed = standard.variables_for_dataset("Dm");

    assert_eq!(upper.len(), lower.len());
    assert_eq!(upper.len(), mixed.len());
}

#[test]
fn variables_for_dataset_nonexistent_returns_empty() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let vars = standard.variables_for_dataset("NONEXISTENT");

    assert!(vars.is_empty());
}

#[test]
fn variables_for_dataset_preserves_order() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let dm_vars = standard.variables_for_dataset("DM");

    // Variables should be in order
    let mut prev_order = 0;
    for var in &dm_vars {
        assert!(
            var.order >= prev_order,
            "Variable order should be non-decreasing"
        );
        prev_order = var.order;
    }
}

#[test]
fn variables_for_dataset_sdtm_dm_has_studyid() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let dm_vars = standard.variables_for_dataset("DM");

    let has_studyid = dm_vars.iter().any(|v| v.name == "STUDYID");
    assert!(has_studyid, "DM should have STUDYID variable");
}

#[test]
fn variables_for_dataset_send_dm_has_studyid() {
    let standard = send_ig_v3_1_1().expect("Failed to load SEND");
    let dm_vars = standard.variables_for_dataset("DM");

    let has_studyid = dm_vars.iter().any(|v| v.name == "STUDYID");
    assert!(has_studyid, "DM should have STUDYID variable in SEND");
}

#[test]
fn variables_for_dataset_adam_adsl_has_studyid() {
    let standard = adam_ig_v1_3().expect("Failed to load ADaM");
    // ADaM Variables.csv uses full names, not abbreviations
    let adsl_vars = standard.variables_for_dataset("Subject-Level Analysis Dataset");

    let has_studyid = adsl_vars.iter().any(|v| v.name == "STUDYID");
    assert!(has_studyid, "ADSL should have STUDYID variable");
}

// =============================================================================
// dataset() Tests
// =============================================================================

#[test]
fn dataset_returns_correct_dataset() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let dm = standard.dataset("DM");

    assert!(dm.is_some());
    let dm = dm.unwrap();
    assert_eq!(dm.name, "DM");
}

#[test]
fn dataset_case_insensitive() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");

    let upper = standard.dataset("DM");
    let lower = standard.dataset("dm");
    let mixed = standard.dataset("Dm");

    assert!(upper.is_some());
    assert!(lower.is_some());
    assert!(mixed.is_some());

    assert_eq!(upper.unwrap().name, lower.unwrap().name);
    assert_eq!(upper.unwrap().name, mixed.unwrap().name);
}

#[test]
fn dataset_nonexistent_returns_none() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let result = standard.dataset("NONEXISTENT");

    assert!(result.is_none());
}

#[test]
fn dataset_has_label_and_class() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let dm = standard.dataset("DM").expect("DM should exist");

    assert!(!dm.label.is_empty(), "DM should have a label");
    assert!(!dm.class.is_empty(), "DM should have a class");
}

#[test]
fn dataset_adam_has_adsl() {
    let standard = adam_ig_v1_3().expect("Failed to load ADaM");
    let adsl = standard.dataset("ADSL");

    assert!(adsl.is_some(), "ADaM should have ADSL");
}

#[test]
fn dataset_adam_has_bds() {
    let standard = adam_ig_v1_3().expect("Failed to load ADaM");
    let bds = standard.dataset("BDS");

    assert!(bds.is_some(), "ADaM should have BDS");
}

// =============================================================================
// dataset_names() Tests
// =============================================================================

#[test]
fn dataset_names_returns_all_datasets() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let names = standard.dataset_names();

    assert_eq!(
        names.len(),
        standard.datasets.len(),
        "dataset_names should return all dataset names"
    );
}

#[test]
fn dataset_names_contains_dm() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let names = standard.dataset_names();

    assert!(
        names.contains(&"DM"),
        "SDTM dataset names should contain DM"
    );
}

#[test]
fn dataset_names_send_contains_dm() {
    let standard = send_ig_v3_1_1().expect("Failed to load SEND");
    let names = standard.dataset_names();

    assert!(
        names.contains(&"DM"),
        "SEND dataset names should contain DM"
    );
}

#[test]
fn dataset_names_adam_contains_adsl() {
    let standard = adam_ig_v1_3().expect("Failed to load ADaM");
    let names = standard.dataset_names();

    assert!(
        names.contains(&"ADSL"),
        "ADaM dataset names should contain ADSL"
    );
}

#[test]
fn dataset_names_no_duplicates() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let names = standard.dataset_names();

    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();

    assert_eq!(
        names.len(),
        sorted.len(),
        "dataset_names should not contain duplicates"
    );
}
