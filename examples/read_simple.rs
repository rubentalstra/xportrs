//! Simple XPT file reading example.
//!
//! This example demonstrates reading an XPT file that we create first.
//!
//! Run with: `cargo run --example read_simple`

use xportrs::{Column, ColumnData, Dataset, Xpt};

fn main() -> xportrs::Result<()> {
    // First, create a test file
    let path = "target/example_ae.xpt";
    create_test_file(path)?;

    // Read the dataset from the XPT file
    let dataset = Xpt::read(path)?;

    // Access basic dataset information
    println!("Domain code: {}", dataset.domain_code());
    println!("Rows: {}", dataset.nrows());
    println!("Columns: {}", dataset.ncols());

    // Print column names and first values
    println!("\nColumn data:");
    for col in dataset.columns() {
        print!("  {} - ", col.name());
        match col.data() {
            ColumnData::String(values) => {
                let first = values.first().and_then(|v| v.as_deref()).unwrap_or("(null)");
                println!("{} values, first: \"{}\"", values.len(), first);
            }
            ColumnData::I64(values) => {
                let first = values.first().and_then(|v| *v).map_or("(null)".to_string(), |v| v.to_string());
                println!("{} values, first: {}", values.len(), first);
            }
            ColumnData::F64(values) => {
                let first = values.first().and_then(|v| *v).map_or("(null)".to_string(), |v| format!("{:.2}", v));
                println!("{} values, first: {}", values.len(), first);
            }
            _ => println!("{} values", col.len()),
        }
    }

    Ok(())
}

fn create_test_file(path: &str) -> xportrs::Result<()> {
    let dataset = Dataset::new(
        "AE",
        vec![
            Column::new(
                "USUBJID",
                ColumnData::String(vec![
                    Some("STUDY01-001".into()),
                    Some("STUDY01-002".into()),
                ]),
            ),
            Column::new("AESEQ", ColumnData::I64(vec![Some(1), Some(1)])),
            Column::new("AESTDY", ColumnData::F64(vec![Some(15.0), Some(22.0)])),
        ],
    )?;
    Xpt::writer(dataset).finalize()?.write_path(path)?;
    Ok(())
}
