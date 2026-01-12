# Changelog

All notable changes to xportrs are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Comprehensive mdbook documentation with mermaid diagrams
- Full regulatory compliance documentation
- API reference documentation
- Usage guides and troubleshooting

## [0.0.6] - 2026

### Added

- **Format type with parsing**: New `Format` struct for SAS format handling
    - `Format::parse("DATE9.")` - Parse format strings
    - `Format::numeric(8, 2)` - Create numeric formats
    - `Format::character(200)` - Create character formats
    - `Format::from_namestr()` - Reconstruct from XPT fields

- **Column metadata support**: Extended `Column` struct
    - `.with_label("...")` - Set variable label
    - `.with_format(Format)` - Set display format
    - `.with_format_str("DATE9.")` - Parse and set format
    - `.with_informat(Format)` - Set input format
    - `.with_length(n)` - Set explicit character length

- **Metadata preservation on read**: XPT files now preserve:
    - Variable labels from NAMESTR `nlabel`
    - Display formats from NAMESTR `nform`, `nfl`, `nfd`, `nfj`
    - Input formats from NAMESTR `niform`, `nifl`, `nifd`
    - Explicit character lengths

- **Validation warnings**: New validation issues
    - `MissingVariableLabel` - Warning when label is empty
    - `MissingDatasetLabel` - Warning when dataset label is empty
    - `InvalidFormatSyntax` - Error for malformed format strings

### Changed

- `Dataset::with_label()` now takes `impl Into<Label>` instead of `Option<impl Into<Label>>`
- Added `Dataset::set_label()` for conditional label setting
- Format metadata now correctly written to NAMESTR records (previously hardcoded to 0)

### Fixed

- Format fields `nfl`, `nfd`, `nfj`, `nifl`, `nifd` now contain actual values instead of zeros
- Metadata roundtrip: labels and formats preserved through read → write cycle

## [0.0.5] - 2026

### Added

- CITATION.cff for academic citation
- codemeta.json for metadata
- JSON schema support

## [0.0.4] - 2026

### Added

- Initial public release
- XPT V5 reading and writing
- FDA/PMDA/NMPA agency validation
- Automatic file splitting at 5GB
- IBM floating-point encoding/decoding
- SAS epoch date handling

### Features

- `Xpt::read()` - Read XPT files
- `Xpt::write()` - Write XPT files
- `Xpt::inspect()` - Get file metadata
- `Dataset`, `Column`, `ColumnData` types
- `Agency` enum for regulatory validation
- `Issue` and `Severity` for validation results

## Migration Guide

### From 0.0.5 to 0.0.6

#### Dataset::with_label signature change

```rust,no_run
// Before (0.0.5)
Dataset::with_label("AE", Some("Adverse Events"), columns)

// After (0.0.6)
Dataset::with_label("AE", "Adverse Events", columns)

// For conditional labels
let mut ds = Dataset::new("AE", columns)?;
if let Some(label) = maybe_label {
    ds.set_label(label);
}
```

#### Adding metadata to columns

```rust,no_run
// New in 0.0.6
Column::new("VAR", data)
    .with_label("Variable Label")
    .with_format(Format::character(200))
    .with_length(200)
```

#### Checking for warnings

```rust,no_run
// New warnings in 0.0.6
let validated = Xpt::writer(dataset).finalize()?;

for issue in validated.issues() {
    match issue {
        Issue::MissingVariableLabel { variable } => {
            println!("Warning: {} missing label", variable);
        }
        Issue::MissingDatasetLabel { dataset } => {
            println!("Warning: {} missing label", dataset);
        }
        _ => {}
    }
}
```

## Compatibility

| xportrs Version | Rust Version | MSRV |
|-----------------|--------------|------|
| 0.0.6           | 1.70+        | 1.70 |
| 0.0.5           | 1.70+        | 1.70 |
| 0.0.4           | 1.70+        | 1.70 |

## License

xportrs is dual-licensed under MIT and Apache 2.0.
