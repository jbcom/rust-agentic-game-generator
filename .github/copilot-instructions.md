# Rust Repository Instructions

## Language & Framework
- **Language**: Rust (edition 2021)
- **Game Engine**: Bevy (where applicable)
- **Async Runtime**: Tokio
- **Error Handling**: thiserror (libraries), anyhow (applications)

## Code Style
- Format with `rustfmt`
- Lint with `clippy` (deny warnings)
- Follow Rust API Guidelines: https://rust-lang.github.io/api-guidelines/

## Commands
```bash
# Check compilation
cargo check --all

# Run tests
cargo test --all

# Format
cargo fmt

# Lint
cargo clippy -- -D warnings

# Build release
cargo build --release

# Build WASM (if applicable)
cargo build --target wasm32-unknown-unknown
```

## Project Structure
- `src/lib.rs` - Library root
- `src/main.rs` - Binary entry (if applicable)
- `tests/` - Integration tests
- `examples/` - Usage examples
- `benches/` - Benchmarks

## Error Handling
- Use `Result<T, E>` for fallible operations
- Use `?` operator for propagation
- Define custom error types with `thiserror`
- Wrap errors with context using `anyhow`

## Documentation
- Document all public items
- Include examples in doc comments
- Use `cargo doc --open` to preview

## Dependencies
- Prefer well-maintained crates
- Pin versions appropriately
- Use workspace dependencies for multi-crate projects
