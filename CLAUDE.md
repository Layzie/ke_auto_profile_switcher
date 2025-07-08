# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# 言語設定
- 日本語で応答してください

## Project Overview

This is a Rust CLI tool called `kaps` (Karabiner Auto Profile Switcher) that automatically switches Karabiner-Elements profiles based on USB keyboard connection status. The tool monitors USB device events and switches between configured profiles when external keyboards are connected or disconnected.

## Architecture

- **Single binary**: The entire application is contained in `src/main.rs`
- **Configuration**: Uses YAML configuration files stored in `~/.config/ke_auto_profile_switcher/config.yml`
- **USB monitoring**: Uses the `usb_enumeration` crate to watch for USB device events
- **Profile switching**: Executes Karabiner-Elements CLI commands to change profiles
- **CLI interface**: Built with `clap` for command-line argument parsing

## Key Components

- `Config` struct: Handles configuration file loading from `~/.config/ke_auto_profile_switcher/config.yml`
- `WatchArgs` struct: Command-line arguments for the watch command
- `change_karabiner_profile()`: Main function that monitors USB events and switches profiles
- `check_keyboard_id()`: Utility function to enumerate connected USB devices

## Common Commands

```bash
# Build the project
cargo build

# Run in development
cargo run -- check
cargo run -- watch

# Build release version
cargo build --release

# Install locally
cargo install --path .

# Test the application
cargo test

# Check for linting issues
cargo clippy

# Format code
cargo fmt
```

## Configuration

The application loads configuration from `~/.config/ke_auto_profile_switcher/config.yml` with the following structure:
```yaml
keyboard_id: 1234  # USB product ID of the external keyboard
product_profile: "External Keyboard"  # Profile name when keyboard is connected
default_profile: "Default"  # Profile name when keyboard is disconnected
```

## Dependencies

- `usb_enumeration`: For monitoring USB device events
- `clap`: For CLI argument parsing
- `serde` + `serde_yaml`: For configuration file handling
- `dirs`: For finding home directory (implied by the code structure)

## Platform Notes

- macOS specific: Uses hardcoded path to Karabiner-Elements CLI at `/Library/Application Support/org.pqrs/Karabiner-Elements/bin/karabiner_cli`
- Requires Karabiner-Elements to be installed on the system