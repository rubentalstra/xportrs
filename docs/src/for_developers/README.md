# For Developers

This section contains information for contributors and developers working on xportrs.

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Git

### Clone and Build

```bash
git clone https://github.com/rubentalstra/xportrs.git
cd xportrs
cargo build
```

### Run Tests

```bash
cargo test --all-features
```

### Run Clippy

```bash
cargo clippy -- -D warnings
```

## Project Structure

```
xportrs/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── dataset/            # Dataset, Column, ColumnData
│   ├── schema/             # Schema derivation
│   ├── validate/           # Validation rules
│   ├── xpt/
│   │   └── v5/             # XPT V5 implementation
│   │       ├── read/       # Reading logic
│   │       └── write/      # Writing logic
│   ├── config/             # Configuration types
│   ├── error/              # Error types
│   └── metadata/           # Metadata types
├── tests/                  # Integration tests
├── docs/                   # mdbook documentation
└── benches/                # Benchmarks (if any)
```

## Adding New Features

### Adding a New Validation Rule

1. Add variant to `Issue` enum in `src/validate/issues.rs`
2. Implement `severity()` method for the new variant
3. Implement `Display` for the new variant
4. Add check in `src/validate/checks_v5.rs`
5. Add tests

### Adding a New Column Type

1. Add variant to `ColumnData` enum in `src/dataset/domain_dataset.rs`
2. Handle in reader (`src/xpt/v5/read/reader.rs`)
3. Handle in writer (`src/xpt/v5/write/writer.rs`)
4. Add `From` implementation
5. Add tests

### Supporting a New Agency

1. Add variant to `Agency` enum
2. Add agency-specific validation in `src/validate/checks_v5.rs`
3. Document in regulatory section

## Code Style

- Follow Rust API Guidelines
- Use `cargo fmt` before committing
- Ensure `cargo clippy -- -D warnings` passes
- Add doc comments for public items
- Include examples in documentation

## Testing

### Unit Tests

Located alongside the code in `mod tests` blocks.

### Integration Tests

Located in `tests/` directory:
- `tests/v5/read.rs` - Reading tests
- `tests/v5/write.rs` - Writing tests
- `tests/api_guidelines.rs` - API compliance tests

### Test Data

Test XPT files are in `tests/data/`.

## Documentation

### Building Docs

```bash
cd docs
mdbook build
```

### Serving Locally

```bash
cd docs
mdbook serve
```

Then open http://localhost:3000

### Adding Pages

1. Create `.md` file in appropriate directory
2. Add entry to `SUMMARY.md`
3. Use mermaid for diagrams

## Pull Request Guidelines

1. Fork the repository
2. Create a feature branch
3. Make changes with tests
4. Ensure CI passes
5. Submit PR with clear description

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag
4. CI publishes to crates.io
