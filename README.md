<div align="center">

<img src=".github/logo.svg" alt="xportrs logo" width="140" />

# xportrs

**Pure Rust SAS XPORT (XPT) library for CDISC clinical trial data submissions**

[![Crates.io](https://img.shields.io/crates/v/xportrs.svg)](https://crates.io/crates/xportrs)
[![Documentation](https://docs.rs/xportrs/badge.svg)](https://docs.rs/xportrs)
[![CI](https://github.com/rubentalstra/xportrs/actions/workflows/ci.yml/badge.svg)](https://github.com/rubentalstra/xportrs/actions/workflows/ci.yml)
[![dependency status](https://deps.rs/repo/github/rubentalstra/xportrs/status.svg)](https://deps.rs/repo/github/rubentalstra/xportrs)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![DOI](https://zenodo.org/badge/1130520836.svg)](https://doi.org/10.5281/zenodo.18219718)

</div>

A safe, DataFrame-agnostic implementation of SAS Transport v5 (XPT) file I/O with built-in regulatory compliance validation for FDA, PMDA, and NMPA submissions.

## Features

- **Regulatory Compliance** — Built-in validation for FDA, PMDA, and NMPA submission requirements
- **Read & Write** — Full support for SAS Transport v5 format
- **Auto File Splitting** — Automatically splits files exceeding agency size limits (5 GB)
- **Framework Agnostic** — Works with any in-memory data representation
- **Safe by Design** — Zero unsafe code, no C dependencies

## Regulatory Agency Support

| Agency   | Region        | Character Encoding | Max File Size |
|----------|---------------|--------------------|---------------|
| **FDA**  | United States | ASCII              | 5 GB          |
| **PMDA** | Japan         | UTF-8 (Japanese)   | 5 GB          |
| **NMPA** | China         | UTF-8 (Chinese)    | 5 GB          |

All agencies enforce: 8-byte variable names, 40-byte labels, 200-byte character values.

## Installation

```toml
[dependencies]
xportrs = "0.0.6"
```

## Quick Example

```rust
use xportrs::{Xpt, Agency};

fn main() -> Result<(), xportrs::Error> {
    // Read an XPT file
    let dataset = Xpt::read("dm.xpt")?;

    // Write with FDA compliance validation
    Xpt::writer(dataset)
        .agency(Agency::FDA)
        .finalize()?
        .write_path("dm.xpt")?;

    Ok(())
}
```

For comprehensive examples and API documentation, see [docs.rs/xportrs](https://docs.rs/xportrs).

## Documentation

- [API Reference](https://docs.rs/xportrs) — Full API documentation
- [Examples](examples/README.md) — Code examples for reading, writing, and validation
- [CONTRIBUTING](CONTRIBUTING.md) — Contribution guidelines

## Safety & Quality

| Aspect                  | Status                    |
|-------------------------|---------------------------|
| Unsafe Code             | `#![forbid(unsafe_code)]` |
| External C Dependencies | None                      |
| Minimum Rust Version    | 1.92                      |

## Related Projects

- [xportr](https://github.com/atorus-research/xportr) — R package that inspired this crate
- [Trial Submission Studio](https://github.com/rubentalstra/trial-submission-studio) — Desktop application using xportrs

## License

MIT License — see [LICENSE](LICENSE) for details.
