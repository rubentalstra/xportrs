# Read-Modify-Write Workflows

This guide covers common patterns for reading, modifying, and writing XPT files.

## Basic Roundtrip

```rust
use xportrs::Xpt;

fn basic_roundtrip(input: &str, output: &str) -> xportrs::Result<()> {
    // Read
    let dataset = Xpt::read(input)?;

    // (Modify here if needed)

    // Write
    Xpt::writer(dataset)
        .finalize()?
        .write_path(output)?;

    Ok(())
}
```

## Preserving Metadata

xportrs automatically preserves metadata during roundtrip:

```rust
use xportrs::Xpt;

fn verify_metadata_preservation(path: &str) -> xportrs::Result<()> {
    // Read original
    let original = Xpt::read(path)?;

    // Write to temp
    let temp_path = "/tmp/roundtrip.xpt";
    Xpt::writer(original.clone())
        .finalize()?
        .write_path(temp_path)?;

    // Read back
    let reloaded = Xpt::read(temp_path)?;

    // Verify metadata preserved
    assert_eq!(original.domain_code(), reloaded.domain_code());
    assert_eq!(original.dataset_label(), reloaded.dataset_label());

    for (orig_col, new_col) in original.columns().iter()
        .zip(reloaded.columns().iter())
    {
        assert_eq!(orig_col.name(), new_col.name());
        assert_eq!(
            orig_col.label().map(|l| l.to_string()),
            new_col.label().map(|l| l.to_string())
        );
        // Format, length, etc. also preserved
    }

    Ok(())
}
```

## Adding Columns

```rust
use xportrs::{Column, ColumnData, Format, Xpt};

fn add_derived_column(input: &str, output: &str) -> xportrs::Result<()> {
    let mut dataset = Xpt::read(input)?;

    // Get row count
    let nrows = dataset.nrows();

    // Create new column
    let new_column = Column::new(
        "DERIVED",
        ColumnData::F64(vec![Some(1.0); nrows]),
    )
        .with_label("Derived Variable")
        .with_format(Format::numeric(8, 0));

    // Add to dataset
    dataset.extend([new_column]);

    // Write
    Xpt::writer(dataset)
        .finalize()?
        .write_path(output)?;

    Ok(())
}
```

## Modifying Column Data

```rust
use xportrs::{Column, ColumnData, Xpt};

fn modify_column_data(input: &str, output: &str) -> xportrs::Result<()> {
    let dataset = Xpt::read(input)?;

    // Create modified columns
    let modified_columns: Vec<Column> = dataset.columns().iter()
        .map(|col| {
            if col.name() == "AESEQ" {
                // Modify AESEQ: multiply by 10
                if let ColumnData::F64(values) = col.data() {
                    let new_values: Vec<Option<f64>> = values.iter()
                        .map(|v| v.map(|x| x * 10.0))
                        .collect();

                    let mut new_col = Column::new(col.name(), ColumnData::F64(new_values));

                    // Preserve metadata
                    if let Some(label) = col.label() {
                        new_col = new_col.with_label(label.to_string());
                    }
                    if let Some(format) = col.format() {
                        new_col = new_col.with_format(format.clone());
                    }

                    return new_col;
                }
            }
            col.clone()
        })
        .collect();

    // Create new dataset with modified columns
    let mut new_dataset = xportrs::Dataset::new(
        dataset.domain_code(),
        modified_columns,
    )?;

    if let Some(label) = dataset.dataset_label() {
        new_dataset.set_label(label);
    }

    Xpt::writer(new_dataset)
        .finalize()?
        .write_path(output)?;

    Ok(())
}
```

## Filtering Rows

```rust
use xportrs::{Column, ColumnData, Dataset, Xpt};

fn filter_rows(input: &str, output: &str, keep_indices: &[usize]) -> xportrs::Result<()> {
    let dataset = Xpt::read(input)?;

    // Filter each column
    let filtered_columns: Vec<Column> = dataset.columns().iter()
        .map(|col| {
            let filtered_data = match col.data() {
                ColumnData::F64(values) => {
                    let filtered: Vec<_> = keep_indices.iter()
                        .map(|&i| values[i].clone())
                        .collect();
                    ColumnData::F64(filtered)
                }
                ColumnData::String(values) => {
                    let filtered: Vec<_> = keep_indices.iter()
                        .map(|&i| values[i].clone())
                        .collect();
                    ColumnData::String(filtered)
                }
                // Handle other types...
                _ => col.data().clone(),
            };

            let mut new_col = Column::new(col.name(), filtered_data);
            if let Some(label) = col.label() {
                new_col = new_col.with_label(label.to_string());
            }
            if let Some(format) = col.format() {
                new_col = new_col.with_format(format.clone());
            }
            new_col
        })
        .collect();

    let mut filtered_dataset = Dataset::new(
        dataset.domain_code(),
        filtered_columns,
    )?;

    if let Some(label) = dataset.dataset_label() {
        filtered_dataset.set_label(label);
    }

    Xpt::writer(filtered_dataset)
        .finalize()?
        .write_path(output)?;

    Ok(())
}
```

## Merging Datasets

```rust
use xportrs::{Column, ColumnData, Dataset, Xpt};

fn merge_datasets(input1: &str, input2: &str, output: &str) -> xportrs::Result<()> {
    let ds1 = Xpt::read(input1)?;
    let ds2 = Xpt::read(input2)?;

    // Verify same structure
    assert_eq!(ds1.ncols(), ds2.ncols(), "Column count mismatch");

    // Concatenate data
    let merged_columns: Vec<Column> = ds1.columns().iter()
        .zip(ds2.columns().iter())
        .map(|(col1, col2)| {
            let merged_data = match (col1.data(), col2.data()) {
                (ColumnData::F64(v1), ColumnData::F64(v2)) => {
                    let mut merged = v1.clone();
                    merged.extend(v2.clone());
                    ColumnData::F64(merged)
                }
                (ColumnData::String(v1), ColumnData::String(v2)) => {
                    let mut merged = v1.clone();
                    merged.extend(v2.clone());
                    ColumnData::String(merged)
                }
                _ => panic!("Type mismatch"),
            };

            let mut col = Column::new(col1.name(), merged_data);
            if let Some(label) = col1.label() {
                col = col.with_label(label.to_string());
            }
            if let Some(format) = col1.format() {
                col = col.with_format(format.clone());
            }
            col
        })
        .collect();

    let mut merged = Dataset::new(ds1.domain_code(), merged_columns)?;
    if let Some(label) = ds1.dataset_label() {
        merged.set_label(label);
    }

    Xpt::writer(merged)
        .finalize()?
        .write_path(output)?;

    Ok(())
}
```

## Updating Labels

```rust
use xportrs::{Column, Dataset, Xpt};
use std::collections::HashMap;

fn update_labels(
    input: &str,
    output: &str,
    label_updates: &HashMap<&str, &str>,
) -> xportrs::Result<()> {
    let dataset = Xpt::read(input)?;

    let updated_columns: Vec<Column> = dataset.columns().iter()
        .map(|col| {
            let mut new_col = Column::new(col.name(), col.data().clone());

            // Apply label update if specified
            if let Some(&new_label) = label_updates.get(col.name()) {
                new_col = new_col.with_label(new_label);
            } else if let Some(label) = col.label() {
                new_col = new_col.with_label(label.to_string());
            }

            if let Some(format) = col.format() {
                new_col = new_col.with_format(format.clone());
            }

            new_col
        })
        .collect();

    let mut updated = Dataset::new(dataset.domain_code(), updated_columns)?;
    if let Some(label) = dataset.dataset_label() {
        updated.set_label(label);
    }

    Xpt::writer(updated)
        .finalize()?
        .write_path(output)?;

    Ok(())
}

// Usage
fn main() -> xportrs::Result<()> {
    let mut updates = HashMap::new();
    updates.insert("USUBJID", "Unique Subject Identifier");
    updates.insert("AETERM", "Reported Adverse Event Term");

    update_labels("ae.xpt", "ae_updated.xpt", &updates)
}
```

## Batch Processing

```rust
use xportrs::Xpt;
use std::path::Path;

fn process_directory(input_dir: &Path, output_dir: &Path) -> xportrs::Result<()> {
    std::fs::create_dir_all(output_dir)?;

    for entry in std::fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "xpt") {
            let filename = path.file_name().unwrap();
            let output_path = output_dir.join(filename);

            println!("Processing: {}", path.display());

            let dataset = Xpt::read(&path)?;

            // Process...

            Xpt::writer(dataset)
                .finalize()?
                .write_path(&output_path)?;

            println!("  Wrote: {}", output_path.display());
        }
    }

    Ok(())
}
```

## Error Handling in Roundtrips

```rust
use xportrs::{Error, Xpt};

fn safe_roundtrip(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read with error handling
    let dataset = match Xpt::read(input) {
        Ok(ds) => ds,
        Err(Error::Io(e)) => {
            eprintln!("Failed to read {}: {}", input, e);
            return Err(e.into());
        }
        Err(e) => return Err(e.into()),
    };

    // Validate
    let validated = Xpt::writer(dataset).finalize()?;

    if validated.has_errors() {
        for issue in validated.issues() {
            eprintln!("Validation error: {}", issue);
        }
        return Err("Validation failed".into());
    }

    // Write
    validated.write_path(output)?;

    // Verify
    let _ = Xpt::read(output)?;

    Ok(())
}
```
