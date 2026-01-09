//! FDA compliance validation example.
//!
//! This example demonstrates how to validate datasets against FDA requirements
//! using strict compliance checking.
//!
//! Run with: `cargo run --example fda_compliance`

use xportrs::policy::{AgencyPolicy, FdaPolicy};
use xportrs::spec::{DatasetSpec, VariableSpec};
use xportrs::validation::{Validator, rules::SpecConformanceConfig};
use xportrs::{ActionLevel, XptColumn, XptDataset};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FDA Compliance Validation Example ===\n");

    // Define a specification
    let spec = DatasetSpec::new("DM")
        .with_label("Demographics")
        .add_variable(
            VariableSpec::character("USUBJID", 20)
                .with_label("Unique Subject Identifier")
                .with_order(1),
        )
        .add_variable(VariableSpec::numeric("AGE").with_label("Age").with_order(2))
        .add_variable(
            VariableSpec::character("SEX", 1)
                .with_label("Sex")
                .with_order(3),
        );

    // Example 1: Valid FDA-compliant dataset
    println!("Example 1: Valid FDA-Compliant Dataset\n");

    let mut valid_dataset = XptDataset::new("DM");
    valid_dataset.label = Some("Demographics".to_string());
    valid_dataset
        .columns
        .push(XptColumn::character("USUBJID", 20).with_label("Unique Subject Identifier"));
    valid_dataset
        .columns
        .push(XptColumn::numeric("AGE").with_label("Age"));
    valid_dataset
        .columns
        .push(XptColumn::character("SEX", 1).with_label("Sex"));

    let validator = Validator::fda();
    let result = validator.validate(&valid_dataset);

    println!("  Dataset: {}", valid_dataset.name);
    println!("  Result: {}", result);
    if result.is_fda_compliant() {
        println!("  Status: PASS - Dataset is FDA compliant!\n");
    }

    // Example 2: Dataset with FDA violations
    println!("Example 2: Dataset with FDA Violations\n");

    let mut invalid_dataset = XptDataset::new("DEMOGRAPHICS"); // Name too long for V5
    invalid_dataset.label = Some(
        "A label that is way too long for FDA V5 compliance - it exceeds the 40 character limit"
            .to_string(),
    );
    invalid_dataset.columns.push(
        XptColumn::character("SUBJECT_IDENTIFIER", 20), // Name too long for V5
    );

    let result = validator.validate(&invalid_dataset);

    println!("  Dataset: {}", invalid_dataset.name);
    println!("  Result: {}", result);
    println!("  Errors:");
    for error in &result.errors {
        println!("    - [{}] {}", error.code, error.message);
    }
    println!();

    // Example 3: Spec conformance validation with strict checking
    println!("Example 3: Strict Spec Conformance\n");

    let mut mismatched_dataset = XptDataset::new("DM");
    mismatched_dataset.label = Some("Demographics".to_string());
    // Type mismatch: AGE should be numeric but we make it character
    mismatched_dataset
        .columns
        .push(XptColumn::character("AGE", 10));
    // Missing USUBJID
    // Extra column not in spec
    mismatched_dataset
        .columns
        .push(XptColumn::character("EXTRA", 10));

    let result = validator.validate_against_spec_strict(&mismatched_dataset, &spec);

    println!("  Dataset: {}", mismatched_dataset.name);
    println!("  Result: {}", result);
    println!("  Issues:");
    for error in result.errors.iter().chain(result.warnings.iter()) {
        println!(
            "    - [{}] ({}) {}",
            error.code, error.severity, error.message
        );
    }
    println!();

    // Example 4: Custom conformance config
    println!("Example 4: Custom Conformance Configuration\n");

    let custom_config = SpecConformanceConfig {
        variable_in_spec_action: ActionLevel::Warn, // Warn about extra variables
        variable_in_data_action: ActionLevel::Stop, // Error on missing variables
        type_action: ActionLevel::Stop,             // Error on type mismatches
        label_action: ActionLevel::None,            // Don't check labels
        order_action: ActionLevel::None,            // Don't check order
        ..Default::default()
    };

    let result =
        validator.validate_against_spec_with_config(&mismatched_dataset, &spec, custom_config);

    println!("  Using custom config (ignoring labels and order):");
    println!("  Result: {}", result);
    println!(
        "  Errors: {}, Warnings: {}",
        result.errors.len(),
        result.warnings.len()
    );
    println!();

    // Example 5: FDA Policy details
    println!("Example 5: FDA Policy Requirements\n");

    let strict_policy = FdaPolicy::strict();
    let lenient_policy = FdaPolicy::lenient();

    println!("  FDA Strict Policy:");
    println!(
        "    - Required version: {:?}",
        strict_policy.required_version()
    );
    println!(
        "    - Max variable name: {} chars",
        strict_policy.max_variable_name_length()
    );
    println!(
        "    - Max variable label: {} chars",
        strict_policy.max_variable_label_length()
    );
    println!(
        "    - Require uppercase: {}",
        strict_policy.require_uppercase_names()
    );
    println!("    - Require ASCII: {}", strict_policy.require_ascii());
    println!(
        "    - Max file size: {} bytes",
        strict_policy.max_file_size().unwrap_or(0)
    );
    println!();

    println!("  FDA Lenient Policy:");
    println!("    - Is strict: {}", lenient_policy.is_strict());
    println!();

    // Example 6: File naming validation
    println!("Example 6: File Naming Rules\n");

    let rules = strict_policy.file_naming_rules();
    println!("  FDA File Naming Rules:");
    println!(
        "    - Max filename length: {} chars",
        rules.max_filename_length
    );
    println!("    - Required extension: {}", rules.required_extension);
    println!("    - Require lowercase: {}", rules.require_lowercase);
    println!("    - Match dataset name: {}", rules.match_dataset_name);

    // Test some filenames
    let test_filenames = ["dm.xpt", "DM.XPT", "demographics.xpt", "dm.sas7bdat"];
    println!("\n  Testing filenames:");
    for filename in &test_filenames {
        let issues = rules.validate(filename);
        if issues.is_empty() {
            println!("    - {}: VALID", filename);
        } else {
            println!("    - {}: INVALID", filename);
            for issue in &issues {
                println!("        {}", issue);
            }
        }
    }
    println!();

    println!("=== Example Complete ===");
    Ok(())
}
