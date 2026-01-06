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
# List all devices (USB and Bluetooth)
cargo run -- check

# List USB devices only
cargo run -- check -t usb

# List Bluetooth devices only
cargo run -- check -t bluetooth

# Start monitoring (interactive setup if no config)
cargo run -- watch

# Start monitoring with CLI args (legacy USB-only mode)
cargo run -- watch -k 1234 -p "External" -d "Default"

# Show CLI help
cargo run -- --help
cargo run -- watch --help
cargo run -- check --help
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

## Configuration Testing
```bash
# Test with new v2 config format (multiple keyboards)
# Create ~/.config/ke_auto_profile_switcher/config.yml with v2 format

# Test legacy v1 config backward compatibility
# Create config with keyboard_id, product_profile, default_profile fields

# Interactive setup test (remove config file first)
rm ~/.config/ke_auto_profile_switcher/config.yml
cargo run -- watch
```
