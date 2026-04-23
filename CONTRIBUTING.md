# Contributing to NTL

Thank you for your interest in contributing to the Neural Transfer Layer. NTL is built on the Ubuntu philosophy — *"I am because we are."*

## Getting Started

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/ntl.git
cd ntl

# Build
cargo build --workspace

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace --all-features

# Format
cargo fmt --all
```

## Development Requirements

- Rust 1.85+ (install via [rustup](https://rustup.rs))
- Git

## Making Changes

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes
4. Add tests for new functionality
5. Ensure all checks pass:
   ```bash
   cargo test --workspace
   cargo clippy --workspace --all-features -- -D warnings
   cargo fmt --all -- --check
   ```
6. Submit a pull request

## Project Structure

```
ntl/
├── runtime/ntl-core/    # Core library (signals, synapses, propagation, crypto)
├── runtime/ntl-cli/     # CLI binary
├── runtime/ntl-node/    # Full node binary
├── runtime/ntl-edge/    # Edge node (lightweight)
├── adapters/            # Protocol adapters (web2, web3, legacy)
├── examples/            # Example applications
├── benchmarks/          # Performance benchmarks
├── docs/                # Mintlify documentation
├── rfcs/                # Protocol change proposals
└── spec/                # Formal specification
```

## Specification Changes

Changes to the NTL protocol specification require an RFC. See `rfcs/0000-template.md` for the template and the [RFC process](https://openntl.org/governance/rfc-process) for details.

## Code Style

- Run `cargo fmt` before committing
- Follow clippy recommendations (`cargo clippy`)
- Write doc comments for all public items
- Use `thiserror` for error types
- Prefer `tracing` over `println!` for logging
- No `unsafe` code (enforced by `#![forbid(unsafe_code)]`)

## Testing

- Unit tests go in the same file as the code they test (`#[cfg(test)]` module)
- Integration tests go in `tests/` directories
- Property-based tests use `proptest`
- Benchmarks use `criterion`

## Commit Messages

Use conventional commits:
- `feat:` new feature
- `fix:` bug fix
- `docs:` documentation
- `refactor:` code restructuring
- `test:` adding or updating tests
- `bench:` benchmark changes
- `ci:` CI/CD changes
- `chore:` maintenance

## Code of Conduct

All participants are expected to treat each other with respect, kindness, and good faith. We are building infrastructure for everyone.

## License

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.
