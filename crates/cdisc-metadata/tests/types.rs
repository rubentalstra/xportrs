//! Tests for core types: VarType, Variable, DatasetDef, Standard.

use cdisc_metadata::{VarType, sdtm_ig_v3_4};

// =============================================================================
// VarType Tests
// =============================================================================

#[test]
fn var_type_char_is_character() {
    let vt = VarType::Char;
    assert!(vt.is_character());
    assert!(!vt.is_numeric());
}

#[test]
fn var_type_num_is_numeric() {
    let vt = VarType::Num;
    assert!(vt.is_numeric());
    assert!(!vt.is_character());
}

#[test]
fn var_type_equality() {
    assert_eq!(VarType::Char, VarType::Char);
    assert_eq!(VarType::Num, VarType::Num);
    assert_ne!(VarType::Char, VarType::Num);
}

#[test]
fn var_type_clone() {
    let vt = VarType::Char;
    let cloned = vt;
    assert_eq!(vt, cloned);
}

#[test]
fn var_type_debug() {
    let char_debug = format!("{:?}", VarType::Char);
    let num_debug = format!("{:?}", VarType::Num);
    assert_eq!(char_debug, "Char");
    assert_eq!(num_debug, "Num");
}

// =============================================================================
// Variable Tests
// =============================================================================

#[test]
fn variable_from_standard_has_required_fields() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let var = standard
        .variables
        .iter()
        .find(|v| v.name == "STUDYID")
        .expect("STUDYID should exist");

    assert_eq!(var.name, "STUDYID");
    assert!(!var.label.is_empty());
    assert!(!var.dataset.is_empty());
    assert!(var.order > 0);
}

#[test]
fn variable_types_are_correct() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");

    // STUDYID is always Char
    let studyid = standard
        .variables
        .iter()
        .find(|v| v.name == "STUDYID")
        .expect("STUDYID should exist");
    assert!(studyid.var_type.is_character());

    // Find a numeric variable (e.g., VISITNUM)
    let visitnum = standard.variables.iter().find(|v| v.name == "VISITNUM");
    if let Some(v) = visitnum {
        assert!(v.var_type.is_numeric());
    }
}

#[test]
fn variable_clone_preserves_all_fields() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let var = &standard.variables[0];
    let cloned = var.clone();

    assert_eq!(var.order, cloned.order);
    assert_eq!(var.name, cloned.name);
    assert_eq!(var.label, cloned.label);
    assert_eq!(var.var_type, cloned.var_type);
    assert_eq!(var.dataset, cloned.dataset);
    assert_eq!(var.role, cloned.role);
    assert_eq!(var.core, cloned.core);
    assert_eq!(var.notes, cloned.notes);
}

// =============================================================================
// DatasetDef Tests
// =============================================================================

#[test]
fn dataset_def_from_standard_has_required_fields() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let dm = standard.dataset("DM").expect("DM dataset should exist");

    assert_eq!(dm.name, "DM");
    assert!(!dm.label.is_empty());
    assert!(!dm.class.is_empty());
}

#[test]
fn dataset_def_clone_preserves_all_fields() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let dataset = &standard.datasets[0];
    let cloned = dataset.clone();

    assert_eq!(dataset.name, cloned.name);
    assert_eq!(dataset.label, cloned.label);
    assert_eq!(dataset.class, cloned.class);
    assert_eq!(dataset.structure, cloned.structure);
}

// =============================================================================
// Standard Tests
// =============================================================================

#[test]
fn standard_has_metadata() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");

    assert!(!standard.name.is_empty());
    assert!(!standard.version.is_empty());
    assert!(!standard.full_name.is_empty());
    assert!(!standard.publishing_set.is_empty());
}

#[test]
fn standard_clone_preserves_all_data() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let cloned = standard.clone();

    assert_eq!(standard.name, cloned.name);
    assert_eq!(standard.version, cloned.version);
    assert_eq!(standard.full_name, cloned.full_name);
    assert_eq!(standard.publishing_set, cloned.publishing_set);
    assert_eq!(standard.effective_date, cloned.effective_date);
    assert_eq!(standard.datasets.len(), cloned.datasets.len());
    assert_eq!(standard.variables.len(), cloned.variables.len());
}

#[test]
fn standard_debug_format() {
    let standard = sdtm_ig_v3_4().expect("Failed to load SDTM");
    let debug = format!("{:?}", standard);

    assert!(debug.contains("SDTM-IG"));
    assert!(debug.contains("3.4"));
}
