# Architecture and Code Structure

## Module Architecture
The application follows a modular architecture with clear separation of concerns:

- **`src/main.rs`**: Minimal entry point - orchestrates CLI parsing and delegates to modules
- **`src/lib.rs`**: Module exports and organization
- **`src/config/`**: Configuration management module
  - `mod.rs`: Core configuration logic, interactive setup, config resolution, legacy format support
  - `tests.rs`: Comprehensive test suite for configuration functionality
- **`src/cli.rs`**: CLI argument definitions and parsing structures (clap-based)
- **`src/monitor/`**: Device monitoring module
  - `mod.rs`: Common traits (`DeviceMonitor`) and types (`DeviceIdentifier`, `KeyboardMapping`, `DeviceEvent`, `DeviceInfo`)
  - `usb.rs`: USB device enumeration and event monitoring using `usb_enumeration` crate
  - `bluetooth.rs`: Bluetooth device monitoring using macOS `system_profiler` command
  - `combined.rs`: Combined monitor for simultaneous USB and Bluetooth monitoring with thread management
- **`src/karabiner.rs`**: Karabiner-Elements CLI integration and profile switching
- **`src/error.rs`**: Custom error types (`AppError`) with proper error chaining
- **`src/constants.rs`**: Centralized application constants (paths, defaults, intervals)
- **`src/usb_monitor.rs`**: Legacy USB monitor (deprecated, kept for backward compatibility)

## Key Architecture Patterns

### Configuration Resolution
- **Priority-based config sourcing**: config file → CLI args → interactive setup
- **`resolve_config()` function**: Implements the priority logic in `src/config/mod.rs`
- **Interactive setup**: Guided setup when neither config file nor complete CLI args exist
- **Backward compatibility**: Automatic conversion from legacy v1 to v2 config format

### Device Monitoring
- **`DeviceMonitor` trait**: Unified interface for USB and Bluetooth monitoring
- **`DeviceIdentifier` enum**: Distinguishes between USB (product_id) and Bluetooth (device_name) devices
- **`KeyboardMapping` struct**: Maps device identifiers to profile names
- **`CombinedMonitor`**: Orchestrates multiple monitors in separate threads

### Event-Driven Monitoring
- **USB**: Real-time event-driven monitoring using `usb_enumeration` crate
- **Bluetooth**: Polling-based monitoring using `system_profiler SPBluetoothDataType`
- **Callback-based design**: Allows flexible response to device events
- **Event types**: `Initial`, `Connected`, `Disconnected`

### Error Handling
- **Custom `Result<T>` type**: Throughout the application for consistent error handling
- **Error propagation**: Proper error chaining with thiserror
- **Detailed error reporting**: Context-aware error messages
- **Error variants**: `Config`, `UsbDevice`, `Bluetooth`, `Karabiner`, `Io`, `Yaml`

### Modular Testing
- **Per-module test suites**: Each module has dedicated tests
- **Filesystem testing**: Uses `tempfile` for temporary directories in tests
- **Configuration tests**: Tests for v1/v2 formats, serialization, CLI args conversion
- **Backward compatibility tests**: Ensures legacy config format still works
