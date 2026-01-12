//! Tests for validating the bundled CDISC metadata content.

use cdisc_metadata::{adam_ig_v1_3, sdtm_ig_v3_4, send_ig_v3_1_1};

// =============================================================================
// SDTM Data Validation Tests
// =============================================================================

#[test]
fn sdtm_has_expected_dataset_count() {
    let standard = sdtm_ig_v3_4().unwrap();
    // SDTM-IG v3.4 has 63 datasets (64 lines minus header)
    assert!(
        standard.datasets.len() >= 60,
        "SDTM should have at least 60 datasets, got {}",
        standard.datasets.len()
    );
}

#[test]
fn sdtm_has_expected_variable_count() {
    let standard = sdtm_ig_v3_4().unwrap();
    // SDTM-IG v3.4 has ~1917 variables (1918 lines minus header)
    assert!(
        standard.variables.len() >= 1900,
        "SDTM should have at least 1900 variables, got {}",
        standard.variables.len()
    );
}

#[test]
fn sdtm_dm_domain_exists() {
    let standard = sdtm_ig_v3_4().unwrap();
    let dm = standard.dataset("DM");
    assert!(dm.is_some(), "SDTM should have DM (Demographics) domain");
}

#[test]
fn sdtm_ae_domain_exists() {
    let standard = sdtm_ig_v3_4().unwrap();
    let ae = standard.dataset("AE");
    assert!(ae.is_some(), "SDTM should have AE (Adverse Events) domain");
}

#[test]
fn sdtm_ex_domain_exists() {
    let standard = sdtm_ig_v3_4().unwrap();
    let ex = standard.dataset("EX");
    assert!(ex.is_some(), "SDTM should have EX (Exposure) domain");
}

#[test]
fn sdtm_dm_has_required_variables() {
    let standard = sdtm_ig_v3_4().unwrap();
    let dm_vars = standard.variables_for_dataset("DM");
    let var_names: Vec<&str> = dm_vars.iter().map(|v| v.name.as_str()).collect();

    assert!(var_names.contains(&"STUDYID"), "DM should have STUDYID");
    assert!(var_names.contains(&"USUBJID"), "DM should have USUBJID");
    assert!(var_names.contains(&"DOMAIN"), "DM should have DOMAIN");
}

#[test]
fn sdtm_variables_have_valid_types() {
    let standard = sdtm_ig_v3_4().unwrap();

    for var in &standard.variables {
        // Each variable should be either Char or Num
        assert!(
            var.var_type.is_character() || var.var_type.is_numeric(),
            "Variable {} should have valid type",
            var.name
        );
    }
}

#[test]
fn sdtm_variables_have_valid_order() {
    let standard = sdtm_ig_v3_4().unwrap();

    for var in &standard.variables {
        assert!(
            var.order > 0,
            "Variable {} should have positive order",
            var.name
        );
    }
}

#[test]
fn sdtm_variables_have_non_empty_names() {
    let standard = sdtm_ig_v3_4().unwrap();

    for var in &standard.variables {
        assert!(!var.name.is_empty(), "Variable should have non-empty name");
        assert!(
            var.name.len() <= 8,
            "SDTM variable name {} should be max 8 chars",
            var.name
        );
    }
}

#[test]
fn sdtm_variables_have_non_empty_labels() {
    let standard = sdtm_ig_v3_4().unwrap();

    for var in &standard.variables {
        assert!(
            !var.label.is_empty(),
            "Variable {} should have non-empty label",
            var.name
        );
    }
}

#[test]
fn sdtm_datasets_have_non_empty_names() {
    let standard = sdtm_ig_v3_4().unwrap();

    for dataset in &standard.datasets {
        assert!(
            !dataset.name.is_empty(),
            "Dataset should have non-empty name"
        );
    }
}

// =============================================================================
// SEND Data Validation Tests
// =============================================================================

#[test]
fn send_has_expected_dataset_count() {
    let standard = send_ig_v3_1_1().unwrap();
    // SEND-IG v3.1.1 has 29 datasets
    assert!(
        standard.datasets.len() >= 25,
        "SEND should have at least 25 datasets, got {}",
        standard.datasets.len()
    );
}

#[test]
fn send_has_expected_variable_count() {
    let standard = send_ig_v3_1_1().unwrap();
    // SEND-IG v3.1.1 has ~675 variables
    assert!(
        standard.variables.len() >= 650,
        "SEND should have at least 650 variables, got {}",
        standard.variables.len()
    );
}

#[test]
fn send_dm_domain_exists() {
    let standard = send_ig_v3_1_1().unwrap();
    let dm = standard.dataset("DM");
    assert!(dm.is_some(), "SEND should have DM domain");
}

#[test]
fn send_ts_domain_exists() {
    let standard = send_ig_v3_1_1().unwrap();
    let ts = standard.dataset("TS");
    assert!(ts.is_some(), "SEND should have TS (Trial Summary) domain");
}

#[test]
fn send_variables_have_valid_types() {
    let standard = send_ig_v3_1_1().unwrap();

    for var in &standard.variables {
        assert!(
            var.var_type.is_character() || var.var_type.is_numeric(),
            "Variable {} should have valid type",
            var.name
        );
    }
}

#[test]
fn send_dm_has_required_variables() {
    let standard = send_ig_v3_1_1().unwrap();
    let dm_vars = standard.variables_for_dataset("DM");
    let var_names: Vec<&str> = dm_vars.iter().map(|v| v.name.as_str()).collect();

    assert!(
        var_names.contains(&"STUDYID"),
        "SEND DM should have STUDYID"
    );
    assert!(
        var_names.contains(&"USUBJID"),
        "SEND DM should have USUBJID"
    );
}

// =============================================================================
// ADaM Data Validation Tests
// =============================================================================

#[test]
fn adam_has_expected_dataset_count() {
    let standard = adam_ig_v1_3().unwrap();
    // ADaM-IG v1.3 has 3 data structures: ADSL, BDS, TTE
    assert!(
        standard.datasets.len() >= 2,
        "ADaM should have at least 2 data structures, got {}",
        standard.datasets.len()
    );
}

#[test]
fn adam_has_expected_variable_count() {
    let standard = adam_ig_v1_3().unwrap();
    // ADaM-IG v1.3 has ~336 variables
    assert!(
        standard.variables.len() >= 300,
        "ADaM should have at least 300 variables, got {}",
        standard.variables.len()
    );
}

#[test]
fn adam_adsl_exists() {
    let standard = adam_ig_v1_3().unwrap();
    let adsl = standard.dataset("ADSL");
    assert!(adsl.is_some(), "ADaM should have ADSL data structure");
}

#[test]
fn adam_bds_exists() {
    let standard = adam_ig_v1_3().unwrap();
    let bds = standard.dataset("BDS");
    assert!(bds.is_some(), "ADaM should have BDS data structure");
}

#[test]
fn adam_adsl_has_required_variables() {
    let standard = adam_ig_v1_3().unwrap();
    // Note: ADaM Variables.csv uses full names like "Subject-Level Analysis Dataset"
    // while DataStructures.csv uses abbreviations like "ADSL"
    let adsl_vars = standard.variables_for_dataset("Subject-Level Analysis Dataset");
    let var_names: Vec<&str> = adsl_vars.iter().map(|v| v.name.as_str()).collect();

    assert!(var_names.contains(&"STUDYID"), "ADSL should have STUDYID");
    assert!(var_names.contains(&"USUBJID"), "ADSL should have USUBJID");
}

#[test]
fn adam_variables_have_valid_types() {
    let standard = adam_ig_v1_3().unwrap();

    for var in &standard.variables {
        assert!(
            var.var_type.is_character() || var.var_type.is_numeric(),
            "Variable {} should have valid type",
            var.name
        );
    }
}

// =============================================================================
// Cross-Standard Validation Tests
// =============================================================================

#[test]
fn all_standards_have_studyid_variable() {
    let sdtm = sdtm_ig_v3_4().unwrap();
    let send = send_ig_v3_1_1().unwrap();
    let adam = adam_ig_v1_3().unwrap();

    let sdtm_has_studyid = sdtm.variables.iter().any(|v| v.name == "STUDYID");
    let send_has_studyid = send.variables.iter().any(|v| v.name == "STUDYID");
    let adam_has_studyid = adam.variables.iter().any(|v| v.name == "STUDYID");

    assert!(sdtm_has_studyid, "SDTM should have STUDYID");
    assert!(send_has_studyid, "SEND should have STUDYID");
    assert!(adam_has_studyid, "ADaM should have STUDYID");
}

#[test]
fn all_standards_have_usubjid_variable() {
    let sdtm = sdtm_ig_v3_4().unwrap();
    let send = send_ig_v3_1_1().unwrap();
    let adam = adam_ig_v1_3().unwrap();

    let sdtm_has = sdtm.variables.iter().any(|v| v.name == "USUBJID");
    let send_has = send.variables.iter().any(|v| v.name == "USUBJID");
    let adam_has = adam.variables.iter().any(|v| v.name == "USUBJID");

    assert!(sdtm_has, "SDTM should have USUBJID");
    assert!(send_has, "SEND should have USUBJID");
    assert!(adam_has, "ADaM should have USUBJID");
}

#[test]
fn studyid_is_always_character_type() {
    let sdtm = sdtm_ig_v3_4().unwrap();
    let send = send_ig_v3_1_1().unwrap();
    let adam = adam_ig_v1_3().unwrap();

    for standard in [&sdtm, &send, &adam] {
        for var in &standard.variables {
            if var.name == "STUDYID" {
                assert!(
                    var.var_type.is_character(),
                    "STUDYID should be Char in {}",
                    standard.name
                );
            }
        }
    }
}

#[test]
fn usubjid_is_always_character_type() {
    let sdtm = sdtm_ig_v3_4().unwrap();
    let send = send_ig_v3_1_1().unwrap();
    let adam = adam_ig_v1_3().unwrap();

    for standard in [&sdtm, &send, &adam] {
        for var in &standard.variables {
            if var.name == "USUBJID" {
                assert!(
                    var.var_type.is_character(),
                    "USUBJID should be Char in {}",
                    standard.name
                );
            }
        }
    }
}

#[test]
fn all_standards_have_unique_dataset_names() {
    let sdtm = sdtm_ig_v3_4().unwrap();
    let send = send_ig_v3_1_1().unwrap();
    let adam = adam_ig_v1_3().unwrap();

    for standard in [&sdtm, &send, &adam] {
        let names = standard.dataset_names();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();

        assert_eq!(
            names.len(),
            sorted.len(),
            "{} should have unique dataset names",
            standard.name
        );
    }
}

#[test]
fn all_standards_have_effective_date() {
    let sdtm = sdtm_ig_v3_4().unwrap();
    let send = send_ig_v3_1_1().unwrap();
    let adam = adam_ig_v1_3().unwrap();

    assert!(
        sdtm.effective_date.is_some(),
        "SDTM should have effective_date"
    );
    assert!(
        send.effective_date.is_some(),
        "SEND should have effective_date"
    );
    assert!(
        adam.effective_date.is_some(),
        "ADaM should have effective_date"
    );
}
