{{#title Quick Start - xportrs}}

# Quick Start

Get up and running with xportrs in 5 minutes.

## Installation

Add xportrs to your `Cargo.toml`:

```toml
[dependencies]
xportrs = "0.0.7"
```

## Reading an XPT File

```rust
use xportrs::Xpt;

fn main() -> xportrs::Result<()> {
    // Read an XPT file
    let dataset = Xpt::read("ae.xpt")?;
    
    // Basic info
    println!("Domain: {}", dataset.domain_code());
    println!("Rows: {}", dataset.nrows());
    println!("Columns: {}", dataset.ncols());
    
    // List columns
    for col in dataset.columns() {
        println!("  - {}", col.name());
    }
    
    Ok(())
}
```

## Creating a Dataset

```rust
use xportrs::{Column, ColumnData, Dataset};

# fn main() -> xportrs::Result<()> {
let dataset = Dataset::new("AE", vec![
    Column::new("USUBJID", ColumnData::String(vec![
        Some("001".into()),
        Some("002".into()),
        Some("003".into()),
    ])),
    Column::new("AESEQ", ColumnData::F64(vec![
        Some(1.0),
        Some(1.0),
        Some(2.0),
    ])),
    Column::new("AETERM", ColumnData::String(vec![
        Some("HEADACHE".into()),
        Some("NAUSEA".into()),
        Some("FATIGUE".into()),
    ])),
])?;

println!("Created {} with {} rows", dataset.domain_code(), dataset.nrows());
# Ok(())
# }
```

## Writing an XPT File

```rust
use xportrs::{Column, ColumnData, Dataset, Xpt};

# fn main() -> xportrs::Result<()> {
let dataset = Dataset::new("AE", vec![
    Column::new("USUBJID", ColumnData::String(vec![Some("001".into())])),
    Column::new("AESEQ", ColumnData::F64(vec![Some(1.0)])),
])?;

// Write to file
Xpt::writer(dataset)
    .finalize()?
    .write_path("ae_output.xpt")?;

println!("Wrote ae_output.xpt");
# Ok(())
# }
```

## Adding Metadata

For regulatory submissions, include metadata:

```rust
use xportrs::{Column, ColumnData, Dataset, Format, Xpt};

# fn main() -> xportrs::Result<()> {
let dataset = Dataset::with_label("AE", "Adverse Events", vec![
    Column::new("USUBJID", ColumnData::String(vec![Some("001".into())]))
        .with_label("Unique Subject Identifier")
        .with_format(Format::character(40)),
    
    Column::new("AESEQ", ColumnData::F64(vec![Some(1.0)]))
        .with_label("Sequence Number")
        .with_format(Format::numeric(8, 0)),
    
    Column::new("AETERM", ColumnData::String(vec![Some("HEADACHE".into())]))
        .with_label("Reported Term for the Adverse Event")
        .with_format(Format::character(200))
        .with_length(200),
])?;

Xpt::writer(dataset)
    .finalize()?
    .write_path("ae_metadata.xpt")?;
# Ok(())
# }
```

## FDA Validation

Validate for FDA submission:

```rust
use xportrs::{Agency, Column, ColumnData, Dataset, Xpt};

# fn main() -> xportrs::Result<()> {
# let dataset = Dataset::new("AE", vec![
#     Column::new("USUBJID", ColumnData::String(vec![Some("001".into())])),
# ])?;
let validated = Xpt::writer(dataset)
    .agency(Agency::FDA)
    .finalize()?;

// Check for issues
if validated.has_errors() {
    eprintln!("Validation errors:");
    for issue in validated.issues() {
        eprintln!("  {}", issue);
    }
    return Ok(());
}

if validated.has_warnings() {
    println!("Warnings (proceeding anyway):");
    for issue in validated.issues() {
        println!("  {}", issue);
    }
}

validated.write_path("ae.xpt")?;
# Ok(())
# }
```

## Round-Trip (Read → Modify → Write)

```rust,noplayground
use xportrs::Xpt;

fn main() -> xportrs::Result<()> {
    // Read existing file
    let dataset = Xpt::read("ae.xpt")?;
    
    // Modify (example: add column)
    // dataset.extend([new_column]);
    
    // Write back
    Xpt::writer(dataset)
        .finalize()?
        .write_path("ae_modified.xpt")?;
    
    Ok(())
}
```

## Common Patterns

### Using From Conversions

```rust
use xportrs::{Column, ColumnData, Dataset};

# fn main() -> xportrs::Result<()> {
// Simpler syntax with From implementations
let dataset = Dataset::new("LB", vec![
    Column::new("LBSEQ", vec![1.0, 2.0, 3.0].into()),  // Vec<f64> → ColumnData
    Column::new("LBTEST", vec!["HGB", "WBC", "PLT"].into()),  // Vec<&str> → ColumnData
])?;
# Ok(())
# }
```

### Accessing Column Data

```rust,noplayground
use xportrs::{ColumnData, Xpt};

# fn main() -> xportrs::Result<()> {
let dataset = Xpt::read("ae.xpt")?;

// By name
let col = &dataset["USUBJID"];

// Match on data type
match col.data() {
    ColumnData::String(values) => {
        for (i, val) in values.iter().enumerate() {
            match val {
                Some(s) => println!("Row {}: {}", i, s),
                None => println!("Row {}: <missing>", i),
            }
        }
    }
    ColumnData::F64(values) => {
        for (i, val) in values.iter().enumerate() {
            match val {
                Some(n) => println!("Row {}: {}", i, n),
                None => println!("Row {}: <missing>", i),
            }
        }
    }
    _ => {}
}
# Ok(())
# }
```

### Handling Errors

```rust,noplayground
use xportrs::{Error, Xpt};

match Xpt::read("missing.xpt") {
    Ok(dataset) => println!("Loaded"),
    Err(Error::Io(e)) => eprintln!("File error: {}", e),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Next Steps

- [FDA Submission Workflow](fda-submission.md) — Complete FDA submission guide
- [API Reference](../api/dataset.md) — Full API documentation
- [XPT Format](../format/structure.md) — Understanding the file format
- [Validation](../api/validation.md) — Validation rules and handling
