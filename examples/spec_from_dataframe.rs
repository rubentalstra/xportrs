//! Loading specifications from `DataFrames` example.
//!
//! This example demonstrates how to load variable specifications from
//! a Polars `DataFrame`. In practice, you would typically load this from
//! a CSV, Excel, or other file using Polars.
//!
//! Run with: `cargo run --example spec_from_dataframe`

use polars::prelude::*;
use xportrs::spec::{ColumnMapping, DataFrameMetadataSource, MetadataSource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Loading Specs from DataFrame Example ===\n");

    // Example 1: Standard column names
    println!("Example 1: Standard Column Names\n");

    let spec_df = df! {
        "dataset" => &["DM", "DM", "DM", "AE", "AE"],
        "variable" => &["USUBJID", "AGE", "SEX", "USUBJID", "AETERM"],
        "label" => &["Unique Subject ID", "Age", "Sex", "Unique Subject ID", "Adverse Event Term"],
        "type" => &["Char", "Num", "Char", "Char", "Char"],
        "length" => &[20i64, 8, 1, 20, 200],
        "order" => &[1i64, 2, 3, 1, 2],
    }?;

    println!("Specification DataFrame:");
    println!("{}\n", spec_df);

    let source = DataFrameMetadataSource::new(spec_df);

    // List available datasets
    println!("Available datasets: {:?}\n", source.dataset_names());

    // Load a specific dataset spec
    let dm_spec = source.load_dataset_spec("DM")?;
    println!("DM Specification:");
    println!("  Name: {}", dm_spec.name);
    println!("  Variables:");
    for var in &dm_spec.variables {
        println!(
            "    - {} ({}): {:?}",
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

    // Load all specs at once
    let all_specs = source.load_all_specs()?;
    println!("Total datasets loaded: {}\n", all_specs.len());

    // Example 2: Custom column names (e.g., Pinnacle 21 style)
    println!("Example 2: Pinnacle 21 Style Column Names\n");

    let pinnacle_df = df! {
        "DOMAIN" => &["DM", "DM"],
        "VARNAME" => &["USUBJID", "BRTHDTC"],
        "VARLABEL" => &["Unique Subject ID", "Date/Time of Birth"],
        "VARTYPE" => &["Char", "Char"],
        "LENGTH" => &[20i64, 19],
        "VARNUM" => &[1i64, 2],
        "ORIGIN" => &["CRF", "Derived"],
    }?;

    println!("Pinnacle 21 Style DataFrame:");
    println!("{}\n", pinnacle_df);

    let pinnacle_source =
        DataFrameMetadataSource::new(pinnacle_df).with_mapping(ColumnMapping::pinnacle21());

    let dm_spec_p21 = pinnacle_source.load_dataset_spec("DM")?;
    println!("DM Specification (from Pinnacle 21 format):");
    for var in &dm_spec_p21.variables {
        println!(
            "  - {} (order {}): {:?}",
            var.name,
            var.order.unwrap_or(0),
            var.label
        );
    }
    println!();

    // Example 3: Custom column mapping
    println!("Example 3: Custom Column Mapping\n");

    let custom_df = df! {
        "DatasetName" => &["LB", "LB"],
        "VariableName" => &["LBTESTCD", "LBORRES"],
        "VariableLabel" => &["Lab Test Code", "Result"],
        "DataType" => &["Char", "Char"],
        "Width" => &[8i64, 200],
    }?;

    println!("Custom Format DataFrame:");
    println!("{}\n", custom_df);

    let custom_mapping = ColumnMapping::new()
        .with_dataset_col("DatasetName")
        .with_variable_col("VariableName")
        .with_label_col("VariableLabel")
        .with_type_col("DataType")
        .with_length_col("Width");

    let custom_source = DataFrameMetadataSource::new(custom_df).with_mapping(custom_mapping);

    let lb_spec = custom_source.load_dataset_spec("LB")?;
    println!("LB Specification:");
    for var in &lb_spec.variables {
        println!(
            "  - {} (length {}): {:?}",
            var.name,
            var.length.unwrap_or(8),
            var.label
        );
    }
    println!();

    // Example 4: Missing dataset handling
    println!("Example 4: Error Handling\n");

    match source.load_dataset_spec("NONEXISTENT") {
        Ok(_) => println!("  Unexpectedly found dataset"),
        Err(e) => println!("  Expected error: {}", e),
    }
    println!();

    println!("=== Example Complete ===");
    Ok(())
}
