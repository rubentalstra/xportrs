//! Simple XPT file writing example.
//!
//! This example demonstrates creating a dataset and writing it to an XPT file.
//!
//! Run with: `cargo run --example write_simple`

use xportrs::{Column, ColumnData, Dataset, Xpt};

fn main() -> xportrs::Result<()> {
    // Create a simple adverse events dataset
    let dataset = Dataset::new(
        "AE", // Domain code
        vec![
            Column::new(
                "USUBJID",
                ColumnData::String(vec![
                    Some("STUDY01-001".into()),
                    Some("STUDY01-002".into()),
                    Some("STUDY01-003".into()),
                ]),
            ),
            Column::new("AESEQ", ColumnData::I64(vec![Some(1), Some(1), Some(2)])),
            Column::new(
                "AETERM",
                ColumnData::String(vec![
                    Some("Headache".into()),
                    Some("Nausea".into()),
                    Some("Fatigue".into()),
                ]),
            ),
            Column::new(
                "AESTDY",
                ColumnData::F64(vec![Some(15.0), Some(22.0), Some(8.0)]),
            ),
        ],
    )?;

    println!("Created dataset: {}", dataset.domain_code());
    println!("Rows: {}", dataset.nrows());
    println!("Columns: {}", dataset.ncols());

    // Write to an XPT file (structural validation only)
    let output_path = "target/ae_simple.xpt";
    let files = Xpt::writer(dataset).finalize()?.write_path(output_path)?;

    println!("\nWrote {} file(s):", files.len());
    for path in &files {
        println!("  - {}", path.display());
    }

    Ok(())
}
