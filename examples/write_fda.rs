//! FDA-compliant XPT file writing example.
//!
//! This example demonstrates writing an XPT file with FDA validation rules.
//! It shows how to check for validation issues before writing.
//!
//! Run with: `cargo run --example write_fda`

use xportrs::{Agency, Column, ColumnData, Dataset, Severity, Xpt};

fn main() -> xportrs::Result<()> {
    // Create a demographics dataset
    let dataset = Dataset::new(
        "DM", // Domain code
        vec![
            Column::new(
                "STUDYID",
                ColumnData::String(vec![Some("STUDY001".into()), Some("STUDY001".into())]),
            ),
            Column::new(
                "USUBJID",
                ColumnData::String(vec![
                    Some("STUDY001-001".into()),
                    Some("STUDY001-002".into()),
                ]),
            ),
            Column::new("AGE", ColumnData::I64(vec![Some(45), Some(62)])),
            Column::new(
                "SEX",
                ColumnData::String(vec![Some("M".into()), Some("F".into())]),
            ),
        ],
    )?;

    println!("Created dataset: {}", dataset.domain_code());
    println!("Validating for FDA submission...\n");

    // Build with FDA agency validation
    let mut builder = Xpt::writer(dataset);
    builder.agency(Agency::FDA);
    let validated = builder.finalize()?;

    // Check for validation issues
    let issues = validated.issues();
    if !issues.is_empty() {
        println!("Validation issues found:");
        for issue in issues {
            let severity = match issue.severity() {
                Severity::Error => "ERROR",
                Severity::Warning => "WARN",
                _ => "INFO",
            };
            println!("  [{severity}] {issue}");
        }
        println!();
    }

    // Check if we can proceed
    if validated.has_errors() {
        println!("Cannot write file: validation errors present");
        return Ok(());
    }

    // Write the file
    let output_path = "target/dm_fda.xpt";
    let files = validated.write_path(output_path)?;

    println!("Successfully wrote {} file(s):", files.len());
    for path in &files {
        println!("  - {}", path.display());
    }

    Ok(())
}
