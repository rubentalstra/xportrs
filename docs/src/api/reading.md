# Reading XPT Files

xportrs provides multiple ways to read XPT files, from simple one-liners to detailed inspection.

## Quick Read

The simplest way to read an XPT file:

```rust
use xportrs::Xpt;

let dataset = Xpt::read("ae.xpt")?;

println!("Domain: {}", dataset.domain_code());
println!("Rows: {}", dataset.nrows());
println!("Columns: {}", dataset.ncols());
```

## Reading Multiple Members

XPT files can contain multiple datasets (members):

```rust
use xportrs::Xpt;

// Read all members
let datasets = Xpt::read_all("multi.xpt")?;

for dataset in datasets {
    println!("{}: {} rows", dataset.domain_code(), dataset.nrows());
}

// Read specific member
let ae = Xpt::read_member("multi.xpt", "AE")?;
```

## Inspecting Files

Get file metadata without loading all data:

```rust
use xportrs::Xpt;

let info = Xpt::inspect("data.xpt")?;

// File timestamps
if let Some(created) = &info.created {
    println!("Created: {}", created);
}

// List members
for name in info.member_names() {
    println!("Member: {}", name);
}

// Find specific member
if let Some(member) = info.find_member("AE") {
    println!("AE has {} variables", member.variables.len());
}
```

## Builder API

For more control, use the reader builder:

```rust
use xportrs::Xpt;

let dataset = Xpt::reader("data.xpt")
    .row_limit(1000)     // Read only first 1000 rows
    .read()?;            // Read first/only member
```

### Row Limiting

```rust
// Read only first 100 rows (useful for previews)
let preview = Xpt::reader("large.xpt")
    .row_limit(100)
    .read()?;

println!("Preview: {} rows", preview.nrows());
```

## Reading from Buffers

Read from in-memory data:

```rust
use std::io::Cursor;
use xportrs::Xpt;

let xpt_bytes: Vec<u8> = /* ... */;
let cursor = Cursor::new(xpt_bytes);

let dataset = Xpt::reader_from(cursor).read()?;
```

## Accessing Data

Once loaded, access the data through the Dataset API:

```rust
use xportrs::{ColumnData, Xpt};

let dataset = Xpt::read("ae.xpt")?;

// Access by column name
let usubjid = &dataset["USUBJID"];
let aeseq = &dataset["AESEQ"];

// Iterate over column data
if let ColumnData::String(values) = usubjid.data() {
    for (i, value) in values.iter().enumerate() {
        match value {
            Some(s) => println!("Row {}: {}", i, s),
            None => println!("Row {}: <missing>", i),
        }
    }
}

if let ColumnData::F64(values) = aeseq.data() {
    for (i, value) in values.iter().enumerate() {
        match value {
            Some(v) => println!("Row {}: {}", i, v),
            None => println!("Row {}: <missing>", i),
        }
    }
}
```

## Metadata Preservation

xportrs preserves metadata when reading:

```rust
let dataset = Xpt::read("ae.xpt")?;

// Dataset label
if let Some(label) = dataset.dataset_label() {
    println!("Dataset label: {}", label);
}

// Column metadata
for col in dataset.columns() {
    println!("Variable: {}", col.name());
    
    if let Some(label) = col.label() {
        println!("  Label: {}", label);
    }
    
    if let Some(format) = col.format() {
        println!("  Format: {}", format);
    }
    
    if let Some(len) = col.explicit_length() {
        println!("  Length: {}", len);
    }
}
```

## Error Handling

```rust
use xportrs::{Error, Xpt};

match Xpt::read("missing.xpt") {
    Ok(dataset) => println!("Loaded {} rows", dataset.nrows()),
    Err(Error::Io(e)) => eprintln!("IO error: {}", e),
    Err(Error::MemberNotFound { domain_code }) => {
        eprintln!("Member not found: {}", domain_code);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Reading Large Files

For large files, consider:

```rust
use xportrs::Xpt;

// 1. Preview first to understand structure
let info = Xpt::inspect("large.xpt")?;
println!("File has {} members", info.members.len());

// 2. Read with row limit for preview
let preview = Xpt::reader("large.xpt")
    .row_limit(100)
    .read()?;

// 3. Read specific columns of interest
let full = Xpt::read("large.xpt")?;
let columns_of_interest = ["USUBJID", "AETERM", "AESTDTC"];
for name in columns_of_interest {
    if let Some(col) = full.column(name) {
        println!("{}: {} values", name, col.len());
    }
}
```

## Thread Safety

Datasets are `Send + Sync`, allowing concurrent access:

```rust
use std::sync::Arc;
use xportrs::Xpt;

let dataset = Arc::new(Xpt::read("ae.xpt")?);

let handles: Vec<_> = (0..4).map(|i| {
    let ds = Arc::clone(&dataset);
    std::thread::spawn(move || {
        println!("Thread {}: {} rows", i, ds.nrows());
    })
}).collect();

for handle in handles {
    handle.join().unwrap();
}
```

## Example: Read and Process

```rust
use xportrs::{ColumnData, Xpt};

fn process_adverse_events(path: &str) -> xportrs::Result<()> {
    let dataset = Xpt::read(path)?;
    
    // Verify expected columns
    let required = ["USUBJID", "AETERM", "AESEV"];
    for name in required {
        if dataset.column(name).is_none() {
            return Err(xportrs::Error::invalid_data(
                format!("Missing required column: {}", name)
            ));
        }
    }
    
    // Process data
    let usubjid = &dataset["USUBJID"];
    let aeterm = &dataset["AETERM"];
    let aesev = &dataset["AESEV"];
    
    if let (
        ColumnData::String(subjects),
        ColumnData::String(terms),
        ColumnData::String(severities),
    ) = (usubjid.data(), aeterm.data(), aesev.data()) {
        for i in 0..dataset.nrows() {
            let subj = subjects[i].as_deref().unwrap_or("?");
            let term = terms[i].as_deref().unwrap_or("?");
            let sev = severities[i].as_deref().unwrap_or("?");
            println!("{}: {} ({})", subj, term, sev);
        }
    }
    
    Ok(())
}
```
