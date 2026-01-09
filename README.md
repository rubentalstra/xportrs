# xportrs

[![Crates.io](https://img.shields.io/crates/v/xportrs.svg)](https://crates.io/crates/xportrs)
[![Documentation](https://docs.rs/xportrs/badge.svg)](https://docs.rs/xportrs)
[![CI](https://github.com/rubentalstra/xportrs/actions/workflows/ci.yml/badge.svg)](https://github.com/rubentalstra/xportrs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.92-blue.svg)](https://blog.rust-lang.org/)

**A Rust implementation of CDISC-compliant XPT file generation, inspired
by [R's xportr package](https://github.com/atorus-research/xportr).**

xportrs brings the power of metadata-driven clinical data transformation to the Rust ecosystem, with first-class Polars
integration and support for FDA, NMPA, and PMDA regulatory requirements.

## Features

- **Metadata-driven transformations** - Apply type, length, label, order, and format from specifications
- **Agency compliance** - Built-in policies for FDA, NMPA (China), and PMDA (Japan)
- **Polars-first design** - Native DataFrame integration with extension traits
- **Full XPT V5/V8 support** - Read and write SAS Transport files
- **IEEE to IBM float conversion** - Accurate floating-point handling
- **All 28 SAS missing value codes** - Standard (`.`) and special (`.A`-`.Z`, `._`)
- **Streaming API** - Memory-efficient processing of large datasets
- **Validation framework** - Configurable strictness with detailed reporting

## xportr-Style Workflow

xportrs provides the same metadata-driven workflow as R's xportr:

| R xportr            | Rust xportrs         | Description                       |
|---------------------|----------------------|-----------------------------------|
| `xportr_type()`     | `.xportr_type()`     | Coerce column types to match spec |
| `xportr_length()`   | `.xportr_length()`   | Apply variable lengths from spec  |
| `xportr_label()`    | `.xportr_label()`    | Apply variable labels from spec   |
| `xportr_order()`    | `.xportr_order()`    | Reorder columns to match spec     |
| `xportr_format()`   | `.xportr_format()`   | Apply SAS formats from spec       |
| `xportr_df_label()` | `.xportr_df_label()` | Set dataset label                 |
| `xportr_metadata()` | `.xportr_metadata()` | Attach specification to DataFrame |
| `xportr_write()`    | `.xportr_write()`    | Write compliant XPT file          |
| `xportr()`          | `.xportr()`          | Full pipeline in one call         |

### Step-by-Step Control

```rust
use polars::prelude::*;
use xportrs::polars::XportrTransforms;
use xportrs::spec::{DatasetSpec, VariableSpec};
use xportrs::ActionLevel;

let spec = DatasetSpec::new("DM")
.with_label("Demographics")
.add_variable(VariableSpec::numeric("AGE").with_label("Age").with_order(1))
.add_variable(VariableSpec::character("SEX", 1).with_label("Sex").with_order(2));

let df = df! {
    "AGE" => &[35i64, 42],
    "SEX" => &["M", "F"],
}?;

let result = df
.xportr_metadata(spec.clone())
.xportr_type( & spec, ActionLevel::Warn) ?
.xportr_length( & spec, ActionLevel::Warn) ?
.xportr_label( & spec, ActionLevel::Warn) ?
.xportr_order( & spec, ActionLevel::Message) ?
.xportr_format( & spec, ActionLevel::Message) ?
.xportr_df_label("Demographics");

// Check report before writing
if result.report().has_warnings() {
eprintln ! ("Warnings found during transformation");
}

result.xportr_write("dm.xpt", "DM", & spec, true) ?;
```

## Installation

```bash
cargo add xportrs
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
xportrs = "0.1"
```

## Quick Start

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

Polars DataFrame integration is included by default:

```rust
use std::path::Path;
use xportrs::{read_xpt_to_dataframe, write_dataframe_to_xpt};

// Read XPT to DataFrame
let df = read_xpt_to_dataframe(Path::new("dm.xpt")) ?;

// Write DataFrame to XPT
write_dataframe_to_xpt(Path::new("dm_out.xpt"), & df, "DM") ?;
```

## Agency Compliance

### FDA (Default)

```rust
use xportrs::validation::Validator;
use xportrs::XptVersion;

// Use FDA-compliant validation mode
let validator = Validator::fda_compliant(XptVersion::V5);
let report = validator.validate( & dataset);

if ! report.is_valid() {
for error in & report.errors {
eprintln ! ("{}", error);
}
}
```

### Custom Policy

Create custom policies with specific constraints:

```rust
use xportrs::{CustomPolicy, XptVersion};
use xportrs::policy::AgencyPolicy;

// Research policy - relaxed for V8 format
let policy = CustomPolicy::new()
.with_required_version(XptVersion::V8)
.with_max_variable_name_length(32)
.with_require_ascii(false)
.with_no_file_size_limit();

// Internal QC - stricter than FDA
let qc_policy = CustomPolicy::from_fda_base()
.with_max_file_size(1024 * 1024 * 1024)  // 1 GB limit
.with_strict(true);
```

## Loading Specifications

Load specifications from any source using Polars, then pass to xportrs:

```rust
use polars::prelude::*;
use xportrs::spec::{DataFrameMetadataSource, MetadataSource};

// Load from CSV, Excel, Parquet, etc. via Polars
let spec_df = CsvReadOptions::default ()
.try_into_reader_with_file_path(Some("specs/var_spec.csv".into())) ?
.finish() ?;

// Create specification from DataFrame
let source = DataFrameMetadataSource::new(spec_df);
let dm_spec = source.load_dataset_spec("DM") ?;
```

## Format Comparison

| Feature        | V5 Limit | V8 Limit  |
|----------------|----------|-----------|
| Variable name  | 8 chars  | 32 chars  |
| Variable label | 40 chars | 256 chars |
| Format name    | 8 chars  | 32 chars  |
| Dataset name   | 8 chars  | 32 chars  |

V5 is the default for FDA compliance. Use `XptWriterOptions::with_version(XptVersion::V8)` for extended limits.

## V8 Format

By default, files are written in V5 format. Use `XptWriterOptions` for V8:

```rust
use std::path::Path;
use xportrs::{XptWriterOptions, XptVersion, write_xpt_with_options};

let options = XptWriterOptions::default ().with_version(XptVersion::V8);
write_xpt_with_options(Path::new("dm.xpt"), & dataset, & options) ?;
```

## Feature Flags

| Feature   | Description                         |
|-----------|-------------------------------------|
| `serde`   | Serialization support               |
| `tracing` | Structured logging                  |
| `full`    | All optional features enabled       |

Note: Polars DataFrame integration is included by default (not optional).

## MSRV

The minimum supported Rust version is **1.92**.

## Acknowledgments

This crate is inspired by and aims for feature parity with the excellent [**xportr
**](https://github.com/atorus-research/xportr) R package developed
by [Atorus Research](https://www.atorusresearch.com/). The xportr package has been instrumental in establishing best
practices for CDISC-compliant XPT file generation in the clinical trials community.

Special thanks to the xportr maintainers and contributors:

- [Eli Miller](https://github.com/elimillera)
- [Ben Straub](https://github.com/bms63)
- And all contributors to the xportr project

## Related Projects

- [xportr (R)](https://github.com/atorus-research/xportr) - The R package that inspired this crate
- [Trial Submission Studio](https://github.com/rubentalstra/trial-submission-studio) - Desktop application using xportrs

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
