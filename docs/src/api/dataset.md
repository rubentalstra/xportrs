# Dataset and Column

The `Dataset` and `Column` types are the core data structures in xportrs for representing XPT datasets.

## Dataset

A `Dataset` represents a single SAS dataset (domain) with columns of data.

### Creating a Dataset

```rust
use xportrs::{Dataset, Column, ColumnData};

// Basic creation
let dataset = Dataset::new("AE", vec![
    Column::new("USUBJID", ColumnData::String(vec![Some("001".into())])),
    Column::new("AESEQ", ColumnData::F64(vec![Some(1.0)])),
])?;

// With dataset label
let dataset = Dataset::with_label("AE", "Adverse Events", vec![
    Column::new("USUBJID", ColumnData::String(vec![Some("001".into())])),
    Column::new("AESEQ", ColumnData::F64(vec![Some(1.0)])),
])?;
```

### Dataset Properties

```rust
// Domain code (dataset name)
let code: &str = dataset.domain_code();

// Dataset label (optional)
let label: Option<&str> = dataset.dataset_label();

// Dimensions
let rows: usize = dataset.nrows();
let cols: usize = dataset.ncols();

// Access columns
let columns: &[Column] = dataset.columns();
```

### Setting the Label

```rust
// Using with_label at construction
let dataset = Dataset::with_label("AE", "Adverse Events", columns)?;

// Or set later
let mut dataset = Dataset::new("AE", columns)?;
dataset.set_label("Adverse Events");
```

### Accessing Columns

```rust
// By index
let first_col: &Column = &dataset[0];

// By name
let usubjid: &Column = &dataset["USUBJID"];

// Find column (returns Option)
let col: Option<&Column> = dataset.column("AESEQ");
```

### Iterating

```rust
// Iterate over columns
for col in dataset.iter() {
    println!("{}: {}", col.name(), col.len());
}

// Column names only
for name in dataset.column_names() {
    println!("{}", name);
}

// Consuming iterator
for col in dataset {
    // col is owned Column
}
```

### Extending a Dataset

```rust
let mut dataset = Dataset::new("AE", vec![
    Column::new("A", ColumnData::F64(vec![Some(1.0)])),
])?;

// Add more columns
dataset.extend([
    Column::new("B", ColumnData::F64(vec![Some(2.0)])),
    Column::new("C", ColumnData::F64(vec![Some(3.0)])),
]);

assert_eq!(dataset.ncols(), 3);
```

## Column

A `Column` represents a single variable with its data and metadata.

### Creating a Column

```rust
use xportrs::{Column, ColumnData, Format, VariableRole};

// Basic column
let col = Column::new("USUBJID", ColumnData::String(vec![
    Some("001".into()),
    Some("002".into()),
]));

// With full metadata
let col = Column::new("AESTDTC", ColumnData::String(vec![Some("2024-01-15".into())]))
    .with_label("Start Date/Time of Adverse Event")
    .with_format(Format::character(19))
    .with_length(19);

// With role
let col = Column::with_role(
    "USUBJID",
    VariableRole::Identifier,
    ColumnData::String(vec![Some("001".into())]),
);
```

### Column Properties

```rust
// Name
let name: &str = col.name();

// Label (optional)
let label: Option<&xportrs::Label> = col.label();

// Data
let data: &ColumnData = col.data();

// Length
let len: usize = col.len();

// Explicit length override
let explicit_len: Option<usize> = col.explicit_length();

// Role
let role: Option<VariableRole> = col.role();

// Format
let format: Option<&Format> = col.format();

// Informat
let informat: Option<&Format> = col.informat();
```

### Builder Methods

```rust
let col = Column::new("VAR", data)
    .with_label("Variable Label")
    .with_format(Format::numeric(8, 2))
    .with_informat(Format::numeric(8, 2))
    .with_length(200);

// Parse format from string
let col = Column::new("DATE", data)
    .with_format_str("DATE9.")?;
```

## ColumnData

`ColumnData` is an enum representing the typed data within a column.

### Variants

```rust
use xportrs::ColumnData;

// Floating-point numbers
let floats = ColumnData::F64(vec![Some(1.0), Some(2.0), None]);

// Integers (converted to f64 on write)
let ints = ColumnData::I64(vec![Some(1), Some(2), None]);

// Booleans (converted to f64: 1.0/0.0)
let bools = ColumnData::Bool(vec![Some(true), Some(false), None]);

// Strings
let strings = ColumnData::String(vec![Some("hello".into()), None]);

// Binary data
let bytes = ColumnData::Bytes(vec![Some(vec![0x01, 0x02]), None]);

// Dates (chrono::NaiveDate)
use chrono::NaiveDate;
let dates = ColumnData::Date(vec![
    Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
    None,
]);

// DateTimes (chrono::NaiveDateTime)
use chrono::NaiveDateTime;
let datetimes = ColumnData::DateTime(vec![/* ... */]);

// Times (chrono::NaiveTime)
use chrono::NaiveTime;
let times = ColumnData::Time(vec![/* ... */]);
```

### From Conversions

```rust
// From Vec<f64>
let data: ColumnData = vec![1.0, 2.0, 3.0].into();

// From Vec<&str>
let data: ColumnData = vec!["a", "b", "c"].into();

// From Vec<String>
let data: ColumnData = vec!["a".to_string(), "b".to_string()].into();

// From Vec<i64>
let data: ColumnData = vec![1i64, 2, 3].into();

// From Vec<bool>
let data: ColumnData = vec![true, false, true].into();
```

### Accessing Data

```rust
match col.data() {
    ColumnData::F64(values) => {
        for value in values {
            match value {
                Some(v) => println!("Value: {}", v),
                None => println!("Missing"),
            }
        }
    }
    ColumnData::String(values) => {
        for value in values {
            if let Some(s) = value {
                println!("Value: {}", s);
            }
        }
    }
    // ... handle other variants
    _ => {}
}
```

## Common Traits

Both `Dataset` and `Column` implement standard Rust traits:

```rust
use xportrs::{Dataset, Column};

// Clone
let dataset2 = dataset.clone();
let col2 = col.clone();

// Debug
println!("{:?}", dataset);
println!("{:?}", col);

// Display
println!("{}", dataset);  // "AE (10 rows, 5 cols)"
println!("{}", col);      // "USUBJID: String[10]"

// PartialEq
assert_eq!(dataset1, dataset2);
assert_eq!(col1, col2);

// Send + Sync (thread-safe)
std::thread::spawn(move || {
    println!("{}", dataset.nrows());
});
```

## Error Handling

Dataset creation can fail:

```rust
use xportrs::{Dataset, Column, ColumnData, Error};

// Column length mismatch
let result = Dataset::new("AE", vec![
    Column::new("A", ColumnData::F64(vec![Some(1.0), Some(2.0)])),
    Column::new("B", ColumnData::F64(vec![Some(1.0)])),  // Different length!
]);

match result {
    Ok(ds) => println!("Created dataset"),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Example: Complete Dataset

```rust
use xportrs::{Column, ColumnData, Dataset, Format, VariableRole, Xpt};

fn create_ae_dataset() -> xportrs::Result<Dataset> {
    let dataset = Dataset::with_label("AE", "Adverse Events", vec![
        Column::with_role(
            "STUDYID",
            VariableRole::Identifier,
            ColumnData::String(vec![Some("ABC-123".into())]),
        )
        .with_label("Study Identifier")
        .with_format(Format::character(20)),

        Column::with_role(
            "USUBJID",
            VariableRole::Identifier,
            ColumnData::String(vec![Some("ABC-123-001".into())]),
        )
        .with_label("Unique Subject Identifier")
        .with_format(Format::character(40)),

        Column::with_role(
            "AESEQ",
            VariableRole::Topic,
            ColumnData::F64(vec![Some(1.0)]),
        )
        .with_label("Sequence Number")
        .with_format(Format::numeric(8, 0)),

        Column::new("AETERM", ColumnData::String(vec![Some("HEADACHE".into())]))
            .with_label("Reported Term for the Adverse Event")
            .with_format(Format::character(200))
            .with_length(200),

        Column::new("AESTDTC", ColumnData::String(vec![Some("2024-01-15".into())]))
            .with_label("Start Date/Time of Adverse Event")
            .with_format(Format::character(19))
            .with_length(19),
    ])?;

    Ok(dataset)
}
```
