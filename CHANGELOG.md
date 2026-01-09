# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1] - 2026-01-08

### Added

- Initial release
- Full SAS Transport V5 format support
- Full SAS Transport V8 format support (extended limits)
- IEEE to IBM floating-point conversion and vice versa
- All 28 SAS missing value codes (`.`, `.A`-`.Z`, `._`)
- Streaming API for memory-efficient processing of large files
- Polars DataFrame integration (included by default)
- Optional Serde support (`serde` feature)
- CDISC validation rules
- FDA compliance checking

[unreleased]: https://github.com/rubentalstra/xportrs/compare/v0.1.0...HEAD

[0.1.0]: https://github.com/rubentalstra/xportrs/compare/v0.0.1...v0.1.0

[0.0.1]: https://github.com/rubentalstra/xportrs/releases/tag/v0.0.1
