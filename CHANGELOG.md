# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Agency-specific character encoding validation:
  - FDA: Strict ASCII-only for names, labels, and values
  - PMDA: ASCII for names, Japanese (UTF-8) allowed in labels/values
  - NMPA: ASCII for names, Chinese (UTF-8) allowed in labels/values
- Added multi-byte label warning when labels approach byte limits

### Fixed

- Corrected agency validation rules to match actual regulatory requirements
- Fixed documentation claiming V8 write support (detection only)
- Fixed documentation claiming Polars is included by default (it's optional)

## [0.0.1] - 2026-01-08

### Added

- Initial release
- Full SAS Transport V5 format support
- SAS Transport V8 format detection (write support planned)
- IEEE to IBM floating-point conversion and vice versa
- All 28 SAS missing value codes (`.`, `.A`-`.Z`, `._`)
- Streaming API for memory-efficient processing of large files
- Polars DataFrame integration (optional `polars` feature)
- Optional Serde support (`serde` feature)
- CDISC validation rules
- FDA compliance checking

[unreleased]: https://github.com/rubentalstra/xportrs/compare/v0.1.0...HEAD

[0.1.0]: https://github.com/rubentalstra/xportrs/compare/v0.0.1...v0.1.0

[0.0.1]: https://github.com/rubentalstra/xportrs/releases/tag/v0.0.1
