//! Combined device monitoring module.
//!
//! Monitors both USB and Bluetooth devices simultaneously,
//! managing multiple keyboard-profile mappings.

use crate::error::{AppError, Result};
use crate::karabiner::KarabinerController;
use crate::monitor::bluetooth::BluetoothMonitor;
use crate::monitor::usb::UsbMonitor;
use crate::monitor::{DeviceEvent, DeviceIdentifier, DeviceMonitor, KeyboardMapping};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

/// Combined monitor that handles both USB and Bluetooth devices
pub struct CombinedMonitor {
    /// Keyboard mappings configuration
    mappings: Vec<KeyboardMapping>,
    /// Default profile when no keyboards are connected
    default_profile: String,
}

impl CombinedMonitor {
    /// Create a new combined monitor
    pub fn new(mappings: Vec<KeyboardMapping>, default_profile: impl Into<String>) -> Self {
        CombinedMonitor {
            mappings,
            default_profile: default_profile.into(),
        }
    }

    /// Start monitoring all configured devices
    pub fn start_monitoring(&self) -> Result<()> {
        let karabiner = KarabinerController::new();

        // Separate USB and Bluetooth mappings
        let mut usb_product_ids: Vec<u16> = Vec::new();
        let mut bluetooth_names: Vec<String> = Vec::new();

        // Build a map from device identifier to profile
        let device_to_profile: HashMap<String, String> = self
            .mappings
            .iter()
            .map(|m| {
                let key = match &m.device {
                    DeviceIdentifier::Usb { product_id } => {
                        usb_product_ids.push(*product_id);
                        format!("usb:{}", product_id)
                    }
                    DeviceIdentifier::Bluetooth { device_name } => {
                        bluetooth_names.push(device_name.clone());
                        format!("bluetooth:{}", device_name.to_lowercase())
                    }
                };
                (key, m.profile.clone())
            })
            .collect();

        let device_to_profile = Arc::new(device_to_profile);
        let default_profile = Arc::new(self.default_profile.clone());

        // Track connected devices
        let connected_devices: Arc<Mutex<Vec<DeviceIdentifier>>> = Arc::new(Mutex::new(Vec::new()));

        // Create the event handler
        let create_handler = |device_to_profile: Arc<HashMap<String, String>>,
                              default_profile: Arc<String>,
                              connected_devices: Arc<Mutex<Vec<DeviceIdentifier>>>,
                              karabiner: KarabinerController| {
            move |event: DeviceEvent| -> Result<()> {
                match event {
                    DeviceEvent::Initial(devices) => {
                        let mut connected = connected_devices.lock().unwrap();
                        connected.extend(devices.clone());

                        // Switch to the first matched device's profile
                        if let Some(device) = devices.first() {
                            if let Some(profile) =
                                get_profile_for_device(device, &device_to_profile)
                            {
                                println!(
                                    "Initial device detected, switching to profile: {}",
                                    profile
                                );
                                karabiner.switch_profile(&profile)?;
                            }
                        }
                    }
                    DeviceEvent::Connected(device) => {
                        let mut connected = connected_devices.lock().unwrap();
                        connected.push(device.clone());

                        if let Some(profile) = get_profile_for_device(&device, &device_to_profile) {
                            println!(
                                "Device connected: {}, switching to profile: {}",
                                device.display_name(),
                                profile
                            );
                            karabiner.switch_profile(&profile)?;
                        }
                    }
                    DeviceEvent::Disconnected(device) => {
                        let mut connected = connected_devices.lock().unwrap();
                        connected.retain(|d| d != &device);

                        // Check if any other monitored device is still connected
                        if let Some(remaining) = connected.first() {
                            if let Some(profile) =
                                get_profile_for_device(remaining, &device_to_profile)
                            {
                                println!(
                                    "Device disconnected: {}, switching to profile: {} (another device still connected)",
                                    device.display_name(),
                                    profile
                                );
                                karabiner.switch_profile(&profile)?;
                            }
                        } else {
                            println!(
                                "Device disconnected: {}, switching to default profile: {}",
                                device.display_name(),
                                default_profile
                            );
                            karabiner.switch_profile(&default_profile)?;
                        }
                    }
                }
                Ok(())
            }
        };

        let mut handles = Vec::new();

        // Start USB monitoring if there are USB devices to monitor
        if !usb_product_ids.is_empty() {
            let usb_monitor = UsbMonitor::new(usb_product_ids);
            let handler = create_handler(
                Arc::clone(&device_to_profile),
                Arc::clone(&default_profile),
                Arc::clone(&connected_devices),
                karabiner.clone(),
            );

            let handle = thread::spawn(move || {
                if let Err(e) = usb_monitor.start_monitoring(handler) {
                    eprintln!("USB monitoring error: {}", e);
                }
            });
            handles.push(handle);
        }

        // Start Bluetooth monitoring if there are Bluetooth devices to monitor
        if !bluetooth_names.is_empty() {
            let bluetooth_monitor = BluetoothMonitor::new(bluetooth_names);
            let handler = create_handler(
                Arc::clone(&device_to_profile),
                Arc::clone(&default_profile),
                Arc::clone(&connected_devices),
                karabiner.clone(),
            );

            let handle = thread::spawn(move || {
                if let Err(e) = bluetooth_monitor.start_monitoring(handler) {
                    eprintln!("Bluetooth monitoring error: {}", e);
                }
            });
            handles.push(handle);
        }

        if handles.is_empty() {
            return Err(AppError::Config(
                "No devices configured for monitoring".to_string(),
            ));
        }

        println!("Monitoring started. Press Ctrl+C to stop.");
        println!("Default profile: {}", self.default_profile);
        println!("Configured mappings:");
        for mapping in &self.mappings {
            println!(
                "  - {} -> Profile: {}",
                mapping.device.display_name(),
                mapping.profile
            );
        }

        // Wait for all monitoring threads
        for handle in handles {
            handle
                .join()
                .map_err(|_| AppError::UsbDevice("Monitoring thread panicked".to_string()))?;
        }

        Ok(())
    }
}

/// Get the profile for a device from the mapping
fn get_profile_for_device(
    device: &DeviceIdentifier,
    device_to_profile: &HashMap<String, String>,
) -> Option<String> {
    let key = match device {
        DeviceIdentifier::Usb { product_id } => format!("usb:{}", product_id),
        DeviceIdentifier::Bluetooth { device_name } => {
            format!("bluetooth:{}", device_name.to_lowercase())
        }
    };
    device_to_profile.get(&key).cloned()
}
