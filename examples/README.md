# Examples

Code examples demonstrating `xportrs` functionality.

## Running Examples

```bash
cargo run --example <name>
```

## Available Examples

| Example                           | Description                                     |
|-----------------------------------|-------------------------------------------------|
| [`read_simple`](read_simple.rs)   | Read an XPT file and access dataset information |
| [`write_simple`](write_simple.rs) | Create a dataset and write to XPT format        |
| [`write_fda`](write_fda.rs)       | Write with FDA compliance validation            |
| [`inspect`](inspect.rs)           | Inspect XPT metadata without loading full data  |
| [`roundtrip`](roundtrip.rs)       | Read, inspect, and write back to a new file     |

## Quick Reference

### Reading

```rust
use xportrs::Xpt;

fn main() -> Result<(), xportrs::Error> {
    let dataset = Xpt::read("path/to/file.xpt")?;
    println!("Rows: {}, Columns: {}", dataset.nrows(), dataset.ncols());
    Ok(())
}
```

### Writing

```rust
use xportrs::{Column, ColumnData, Dataset, Xpt};

fn main() -> Result<(), xportrs::Error> {
    let dataset = Dataset::new("DM", vec![
        Column::new("USUBJID", ColumnData::String(vec![Some("SUBJ-001".into())])),
        Column::new("AGE", ColumnData::I64(vec![Some(45)])),
    ])?;

    Xpt::writer(dataset).finalize()?.write_path("dm.xpt")?;
    Ok(())
}
```

### With Agency Validation

```rust
use xportrs::{Agency, Column, ColumnData, Dataset, Xpt};

fn main() -> Result<(), xportrs::Error> {
    let dataset = Dataset::new("DM", vec![
        Column::new("USUBJID", ColumnData::String(vec![Some("SUBJ-001".into())])),
    ])?;

    Xpt::writer(dataset)
        .agency(Agency::FDA)
        .finalize()?
        .write_path("dm.xpt")?;
    Ok(())
}
```

### Inspecting Metadata

```rust
use xportrs::Xpt;

fn main() -> Result<(), xportrs::Error> {
    let info = Xpt::inspect("file.xpt")?;
    println!("Members: {:?}", info.member_names());
    Ok(())
}
```

## More Information

See the [API documentation](https://docs.rs/xportrs) for complete reference.
