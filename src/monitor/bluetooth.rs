//! Bluetooth device listing for the `check` command.
//!
//! Live Bluetooth keyboard monitoring is handled by the unified IOKit monitor
//! (`crate::monitor::iokit`). This module only provides the one-shot snapshot
//! listing used by `kaps check`, which queries
//! `system_profiler SPBluetoothDataType`.

use crate::error::{AppError, Result};
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Maximum number of retries for the system_profiler command.
const MAX_RETRIES: u32 = 3;
/// Initial delay between retries (exponentially increased).
const RETRY_DELAY_MS: u64 = 500;

/// Internal Bluetooth device info.
#[derive(Debug, Clone)]
struct BluetoothDeviceInfo {
    name: String,
    #[allow(dead_code)]
    address: String,
    connected: bool,
    is_keyboard: bool,
}

fn get_connected_devices() -> Result<Vec<BluetoothDeviceInfo>> {
    // Retry only the (transient) command execution; parse the result once.
    // Parse failures are deterministic and must not be retried.
    let stdout = run_system_profiler_with_retry(MAX_RETRIES)?;
    parse_bluetooth_json(&stdout)
}

/// Run `system_profiler`, retrying only on transient execution failures (spawn
/// error or non-zero exit). The raw stdout bytes are returned for parsing.
fn run_system_profiler_with_retry(max_retries: u32) -> Result<Vec<u8>> {
    let mut last_error = None;
    let mut delay = Duration::from_millis(RETRY_DELAY_MS);

    for attempt in 0..=max_retries {
        match run_system_profiler() {
            Ok(stdout) => return Ok(stdout),
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

/// Execute `system_profiler SPBluetoothDataType -json` once, returning raw stdout.
fn run_system_profiler() -> Result<Vec<u8>> {
    let output = Command::new("system_profiler")
        .args(["SPBluetoothDataType", "-json"])
        .output()
        .map_err(|e| AppError::Bluetooth(format!("Failed to run system_profiler: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::Bluetooth(
            "system_profiler command failed".to_string(),
        ));
    }

    Ok(output.stdout)
}

/// Parse the JSON output from system_profiler.
fn parse_bluetooth_json(json_bytes: &[u8]) -> Result<Vec<BluetoothDeviceInfo>> {
    let json: serde_json::Value = serde_json::from_slice(json_bytes)
        .map_err(|e| AppError::Bluetooth(format!("Failed to parse JSON: {}", e)))?;

    let mut devices = Vec::new();

    // Navigate the JSON structure to find connected devices.
    if let Some(bluetooth_data) = json.get("SPBluetoothDataType").and_then(|v| v.as_array()) {
        for controller in bluetooth_data {
            // Check for connected devices in various locations.
            // Structure varies by macOS version.

            // Try "device_connected" (older macOS).
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
                                .is_some_and(|t| t.to_ascii_lowercase().contains("keyboard"));

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

            // Try "device_title" -> devices (newer macOS).
            if let Some(device_title) = controller.get("device_title").and_then(|v| v.as_array()) {
                for section in device_title {
                    if let Some(section_devices) = section.get("_items").and_then(|v| v.as_array())
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
                                .is_some_and(|s| {
                                    s == "attrib_Yes" || s.eq_ignore_ascii_case("yes")
                                });

                            let is_keyboard = device
                                .get("device_minorType")
                                .and_then(|v| v.as_str())
                                .is_some_and(|t| t.to_ascii_lowercase().contains("keyboard"));

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

/// List all Bluetooth devices (for the `check` command).
pub fn list_bluetooth_devices() -> Result<()> {
    let devices = get_connected_devices()?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_device_title_section() {
        let json = br#"{
            "SPBluetoothDataType": [
                {
                    "device_title": [
                        {
                            "_items": [
                                {
                                    "_name": "Magic Keyboard",
                                    "device_address": "AA-BB-CC",
                                    "device_connected": "attrib_Yes",
                                    "device_minorType": "Keyboard"
                                }
                            ]
                        }
                    ]
                }
            ]
        }"#;
        let devices = parse_bluetooth_json(json).expect("parse");
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Magic Keyboard");
        assert!(devices[0].connected);
        assert!(devices[0].is_keyboard);
    }

    #[test]
    fn parses_empty_when_no_bluetooth_data() {
        let devices = parse_bluetooth_json(b"{}").expect("parse");
        assert!(devices.is_empty());
    }
}
