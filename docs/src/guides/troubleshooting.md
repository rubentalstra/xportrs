{{#title Troubleshooting - xportrs Guide}}

# Troubleshooting

This guide covers common issues and their solutions when working with xportrs.

## Validation Errors

### Variable Name Too Long

```
[ERROR] MYLONGVARNAME: Variable name exceeds 8 bytes
```

**Cause**: XPT V5 limits variable names to 8 bytes.

**Solution**: Shorten the variable name to ≤8 characters.

```rust,ignore
# use xportrs::{Column, ColumnData};
# let data = ColumnData::F64(vec![Some(1.0)]);
// Wrong
Column::new("MYLONGVARNAME", data.clone());

// Correct
Column::new("MYVAR", data);
```

### Variable Label Too Long

```
[ERROR] USUBJID: Variable label exceeds 40 bytes
```

**Cause**: XPT V5 limits labels to 40 bytes.

**Solution**: Shorten the label.

```rust,ignore
# use xportrs::{Column, ColumnData};
# let data = ColumnData::F64(vec![Some(1.0)]);
// Wrong (41 characters)
Column::new("VAR", data.clone())
    .with_label("This is a very long label that exceeds 40");

// Correct (40 characters max)
Column::new("VAR", data)
    .with_label("Unique Subject Identifier");
```

### Non-ASCII Characters (FDA)

```
[ERROR] AETERM: Variable label contains non-ASCII characters
```

**Cause**: FDA requires ASCII-only text.

**Solution**: Replace non-ASCII characters.

```rust,ignore
# use xportrs::{Column, ColumnData};
# let data = ColumnData::F64(vec![Some(1.0)]);
// Wrong
Column::new("VAR", data.clone())
    .with_label("Événement indésirable");

// Correct
Column::new("VAR", data)
    .with_label("Adverse Event");

// Or use a helper function
fn to_ascii(s: &str) -> String {
    s.chars().map(|c| match c {
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'à' | 'â' | 'ä' => 'a',
        // ... more mappings
        c if c.is_ascii() => c,
        _ => '?',
    }).collect()
}
```

### Column Length Mismatch

```
Error: Column length mismatch: expected 100, got 99
```

**Cause**: Columns have different numbers of rows.

**Solution**: Ensure all columns have the same length.

```rust,ignore
# use xportrs::{Column, ColumnData, Dataset};
// Wrong
Dataset::new("AE", vec![
    Column::new("A", ColumnData::F64(vec![Some(1.0), Some(2.0)])),  // 2 rows
    Column::new("B", ColumnData::F64(vec![Some(1.0)])),              // 1 row!
]);

// Correct - same length
Dataset::new("AE", vec![
    Column::new("A", ColumnData::F64(vec![Some(1.0), Some(2.0)])),
    Column::new("B", ColumnData::F64(vec![Some(1.0), Some(2.0)])),
]);
```

## Warnings

### Missing Variable Label

```
[WARN] MYVAR: Variable 'MYVAR' is missing a label
```

**Cause**: Variable has no label defined.

**Solution**: Add a label.

```rust,ignore
# use xportrs::{Column, ColumnData};
# let data = ColumnData::F64(vec![Some(1.0)]);
Column::new("MYVAR", data)
    .with_label("My Variable Description");
```

### Missing Dataset Label

```
[WARN] AE: Dataset is missing a label
```

**Cause**: Dataset has no label defined.

**Solution**: Use `with_label` or `set_label`.

```rust,ignore
# use xportrs::{Column, ColumnData, Dataset};
# let columns = vec![Column::new("A", ColumnData::F64(vec![Some(1.0)]))];
// At construction
Dataset::with_label("AE", "Adverse Events", columns.clone());

// Or after
let mut ds = Dataset::new("AE", columns)?;
ds.set_label("Adverse Events");
```

## Reading Errors

### File Not Found

```
Error: No such file or directory (os error 2)
```

**Solution**: Verify the file path exists.

```rust,ignore
use std::path::Path;

let path = "data.xpt";
if !Path::new(path).exists() {
    eprintln!("File not found: {}", path);
}
```

### Invalid XPT Format

```
Error: Invalid header record
```

**Cause**: File is not a valid XPT V5 file.

**Solution**: Verify the file:

- Check it's an XPT file (not XPT V8, SAS7BDAT, etc.)
- Ensure it's not corrupted
- Verify with hex dump that it starts with `HEADER RECORD`

```bash
# Check file header
xxd -l 80 suspect.xpt
```

### Member Not Found

```
Error: MemberNotFound { domain_code: "XX" }
```

**Cause**: Requested member doesn't exist in the file.

**Solution**: Check available members.

```rust,ignore
# use xportrs::Xpt;
# fn main() -> xportrs::Result<()> {
let info = Xpt::inspect("multi.xpt")?;
for name in info.member_names() {
    println!("Available: {}", name);
}
# Ok(())
# }
```

## Writing Errors

### Write Permission Denied

```
Error: Permission denied (os error 13)
```

**Solution**: Check file/directory permissions.

```rust,ignore
use std::fs;

let dir = "/output";
fs::create_dir_all(dir)?;  // Create if missing

// Check write permission
let test_file = format!("{}/test.tmp", dir);
match fs::write(&test_file, "test") {
    Ok(_) => { fs::remove_file(&test_file)?; }
    Err(e) => eprintln!("Cannot write to {}: {}", dir, e),
}
```

### Disk Full

```
Error: No space left on device (os error 28)
```

**Solution**: Free disk space or write to a different location.

## Data Issues

### Precision Loss

```
// Original: 3.141592653589793
// After roundtrip: 3.141592653589792
```

**Cause**: IBM floating-point has slightly less precision than IEEE 754.

**Solution**: For critical values, store as strings or accept minor precision loss (~14-16 digits).

```rust,ignore
# use xportrs::{Column, ColumnData};
// Store as string for exact preservation
Column::new("EXACTVAL", ColumnData::String(vec![
    Some("3.141592653589793".into()),
]));
```

### Missing Values Handling

```rust,ignore
# use xportrs::ColumnData;
# let col_data = ColumnData::F64(vec![Some(1.0), None]);
// Check for missing values
if let ColumnData::F64(values) = &col_data {
    for (i, val) in values.iter().enumerate() {
        if val.is_none() {
            println!("Row {} is missing", i);
        }
    }
}
```

## Format Issues

### Invalid Format String

```
Error: Invalid format syntax: "DATE"
```

**Cause**: Format string missing trailing period.

**Solution**: SAS formats end with a period.

```rust,ignore
# use xportrs::Format;
// Wrong
Format::parse("DATE9");

// Correct
Format::parse("DATE9.");
```

### Format Not Preserved

**Cause**: Format might not be written if name is empty.

**Solution**: Use named formats.

```rust,ignore
# use xportrs::Format;
// May not be preserved (bare numeric format)
Format::parse("8.2");

// Will be preserved (named format)
Format::parse("BEST12.");
Format::parse("DATE9.");
Format::character(200);
```

## Performance Issues

### Slow Reading Large Files

**Solution**: Use row limiting for previews.

```rust,ignore
# use xportrs::Xpt;
# fn main() -> xportrs::Result<()> {
// Preview first 100 rows
let preview = Xpt::reader("large.xpt")
    .row_limit(100)
    .read()?;
# Ok(())
# }
```

### Memory Usage

**Solution**: Process in chunks for very large datasets.

```rust,ignore
# use xportrs::{Dataset, Xpt};
# fn process(_ds: &Dataset) {}
# fn main() -> xportrs::Result<()> {
// Read, process, and release
{
    let dataset = Xpt::read("chunk1.xpt")?;
    process(&dataset);
} // dataset dropped, memory freed

{
    let dataset = Xpt::read("chunk2.xpt")?;
    process(&dataset);
}
# Ok(())
# }
```

## Pinnacle 21 Validation Failures

### SD0063: Label Mismatch

**Cause**: XPT label doesn't match define.xml.

**Solution**: Ensure labels are consistent.

```rust,ignore
# use xportrs::{Column, ColumnData};
# let data = ColumnData::String(vec![Some("001".into())]);
// Label should match define.xml exactly
Column::new("USUBJID", data)
    .with_label("Unique Subject Identifier");  // As in define.xml
```

### SD1001: Variable Name Invalid

**Cause**: Variable name doesn't follow SAS naming rules.

**Solution**: Use uppercase, alphanumeric, start with letter.

```rust,ignore
# use xportrs::{Column, ColumnData};
# let data = ColumnData::F64(vec![Some(1.0)]);
// Wrong
Column::new("1stVar", data.clone());   // Starts with number
Column::new("my-var", data.clone());   // Contains hyphen

// Correct
Column::new("FIRSTVAR", data.clone());
Column::new("MYVAR", data);
```

## Getting Help

If you encounter issues not covered here:

1. Check the [API documentation](../api/dataset.md)
2. Review the [XPT format specification](../format/structure.md)
3. Open an issue on [GitHub](https://github.com/rubentalstra/xportrs/issues)

When reporting issues, include:

- xportrs version
- Rust version
- Minimal code to reproduce
- Error messages
- Sample data (if not confidential)
