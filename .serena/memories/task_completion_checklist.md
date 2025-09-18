# Task Completion Checklist

## Before Committing Code Changes

### 1. Code Quality Checks
```bash
# Check compilation
cargo check

# Run linting
cargo clippy

# Format code
cargo fmt

# Verify formatting (optional)
cargo fmt -- --check
```

### 2. Testing
```bash
# Run all tests
cargo test

# Run specific module tests if working on a particular module
cargo test config::tests::  # Example for config module
```

### 3. Functionality Testing
```bash
# Test basic functionality
cargo run -- check        # Verify USB device listing works
cargo run -- --help       # Verify CLI help is updated
cargo run -- watch --help # Verify command-specific help

# Test the actual monitoring (if applicable)
cargo run -- watch        # Test interactive setup or existing config
```

### 4. Documentation Updates
- Update relevant comments if public APIs changed
- Update CLAUDE.md if architecture or commands changed
- Ensure CLI help text matches actual functionality

### 5. Error Handling Verification
- Ensure new code follows the `crate::Result<T>` pattern
- Verify error messages are helpful and descriptive
- Test error cases where applicable

## Release Preparation (if applicable)
```bash
# Build release version
cargo build --release

# Install and test locally
cargo install --path .
kaps --help
kaps check
```

## Notes
- This project uses Rust 2021 edition
- All changes should maintain compatibility with macOS and Karabiner-Elements
- Configuration file format should remain backward compatible
- CLI interface should be intuitive and well-documented