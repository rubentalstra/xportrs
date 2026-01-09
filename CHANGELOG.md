# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-01-09

### Added

- **Metadata specification system** - `VariableSpec` and `DatasetSpec` for defining variable and dataset metadata
- **Action levels** - `ActionLevel` enum (None/Message/Warn/Stop) for graduated messaging like R xportr
- **Transform error types** - `TransformError` and `SpecError` for transform and specification operations
- **Agency policy framework** - Foundation for FDA, NMPA, and PMDA compliance policies
- **Enhanced validation** - Spec conformance validation rules
- **Project configuration** - Added `.rustfmt.toml`, `clippy.toml`, `.editorconfig` for consistent development
- **New feature flags** - `miette` for rich error diagnostics, `tracing` for structured logging, `full` for all features

### Changed

- **Version bump** - Clean slate release at 0.1.0 (breaking changes from 0.0.x)
- **Package description** - Updated to reflect xportr-inspired functionality
- **Lint configuration** - Enhanced Clippy and Rust lints for better code quality
- **README** - Complete rewrite with professional formatting and xportr acknowledgments

### Inspiration

This release marks the beginning of xportrs' evolution from a low-level XPT codec into a complete CDISC-compliant metadata management system, inspired by the excellent [xportr R package](https://github.com/atorus-research/xportr) by Atorus Research.

## [0.0.1] - 2026-01-08

### Added

- Initial release
- Full SAS Transport V5 format support
- Full SAS Transport V8 format support (extended limits)
- IEEE to IBM floating-point conversion and vice versa
- All 28 SAS missing value codes (`.`, `.A`-`.Z`, `._`)
- Streaming API for memory-efficient processing of large files
- Optional Polars integration (`polars` feature)
- Optional Serde support (`serde` feature)
- CDISC validation rules
- FDA compliance checking

[unreleased]: https://github.com/rubentalstra/xportrs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/rubentalstra/xportrs/compare/v0.0.1...v0.1.0
[0.0.1]: https://github.com/rubentalstra/xportrs/releases/tag/v0.0.1
