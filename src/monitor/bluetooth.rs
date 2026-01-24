//! Bluetooth device monitoring module.
//!
//! Monitors Bluetooth HID devices (keyboards) on macOS using IOKit/HIDManager.
//! Falls back to system_profiler polling if IOKit is not available.

use crate::constants::BLUETOOTH_POLL_INTERVAL_SECONDS;
use crate::error::{AppError, Result};
use crate::monitor::{DeviceEvent, DeviceIdentifier, DeviceInfo, DeviceMonitor};
use std::collections::HashSet;
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Maximum number of retries for system_profiler command
const MAX_RETRIES: u32 = 3;
/// Initial delay between retries (will be exponentially increased)
const RETRY_DELAY_MS: u64 = 500;

/// Monitor for Bluetooth keyboard connections
pub struct BluetoothMonitor {
    /// Device names to monitor
    device_names: Vec<String>,
}

impl BluetoothMonitor {
    /// Create a new Bluetooth monitor for the given device names
    pub fn new(device_names: Vec<String>) -> Self {
        BluetoothMonitor { device_names }
    }

    /// Create a Bluetooth monitor for a single device name
    pub fn single(device_name: impl Into<String>) -> Self {
        BluetoothMonitor {
            device_names: vec![device_name.into()],
        }
    }

    /// Get currently connected Bluetooth devices using system_profiler with retry
    fn get_connected_devices() -> Result<Vec<BluetoothDeviceInfo>> {
        Self::get_connected_devices_with_retry(MAX_RETRIES)
    }

    /// Get connected devices with specified number of retries
    fn get_connected_devices_with_retry(max_retries: u32) -> Result<Vec<BluetoothDeviceInfo>> {
        let mut last_error = None;
        let mut delay = Duration::from_millis(RETRY_DELAY_MS);

        for attempt in 0..=max_retries {
            match Self::try_get_connected_devices() {
                Ok(devices) => return Ok(devices),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        thread::sleep(delay);
                        delay *= 2; // Exponential backoff
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AppError::Bluetooth("Failed to get Bluetooth devices after retries".to_string())
        }))
    }

    /// Try to get connected devices (single attempt)
    fn try_get_connected_devices() -> Result<Vec<BluetoothDeviceInfo>> {
        let output = Command::new("system_profiler")
            .args(["SPBluetoothDataType", "-json"])
            .output()
            .map_err(|e| AppError::Bluetooth(format!("Failed to run system_profiler: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::Bluetooth(
                "system_profiler command failed".to_string(),
            ));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        Self::parse_bluetooth_json(&json_str)
    }

    /// Parse the JSON output from system_profiler
    fn parse_bluetooth_json(json_str: &str) -> Result<Vec<BluetoothDeviceInfo>> {
        // Parse the JSON structure
        let json: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| AppError::Bluetooth(format!("Failed to parse JSON: {}", e)))?;

        let mut devices = Vec::new();

        // Navigate the JSON structure to find connected devices
        if let Some(bluetooth_data) = json.get("SPBluetoothDataType").and_then(|v| v.as_array()) {
            for controller in bluetooth_data {
                // Check for connected devices in various locations
                // Structure varies by macOS version

                // Try "device_connected" (older macOS)
                if let Some(connected) = controller
                    .get("device_connected")
                    .and_then(|v| v.as_array())
                {
                    for device in connected {
                        if let Some(device_obj) = device.as_object() {
                            for (name, info) in device_obj {
                                let address = info
                                    .get("device_address")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                let is_keyboard = info
                                    .get("device_minorType")
                                    .and_then(|v| v.as_str())
                                    .map(|t| t.to_lowercase().contains("keyboard"))
                                    .unwrap_or(false);

                                devices.push(BluetoothDeviceInfo {
                                    name: name.clone(),
                                    address,
                                    connected: true,
                                    is_keyboard,
                                });
                            }
                        }
                    }
                }

                // Try "device_title" -> devices (newer macOS)
                if let Some(device_title) =
                    controller.get("device_title").and_then(|v| v.as_array())
                {
                    for section in device_title {
                        if let Some(section_devices) =
                            section.get("_items").and_then(|v| v.as_array())
                        {
                            for device in section_devices {
                                let name = device
                                    .get("_name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown")
                                    .to_string();

                                let address = device
                                    .get("device_address")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                let connected = device
                                    .get("device_connected")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s == "attrib_Yes" || s.to_lowercase() == "yes")
                                    .unwrap_or(false);

                                let is_keyboard = device
                                    .get("device_minorType")
                                    .and_then(|v| v.as_str())
                                    .map(|t| t.to_lowercase().contains("keyboard"))
                                    .unwrap_or(false);

                                devices.push(BluetoothDeviceInfo {
                                    name,
                                    address,
                                    connected,
                                    is_keyboard,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(devices)
    }

    /// Check if a device name matches any of the monitored names
    fn matches_device(&self, device_name: &str) -> bool {
        self.device_names
            .iter()
            .any(|name| device_name.to_lowercase().contains(&name.to_lowercase()))
    }
}

/// Internal Bluetooth device info
#[derive(Debug, Clone)]
struct BluetoothDeviceInfo {
    name: String,
    #[allow(dead_code)]
    address: String,
    connected: bool,
    #[allow(dead_code)]
    is_keyboard: bool,
}

impl DeviceMonitor for BluetoothMonitor {
    fn start_monitoring<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(DeviceEvent) -> Result<()> + Send + Sync + 'static,
    {
        // If no device names to monitor, return early
        if self.device_names.is_empty() {
            return Ok(());
        }

        let poll_interval = Duration::from_secs(BLUETOOTH_POLL_INTERVAL_SECONDS);
        let mut previously_connected: HashSet<String> = HashSet::new();
        let mut is_initial = true;

        loop {
            match Self::get_connected_devices() {
                Ok(devices) => {
                    // Filter to devices we're monitoring that are connected
                    let currently_connected: HashSet<String> = devices
                        .iter()
                        .filter(|d| d.connected && self.matches_device(&d.name))
                        .map(|d| d.name.clone())
                        .collect();

                    if is_initial {
                        // Report initial state
                        if !currently_connected.is_empty() {
                            let identifiers: Vec<DeviceIdentifier> = currently_connected
                                .iter()
                                .map(|name| DeviceIdentifier::bluetooth(name.clone()))
                                .collect();
                            callback(DeviceEvent::Initial(identifiers)).map_err(|e| {
                                AppError::Bluetooth(format!(
                                    "Failed to handle initial devices: {}",
                                    e
                                ))
                            })?;
                        }
                        is_initial = false;
                    } else {
                        // Check for newly connected devices
                        for name in currently_connected.difference(&previously_connected) {
                            println!("Bluetooth device connected: {}", name);
                            let identifier = DeviceIdentifier::bluetooth(name.clone());
                            callback(DeviceEvent::Connected(identifier)).map_err(|e| {
                                AppError::Bluetooth(format!(
                                    "Failed to handle device connection: {}",
                                    e
                                ))
                            })?;
                        }

                        // Check for disconnected devices
                        for name in previously_connected.difference(&currently_connected) {
                            println!("Bluetooth device disconnected: {}", name);
                            let identifier = DeviceIdentifier::bluetooth(name.clone());
                            callback(DeviceEvent::Disconnected(identifier)).map_err(|e| {
                                AppError::Bluetooth(format!(
                                    "Failed to handle device disconnection: {}",
                                    e
                                ))
                            })?;
                        }
                    }

                    previously_connected = currently_connected;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get Bluetooth devices: {}", e);
                }
            }

            thread::sleep(poll_interval);
        }
    }

    fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
        let devices = Self::get_connected_devices()?;
        Ok(devices
            .into_iter()
            .map(|d| {
                let identifier = DeviceIdentifier::bluetooth(d.name.clone());
                DeviceInfo::new(identifier, d.name, d.connected)
            })
            .collect())
    }
}

/// List all Bluetooth devices (for the check command)
pub fn list_bluetooth_devices() -> Result<()> {
    let devices = BluetoothMonitor::get_connected_devices()?;

    if devices.is_empty() {
        println!("No Bluetooth devices found.");
        return Ok(());
    }

    println!("Bluetooth Devices:");
    for device in devices.iter() {
        let status = if device.connected {
            "Connected"
        } else {
            "Paired"
        };
        let device_type = if device.is_keyboard {
            "Keyboard"
        } else {
            "Other"
        };
        println!(
            "  Name: {}, Status: {}, Type: {}",
            device.name, status, device_type
        );
    }

    Ok(())
}
