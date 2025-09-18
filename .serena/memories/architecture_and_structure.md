# Architecture and Code Structure

## Module Architecture
The application follows a modular architecture with clear separation of concerns:

- **`src/main.rs`**: Minimal entry point (51 lines) - orchestrates CLI parsing and delegates to modules
- **`src/lib.rs`**: Module exports and organization
- **`src/config/`**: Configuration management module
  - `mod.rs`: Core configuration logic, interactive setup, config resolution
  - `tests.rs`: Comprehensive test suite for configuration functionality
- **`src/cli.rs`**: CLI argument definitions and parsing structures (clap-based)
- **`src/usb_monitor.rs`**: USB device enumeration and event monitoring with callback support
- **`src/karabiner.rs`**: Karabiner-Elements CLI integration and profile switching
- **`src/error.rs`**: Custom error types (`AppError`) with proper error chaining
- **`src/constants.rs`**: Centralized application constants (paths, defaults, intervals)

## Key Architecture Patterns

### Configuration Resolution
- **Priority-based config sourcing**: config file → CLI args → interactive setup
- **`resolve_config()` function**: Implements the priority logic in `src/config/mod.rs`
- **Interactive setup**: Guided setup when neither config file nor complete CLI args exist

### Event-Driven USB Monitoring
- **Callback-based design**: Allows flexible response to USB events
- **Device enumeration**: Lists available USB devices for identification
- **Real-time monitoring**: Continuous monitoring of USB connection events

### Error Handling
- **Custom `Result<T>` type**: Throughout the application for consistent error handling
- **Error propagation**: Proper error chaining with thiserror
- **Detailed error reporting**: Context-aware error messages

### Modular Testing
- **Per-module test suites**: Each module has dedicated tests
- **Filesystem testing**: Uses `tempfile` for temporary directories in tests
- **Integration tests**: Tests configuration resolution and USB monitoring