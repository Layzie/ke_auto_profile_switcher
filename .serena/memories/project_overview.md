# Project Overview

## Purpose
This is a Rust CLI tool called `kaps` (Karabiner Auto Profile Switcher) that automatically switches Karabiner-Elements profiles based on USB keyboard connection status. The tool monitors USB device events and switches between configured profiles when external keyboards are connected or disconnected.

## Tech Stack
- **Language**: Rust (edition 2021)
- **CLI Framework**: clap with derive features
- **Configuration**: serde + serde_yaml for YAML config files
- **USB Monitoring**: usb_enumeration crate
- **Error Handling**: thiserror for custom error types, anyhow for error context
- **Testing**: tempfile for filesystem tests

## Key Features
- Automatic profile switching based on USB keyboard connection/disconnection
- Multiple configuration methods (config file, CLI args, interactive setup)
- Event-driven USB device monitoring
- Cross-platform USB device enumeration
- Priority-based configuration resolution

## Platform Requirements
- macOS with Karabiner-Elements installed
- Karabiner-Elements CLI at standard location (/Applications/Karabiner-Elements.app/Contents/MacOS/Karabiner-Elements)