# xportrs

[![Crates.io](https://img.shields.io/crates/v/xportrs.svg)](https://crates.io/crates/xportrs)
[![Documentation](https://docs.rs/xportrs/badge.svg)](https://docs.rs/xportrs)
[![CI](https://github.com/rubentalstra/xportrs/actions/workflows/ci.yml/badge.svg)](https://github.com/rubentalstra/xportrs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

SAS Transport (XPT) V5/V8 format reader and writer in pure Rust.

## Features

- **Full V5 format support** - Maximum compatibility with regulatory submission systems
- **V8 format support** - Extended limits for variable names (32 chars), labels (256 chars)
- **IEEE ↔ IBM float conversion** - Accurate floating-point conversion between formats
- **All 28 SAS missing value codes** - Standard (`.`) and special (`.A`-`.Z`, `._`)
- **Streaming API** - Memory-efficient processing of large files
- **Optional Polars integration** - Direct DataFrame conversion with `polars` feature

## Installation

```bash
cargo add xportrs
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
xportrs = "0.0.1"
```

### Optional Features

```toml
# Enable Polars DataFrame integration
xportrs = { version = "0.0.1", features = ["polars"] }

# Enable Serde serialization
xportrs = { version = "0.0.1", features = ["serde"] }
```

## Format Comparison

| Feature        | V5 Limit | V8 Limit  |
|----------------|----------|-----------|
| Variable name  | 8 chars  | 32 chars  |
| Variable label | 40 chars | 256 chars |
| Format name    | 8 chars  | 32 chars  |
| Dataset name   | 8 chars  | 32 chars  |

## Usage

### Reading XPT Files

```rust
use std::path::Path;
use xportrs::{read_xpt, XptDataset};

// Read entire dataset
let dataset = read_xpt(Path::new("dm.xpt")) ?;
println!("Dataset: {} ({} rows)", dataset.name, dataset.num_rows());

// Access columns
for column in & dataset.columns {
println!("  {} ({})", column.name, column.xpt_type);
}
```

### Writing XPT Files

```rust
use std::path::Path;
use xportrs::{XptDataset, XptColumn, XptValue, write_xpt};

// Create dataset with columns
let mut dataset = XptDataset::with_columns(
"DM",
vec![
    XptColumn::character("USUBJID", 20).with_label("Unique Subject ID"),
    XptColumn::numeric("AGE").with_label("Age in Years"),
],
);

// Add rows
dataset.add_row(vec![
    XptValue::character("STUDY-001"),
    XptValue::numeric(35.0),
]);

// Write to file
write_xpt(Path::new("dm.xpt"), & dataset) ?;
```

### Streaming Large Files

```rust
use std::path::Path;
use xportrs::read_xpt_streaming;

// Process rows one at a time (memory efficient)
let reader = read_xpt_streaming(Path::new("large_file.xpt")) ?;
for observation in reader {
let row = observation ?;
// Process each row...
}
```

### Missing Values

SAS supports 28 different missing value codes:

```rust
use xportrs::{XptValue, MissingValue};

// Standard missing (.)
let missing = XptValue::numeric_missing();

// Special missing (.A through .Z)
let missing_a = XptValue::numeric_missing_with(MissingValue::Special('A'));

// Check for missing
assert!(missing.is_missing());
```

### Polars Integration

Enable the `polars` feature for DataFrame support:

```toml
[dependencies]
xportrs = { version = "0.0.1", features = ["polars"] }
```

```rust
use std::path::Path;
use xportrs::{read_xpt_to_dataframe, write_dataframe_to_xpt};

// Read XPT to DataFrame
let df = read_xpt_to_dataframe(Path::new("dm.xpt")) ?;

// Write DataFrame to XPT
write_dataframe_to_xpt( & df, "DM", Path::new("dm_out.xpt")) ?;
```

## V8 Format

By default, files are written in V5 format. Use `XptWriterOptions` for V8:

```rust
use std::path::Path;
use xportrs::{XptWriterOptions, XptVersion, write_xpt_with_options};

let options = XptWriterOptions::default ().with_version(XptVersion::V8);
write_xpt_with_options(Path::new("dm.xpt"), & dataset, & options) ?;
```

## MSRV

The minimum supported Rust version is **1.92**.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Related Projects

This crate is used by [Trial Submission Studio](https://github.com/rubentalstra/trial-submission-studio), a desktop
application for transforming clinical trial data into FDA-compliant CDISC formats.
