# Essential Development Commands

## Build and Development
```bash
# Build the project
cargo build

# Build release version
cargo build --release

# Install locally
cargo install --path .

# Quick compilation check
cargo check
```

## Running the Application
```bash
# List USB devices
cargo run -- check

# Start monitoring (interactive setup if no config)
cargo run -- watch

# Start monitoring with CLI args
cargo run -- watch -k 1234 -p "External" -d "Default"

# Show CLI help
cargo run -- --help
cargo run -- watch --help
```

## Testing
```bash
# Run all tests
cargo test

# Run specific module tests
cargo test config::tests::

# Run single test
cargo test test_config_serialization

# Run tests with output
cargo test -- --nocapture
```

## Code Quality and Formatting
```bash
# Linting
cargo clippy

# Formatting
cargo fmt

# Check formatting without changing files
cargo fmt -- --check
```

## Development Workflow
1. Make changes to code
2. Run `cargo check` for quick compilation check
3. Run `cargo test` to ensure tests pass
4. Run `cargo clippy` for linting
5. Run `cargo fmt` for formatting
6. Test functionality with `cargo run -- check` and `cargo run -- watch`