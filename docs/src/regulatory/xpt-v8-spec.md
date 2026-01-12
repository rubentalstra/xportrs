# XPT V8/V9 Specification

The XPT V8/V9 format extends the original V5 format with support for longer variable names and labels.

> [!WARNING]
> XPT V8/V9 format is **not accepted** for FDA regulatory submissions. For regulatory submissions, use [XPT V5 format](xpt-v5-spec.md) only.

## Key Differences from V5

| Feature              | V5 (TS-140)   | V8/V9            |
|----------------------|---------------|------------------|
| Variable name length | 8 bytes       | 32 bytes         |
| Label length         | 40 bytes      | 256 bytes        |
| Number encoding      | IBM float     | IEEE 754         |
| Max observations     | ~2 billion    | Unlimited        |
| Regulatory support   | FDA/PMDA/NMPA | **Not accepted** |

## Format Overview

XPT V8/V9 maintains the same basic structure as V5:

- **80-byte records** for headers
- **Big-endian** byte order
- **Fixed-width** text fields (space-padded)

However, it differs in:

- **Variable names**: Extended from 8 to 32 characters
- **Labels**: Extended from 40 to 256 characters
- **Numeric encoding**: Uses IEEE 754 instead of IBM floating-point

## Use Cases

V8/V9 format may be appropriate for:

- Internal data storage where longer names improve readability
- Non-regulatory data exchange between systems
- Archival purposes where V5 limitations are problematic
- Academic or research datasets not intended for regulatory submission

## Regulatory Considerations

### FDA Submissions

The FDA Data Standards Catalog explicitly requires XPT V5 format. Files in V8/V9 format will be rejected during technical validation.

### CDISC Standards

CDISC standards (SDTM, ADaM) are designed around V5 limitations:

- Variable names: 8 characters maximum
- Labels: 40 characters maximum

Using V8/V9 format with CDISC data defeats the purpose of standardization.

### Best Practice

If your data requires longer names or labels:

1. Use V5-compliant short names in the XPT file
2. Document full names in define.xml metadata
3. Use controlled terminology for consistency

## Official Specification

**SAS Technical Note: Record Layout of a SAS Version 8 or 9 Data Set in SAS Transport Format**

<iframe
src="../assets/pdfs/ts-140-xpt-v8-v9-spec.pdf"
width="100%"
height="600px"
style="border: 1px solid #ccc; border-radius: 4px;">
</iframe>

<p style="text-align: center; margin-top: 10px;">
<a href="../assets/pdfs/ts-140-xpt-v8-v9-spec.pdf" download>Download PDF</a> |
<a href="https://support.sas.com/content/dam/SAS/support/en/technical-papers/record-layout-of-a-sas-version-8-or-9-data-set-in-sas-transport-format.pdf" target="_blank">View on SAS Support</a>
</p>

## xportrs Support

xportrs currently focuses on V5 format for regulatory compliance. V8/V9 support is not a priority as it cannot be used for regulatory submissions.

If you need V8/V9 support for non-regulatory purposes, please [open an issue](https://github.com/rubentalstra/xportrs/issues) to discuss your use case.
