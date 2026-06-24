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
cargo run -- check            # Verify device listing works (USB and Bluetooth)
cargo run -- check -t usb     # Verify USB device listing
cargo run -- check -t bluetooth  # Verify Bluetooth device listing
cargo run -- --help           # Verify CLI help is updated
cargo run -- watch --help     # Verify command-specific help
cargo run -- check --help     # Verify check command help

# Test the actual monitoring (if applicable)
cargo run -- watch            # Test interactive setup or existing config
```

### 4. Documentation Updates
- Update relevant comments if public APIs changed
- Update CLAUDE.md if architecture or commands changed
- Update README.md if user-facing features changed
- Update .serena/memories/*.md if architecture patterns changed
- Ensure CLI help text matches actual functionality

### 5. Error Handling Verification
- Ensure new code follows the `crate::Result<T>` pattern
- Verify error messages are helpful and descriptive
- Test error cases where applicable
- Check both USB and Bluetooth error paths

### 6. Backward Compatibility
- Verify legacy v1 config format still works
- Verify legacy CLI arguments still work
- Run test_legacy_config_compatibility test

## Release Preparation (if applicable)
```bash
# Build release version
cargo build --release

# Install and test locally
cargo install --path .
kaps --help
kaps check
kaps check -t usb
kaps check -t bluetooth
```

## Notes
- This project uses Rust 2021 edition
- All changes should maintain compatibility with macOS and Karabiner-Elements
- Configuration file format should remain backward compatible (v1 and v2)
- CLI interface should be intuitive and well-documented
- Both USB and Bluetooth monitoring should be tested when possible
- Multiple keyboard configurations should be tested
