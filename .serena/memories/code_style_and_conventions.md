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
- Prefer thiserror for custom error types over manual implementations

### Module Organization
- Keep modules focused on single responsibilities
- Each module should have a clear, well-defined purpose
- Place module-specific constants in `src/constants.rs`
- Organize related functionality into subdirectories (like `src/config/`)

### Configuration Management
- Use serde for serialization/deserialization
- YAML format for configuration files
- Implement priority-based configuration resolution
- Provide interactive setup for first-time users

### Testing Patterns
- Use `tempfile::tempdir()` for filesystem-related tests
- Each module should have dedicated test suites
- Place tests in separate files (e.g., `src/config/tests.rs`)
- Test both success and error cases

### CLI Interface
- Use clap with derive features for argument parsing
- Update both `src/cli.rs` structs and help text consistently
- Provide comprehensive help messages and examples

### Constants Management
- Add new hardcoded values to `src/constants.rs`
- Use descriptive names for constants
- Group related constants together

## Development Guidelines
- Prefer editing existing files over creating new ones
- Follow the existing error handling patterns
- Keep the main.rs minimal - delegate functionality to modules
- Use callback-based design for event handling (USB monitoring)