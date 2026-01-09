//! Polars DataFrame pipeline example.
//!
//! This example demonstrates the xportr-style workflow using Polars DataFrames.
//! It shows how to apply metadata-driven transformations to create compliant XPT files.
//!
//! Run with: `cargo run --example polars_pipeline`

use polars::prelude::*;
use std::path::Path;
use xportrs::polars::XportrTransforms;
use xportrs::spec::{DatasetSpec, VariableSpec};
use xportrs::{ActionLevel, FormatSpec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== xportrs Polars Pipeline Example ===\n");

    // Step 1: Define the specification (metadata)
    // This mirrors how xportr uses specification files
    let spec = DatasetSpec::new("DM")
        .with_label("Demographics")
        .add_variable(
            VariableSpec::character("STUDYID", 12)
                .with_label("Study Identifier")
                .with_order(1),
        )
        .add_variable(
            VariableSpec::character("USUBJID", 20)
                .with_label("Unique Subject Identifier")
                .with_order(2),
        )
        .add_variable(
            VariableSpec::numeric("AGE")
                .with_label("Age")
                .with_format(FormatSpec::best(8))
                .with_order(3),
        )
        .add_variable(
            VariableSpec::character("SEX", 1)
                .with_label("Sex")
                .with_order(4),
        )
        .add_variable(
            VariableSpec::character("RACE", 40)
                .with_label("Race")
                .with_order(5),
        );

    println!("Specification:");
    println!("  Dataset: {} - {:?}", spec.name, spec.label);
    println!("  Variables: {}", spec.variables.len());
    for var in &spec.variables {
        println!(
            "    - {} ({:?}): {:?}",
            var.name,
            if var.data_type.is_numeric() {
                "Num"
            } else {
                "Char"
            },
            var.label
        );
    }
    println!();

    // Step 2: Create a DataFrame with raw data
    // In practice, this would come from your data source
    let df = df! {
        "USUBJID" => &["STUDY001-001", "STUDY001-002", "STUDY001-003"],
        "AGE" => &[35i64, 42, 28],  // Will be coerced to f64
        "SEX" => &["M", "F", "M"],
        "STUDYID" => &["STUDY001", "STUDY001", "STUDY001"],
        "RACE" => &["WHITE", "BLACK OR AFRICAN AMERICAN", "ASIAN"],
    }?;

    println!("Input DataFrame:");
    println!("{}\n", df);

    // Step 3: Apply the xportr pipeline
    // This is similar to R's: df %>% xportr_metadata() %>% xportr_type() %>% ...
    println!("Applying xportr transforms...\n");

    let result = df
        .clone()
        .xportr_metadata(spec.clone())
        .xportr_type(&spec, ActionLevel::Warn)?
        .xportr_length(&spec, ActionLevel::Warn)?
        .xportr_label(&spec, ActionLevel::Warn)?
        .xportr_order(&spec, ActionLevel::Message)?
        .xportr_format(&spec, ActionLevel::Message)?
        .xportr_df_label("Demographics");

    // Step 4: Check the transform report
    let report = result.report();
    println!("Transform Report:");
    println!("  Type conversions: {}", report.type_conversions.len());
    for conv in &report.type_conversions {
        println!(
            "    - {}: {} -> {} ({} values)",
            conv.variable, conv.from_type, conv.to_type, conv.values_converted
        );
    }
    println!("  Label changes: {}", report.label_changes.len());
    println!("  Order changes: {}", report.order_changes.len());
    if report.has_warnings() {
        println!("  Warnings: {}", report.warnings.len());
        for warning in &report.warnings {
            println!("    - {}", warning);
        }
    }
    println!();

    // Step 5: View the transformed DataFrame
    println!("Transformed DataFrame:");
    println!("{}\n", result.df());

    // Step 6: Write to XPT file (optional)
    let output_path = Path::new("/tmp/dm_example.xpt");
    println!("Writing to XPT file: {}\n", output_path.display());

    // Clone and write
    let write_result = result.clone();
    let pipeline_report = write_result.xportr_write(output_path, "DM", &spec, false)?;

    println!("Write completed!");
    println!("Pipeline errors: {}", pipeline_report.errors.len());
    if !pipeline_report.errors.is_empty() {
        for err in &pipeline_report.errors {
            println!("  - {}", err);
        }
    }

    // Verify the file was written
    if output_path.exists() {
        let metadata = std::fs::metadata(output_path)?;
        println!("XPT file size: {} bytes", metadata.len());

        // Read it back to verify
        let read_back = xportrs::read_xpt(output_path)?;
        println!(
            "Verified: Read back {} rows, {} columns",
            read_back.num_rows(),
            read_back.columns.len()
        );
    }

    println!("\n=== Example Complete ===");
    Ok(())
}
