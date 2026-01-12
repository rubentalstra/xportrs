# Contributing to xportrs

Thank you for your interest in contributing! This guide will help you get started.

## Quick Start

```bash
git clone https://github.com/rubentalstra/xportrs.git
cd xportrs
cargo build
cargo test --all-features
```

**Requirements:** Rust 1.92+ (MSRV)

## Development Commands

| Task   | Command                                                    |
|--------|------------------------------------------------------------|
| Build  | `cargo build`                                              |
| Test   | `cargo test --all-features`                                |
| Lint   | `cargo clippy --all-targets --all-features -- -D warnings` |
| Format | `cargo fmt --all`                                          |
| Docs   | `cargo doc --all-features --open`                          |

## Code Guidelines

- **No unsafe code** — This crate uses `#![forbid(unsafe_code)]`
- **Documentation** — All public items must be documented
- **Testing** — Add tests for new functionality
- **Formatting** — Use `rustfmt` with default settings
- **CDISC terminology** — Use Dataset, Variable, Observation where appropriate

## Pull Request Checklist

Before submitting, ensure:

- [ ] `cargo test --all-features` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo +1.92 check --all-features` (MSRV check)
- [ ] Documentation updated if needed

## Reporting Issues

Please include:

- Rust version (`rustc --version`)
- xportrs version
- Minimal reproduction case
- Expected vs actual behavior

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
