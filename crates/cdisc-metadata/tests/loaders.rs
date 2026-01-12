//! Tests for loading functions and bundled standards.

use cdisc_metadata::{
    adam_ig_v1_3, data_dir, load_adam, load_sdtm, load_send, load_standard, sdtm_ig_v3_4,
    send_ig_v3_1_1, standard_path,
};

// =============================================================================
// Path Functions Tests
// =============================================================================

#[test]
fn data_dir_exists() {
    let dir = data_dir();
    assert!(dir.exists(), "Data directory should exist");
    assert!(dir.is_dir(), "Data directory should be a directory");
}

#[test]
fn data_dir_contains_standards() {
    let dir = data_dir();

    assert!(dir.join("sdtm").exists(), "SDTM directory should exist");
    assert!(dir.join("send").exists(), "SEND directory should exist");
    assert!(dir.join("adam").exists(), "ADaM directory should exist");
}

#[test]
fn standard_path_sdtm() {
    let path = standard_path("sdtm", "v3.4");
    assert!(path.exists(), "SDTM v3.4 path should exist");
    assert!(
        path.join("metadata.toml").exists(),
        "metadata.toml should exist"
    );
}

#[test]
fn standard_path_send() {
    let path = standard_path("send", "v3.1.1");
    assert!(path.exists(), "SEND v3.1.1 path should exist");
    assert!(
        path.join("metadata.toml").exists(),
        "metadata.toml should exist"
    );
}

#[test]
fn standard_path_adam() {
    let path = standard_path("adam", "v1.3");
    assert!(path.exists(), "ADaM v1.3 path should exist");
    assert!(
        path.join("metadata.toml").exists(),
        "metadata.toml should exist"
    );
}

// =============================================================================
// Bundled Standard Loaders Tests
// =============================================================================

#[test]
fn sdtm_ig_v3_4_loads_successfully() {
    let result = sdtm_ig_v3_4();
    assert!(
        result.is_ok(),
        "SDTM-IG v3.4 should load: {:?}",
        result.err()
    );
}

#[test]
fn sdtm_ig_v3_4_has_correct_metadata() {
    let standard = sdtm_ig_v3_4().unwrap();

    assert_eq!(standard.name, "SDTM-IG");
    assert_eq!(standard.version, "3.4");
    assert_eq!(standard.publishing_set, "SDTM");
    assert!(standard.effective_date.is_some());
}

#[test]
fn send_ig_v3_1_1_loads_successfully() {
    let result = send_ig_v3_1_1();
    assert!(
        result.is_ok(),
        "SEND-IG v3.1.1 should load: {:?}",
        result.err()
    );
}

#[test]
fn send_ig_v3_1_1_has_correct_metadata() {
    let standard = send_ig_v3_1_1().unwrap();

    assert_eq!(standard.name, "SEND-IG");
    assert_eq!(standard.version, "3.1.1");
    assert_eq!(standard.publishing_set, "SEND");
    assert!(standard.effective_date.is_some());
}

#[test]
fn adam_ig_v1_3_loads_successfully() {
    let result = adam_ig_v1_3();
    assert!(
        result.is_ok(),
        "ADaM-IG v1.3 should load: {:?}",
        result.err()
    );
}

#[test]
fn adam_ig_v1_3_has_correct_metadata() {
    let standard = adam_ig_v1_3().unwrap();

    assert_eq!(standard.name, "ADaM-IG");
    assert_eq!(standard.version, "1.3");
    assert_eq!(standard.publishing_set, "ADaM");
    assert!(standard.effective_date.is_some());
}

// =============================================================================
// Generic load_standard() Tests
// =============================================================================

#[test]
fn load_standard_sdtm() {
    let path = standard_path("sdtm", "v3.4");
    let result = load_standard(&path);

    assert!(result.is_ok(), "load_standard should work for SDTM");
    assert_eq!(result.unwrap().name, "SDTM-IG");
}

#[test]
fn load_standard_send() {
    let path = standard_path("send", "v3.1.1");
    let result = load_standard(&path);

    assert!(result.is_ok(), "load_standard should work for SEND");
    assert_eq!(result.unwrap().name, "SEND-IG");
}

#[test]
fn load_standard_adam() {
    let path = standard_path("adam", "v1.3");
    let result = load_standard(&path);

    assert!(result.is_ok(), "load_standard should work for ADaM");
    assert_eq!(result.unwrap().name, "ADaM-IG");
}

// =============================================================================
// Direct Standard Loaders Tests
// =============================================================================

#[test]
fn load_sdtm_direct() {
    let path = standard_path("sdtm", "v3.4");
    let result = load_sdtm(&path);

    assert!(result.is_ok(), "load_sdtm should succeed");
    let standard = result.unwrap();
    assert!(!standard.datasets.is_empty());
    assert!(!standard.variables.is_empty());
}

#[test]
fn load_send_direct() {
    let path = standard_path("send", "v3.1.1");
    let result = load_send(&path);

    assert!(result.is_ok(), "load_send should succeed");
    let standard = result.unwrap();
    assert!(!standard.datasets.is_empty());
    assert!(!standard.variables.is_empty());
}

#[test]
fn load_adam_direct() {
    let path = standard_path("adam", "v1.3");
    let result = load_adam(&path);

    assert!(result.is_ok(), "load_adam should succeed");
    let standard = result.unwrap();
    assert!(!standard.datasets.is_empty());
    assert!(!standard.variables.is_empty());
}

// =============================================================================
// Consistency Tests
// =============================================================================

#[test]
fn bundled_and_direct_loaders_produce_same_result_sdtm() {
    let bundled = sdtm_ig_v3_4().unwrap();
    let direct = load_sdtm(&standard_path("sdtm", "v3.4")).unwrap();

    assert_eq!(bundled.datasets.len(), direct.datasets.len());
    assert_eq!(bundled.variables.len(), direct.variables.len());
}

#[test]
fn bundled_and_direct_loaders_produce_same_result_send() {
    let bundled = send_ig_v3_1_1().unwrap();
    let direct = load_send(&standard_path("send", "v3.1.1")).unwrap();

    assert_eq!(bundled.datasets.len(), direct.datasets.len());
    assert_eq!(bundled.variables.len(), direct.variables.len());
}

#[test]
fn bundled_and_direct_loaders_produce_same_result_adam() {
    let bundled = adam_ig_v1_3().unwrap();
    let direct = load_adam(&standard_path("adam", "v1.3")).unwrap();

    assert_eq!(bundled.datasets.len(), direct.datasets.len());
    assert_eq!(bundled.variables.len(), direct.variables.len());
}

#[test]
fn load_standard_and_direct_produce_same_result() {
    let via_load_standard = load_standard(&standard_path("sdtm", "v3.4")).unwrap();
    let via_direct = load_sdtm(&standard_path("sdtm", "v3.4")).unwrap();

    assert_eq!(via_load_standard.datasets.len(), via_direct.datasets.len());
    assert_eq!(
        via_load_standard.variables.len(),
        via_direct.variables.len()
    );
}
