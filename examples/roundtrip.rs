//! XPT roundtrip example.
//!
//! This example demonstrates reading an XPT file, accessing its data,
//! and writing it back to a new file.
//!
//! Run with: `cargo run --example roundtrip`

use xportrs::{Column, ColumnData, Dataset, Xpt};

fn main() -> xportrs::Result<()> {
    // Create and write an initial dataset
    let original_path = "target/roundtrip_original.xpt";
    let copy_path = "target/roundtrip_copy.xpt";

    let original = Dataset::new(
        "DM",
        vec![
            Column::new(
                "STUDYID",
                ColumnData::String(vec![
                    Some("STUDY001".into()),
                    Some("STUDY001".into()),
                    Some("STUDY001".into()),
                ]),
            ),
            Column::new(
                "USUBJID",
                ColumnData::String(vec![
                    Some("STUDY001-001".into()),
                    Some("STUDY001-002".into()),
                    Some("STUDY001-003".into()),
                ]),
            ),
            Column::new("AGE", ColumnData::I64(vec![Some(45), Some(62), Some(38)])),
            Column::new(
                "SEX",
                ColumnData::String(vec![
                    Some("M".into()),
                    Some("F".into()),
                    Some("M".into()),
                ]),
            ),
        ],
    )?;

    println!("Writing original dataset...");
    Xpt::writer(original).finalize()?.write_path(original_path)?;

    // Read the dataset back
    let dataset = Xpt::read(original_path)?;
    println!("\nRead dataset: {}", dataset.domain_code());
    println!("  Rows: {}", dataset.nrows());
    println!("  Columns: {}", dataset.ncols());

    // Inspect column types
    println!("\nColumn details:");
    for col in dataset.columns() {
        let type_name = match col.data() {
            ColumnData::F64(_) => "Numeric (f64)",
            ColumnData::I64(_) => "Integer (i64)",
            ColumnData::String(_) => "Character",
            ColumnData::Bool(_) => "Boolean",
            ColumnData::Bytes(_) => "Binary",
            ColumnData::Date(_) => "Date",
            ColumnData::DateTime(_) => "DateTime",
            ColumnData::Time(_) => "Time",
            _ => "Unknown",
        };
        println!("  {} - {}", col.name(), type_name);
    }

    // Write to a new file
    let files = Xpt::writer(dataset).finalize()?.write_path(copy_path)?;
    println!("\nWrote {} file(s) to:", files.len());
    for path in &files {
        println!("  - {}", path.display());
    }

    // Verify by reading back
    let reread = Xpt::read(copy_path)?;
    println!("\nVerification - re-read dataset:");
    println!("  Domain: {}", reread.domain_code());
    println!("  Rows: {}", reread.nrows());
    println!("  Columns: {}", reread.ncols());

    Ok(())
}
