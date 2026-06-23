//! USB device listing for the `check` command.
//!
//! Live USB keyboard monitoring is handled by the unified IOKit monitor
//! (`crate::monitor::iokit`). This module only provides the device listing used
//! by `kaps check`.

/// List all USB devices (for the `check` command).
pub fn list_usb_devices() {
    #[cfg(target_os = "macos")]
    crate::monitor::iokit::list_usb_devices();

    #[cfg(not(target_os = "macos"))]
    println!("No USB devices found.");
}
