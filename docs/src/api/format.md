# Format Type

The `Format` type represents a SAS display format or informat. It provides parsing and construction of format specifications.

## Overview

SAS formats control how values are displayed or read:

| Format | Description | Example Output |
|--------|-------------|----------------|
| `DATE9.` | Date format | `15JAN2024` |
| `8.2` | Numeric with decimals | `123.45` |
| `$CHAR200.` | Character format | `Hello World` |
| `BEST12.` | Best numeric representation | `123456789012` |

## Creating Formats

### Parsing from String

```rust
use xportrs::Format;

// Date format
let date_fmt = Format::parse("DATE9.")?;
assert_eq!(date_fmt.name(), "DATE");
assert_eq!(date_fmt.length(), 9);

// Numeric format with decimals
let num_fmt = Format::parse("8.2")?;
assert_eq!(num_fmt.name(), "");
assert_eq!(num_fmt.length(), 8);
assert_eq!(num_fmt.decimals(), 2);

// Character format
let char_fmt = Format::parse("$CHAR200.")?;
assert_eq!(char_fmt.name(), "$CHAR");
assert_eq!(char_fmt.length(), 200);
assert!(char_fmt.is_character());
```

### Using Constructors

```rust
use xportrs::Format;

// Numeric format
let num = Format::numeric(8, 2);
assert_eq!(num.length(), 8);
assert_eq!(num.decimals(), 2);

// Character format
let char_fmt = Format::character(200);
assert_eq!(char_fmt.name(), "$CHAR");
assert_eq!(char_fmt.length(), 200);
```

### From NAMESTR Fields

When reading XPT files, formats are reconstructed from NAMESTR fields:

```rust
use xportrs::Format;

// Reconstruct from XPT fields
let format = Format::from_namestr(
    "DATE    ",  // nform (8 bytes, space-padded)
    9,           // nfl (format length)
    0,           // nfd (format decimals)
    1,           // nfj (justification: 0=left, 1=right)
);

assert_eq!(format.name(), "DATE");
assert_eq!(format.length(), 9);
```

## Format Properties

```rust
use xportrs::Format;

let format = Format::parse("$CHAR200.")?;

// Format name (may include $ prefix)
let name: &str = format.name();  // "$CHAR"

// Name without $ prefix
let stripped: &str = format.name_without_prefix();  // "CHAR"

// Total display width
let length: usize = format.length();  // 200

// Decimal places
let decimals: usize = format.decimals();  // 0

// Is it a character format?
let is_char: bool = format.is_character();  // true

// Display representation
println!("{}", format);  // "$CHAR200."
```

## Common Format Patterns

### Date Formats

```rust
use xportrs::Format;

// Standard date formats
let date9 = Format::parse("DATE9.")?;      // 15JAN2024
let date7 = Format::parse("DATE7.")?;      // 15JAN24
let yymmdd = Format::parse("YYMMDD10.")?;  // 2024-01-15
let e8601 = Format::parse("E8601DA10.")?;  // 2024-01-15
```

### DateTime Formats

```rust
let datetime = Format::parse("DATETIME20.")?;  // 15JAN2024:14:30:00
let e8601dt = Format::parse("E8601DT19.")?;    // 2024-01-15T14:30:00
```

### Numeric Formats

```rust
// Bare numeric format
let bare = Format::parse("8.")?;    // 8 characters, 0 decimals
let decimal = Format::parse("8.2")?;  // 8 characters, 2 decimals

// Named numeric formats
let best = Format::parse("BEST12.")?;    // Best representation
let comma = Format::parse("COMMA10.2")?; // Comma-separated
```

### Character Formats

```rust
// Character formats start with $
let char200 = Format::parse("$CHAR200.")?;
let char40 = Format::parse("$40.")?;  // Shorthand for $CHAR40.
```

## Using Formats with Columns

### Setting Format on Column

```rust
use xportrs::{Column, ColumnData, Format};

// Using Format object
let col = Column::new("AESTDTC", data)
    .with_format(Format::character(19));

// Parsing from string
let col = Column::new("AESTDT", data)
    .with_format_str("DATE9.")?;

// Using constructor
let col = Column::new("VALUE", data)
    .with_format(Format::numeric(8, 2));
```

### Setting Informat

Informats control how data is read:

```rust
let col = Column::new("RAWDATE", data)
    .with_informat(Format::parse("DATE9.")?);
```

## Format in XPT Files

When written to XPT, formats are stored in the NAMESTR record:

| Field | Size | Description |
|-------|------|-------------|
| `nform` | 8 bytes | Format name (space-padded) |
| `nfl` | 2 bytes | Format length |
| `nfd` | 2 bytes | Format decimals |
| `nfj` | 2 bytes | Justification (0=left, 1=right) |

```rust
use xportrs::{Column, ColumnData, Format, Xpt};

let col = Column::new("AESTDT", ColumnData::F64(vec![Some(23391.0)]))
    .with_format_str("DATE9.")?;

// When written, NAMESTR will contain:
// nform = "DATE    "
// nfl = 9
// nfd = 0
// nfj = 1 (right-justified)
```

## Format Validation

Invalid format strings return errors:

```rust
use xportrs::Format;

// Missing period
let result = Format::parse("DATE9");
assert!(result.is_err());

// Invalid syntax
let result = Format::parse("INVALID");
assert!(result.is_err());

// Empty string
let result = Format::parse("");
assert!(result.is_err());
```

## Display and Debug

```rust
use xportrs::Format;

let format = Format::parse("DATE9.")?;

// Display: canonical format string
println!("{}", format);  // "DATE9."

// Debug: detailed representation
println!("{:?}", format);  // Format { name: "DATE", length: 9, ... }
```

## Common Traits

```rust
use xportrs::Format;

// Clone
let format2 = format.clone();

// PartialEq
assert_eq!(Format::parse("DATE9.")?, Format::parse("DATE9.")?);

// Debug
println!("{:?}", format);

// Display
println!("{}", format);
```

## FDA Format Recommendations

> [!TIP]
> The FDA recommends avoiding custom SAS formats. Use standard formats like DATE9., DATETIME20., or simple numeric formats.

Recommended formats:

| Type | Recommended Format |
|------|--------------------|
| Date (numeric) | `DATE9.` |
| DateTime (numeric) | `DATETIME20.` |
| Time (numeric) | `TIME8.` |
| Numeric | `8.`, `8.2` |
| Character | `$CHAR200.`, `$40.` |

Avoid:
- Custom user-defined formats
- Formats requiring external catalogs
- Regional-specific formats
