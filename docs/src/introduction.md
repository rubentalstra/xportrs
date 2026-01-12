# Introduction

**xportrs
** is a Rust library for reading and writing SAS Transport (XPT) files, the standard format for regulatory submissions to the FDA, PMDA, and other health authorities.

## Why xportrs?

Clinical trial data submitted to regulatory agencies must be in XPT V5 format. While SAS has traditionally been the tool of choice, modern data pipelines increasingly use Python, R, and Rust. xportrs provides:

- **Full CDISC/FDA compliance** - Correct NAMESTR structure, IBM floating-point encoding, and metadata handling
- **Type safety** - Rust's type system prevents common errors at compile time
- **Performance** - Zero-copy parsing where possible, efficient memory usage
- **Validation** - Built-in checks for FDA, PMDA, and NMPA requirements

## Quick Example

```rust
use xportrs::{Column, ColumnData, Dataset, Format, Xpt};

// Create a dataset with full CDISC metadata
let dataset = Dataset::with_label("AE", "Adverse Events", vec![
    Column::new("STUDYID", ColumnData::String(vec![Some("ABC123".into())]))
        .with_label("Study Identifier")
        .with_format(Format::character(20)),
    Column::new("USUBJID", ColumnData::String(vec![Some("ABC123-001".into())]))
        .with_label("Unique Subject Identifier")
        .with_format(Format::character(40)),
    Column::new("AESEQ", ColumnData::F64(vec![Some(1.0)]))
        .with_label("Sequence Number")
        .with_format(Format::numeric(8, 0)),
]) ?;

// Write with FDA validation
Xpt::writer(dataset)
.agency(xportrs::Agency::FDA)
.finalize() ?
.write_path("ae.xpt") ?;
```

## Compliance Matrix

| Requirement                        | Status | Implementation |
|------------------------------------|--------|----------------|
| Variable names ≤8 bytes, uppercase | ✅      | Validated      |
| Variable labels ≤40 bytes          | ✅      | Validated      |
| Dataset names ≤8 bytes             | ✅      | Validated      |
| Character length 1-200 bytes       | ✅      | Validated      |
| Numeric = 8 bytes IBM float        | ✅      | Enforced       |
| ASCII-only for FDA                 | ✅      | Agency rules   |
| File splitting at 5GB              | ✅      | Automatic      |
| SAS epoch (1960) dates             | ✅      | Handled        |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
xportrs = "0.0.6"
```

## Next Steps

- [Quick Start Guide](guides/quickstart.md) - Get up and running in 5 minutes
- [FDA Submission Workflow](guides/fda-submission.md) - Complete walkthrough for regulatory submissions
- [API Reference](api/dataset.md) - Detailed API documentation
- [XPT Format Specification](format/structure.md) - Understanding the file format
