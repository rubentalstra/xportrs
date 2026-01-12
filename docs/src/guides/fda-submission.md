# FDA Submission Workflow

This guide walks through creating FDA-compliant XPT files for regulatory submissions.

## Prerequisites

- Understanding of CDISC SDTM/ADaM standards
- Access to define.xml for your study
- Clinical trial data in a structured format

## Step 1: Design Your Dataset

Plan your dataset structure based on SDTM/ADaM:

```rust
// Example: Adverse Events (AE) domain
// Required SDTM variables: STUDYID, DOMAIN, USUBJID, AESEQ, AETERM, ...

use xportrs::{Column, ColumnData, Dataset, Format, VariableRole};
```

## Step 2: Create the Dataset with Full Metadata

```rust
use xportrs::{Column, ColumnData, Dataset, Format, VariableRole};

fn create_ae_dataset(data: &YourDataSource) -> xportrs::Result<Dataset> {
    let dataset = Dataset::with_label("AE", "Adverse Events", vec![
        // Identifier variables
        Column::with_role(
            "STUDYID",
            VariableRole::Identifier,
            ColumnData::String(data.studyid.clone()),
        )
            .with_label("Study Identifier")
            .with_format(Format::character(20)),
        Column::new("DOMAIN", ColumnData::String(
            vec![Some("AE".into()); data.len()]
        ))
            .with_label("Domain Abbreviation")
            .with_format(Format::character(2))
            .with_length(2),
        Column::with_role(
            "USUBJID",
            VariableRole::Identifier,
            ColumnData::String(data.usubjid.clone()),
        )
            .with_label("Unique Subject Identifier")
            .with_format(Format::character(40)),
        Column::with_role(
            "AESEQ",
            VariableRole::Topic,
            ColumnData::F64(data.aeseq.clone()),
        )
            .with_label("Sequence Number")
            .with_format(Format::numeric(8, 0)),

        // Qualifier variables
        Column::with_role(
            "AETERM",
            VariableRole::Qualifier,
            ColumnData::String(data.aeterm.clone()),
        )
            .with_label("Reported Term for the Adverse Event")
            .with_format(Format::character(200))
            .with_length(200),
        Column::new("AEDECOD", ColumnData::String(data.aedecod.clone()))
            .with_label("Dictionary-Derived Term")
            .with_format(Format::character(200))
            .with_length(200),
        Column::new("AESEV", ColumnData::String(data.aesev.clone()))
            .with_label("Severity/Intensity")
            .with_format(Format::character(10))
            .with_length(10),

        // Timing variables
        Column::with_role(
            "AESTDTC",
            VariableRole::Timing,
            ColumnData::String(data.aestdtc.clone()),
        )
            .with_label("Start Date/Time of Adverse Event")
            .with_format(Format::character(19))
            .with_length(19),
        Column::new("AEENDTC", ColumnData::String(data.aeendtc.clone()))
            .with_label("End Date/Time of Adverse Event")
            .with_format(Format::character(19))
            .with_length(19),
    ])?;

    Ok(dataset)
}
```

## Step 3: Validate for FDA Compliance

```rust
use xportrs::{Agency, Severity, Xpt};

fn validate_for_fda(dataset: Dataset) -> xportrs::Result<xportrs::ValidatedWrite> {
    let validated = Xpt::writer(dataset)
        .agency(Agency::FDA)
        .finalize()?;

    // Report all issues
    println!("Validation Results:");
    println!("  Errors: {}", validated.issues().iter()
        .filter(|i| i.severity() == Severity::Error).count());
    println!("  Warnings: {}", validated.issues().iter()
        .filter(|i| i.severity() == Severity::Warning).count());

    // Detail issues
    for issue in validated.issues() {
        let prefix = match issue.severity() {
            Severity::Error => "ERROR",
            Severity::Warning => "WARN",
            Severity::Info => "INFO",
        };
        println!("  [{}] {}: {}", prefix, issue.target(), issue);
    }

    // Fail on errors
    if validated.has_errors() {
        return Err(xportrs::Error::invalid_data(
            "FDA validation failed with errors"
        ));
    }

    Ok(validated)
}
```

## Step 4: Write the XPT File

```rust
use std::path::Path;

fn write_submission_file(
    validated: xportrs::ValidatedWrite,
    output_dir: &Path,
) -> xportrs::Result<()> {
    let output_path = output_dir.join("ae.xpt");

    // Write (may split if >5GB)
    let paths = validated.write_path(&output_path)?;

    for path in &paths {
        println!("Wrote: {}", path.display());

        // Verify file size
        let size = std::fs::metadata(path)?.len();
        println!("  Size: {} bytes ({:.2} GB)",
                 size, size as f64 / 1_073_741_824.0);
    }

    Ok(())
}
```

## Step 5: Verify the Output

```rust
use xportrs::Xpt;

fn verify_output(path: &str) -> xportrs::Result<()> {
    // Read back
    let dataset = Xpt::read(path)?;

    // Verify structure
    println!("\nVerification:");
    println!("  Domain: {}", dataset.domain_code());
    println!("  Label: {:?}", dataset.dataset_label());
    println!("  Rows: {}", dataset.nrows());
    println!("  Columns: {}", dataset.ncols());

    // Check metadata preserved
    for col in dataset.columns() {
        print!("  {} ", col.name());
        if col.label().is_some() { print!("[label] "); }
        if col.format().is_some() { print!("[format] "); }
        if col.explicit_length().is_some() { print!("[length] "); }
        println!();
    }

    Ok(())
}
```

## Complete Example

```rust
use xportrs::{Agency, Column, ColumnData, Dataset, Format, Severity, Xpt};
use std::path::PathBuf;

fn main() -> xportrs::Result<()> {
    // 1. Create dataset
    let dataset = Dataset::with_label("AE", "Adverse Events", vec![
        Column::new("STUDYID", ColumnData::String(vec![
            Some("ABC-123".into()),
            Some("ABC-123".into()),
        ]))
            .with_label("Study Identifier")
            .with_format(Format::character(20)),
        Column::new("DOMAIN", ColumnData::String(vec![
            Some("AE".into()),
            Some("AE".into()),
        ]))
            .with_label("Domain Abbreviation")
            .with_format(Format::character(2)),
        Column::new("USUBJID", ColumnData::String(vec![
            Some("ABC-123-001".into()),
            Some("ABC-123-002".into()),
        ]))
            .with_label("Unique Subject Identifier")
            .with_format(Format::character(40)),
        Column::new("AESEQ", ColumnData::F64(vec![
            Some(1.0),
            Some(1.0),
        ]))
            .with_label("Sequence Number"),
        Column::new("AETERM", ColumnData::String(vec![
            Some("HEADACHE".into()),
            Some("NAUSEA".into()),
        ]))
            .with_label("Reported Term for the Adverse Event")
            .with_format(Format::character(200))
            .with_length(200),
        Column::new("AESTDTC", ColumnData::String(vec![
            Some("2024-01-15".into()),
            Some("2024-01-16".into()),
        ]))
            .with_label("Start Date/Time of Adverse Event")
            .with_format(Format::character(19)),
    ])?;

    // 2. Validate for FDA
    let validated = Xpt::writer(dataset)
        .agency(Agency::FDA)
        .finalize()?;

    // 3. Report issues
    if !validated.issues().is_empty() {
        println!("Validation Issues:");
        for issue in validated.issues() {
            println!("  [{}] {}", issue.severity(), issue);
        }
    }

    // 4. Check for blocking errors
    if validated.has_errors() {
        eprintln!("Cannot proceed due to validation errors");
        return Err(xportrs::Error::invalid_data("Validation failed"));
    }

    // 5. Write file
    let output = PathBuf::from("output/ae.xpt");
    std::fs::create_dir_all(output.parent().unwrap())?;
    validated.write_path(&output)?;

    // 6. Verify
    let loaded = Xpt::read(&output)?;
    assert_eq!(loaded.domain_code(), "AE");
    assert_eq!(loaded.nrows(), 2);

    println!("\nSuccessfully created ae.xpt for FDA submission");

    Ok(())
}
```

## Checklist

Before submission, verify:

- [ ] Dataset name ≤8 characters, uppercase
- [ ] Variable names ≤8 characters, uppercase
- [ ] Variable labels ≤40 characters, ASCII only
- [ ] Character variables ≤200 bytes
- [ ] All variables have labels
- [ ] Dataset has a label
- [ ] File size ≤5GB (or properly split)
- [ ] Pinnacle 21 validation passed
- [ ] Labels match define.xml

## Common Issues

### Missing Labels

```
[WARN] MYVAR: Variable 'MYVAR' is missing a label
```

**Fix**: Add `.with_label("...")` to all columns.

### Non-ASCII Characters

```
[ERROR] AETERM: Variable label contains non-ASCII characters
```

**Fix**: Replace accented characters (é→e, ñ→n) and special symbols.

### Variable Name Too Long

```
[ERROR] MYLONGNAME: Variable name exceeds 8 bytes
```

**Fix**: Shorten variable names to ≤8 characters.

## Next Steps

- Run Pinnacle 21 validation on generated files
- Verify define.xml consistency
- Package with eCTD structure
