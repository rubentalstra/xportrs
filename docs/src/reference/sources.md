# Official Sources

This page lists authoritative sources for XPT format and regulatory requirements.

## SAS Documentation

### TS-140: XPT V5 Specification

**The authoritative specification for XPT V5 format.**

- **Title**: Record Layout of a SAS Version 5 or 6 Data Set in SAS Transport (XPORT) Format
- **Publisher**: SAS Institute Inc.
- **Link**: [TS-140 PDF](https://support.sas.com/content/dam/SAS/support/en/technical-papers/record-layout-of-a-sas-version-5-or-6-data-set-in-sas-transport-xport-format.pdf)

Key contents:

- File structure (headers, NAMESTR, observations)
- 140-byte NAMESTR record layout
- IBM floating-point encoding
- Character encoding rules

## FDA Documentation

### Study Data Technical Conformance Guide

**Requirements for electronic study data submissions.**

- **Publisher**: U.S. Food and Drug Administration
- **Link**: [TCG PDF](https://www.fda.gov/media/153632/download)

Key contents:

- Required file formats (XPT V5)
- File size limits (5GB)
- Character encoding (ASCII)
- eCTD placement

### FDA Data Standards Catalog

**Supported CDISC standards and versions.**

- **Link**: [Data Standards Resources](https://www.fda.gov/industry/fda-data-standards-advisory-board/study-data-standards-resources)

## CDISC Standards

### SDTM Implementation Guide

**Standard for tabulation data structure.**

- **Publisher**: CDISC
- **Link**: [SDTM-IG](https://www.cdisc.org/standards/foundational/sdtmig)

Key contents:

- Domain structures (DM, AE, LB, etc.)
- Variable naming conventions
- Controlled terminology requirements
- Metadata requirements

### ADaM Implementation Guide

**Standard for analysis datasets.**

- **Publisher**: CDISC
- **Link**: [ADaM-IG](https://www.cdisc.org/standards/foundational/adam)

Key contents:

- Analysis dataset structures
- Derived variable conventions
- Traceability requirements

### CDISC Controlled Terminology

**Standard coded values for CDISC variables.**

- **Link**: [CDISC CT](https://www.cdisc.org/standards/terminology)

## Format Registries

### Library of Congress

**Format documentation and preservation information.**

- **XPT Format Family**: [FDD 000464](https://www.loc.gov/preservation/digital/formats/fdd/fdd000464.shtml)
- **XPT V5 Specific**: [FDD 000466](https://www.loc.gov/preservation/digital/formats/fdd/fdd000466.shtml)

## Validation Tools

### Pinnacle 21

**Industry-standard CDISC validation tool.**

- **Publisher**: Certara
- **Link**: [Pinnacle 21](https://www.pinnacle21.com/)

Validates:

- XPT file structure
- CDISC standard compliance
- define.xml consistency
- Controlled terminology

### OpenCDISC (Legacy)

**Open-source validation (now Pinnacle 21 Community).**

- **Link**: [Community Downloads](https://www.pinnacle21.com/downloads)

## International Regulators

### PMDA (Japan)

- **Link**: [PMDA Electronic Submission](https://www.pmda.go.jp/english/review-services/electronic-submissions/0001.html)

### NMPA (China)

- **Link**: [NMPA Drug Center](https://www.cde.org.cn/)

### EMA (Europe)

- **Link**: [EMA eSubmission](https://www.ema.europa.eu/en/human-regulatory-overview/research-development/scientific-guidelines/clinical-efficacy-safety/clinical-efficacy-safety-e-submissions)

## Technical References

### IBM Floating-Point

- **Wikipedia**: [IBM Hexadecimal Floating-Point](https://en.wikipedia.org/wiki/IBM_hexadecimal_floating-point)
- **IBM Documentation**: [System/360 Principles](https://www.ibm.com/support/pages/ibm-system360-principles-operation)

### ISO 8601 Date/Time

- **Standard**: [ISO 8601](https://www.iso.org/iso-8601-date-and-time-format.html)

Used for SDTM timing variables (`--DTC`).

### Character Encodings

- **ASCII**: [ANSI X3.4](https://en.wikipedia.org/wiki/ASCII)
- **Latin-1**: [ISO/IEC 8859-1](https://en.wikipedia.org/wiki/ISO/IEC_8859-1)

## Related Tools

### xportr (R Package)

R package for XPT file handling:

- **Link**: [xportr on GitHub](https://github.com/atorus-research/xportr)
- **Docs**: [xportr Documentation](https://atorus-research.github.io/xportr/)

### pyreadstat (Python)

Python library for reading statistical file formats:

- **Link**: [pyreadstat](https://github.com/Roche/pyreadstat)

### haven (R Package)

R package for reading SAS files:

- **Link**: [haven](https://haven.tidyverse.org/)

## Citation

When referencing xportrs in academic or regulatory contexts:

```bibtex
@software{xportrs,
  title = {xportrs: SAS Transport (XPT) file format library for Rust},
  author = {xportrs contributors},
  year = {2024},
  url = {https://github.com/rubentalstra/xportrs},
  license = {MIT OR Apache-2.0},
}
```
