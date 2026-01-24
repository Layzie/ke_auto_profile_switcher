//! Combined device monitoring module.
//!
//! Monitors both USB and Bluetooth devices simultaneously,
//! managing multiple keyboard-profile mappings with priority support.

use crate::error::{AppError, Result};
use crate::karabiner::KarabinerController;
use crate::monitor::bluetooth::BluetoothMonitor;
use crate::monitor::usb::UsbMonitor;
use crate::monitor::{DeviceEvent, DeviceIdentifier, DeviceMonitor, KeyboardMapping};
use std::sync::{Arc, Mutex};
use std::thread;

/// Connected device with its associated mapping
#[derive(Debug, Clone)]
struct ConnectedDevice {
    identifier: DeviceIdentifier,
    mapping: KeyboardMapping,
}

/// Combined monitor that handles both USB and Bluetooth devices
pub struct CombinedMonitor {
    /// Keyboard mappings configuration
    mappings: Vec<KeyboardMapping>,
    /// Default profile when no keyboards are connected
    default_profile: String,
    /// Enable verbose logging
    verbose: bool,
}

impl CombinedMonitor {
    /// Create a new combined monitor
    pub fn new(mappings: Vec<KeyboardMapping>, default_profile: impl Into<String>) -> Self {
        CombinedMonitor {
            mappings,
            default_profile: default_profile.into(),
            verbose: false,
        }
    }

    /// Enable or disable verbose logging
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Find the mapping that matches a device identifier
    fn find_mapping_for_device(&self, device: &DeviceIdentifier) -> Option<&KeyboardMapping> {
        self.mappings.iter().find(|m| m.device.matches(device))
    }

    /// Get the highest priority connected device's profile
    fn get_highest_priority_profile(
        connected: &[ConnectedDevice],
        default_profile: &str,
    ) -> String {
        connected
            .iter()
            .max_by_key(|d| d.mapping.priority)
            .map(|d| d.mapping.profile.clone())
            .unwrap_or_else(|| default_profile.to_string())
    }

    /// Start monitoring all configured devices
    pub fn start_monitoring(&self) -> Result<()> {
        let karabiner = KarabinerController::new();

        // Separate USB and Bluetooth mappings
        let mut usb_product_ids: Vec<u16> = Vec::new();
        let mut bluetooth_names: Vec<String> = Vec::new();

        for mapping in &self.mappings {
            match &mapping.device {
                DeviceIdentifier::Usb { product_id } => {
                    usb_product_ids.push(*product_id);
                }
                DeviceIdentifier::Bluetooth { device_name } => {
                    bluetooth_names.push(device_name.clone());
                }
            }
        }

        let mappings = Arc::new(self.mappings.clone());
        let default_profile = Arc::new(self.default_profile.clone());
        let verbose = self.verbose;

        // Track connected devices with their mappings
        let connected_devices: Arc<Mutex<Vec<ConnectedDevice>>> = Arc::new(Mutex::new(Vec::new()));

        // Create the event handler
        let create_handler = |mappings: Arc<Vec<KeyboardMapping>>,
                              default_profile: Arc<String>,
                              connected_devices: Arc<Mutex<Vec<ConnectedDevice>>>,
                              karabiner: KarabinerController,
                              verbose: bool| {
            move |event: DeviceEvent| -> Result<()> {
                match event {
                    DeviceEvent::Initial(devices) => {
                        let mut connected = connected_devices.lock().unwrap();

                        for device in &devices {
                            if let Some(mapping) =
                                mappings.iter().find(|m| m.device.matches(device))
                            {
                                if verbose {
                                    println!(
                                        "[DEBUG] Initial device matched: {} -> {}",
                                        device.display_name(),
                                        mapping.name
                                    );
                                }
                                connected.push(ConnectedDevice {
                                    identifier: device.clone(),
                                    mapping: mapping.clone(),
                                });
                            }
                        }

                        // Switch to highest priority device's profile
                        if !connected.is_empty() {
                            let profile =
                                Self::get_highest_priority_profile(&connected, &default_profile);
                            println!(
                                "Initial device detected, switching to profile: {}",
                                profile
                            );
                            karabiner.switch_profile(&profile)?;
                        }
                    }
                    DeviceEvent::Connected(device) => {
                        let mut connected = connected_devices.lock().unwrap();

                        if let Some(mapping) = mappings.iter().find(|m| m.device.matches(&device)) {
                            if verbose {
                                println!(
                                    "[DEBUG] Device matched: {} -> {} (priority: {})",
                                    device.display_name(),
                                    mapping.name,
                                    mapping.priority
                                );
                            }

                            connected.push(ConnectedDevice {
                                identifier: device.clone(),
                                mapping: mapping.clone(),
                            });

                            // Get the highest priority profile among all connected devices
                            let profile =
                                Self::get_highest_priority_profile(&connected, &default_profile);
                            println!(
                                "Device connected: {}, switching to profile: {}",
                                device.display_name(),
                                profile
                            );
                            karabiner.switch_profile(&profile)?;
                        } else if verbose {
                            println!(
                                "[DEBUG] Device connected but no matching mapping: {}",
                                device.display_name()
                            );
                        }
                    }
                    DeviceEvent::Disconnected(device) => {
                        let mut connected = connected_devices.lock().unwrap();

                        // Remove the disconnected device (using matches for Bluetooth partial matching)
                        let before_len = connected.len();
                        connected.retain(|d| !d.identifier.matches(&device));
                        let removed = before_len != connected.len();

                        if removed {
                            if connected.is_empty() {
                                println!(
                                    "Device disconnected: {}, switching to default profile: {}",
                                    device.display_name(),
                                    default_profile
                                );
                                karabiner.switch_profile(&default_profile)?;
                            } else {
                                // Switch to the highest priority remaining device's profile
                                let profile =
                                    Self::get_highest_priority_profile(&connected, &default_profile);
                                println!(
                                    "Device disconnected: {}, switching to profile: {} (another device still connected)",
                                    device.display_name(),
                                    profile
                                );
                                karabiner.switch_profile(&profile)?;
                            }
                        } else if verbose {
                            println!(
                                "[DEBUG] Device disconnected but was not tracked: {}",
                                device.display_name()
                            );
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
                Arc::clone(&mappings),
                Arc::clone(&default_profile),
                Arc::clone(&connected_devices),
                karabiner.clone(),
                verbose,
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
                Arc::clone(&mappings),
                Arc::clone(&default_profile),
                Arc::clone(&connected_devices),
                karabiner.clone(),
                verbose,
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
        println!("Configured mappings (sorted by priority):");

        // Sort and display by priority
        let mut sorted_mappings = self.mappings.clone();
        sorted_mappings.sort_by(|a, b| b.priority.cmp(&a.priority));
        for mapping in &sorted_mappings {
            println!(
                "  - {} [priority: {}] -> Profile: {}",
                mapping.device.display_name(),
                mapping.priority,
                mapping.profile
            );
        }

        if self.verbose {
            println!("[DEBUG] Verbose logging enabled");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_highest_priority_profile() {
        let connected = vec![
            ConnectedDevice {
                identifier: DeviceIdentifier::usb(1234),
                mapping: KeyboardMapping::with_priority(
                    "Low priority",
                    DeviceIdentifier::usb(1234),
                    "Profile A",
                    1,
                ),
            },
            ConnectedDevice {
                identifier: DeviceIdentifier::usb(5678),
                mapping: KeyboardMapping::with_priority(
                    "High priority",
                    DeviceIdentifier::usb(5678),
                    "Profile B",
                    10,
                ),
            },
        ];

        let profile = CombinedMonitor::get_highest_priority_profile(&connected, "Default");
        assert_eq!(profile, "Profile B");
    }

    #[test]
    fn test_get_highest_priority_profile_empty() {
        let connected: Vec<ConnectedDevice> = vec![];
        let profile = CombinedMonitor::get_highest_priority_profile(&connected, "Default");
        assert_eq!(profile, "Default");
    }

    #[test]
    fn test_device_identifier_matches_usb() {
        let device1 = DeviceIdentifier::usb(1234);
        let device2 = DeviceIdentifier::usb(1234);
        let device3 = DeviceIdentifier::usb(5678);

        assert!(device1.matches(&device2));
        assert!(!device1.matches(&device3));
    }

    #[test]
    fn test_device_identifier_matches_bluetooth_partial() {
        let config_name = DeviceIdentifier::bluetooth("HHKB");
        let actual_name = DeviceIdentifier::bluetooth("HHKB-BT");

        // Partial match should work in both directions
        assert!(config_name.matches(&actual_name));
        assert!(actual_name.matches(&config_name));
    }

    #[test]
    fn test_device_identifier_matches_bluetooth_case_insensitive() {
        let config_name = DeviceIdentifier::bluetooth("Magic Keyboard");
        let actual_name = DeviceIdentifier::bluetooth("magic keyboard");

        assert!(config_name.matches(&actual_name));
    }
}
