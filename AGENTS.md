# AGENTS.md

## Cursor Cloud specific instructions

### Overview

Rust CLI tool (`kaps`) that auto-switches Karabiner-Elements profiles based on USB keyboard connection. macOS-only at runtime, but builds and tests on Linux.

### System dependency

`libudev-dev` is required on Linux to compile the `usb_enumeration` crate. The update script handles this.

### Rust edition 2024

This project uses Rust edition 2024, which requires **Rust 1.85+**. If `cargo build` fails with `feature edition2024 is required`, run `rustup update stable && rustup default stable`.

### Commands

Standard commands are documented in `CLAUDE.md`. Key ones:

- **Build:** `cargo build`
- **Test:** `cargo test`
- **Lint:** `cargo clippy` (expect warnings for redundant closures and `serde_yaml` deprecation — these are known)
- **Format check:** `cargo fmt --check` (existing code has formatting drift; do not auto-format without maintainer approval)
- **Run:** `cargo run -- check` (lists USB devices), `cargo run -- --help`

### Gotchas

- `cargo run -- check` returns "No USB devices found" in Cloud VMs (no USB hardware). This is expected.
- `cargo run -- watch` requires interactive stdin input or CLI flags; avoid running it without arguments in non-interactive environments.
- `serde_yaml` is deprecated — build warnings are expected.
