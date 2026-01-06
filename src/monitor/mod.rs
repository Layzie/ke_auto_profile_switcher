//! Device monitoring module for USB and Bluetooth keyboards.
//!
//! This module provides a unified interface for monitoring different types of
//! keyboard connections and triggering profile switches.

pub mod bluetooth;
pub mod combined;
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
    /// Create a USB device identifier
    pub fn usb(product_id: u16) -> Self {
        DeviceIdentifier::Usb { product_id }
    }

    /// Create a Bluetooth device identifier
    pub fn bluetooth(device_name: impl Into<String>) -> Self {
        DeviceIdentifier::Bluetooth {
            device_name: device_name.into(),
        }
    }

    /// Check if this is a USB device
    pub fn is_usb(&self) -> bool {
        matches!(self, DeviceIdentifier::Usb { .. })
    }

    /// Check if this is a Bluetooth device
    pub fn is_bluetooth(&self) -> bool {
        matches!(self, DeviceIdentifier::Bluetooth { .. })
    }

    /// Get display name for the device identifier
    pub fn display_name(&self) -> String {
        match self {
            DeviceIdentifier::Usb { product_id } => format!("USB (Product ID: {})", product_id),
            DeviceIdentifier::Bluetooth { device_name } => {
                format!("Bluetooth (Name: {})", device_name)
            }
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
}

impl KeyboardMapping {
    /// Create a new keyboard mapping
    pub fn new(
        name: impl Into<String>,
        device: DeviceIdentifier,
        profile: impl Into<String>,
    ) -> Self {
        KeyboardMapping {
            name: name.into(),
            device,
            profile: profile.into(),
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
