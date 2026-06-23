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
  - `iokit.rs`: Unified IOKit event-driven monitor (`IoKitMonitor`) for USB + Bluetooth via `IOServiceAddMatchingNotification` on `IOHIDDevice` (macOS only)
  - `usb.rs`: USB device listing for the `check` command (IOKit `IOUSBHostDevice` enumeration)
  - `bluetooth.rs`: Bluetooth device listing for the `check` command using macOS `system_profiler`
  - `combined.rs`: Drives a single `IoKitMonitor`, applying priority-based profile mappings
- **`src/karabiner.rs`**: Karabiner-Elements CLI integration and profile switching
- **`src/error.rs`**: Custom error types (`AppError`) with proper error chaining
- **`src/constants.rs`**: Centralized application constants (paths, defaults, intervals)

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
- **`CombinedMonitor`**: Drives a single `IoKitMonitor` on the calling thread's CFRunLoop

### Event-Driven Monitoring
- **Unified IOKit monitor**: USB and Bluetooth keyboards are watched together via `IOServiceAddMatchingNotification` on the `IOHIDDevice` class (first-match + terminated). Real-time, no polling.
- **No Input Monitoring permission**: Only IORegistry metadata is read; the HID device is never opened (`IOHIDManagerOpen` is avoided).
- **Disconnect lookup**: connect-time `IORegistryEntryGetRegistryEntryID` → `DeviceIdentifier` map, reverse-looked-up on terminate (properties may be gone at removal).
- **Callback-based design**: Allows flexible response to device events
- **Event types**: `Initial`, `Connected`, `Disconnected`

### Error Handling
- **Custom `Result<T>` type**: Throughout the application for consistent error handling
- **Error propagation**: Proper error chaining with thiserror
- **Detailed error reporting**: Context-aware error messages
- **Error variants**: `Config`, `UsbDevice`, `Bluetooth`, `Monitor`, `Karabiner`, `Io`, `Yaml`, `HomeDirectoryNotFound`, `MissingArgument`, `InvalidInput`

### Modular Testing
- **Per-module test suites**: Each module has dedicated tests
- **Filesystem testing**: Uses `tempfile` for temporary directories in tests
- **Configuration tests**: Tests for v1/v2 formats, serialization, CLI args conversion
- **Backward compatibility tests**: Ensures legacy config format still works

### Concurrency in CombinedMonitor
- **Single CFRunLoop thread**: `IoKitMonitor` がすべてのイベントを単一スレッド上で直列に配信するため、handler は並行実行されない。`Arc<Mutex<...>>` の共有状態はインタフェース安定のため残しているが、実質的に競合しない。
- **Apply cache (`last_applied: Mutex<Option<String>>`)**: 直前に適用したプロファイル名を保持し、同じプロファイルへの重複 switch を抑止する。`apply_target_profile` がこのロックを保持したまま subprocess を呼ぶ。
