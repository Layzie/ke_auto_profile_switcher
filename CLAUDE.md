# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# 言語設定
- 日本語で応答してください

## Project Overview

This is a Rust CLI tool called `kaps` (Karabiner Auto Profile Switcher) that automatically switches Karabiner-Elements profiles based on keyboard connection status. The tool monitors both **USB** and **Bluetooth** device events and switches between configured profiles when external keyboards are connected or disconnected. It supports multiple keyboards with different profile mappings.

## Architecture

The application follows a modular architecture with clear separation of concerns:

- **Modular Design**: Code is organized into focused modules in `src/lib.rs`
- **Error Handling**: Custom error types with `thiserror` for detailed error reporting
- **Configuration Management**: Multiple configuration sources with priority: config file → CLI args → interactive setup
- **Device Monitoring**: Event-driven USB and Bluetooth device monitoring with callback-based profile switching
- **CLI Interface**: Built with `clap` for comprehensive command-line argument parsing

### Module Structure

- **`src/main.rs`**: Minimal entry point - orchestrates CLI parsing and delegates to modules
- **`src/config/`**: Configuration management with YAML serialization, interactive setup, and comprehensive tests
- **`src/cli.rs`**: CLI argument definitions and parsing structures
- **`src/monitor/`**: Device monitoring module (USB and Bluetooth)
  - `mod.rs`: Common traits (`DeviceMonitor`) and types (`DeviceIdentifier`, `KeyboardMapping`, `DeviceEvent`)
  - `iokit.rs`: Unified IOKit event-driven monitor (`IoKitMonitor`) watching USB + Bluetooth keyboards via `IOServiceAddMatchingNotification` on the `IOHIDDevice` service class (macOS only); also provides USB enumeration for `check`
  - `usb.rs`: Thin shim — USB device listing for the `check` command (delegates to `iokit::list_usb_devices` on macOS)
  - `bluetooth.rs`: Bluetooth device listing for the `check` command using macOS `system_profiler` (snapshot only; not used by `watch`)
  - `combined.rs`: Drives a single `IoKitMonitor`, applying priority-based profile mappings
- **`src/karabiner.rs`**: Karabiner-Elements CLI integration and profile switching
- **`src/error.rs`**: Custom error types (`AppError`) with proper error chaining
- **`src/constants.rs`**: Centralized application constants (paths, defaults)

### Key Architecture Patterns

- **Configuration Resolution**: `resolve_config()` function implements priority-based config sourcing
- **Device Monitor Trait**: `DeviceMonitor` trait provides unified interface for USB and Bluetooth monitoring
- **Event-Driven Monitoring**: Callback-based design allows flexible response to device events
- **Error Propagation**: Custom `Result<T>` type throughout for consistent error handling
- **Modular Testing**: Each module has dedicated test suites with `tempfile` for filesystem testing

## Common Commands

```bash
# Build the project
cargo build

# Run in development
cargo run -- check                    # List all devices (USB and Bluetooth)
cargo run -- check -t usb             # List USB devices only
cargo run -- check -t bluetooth       # List Bluetooth devices only
cargo run -- watch                    # Start monitoring (interactive setup if no config)
cargo run -- watch -k 1234 -p "External" -d "Default"  # With CLI args (legacy)

# Build release version
cargo build --release

# Install locally
cargo install --path .

# Testing
cargo test                            # Run all tests
cargo test config::tests::           # Run specific module tests
cargo test test_config_serialization # Run single test
cargo test -- --nocapture            # Run tests with output

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
2. **CLI arguments**: When keyboard-id and product-profile are provided (legacy USB-only)
3. **Interactive setup** (lowest): Guided setup when neither config file nor complete CLI args exist

### Config File Format (Version 2 - Recommended)
```yaml
version: 2
default_profile: "Default"
keyboards:
  - name: "USB Keyboard"
    device:
      type: usb
      product_id: 1234
    profile: "USB Profile"
  - name: "Magic Keyboard"
    device:
      type: bluetooth
      device_name: "Magic Keyboard"
    profile: "Bluetooth Profile"
```

### Legacy Config File Format (Version 1 - Still Supported)
```yaml
keyboard_id: 1234  # USB product ID of the external keyboard
product_profile: "External Keyboard"  # Profile name when keyboard is connected
default_profile: "Default"  # Profile name when keyboard is disconnected
```

### Configuration Resolution
The `resolve_config()` function in `src/config/mod.rs` implements the priority logic and handles interactive setup when needed. It automatically converts legacy v1 config to v2 format internally.

## Dependencies

- **Core functionality**:
  - `clap`: CLI argument parsing with derive features
  - `serde` + `serde_yaml` + `serde_json`: Configuration and data serialization
  - `dirs`: System directory location
- **macOS device monitoring** (`[target.'cfg(target_os = "macos")'.dependencies]`):
  - `io-kit-sys`: IOKit FFI for event-driven device monitoring (`IOServiceAddMatchingNotification`)
  - `core-foundation` + `core-foundation-sys`: Core Foundation types (CFRunLoop, CFString, CFNumber) for the IOKit monitor
- **Error handling**:
  - `thiserror`: Custom error type derivation
  - `anyhow`: Error context (imported but minimal usage)
- **Testing**:
  - `tempfile`: Temporary directories for filesystem tests

## Platform Notes

- **macOS specific**: Uses hardcoded path to Karabiner-Elements CLI defined in `src/constants.rs`
- **Requires Karabiner-Elements** to be installed on the system
- **Device monitoring (`watch`)**: True event-driven via IOKit `IOServiceAddMatchingNotification` on `IOHIDDevice`. Reads only IORegistry metadata (never opens the device), so it does **not** require the Input Monitoring permission. No polling.
- **Device listing (`check`)**: One-shot snapshot — USB via IOKit, Bluetooth via macOS `system_profiler SPBluetoothDataType`
- **Rust Edition**: Uses Rust 2021 edition

## Development Guidelines

- **Error Handling**: Use `crate::Result<T>` for consistent error propagation
- **Constants**: Add new hardcoded values to `src/constants.rs`
- **Testing**: Use `tempfile::tempdir()` for filesystem-related tests
- **Module Organization**: Keep modules focused on single responsibilities
- **CLI Changes**: Update both `src/cli.rs` structs and help text consistently
- **Device Monitoring**: Implement `DeviceMonitor` trait for new device types
- **Backward Compatibility**: Maintain support for legacy v1 configuration format
