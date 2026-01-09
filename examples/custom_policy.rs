//! Custom agency policy example.
//!
//! This example demonstrates how to create and use custom agency policies
//! for validation beyond the built-in FDA, NMPA, and PMDA policies.
//!
//! Run with: `cargo run --example custom_policy`

use xportrs::policy::AgencyPolicy;
use xportrs::validation::{ValidationMode, Validator};
use xportrs::{CustomPolicy, FdaPolicy, NmpaPolicy, PmdaPolicy, XptDataset, XptVersion};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== xportrs Custom Policy Example ===\n");

    // Create a sample dataset for validation
    let mut dataset = XptDataset::new("DM");
    dataset.label = Some("Demographics".to_string());
    dataset.columns.push(
        xportrs::XptColumn::character("STUDYID", 12).with_label("Study Identifier"),
    );
    dataset.columns.push(
        xportrs::XptColumn::character("USUBJID", 20).with_label("Unique Subject Identifier"),
    );
    dataset.columns.push(xportrs::XptColumn::numeric("AGE").with_label("Age"));

    // =========================================
    // Built-in FDA Policy (Strict)
    // =========================================
    println!("1. FDA Strict Policy");
    let fda_strict = FdaPolicy::strict();
    println!("   - Required version: {:?}", fda_strict.required_version());
    println!(
        "   - Max variable name: {} characters",
        fda_strict.max_variable_name_length()
    );
    println!(
        "   - Max dataset name: {} characters",
        fda_strict.max_dataset_name_length()
    );
    println!(
        "   - Max variable label: {} characters",
        fda_strict.max_variable_label_length()
    );
    println!("   - Max file size: {:?}", fda_strict.max_file_size());
    println!("   - Requires ASCII: {}", fda_strict.require_ascii());
    println!(
        "   - Requires uppercase: {}",
        fda_strict.require_uppercase_names()
    );

    let validator = Validator::fda_compliant(XptVersion::V5);
    let report = validator.validate(&dataset);
    println!("   Validation: {} issues found\n", report.errors.len() + report.warnings.len());

    // =========================================
    // Built-in FDA Policy (Lenient)
    // =========================================
    println!("2. FDA Lenient Policy");
    let fda_lenient = FdaPolicy::lenient();
    println!("   - Is strict: {}", fda_lenient.is_strict());
    println!("   - Description: {}", fda_lenient.description());

    let validator = Validator::new(XptVersion::V5);
    let report = validator.validate(&dataset);
    println!("   Validation: {} issues found\n", report.errors.len() + report.warnings.len());

    // =========================================
    // Built-in NMPA Policy (China)
    // =========================================
    println!("3. NMPA Policy (China)");
    let nmpa = NmpaPolicy::default();
    println!("   - Description: {}", nmpa.description());
    println!("   - Requires ASCII: {}", nmpa.require_ascii());

    let validator = Validator::new(XptVersion::V5);
    let report = validator.validate(&dataset);
    println!("   Validation: {} issues found\n", report.errors.len() + report.warnings.len());

    // =========================================
    // Built-in PMDA Policy (Japan)
    // =========================================
    println!("4. PMDA Policy (Japan)");
    let pmda = PmdaPolicy::default();
    println!("   - Description: {}", pmda.description());

    let validator = Validator::new(XptVersion::V5);
    let report = validator.validate(&dataset);
    println!("   Validation: {} issues found\n", report.errors.len() + report.warnings.len());

    // =========================================
    // Custom Policy - Research Use
    // =========================================
    println!("5. Custom Policy - Research Use (Relaxed)");
    println!("   - Allow V8 for longer names");
    println!("   - No file size limit");
    println!("   - Allow non-ASCII characters");

    let research_policy = CustomPolicy::new()
        .with_required_version(XptVersion::V8)
        .with_max_variable_name_length(32)
        .with_max_dataset_name_length(32)
        .with_max_variable_label_length(256)
        .with_max_dataset_label_length(256)
        .with_require_ascii(false)
        .with_require_uppercase_names(false)
        .with_no_file_size_limit();

    println!("   - Policy description: {}", research_policy.description());

    let validator = Validator::new(XptVersion::V8);
    let report = validator.validate(&dataset);
    println!("   Validation: {} issues found\n", report.errors.len() + report.warnings.len());

    // =========================================
    // Custom Policy - Internal QC
    // =========================================
    println!("6. Custom Policy - Internal QC (Extra Strict)");
    println!("   - Max file size: 1 GB");
    println!("   - All FDA rules plus custom limits");

    let qc_policy = CustomPolicy::from_fda_base()
        .with_max_file_size(1024 * 1024 * 1024) // 1 GB
        .with_strict(true)
        .with_description("Internal QC Standards");

    println!("   - Policy description: {}", qc_policy.description());
    println!("   - Max file size: {:?}", qc_policy.max_file_size());

    let validator = Validator::fda_compliant(XptVersion::V5);
    let report = validator.validate(&dataset);
    println!("   Validation: {} issues found\n", report.errors.len() + report.warnings.len());

    // =========================================
    // Custom Policy - External Partner
    // =========================================
    println!("7. Custom Policy - External Partner");
    println!("   - V8 allowed");
    println!("   - Max 10 GB file size");
    println!("   - Uppercase names required");

    let partner_policy = CustomPolicy::v8_extended()
        .with_max_file_size(10 * 1024 * 1024 * 1024) // 10 GB
        .with_require_uppercase_names(true)
        .with_description("Partner Data Exchange v2.0");

    println!("   - Policy description: {}", partner_policy.description());

    let validator = Validator::new(XptVersion::V8).with_mode(ValidationMode::Custom);
    let report = validator.validate(&dataset);
    println!("   Validation: {} issues found\n", report.errors.len() + report.warnings.len());

    // =========================================
    // Comparing Policy Constraints
    // =========================================
    println!("=== Policy Constraint Comparison ===\n");
    println!("| Policy      | Max Var Name | Max Label | File Size | ASCII | Uppercase |");
    println!("|-------------|--------------|-----------|-----------|-------|-----------|");

    print_policy_row("FDA Strict", &FdaPolicy::strict());
    print_policy_row("FDA Lenient", &FdaPolicy::lenient());
    print_policy_row("NMPA", &NmpaPolicy::default());
    print_policy_row("PMDA", &PmdaPolicy::default());
    print_policy_row("Research", &research_policy);
    print_policy_row("Internal QC", &qc_policy);
    print_policy_row("Partner", &partner_policy);

    println!("\n=== Example Complete ===");
    Ok(())
}

fn print_policy_row<P: AgencyPolicy>(name: &str, policy: &P) {
    let file_size = match policy.max_file_size() {
        Some(size) => {
            let gb = size as f64 / (1024.0 * 1024.0 * 1024.0);
            format!("{gb:.0} GB")
        }
        None => "None".to_string(),
    };

    println!(
        "| {:<11} | {:<12} | {:<9} | {:<9} | {:<5} | {:<9} |",
        name,
        policy.max_variable_name_length(),
        policy.max_variable_label_length(),
        file_size,
        if policy.require_ascii() { "Yes" } else { "No" },
        if policy.require_uppercase_names() {
            "Yes"
        } else {
            "No"
        }
    );
}
