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

#[derive(Debug, Clone)]
struct ConnectedDevice {
    identifier: DeviceIdentifier,
    mapping: KeyboardMapping,
}

pub struct CombinedMonitor {
    mappings: Vec<KeyboardMapping>,
    default_profile: String,
    verbose: bool,
}

impl CombinedMonitor {
    pub fn new(mappings: Vec<KeyboardMapping>, default_profile: impl Into<String>) -> Self {
        CombinedMonitor {
            mappings,
            default_profile: default_profile.into(),
            verbose: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

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

    /// Apply `target` via Karabiner only if it differs from the last applied
    /// profile. Cache lock spans the subprocess call so concurrent USB and
    /// Bluetooth threads serialize switches and observe a consistent cache.
    fn apply_target_profile(
        target: &str,
        last_applied: &Mutex<Option<String>>,
        karabiner: &KarabinerController,
        verbose: bool,
    ) -> Result<()> {
        let mut applied = last_applied
            .lock()
            .expect("last_applied mutex poisoned");
        if applied.as_deref() == Some(target) {
            if verbose {
                println!("[DEBUG] Profile already '{}', skipping switch", target);
            }
            return Ok(());
        }
        println!("Switching to profile: {}", target);
        karabiner.switch_profile(target)?;
        *applied = Some(target.to_string());
        Ok(())
    }

    pub fn start_monitoring(&self) -> Result<()> {
        let karabiner = KarabinerController::new();

        let mut usb_product_ids: Vec<u16> = Vec::new();
        let mut bluetooth_names: Vec<String> = Vec::new();
        for mapping in &self.mappings {
            match &mapping.device {
                DeviceIdentifier::Usb { product_id } => usb_product_ids.push(*product_id),
                DeviceIdentifier::Bluetooth { device_name } => {
                    bluetooth_names.push(device_name.clone())
                }
            }
        }

        let mappings = Arc::new(self.mappings.clone());
        let default_profile = Arc::new(self.default_profile.clone());
        let connected_devices: Arc<Mutex<Vec<ConnectedDevice>>> = Arc::new(Mutex::new(Vec::new()));
        let last_applied: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let verbose = self.verbose;

        let create_handler = |mappings: Arc<Vec<KeyboardMapping>>,
                              default_profile: Arc<String>,
                              connected_devices: Arc<Mutex<Vec<ConnectedDevice>>>,
                              last_applied: Arc<Mutex<Option<String>>>,
                              karabiner: KarabinerController,
                              verbose: bool| {
            move |event: DeviceEvent| -> Result<()> {
                let target_profile: Option<String> = {
                    let mut connected = connected_devices
                        .lock()
                        .expect("connected_devices mutex poisoned");
                    match event {
                        DeviceEvent::Initial(devices) => {
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
                            if connected.is_empty() {
                                None
                            } else {
                                let profile = Self::get_highest_priority_profile(
                                    &connected,
                                    &default_profile,
                                );
                                println!("Initial device detected, target profile: {}", profile);
                                Some(profile)
                            }
                        }
                        DeviceEvent::Connected(device) => {
                            let Some(mapping) =
                                mappings.iter().find(|m| m.device.matches(&device))
                            else {
                                if verbose {
                                    println!(
                                        "[DEBUG] Device connected but no matching mapping: {}",
                                        device.display_name()
                                    );
                                }
                                return Ok(());
                            };
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
                            let profile =
                                Self::get_highest_priority_profile(&connected, &default_profile);
                            println!(
                                "Device connected: {}, target profile: {}",
                                device.display_name(),
                                profile
                            );
                            Some(profile)
                        }
                        DeviceEvent::Disconnected(device) => {
                            let before_len = connected.len();
                            connected.retain(|d| !d.identifier.matches(&device));
                            if before_len == connected.len() {
                                if verbose {
                                    println!(
                                        "[DEBUG] Device disconnected but was not tracked: {}",
                                        device.display_name()
                                    );
                                }
                                return Ok(());
                            }
                            let profile =
                                Self::get_highest_priority_profile(&connected, &default_profile);
                            println!(
                                "Device disconnected: {}, target profile: {}",
                                device.display_name(),
                                profile
                            );
                            Some(profile)
                        }
                    }
                };

                if let Some(profile) = target_profile {
                    Self::apply_target_profile(&profile, &last_applied, &karabiner, verbose)?;
                }
                Ok(())
            }
        };

        let mut handles = Vec::new();

        if !usb_product_ids.is_empty() {
            let usb_monitor = UsbMonitor::new(usb_product_ids);
            let handler = create_handler(
                Arc::clone(&mappings),
                Arc::clone(&default_profile),
                Arc::clone(&connected_devices),
                Arc::clone(&last_applied),
                karabiner.clone(),
                verbose,
            );
            handles.push(thread::spawn(move || {
                if let Err(e) = usb_monitor.start_monitoring(handler) {
                    eprintln!("USB monitoring error: {}", e);
                }
            }));
        }

        if !bluetooth_names.is_empty() {
            let bluetooth_monitor = BluetoothMonitor::new(bluetooth_names);
            let handler = create_handler(
                Arc::clone(&mappings),
                Arc::clone(&default_profile),
                Arc::clone(&connected_devices),
                Arc::clone(&last_applied),
                karabiner.clone(),
                verbose,
            );
            handles.push(thread::spawn(move || {
                if let Err(e) = bluetooth_monitor.start_monitoring(handler) {
                    eprintln!("Bluetooth monitoring error: {}", e);
                }
            }));
        }

        if handles.is_empty() {
            return Err(AppError::Config(
                "No devices configured for monitoring".to_string(),
            ));
        }

        println!("Monitoring started. Press Ctrl+C to stop.");
        println!("Default profile: {}", self.default_profile);
        println!("Configured mappings (sorted by priority):");

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

        for handle in handles {
            handle.join().map_err(|e| {
                AppError::Monitor(format!("Monitoring thread panicked: {:?}", e))
            })?;
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
