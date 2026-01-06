# Project Overview

## Purpose
This is a Rust CLI tool called `kaps` (Karabiner Auto Profile Switcher) that automatically switches Karabiner-Elements profiles based on keyboard connection status. The tool monitors both USB and Bluetooth device events and switches between configured profiles when external keyboards are connected or disconnected. It supports multiple keyboards with different profile mappings.

## Tech Stack
- **Language**: Rust (edition 2021)
- **CLI Framework**: clap with derive features
- **Configuration**: serde + serde_yaml + serde_json for config and data serialization
- **USB Monitoring**: usb_enumeration crate
- **Bluetooth Monitoring**: macOS system_profiler command
- **macOS Integration**: core-foundation, core-foundation-sys
- **Error Handling**: thiserror for custom error types, anyhow for error context
- **Testing**: tempfile for filesystem tests

## Key Features
- Automatic profile switching based on keyboard connection/disconnection
- Support for both USB and Bluetooth keyboards
- Multiple keyboard-profile mappings
- Multiple configuration methods (config file, CLI args, interactive setup)
- Event-driven device monitoring (USB real-time, Bluetooth polling)
- Priority-based configuration resolution
- Backward compatible with legacy single-keyboard configuration

## Platform Requirements
- macOS with Karabiner-Elements installed
- Karabiner-Elements CLI at standard location (/Library/Application Support/org.pqrs/Karabiner-Elements/bin/karabiner_cli)
