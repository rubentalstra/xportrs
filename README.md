# xportrs <img src="logo.png" align="right" alt="" width="120" />

[![Crates.io](https://img.shields.io/crates/v/xportrs.svg)](https://crates.io/crates/xportrs)
[![Documentation](https://docs.rs/xportrs/badge.svg)](https://docs.rs/xportrs)
[![CI](https://github.com/rubentalstra/xportrs/actions/workflows/ci.yml/badge.svg)](https://github.com/rubentalstra/xportrs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.92-blue.svg)](https://blog.rust-lang.org/)
[![dependency status](https://deps.rs/repo/github/rubentalstra/xportrs/status.svg)](https://deps.rs/repo/github/rubentalstra/xportrs)

**Pure Rust SAS XPORT (XPT) reader and writer for CDISC clinical trial data submissions.**

`xportrs` provides a safe, DataFrame-agnostic implementation of XPT v5 I/O with built-in regulatory compliance validation for FDA, PMDA, and NMPA submissions.

## Features

- **DataFrame-agnostic** - Works with any in-memory table representation
- **Agency compliance** - Built-in validation for FDA, PMDA, and NMPA requirements
- **Auto file splitting** - Automatically splits large files to meet agency size limits (5GB)
- **XPT v5 support** - Full read and write support for SAS XPORT v5 format
- **Configurable** - Text encoding modes, validation strictness, and more

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
xportrs = "0.0.2"
```

With optional features:

```toml
[dependencies]
xportrs = { version = "0.0.2", features = ["serde", "tracing"] }
```

## Quick Start

### Reading XPT Files

```rust
use xportrs::Xpt;

// Simple: read the first dataset
let dataset = Xpt::read("ae.xpt") ?;
println!("Domain: {}", dataset.domain_code());
println!("Rows: {}", dataset.nrows());

// Read a specific member from a multi-dataset file
let dm = Xpt::reader("study.xpt") ?.read_member("DM") ?;

// Read all members
let datasets = Xpt::reader("study.xpt") ?.read_all() ?;

// Inspect file metadata without loading data
let info = Xpt::inspect("data.xpt") ?;
for name in info.member_names() {
println ! ("Member: {}", name);
}
```

### Writing XPT Files

```rust
use xportrs::{Xpt, Dataset, Column, ColumnData};

// Create a dataset
let dataset = Dataset::new(
"AE".to_string(),
vec![
    Column::new("USUBJID", ColumnData::String(vec![
        Some("01-001".into()),
        Some("01-002".into()),
    ])),
    Column::new("AESEQ", ColumnData::I64(vec![Some(1), Some(1)])),
    Column::new("AESTDY", ColumnData::F64(vec![Some(15.0), Some(22.0)])),
],
) ?;

// Write with structural validation only
Xpt::writer(dataset)
.finalize() ?
.write_path("ae.xpt") ?;
```

## Agency Compliance

When submitting clinical trial data to regulatory agencies, use the
`agency()` method to enable agency-specific validation rules:

```rust
use xportrs::{Xpt, Agency, Dataset};

let dataset = Dataset::new("AE", vec![/* ... */]) ?;

// FDA submission - applies all FDA validation rules
let files = Xpt::writer(dataset)
.agency(Agency::FDA)
.finalize() ?
.write_path("ae.xpt") ?;

// Returns Vec<PathBuf> - multiple files if splitting occurred
println!("Created {} file(s)", files.len());
```

### Supported Agencies

| Agency         | Description                                      | Max File Size |
|----------------|--------------------------------------------------|---------------|
| `Agency::FDA`  | U.S. Food and Drug Administration                | 5 GB          |
| `Agency::PMDA` | Japan Pharmaceuticals and Medical Devices Agency | 5 GB          |
| `Agency::NMPA` | China National Medical Products Administration   | 5 GB          |

### Agency Validation Rules

When an agency is specified, the following validations are applied:

- **ASCII-only** names, labels, and character values
- **Dataset names**: max 8 bytes, uppercase alphanumeric, must start with letter
- **Variable names**: max 8 bytes, uppercase alphanumeric with underscores
- **Labels**: max 40 bytes
- **Character values**: max 200 bytes
- **File naming**: dataset name must match file stem (case-insensitive)

## Automatic File Splitting

Large XPT files are automatically split when an agency is specified:

```rust
use xportrs::{Xpt, Agency, Dataset};

// Large dataset with millions of rows
let large_dataset = Dataset::new("LB", /* ... */) ?;

// Files > 5GB are automatically split into numbered parts
let files = Xpt::writer(large_dataset)
.agency(Agency::FDA)
.finalize() ?
.write_path("lb.xpt") ?;

// Result: ["lb_001.xpt", "lb_002.xpt", ...] if split
// Result: ["lb.xpt"] if no split needed
```

## Data Types

`xportrs` supports the following column data types:

| Rust Type            | XPT Type  | Description                        |
|----------------------|-----------|------------------------------------|
| `ColumnData::F64`    | Numeric   | 64-bit floating point              |
| `ColumnData::I64`    | Numeric   | 64-bit integer (stored as float)   |
| `ColumnData::String` | Character | Variable-length text (1-200 bytes) |

All types support `Option<T>` for missing values (SAS missing = `.`).

## Validation Issues

The library provides detailed validation feedback:

```rust
use xportrs::{Xpt, Agency, Dataset};

let plan = Xpt::writer(dataset)
.agency(Agency::FDA)
.finalize() ?;

// Check for issues before writing
if plan.has_errors() {
for issue in plan.issues() {
eprintln ! ("{}", issue);
}
}

if plan.has_warnings() {
for issue in plan.issues().iter().filter( | i | i.is_warning()) {
println ! ("Warning: {}", issue);
}
}

plan.write_path("ae.xpt") ?;
```

## CDISC Terminology

This crate uses CDISC SDTM vocabulary:

| Term               | Description                                                          |
|--------------------|----------------------------------------------------------------------|
| **Domain dataset** | A table identified by a domain code (e.g., "AE", "DM", "LB")         |
| **Observation**    | One row/record in the dataset                                        |
| **Variable**       | One column; may have a role (Identifier/Topic/Timing/Qualifier/Rule) |

## Feature Flags

| Feature   | Description                                        |
|-----------|----------------------------------------------------|
| `serde`   | Enable serialization/deserialization support       |
| `tracing` | Enable structured logging with the `tracing` crate |
| `full`    | Enable all optional features                       |

```toml
# Enable all features
xportrs = { version = "0.0.2", features = ["full"] }
```

## Temporal Utilities

Convert between Rust chrono types and SAS date/time values:

```rust
use xportrs::temporal::{
    sas_days_since_1960,
    sas_seconds_since_1960,
    date_from_sas_days,
    datetime_from_sas_seconds,
};
use chrono::NaiveDate;

// Convert Rust date to SAS days
let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
let sas_days = sas_days_since_1960( & date);

// Convert SAS days back to Rust date
let back = date_from_sas_days(sas_days);
```

## Safety

This crate is built with
`#![forbid(unsafe_code)]`. All binary parsing and encoding uses safe Rust constructs. The library has been designed with security in mind:

- No unsafe code blocks
- No external C dependencies
- Comprehensive input validation
- Protection against malformed files

## Minimum Supported Rust Version (MSRV)

The minimum supported Rust version is **1.92**.

## Related Projects

- [xportr (R)](https://github.com/atorus-research/xportr) - The R package that inspired this crate
- [Trial Submission Studio](https://github.com/rubentalstra/trial-submission-studio) - Desktop application using xportrs

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests (`cargo test`)
4. Run clippy (`cargo clippy --all-targets --all-features -- -D warnings`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request
