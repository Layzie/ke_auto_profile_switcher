# Code Style and Conventions

## Rust Conventions
- **Edition**: Rust 2021
- **Formatting**: Standard rustfmt formatting
- **Naming**: Standard Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- **Error Handling**: Custom error types with thiserror, consistent `Result<T>` usage

## Project-Specific Patterns

### Error Handling
- Use `crate::Result<T>` for consistent error propagation throughout the application
- Custom `AppError` type defined in `src/error.rs` with proper error chaining
- Error variants: `Config`, `UsbDevice`, `Bluetooth`, `Karabiner`, `Io`, `Yaml`, `HomeDirectoryNotFound`, `MissingArgument`, `InvalidInput`
- Prefer thiserror for custom error types over manual implementations

### Module Organization
- Keep modules focused on single responsibilities
- Each module should have a clear, well-defined purpose
- Place module-specific constants in `src/constants.rs`
- Organize related functionality into subdirectories (like `src/config/`, `src/monitor/`)
- Use `mod.rs` for module entry points with submodules for specific functionality

### Configuration Management
- Use serde for serialization/deserialization
- YAML format for configuration files
- Support both legacy (v1) and new (v2) configuration formats
- Implement priority-based configuration resolution
- Provide interactive setup for first-time users
- Version field in config for future extensibility

### Device Monitoring
- Implement `DeviceMonitor` trait for new device types
- Use `DeviceIdentifier` enum for type-safe device identification
- Event-driven design with `DeviceEvent` enum
- Separate threads for USB and Bluetooth monitoring in combined mode
- Use Arc and Mutex for shared state between threads

### Testing Patterns
- Use `tempfile::tempdir()` for filesystem-related tests
- Each module should have dedicated test suites
- Place tests in separate files (e.g., `src/config/tests.rs`)
- Test both success and error cases
- Test backward compatibility with legacy formats
- Test serialization/deserialization roundtrips

### CLI Interface
- Use clap with derive features for argument parsing
- Update both `src/cli.rs` structs and help text consistently
- Provide comprehensive help messages and examples
- Support device type filtering with `-t` / `--type` option
- Maintain legacy CLI options for backward compatibility

### Constants Management
- Add new hardcoded values to `src/constants.rs`
- Use descriptive names for constants
- Group related constants together
- Constants include: paths, intervals, default values

## Development Guidelines
- Prefer editing existing files over creating new ones
- Follow the existing error handling patterns
- Keep the main.rs minimal - delegate functionality to modules
- Use callback-based design for event handling
- Mark deprecated code with `#[deprecated]` attribute
- Maintain backward compatibility when possible
