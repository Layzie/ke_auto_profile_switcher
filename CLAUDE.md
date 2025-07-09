# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# 言語設定
- 日本語で応答してください

## Project Overview

This is a Rust CLI tool called `kaps` (Karabiner Auto Profile Switcher) that automatically switches Karabiner-Elements profiles based on USB keyboard connection status. The tool monitors USB device events and switches between configured profiles when external keyboards are connected or disconnected.

## Architecture

The application follows a modular architecture with clear separation of concerns:

- **Modular Design**: Code is organized into focused modules in `src/lib.rs`
- **Error Handling**: Custom error types with `thiserror` for detailed error reporting
- **Configuration Management**: Multiple configuration sources with priority: config file → CLI args → interactive setup
- **USB Monitoring**: Event-driven USB device monitoring with callback-based profile switching
- **CLI Interface**: Built with `clap` for comprehensive command-line argument parsing

### Module Structure

- **`src/main.rs`**: Minimal entry point (51 lines) - orchestrates CLI parsing and delegates to modules
- **`src/config/`**: Configuration management with YAML serialization, interactive setup, and comprehensive tests
- **`src/cli.rs`**: CLI argument definitions and parsing structures
- **`src/usb_monitor.rs`**: USB device enumeration and event monitoring with callback support
- **`src/karabiner.rs`**: Karabiner-Elements CLI integration and profile switching
- **`src/error.rs`**: Custom error types (`AppError`) with proper error chaining
- **`src/constants.rs`**: Centralized application constants (paths, defaults, intervals)

### Key Architecture Patterns

- **Configuration Resolution**: `resolve_config()` function implements priority-based config sourcing
- **Event-Driven USB Monitoring**: Callback-based design allows flexible response to USB events
- **Error Propagation**: Custom `Result<T>` type throughout for consistent error handling
- **Modular Testing**: Each module has dedicated test suites with `tempfile` for filesystem testing

## Common Commands

```bash
# Build the project
cargo build

# Run in development
cargo run -- check                    # List USB devices
cargo run -- watch                    # Start monitoring (interactive setup if no config)
cargo run -- watch -k 1234 -p "External" -d "Default"  # With CLI args

# Build release version
cargo build --release

# Install locally
cargo install --path .

# Testing
cargo test                            # Run all tests
cargo test config::tests::           # Run specific module tests
cargo test test_config_serialization # Run single test

# Code quality
cargo clippy                         # Linting
cargo fmt                           # Formatting
cargo check                         # Quick compilation check

# Development helpers
cargo run -- --help                 # Show CLI help
cargo run -- watch --help          # Show watch command help
```

## Configuration

The application supports three configuration methods with the following priority:

1. **Config file** (highest): `~/.config/ke_auto_profile_switcher/config.yml`
2. **CLI arguments**: When keyboard-id and product-profile are provided
3. **Interactive setup** (lowest): Guided setup when neither config file nor complete CLI args exist

### Config File Format
```yaml
keyboard_id: 1234  # USB product ID of the external keyboard
product_profile: "External Keyboard"  # Profile name when keyboard is connected
default_profile: "Default"  # Profile name when keyboard is disconnected
```

### Configuration Resolution
The `resolve_config()` function in `src/config/mod.rs` implements the priority logic and handles interactive setup when needed.

## Dependencies

- **Core functionality**:
  - `usb_enumeration`: USB device monitoring and events
  - `clap`: CLI argument parsing with derive features
  - `serde` + `serde_yaml`: Configuration serialization
  - `dirs`: System directory location
- **Error handling**:
  - `thiserror`: Custom error type derivation
  - `anyhow`: Error context (imported but minimal usage)
- **Testing**:
  - `tempfile`: Temporary directories for filesystem tests

## Platform Notes

- **macOS specific**: Uses hardcoded path to Karabiner-Elements CLI defined in `src/constants.rs`
- **Requires Karabiner-Elements** to be installed on the system
- **Rust Edition**: Uses Rust 2024 edition for latest language features

## Development Guidelines

- **Error Handling**: Use `crate::Result<T>` for consistent error propagation
- **Constants**: Add new hardcoded values to `src/constants.rs`
- **Testing**: Use `tempfile::tempdir()` for filesystem-related tests
- **Module Organization**: Keep modules focused on single responsibilities
- **CLI Changes**: Update both `src/cli.rs` structs and help text consistently