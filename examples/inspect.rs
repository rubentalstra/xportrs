//! XPT file inspection example.
//!
//! This example demonstrates inspecting XPT file metadata without
//! loading the full dataset into memory.
//!
//! Run with: `cargo run --example inspect`

use xportrs::{Column, ColumnData, Dataset, Xpt};

fn main() -> xportrs::Result<()> {
    // Create a test file first
    let path = "target/example_inspect.xpt";
    create_test_file(path)?;

    println!("Inspecting: {}\n", path);

    // Inspect file metadata without loading data
    let info = Xpt::inspect(path)?;

    println!("XPT File Information:");
    println!("  Members: {}", info.members.len());

    if let Some(label) = &info.library_label {
        println!("  Library label: {}", label);
    }
    if let Some(created) = &info.created {
        println!("  Created: {}", created);
    }
    if let Some(modified) = &info.modified {
        println!("  Modified: {}", modified);
    }

    println!("\nDataset members:");
    for name in info.member_names() {
        println!("  - {}", name);
    }

    // Access detailed member information
    println!("\nDetailed member information:");
    for member in &info.members {
        println!("  {}:", member.name);
        println!("    Variables: {}", member.variables.len());
        println!("    Observations: {}", member.obs_count);
        if let Some(label) = &member.label {
            println!("    Label: {}", label);
        }
    }

    Ok(())
}

fn create_test_file(path: &str) -> xportrs::Result<()> {
    let dataset = Dataset::new(
        "LB",
        vec![
            Column::new(
                "USUBJID",
                ColumnData::String(vec![
                    Some("STUDY01-001".into()),
                    Some("STUDY01-001".into()),
                    Some("STUDY01-002".into()),
                ]),
            ),
            Column::new(
                "LBTEST",
                ColumnData::String(vec![
                    Some("Hemoglobin".into()),
                    Some("Glucose".into()),
                    Some("Hemoglobin".into()),
                ]),
            ),
            Column::new(
                "LBSTRESN",
                ColumnData::F64(vec![Some(14.2), Some(98.0), Some(13.8)]),
            ),
        ],
    )?;
    Xpt::writer(dataset).finalize()?.write_path(path)?;
    Ok(())
}
