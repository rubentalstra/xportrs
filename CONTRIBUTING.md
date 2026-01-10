# Contributing to `xportrs`

Thank you for your interest in contributing to
`xportrs`! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- Rust 1.92 or later (MSRV)
- Git

### Getting Started

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/your-username/xportrs.git
   cd xportrs
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run the tests:
   ```bash
   cargo test --all-features
   ```

## Development Commands

### Building

```bash
# Standard build
cargo build

# Build with all features
cargo build --all-features

# Release build
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test --all-features

# Run a specific test
cargo test test_name

# Run tests with output
cargo test --all-features -- --nocapture
```

### Linting

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

### Documentation

```bash
# Build and open documentation
cargo doc --all-features --open

# Check MSRV compatibility
cargo +1.92 check --all-features
```

## Code Style

### General Guidelines

- Follow Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
- Use `rustfmt` for formatting (default settings)
- All public items must have documentation
- Use meaningful variable and function names

### Specific Conventions

- **No unsafe code**: This crate uses `#![forbid(unsafe_code)]`
- **Error handling**: Use the crate's `Error` type and `Result` alias
- **Documentation**: Use backticks for code references (e.g., `DataFrame`)
- **Tests**: Add tests for new functionality

### Naming

- Use CDISC terminology where appropriate (Dataset, Variable, Observation)
- Use standard Rust conventions (snake_case for functions, CamelCase for types)
- Prefix internal items with `_` or use `pub(crate)` visibility

## Architecture Overview

### Module Structure

```
src/
├── lib.rs          # Public exports and crate documentation
├── api.rs          # Xpt entry point and XptReaderBuilder
├── write_plan.rs   # XptWriterBuilder and ValidatedWrite
├── dataset/        # Dataset, Column, ColumnData types
├── agency/         # Agency validation rules (FDA, PMDA, NMPA)
├── validate/       # Validation logic and Issue reporting
├── xpt/            # XPT format implementation
│   ├── v5/         # XPT v5 (fully implemented)
│   └── v8/         # XPT v8 (API-ready, not implemented)
├── schema/         # Schema derivation (internal)
├── metadata/       # Metadata types (internal)
└── config/         # Configuration types (internal)
```

### Key Design Decisions

1. **Builder Pattern**: Write operations use a builder pattern with validation:
    - `Xpt::writer(dataset)` returns `XptWriterBuilder`
    - `.finalize()` validates and returns `ValidatedWrite`
    - `.write_path()` performs the actual write

2. **Private Fields
   **: All struct fields are private with accessor methods to allow future changes without breaking the API.

3. **Newtypes**: `DomainCode` and `Label` are newtypes for type safety.

4. **Feature Flags**: Optional integrations are behind feature flags:
    - `polars` - Polars DataFrame integration
    - `serde` - Serialization support
    - `tracing` - Structured logging

## Pull Request Process

1. **Create a branch** from `main` for your changes
2. **Make your changes** with appropriate tests
3. **Run the full test suite**: `cargo test --all-features`
4. **Run clippy**: `cargo clippy --all-targets --all-features -- -D warnings`
5. **Format your code**: `cargo fmt --all`
6. **Update documentation** if needed
7. **Submit a pull request** with a clear description

### PR Checklist

- [ ] Tests pass (`cargo test --all-features`)
- [ ] Clippy passes (`cargo clippy --all-targets --all-features -- -D warnings`)
- [ ] Code is formatted (`cargo fmt --all --check`)
- [ ] Documentation is updated
- [ ] MSRV is maintained (`cargo +1.92 check --all-features`)

## Reporting Issues

When reporting bugs, please include:

- Rust version (`rustc --version`)
- `xportrs` version
- Minimal reproduction case
- Expected vs actual behavior

## License

By contributing to `xportrs`, you agree that your contributions will be licensed under the MIT License.
