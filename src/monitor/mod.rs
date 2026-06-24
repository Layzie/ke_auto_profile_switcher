//! Device monitoring module for USB and Bluetooth keyboards.
//!
//! This module provides a unified interface for monitoring different types of
//! keyboard connections and triggering profile switches.

pub mod bluetooth;
pub mod combined;
#[cfg(target_os = "macos")]
pub mod iokit;
pub mod usb;

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Represents the type of device connection to monitor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DeviceIdentifier {
    /// USB device identified by product ID
    Usb { product_id: u16 },
    /// Bluetooth device identified by device name
    Bluetooth { device_name: String },
}

impl DeviceIdentifier {
    pub fn usb(product_id: u16) -> Self {
        DeviceIdentifier::Usb { product_id }
    }

    pub fn bluetooth(device_name: impl Into<String>) -> Self {
        DeviceIdentifier::Bluetooth {
            device_name: device_name.into(),
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            DeviceIdentifier::Usb { product_id } => format!("USB (Product ID: {})", product_id),
            DeviceIdentifier::Bluetooth { device_name } => {
                format!("Bluetooth (Name: {})", device_name)
            }
        }
    }

    /// USB: exact product_id equality. Bluetooth: case-insensitive bidirectional
    /// substring match so a config name can be a prefix/suffix of the actual
    /// system-reported name (e.g. "HHKB" matches "HHKB-BT").
    pub fn matches(&self, other: &DeviceIdentifier) -> bool {
        match (self, other) {
            (
                DeviceIdentifier::Usb { product_id: id1 },
                DeviceIdentifier::Usb { product_id: id2 },
            ) => id1 == id2,
            (
                DeviceIdentifier::Bluetooth { device_name: name1 },
                DeviceIdentifier::Bluetooth { device_name: name2 },
            ) => {
                let name1_lower = name1.to_ascii_lowercase();
                let name2_lower = name2.to_ascii_lowercase();
                // An empty name must never match: an empty configured name would
                // otherwise substring-match (via `contains("")`) every device.
                !name1_lower.is_empty()
                    && !name2_lower.is_empty()
                    && (name1_lower.contains(&name2_lower) || name2_lower.contains(&name1_lower))
            }
            _ => false,
        }
    }

    /// Exact-identity comparison used to track the *same* physical device across
    /// connect/disconnect events. Unlike [`matches`](Self::matches) (which does
    /// partial Bluetooth-name matching against a *configured* name), this
    /// requires full equality, so a disconnect only removes the device that
    /// actually disconnected rather than any tracked device whose name is a
    /// substring of it.
    pub fn is_same_device(&self, other: &DeviceIdentifier) -> bool {
        match (self, other) {
            (
                DeviceIdentifier::Usb { product_id: id1 },
                DeviceIdentifier::Usb { product_id: id2 },
            ) => id1 == id2,
            (
                DeviceIdentifier::Bluetooth { device_name: name1 },
                DeviceIdentifier::Bluetooth { device_name: name2 },
            ) => name1.eq_ignore_ascii_case(name2),
            _ => false,
        }
    }
}

/// Configuration for a single keyboard-profile mapping
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardMapping {
    /// Human-readable name for this mapping
    pub name: String,
    /// Device identifier (USB or Bluetooth)
    pub device: DeviceIdentifier,
    /// Profile to switch to when this keyboard is connected
    pub profile: String,
    /// Priority for this keyboard (higher = more important, default: 0)
    /// When multiple keyboards are connected, the one with higher priority is used
    #[serde(default)]
    pub priority: i32,
}

impl KeyboardMapping {
    pub fn new(
        name: impl Into<String>,
        device: DeviceIdentifier,
        profile: impl Into<String>,
    ) -> Self {
        KeyboardMapping {
            name: name.into(),
            device,
            profile: profile.into(),
            priority: 0,
        }
    }

    pub fn with_priority(
        name: impl Into<String>,
        device: DeviceIdentifier,
        profile: impl Into<String>,
        priority: i32,
    ) -> Self {
        KeyboardMapping {
            name: name.into(),
            device,
            profile: profile.into(),
            priority,
        }
    }
}

/// Event emitted when a device connection state changes
#[derive(Debug, Clone)]
pub enum DeviceEvent {
    /// Device was connected
    Connected(DeviceIdentifier),
    /// Device was disconnected
    Disconnected(DeviceIdentifier),
    /// Initial state of devices
    Initial(Vec<DeviceIdentifier>),
}

/// Trait for device monitors
pub trait DeviceMonitor: Send + Sync {
    /// Start monitoring for device events
    ///
    /// The callback will be called for each device event (connect/disconnect)
    fn start_monitoring<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(DeviceEvent) -> Result<()> + Send + Sync + 'static;

    /// List currently connected devices that match the monitored criteria
    fn list_devices(&self) -> Result<Vec<DeviceInfo>>;
}

/// Information about a detected device
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Device identifier
    pub identifier: DeviceIdentifier,
    /// Device description/name
    pub description: String,
    /// Whether the device is currently connected
    pub connected: bool,
}

impl DeviceInfo {
    pub fn new(
        identifier: DeviceIdentifier,
        description: impl Into<String>,
        connected: bool,
    ) -> Self {
        DeviceInfo {
            identifier,
            description: description.into(),
            connected,
        }
    }
}
