//! Combined device monitoring module.
//!
//! Monitors USB and Bluetooth keyboards together through a single IOKit
//! event-driven monitor, managing multiple keyboard-profile mappings with
//! priority support.

use crate::error::{AppError, Result};
#[cfg(target_os = "macos")]
use crate::karabiner::KarabinerController;
#[cfg(target_os = "macos")]
use crate::monitor::{iokit::IoKitMonitor, DeviceEvent, DeviceMonitor};
use crate::monitor::{DeviceIdentifier, KeyboardMapping};
#[cfg(target_os = "macos")]
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
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

    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
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
    /// profile. The cache lock is kept while switching so that concurrent events
    /// (should they ever overlap) serialize and observe a consistent cache.
    #[cfg(target_os = "macos")]
    fn apply_target_profile(
        target: &str,
        last_applied: &Mutex<Option<String>>,
        karabiner: &KarabinerController,
        verbose: bool,
    ) -> Result<()> {
        let mut applied = last_applied.lock().expect("last_applied mutex poisoned");
        if applied.as_deref() == Some(target) {
            if verbose {
                println!("[DEBUG] Profile already '{}', skipping switch", target);
            }
            return Ok(());
        }
        // `switch_profile` itself prints "Switched to profile: ..." on success,
        // so we intentionally do not print a separate "Switching" line here.
        karabiner.switch_profile(target)?;
        *applied = Some(target.to_string());
        Ok(())
    }

    pub fn start_monitoring(&self) -> Result<()> {
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

        if usb_product_ids.is_empty() && bluetooth_names.is_empty() {
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

        #[cfg(target_os = "macos")]
        {
            let karabiner = KarabinerController::new();
            let mappings = Arc::new(self.mappings.clone());
            let default_profile = Arc::new(self.default_profile.clone());
            let connected_devices: Arc<Mutex<Vec<ConnectedDevice>>> =
                Arc::new(Mutex::new(Vec::new()));
            let last_applied: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
            let verbose = self.verbose;

            let handler = move |event: DeviceEvent| -> Result<()> {
                let target_profile: Option<String> = {
                    let mut connected = connected_devices
                        .lock()
                        .expect("connected_devices mutex poisoned");
                    match event {
                        DeviceEvent::Initial(devices) => {
                            for device in &devices {
                                // Skip devices already tracked: one physical
                                // keyboard can expose several HID nodes that each
                                // appear in the initial set.
                                if connected
                                    .iter()
                                    .any(|d| d.identifier.is_same_device(device))
                                {
                                    continue;
                                }
                                // Bind to the highest-priority matching mapping,
                                // not merely the first one in config order.
                                if let Some(mapping) = mappings
                                    .iter()
                                    .filter(|m| m.device.matches(device))
                                    .max_by_key(|m| m.priority)
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
                            // Always reconcile at startup: when no monitored
                            // keyboard is connected this resolves to the default
                            // profile, correcting any stale profile left active.
                            let profile =
                                Self::get_highest_priority_profile(&connected, &default_profile);
                            println!("Initial state resolved, target profile: {}", profile);
                            Some(profile)
                        }
                        DeviceEvent::Connected(device) => {
                            let Some(mapping) = mappings
                                .iter()
                                .filter(|m| m.device.matches(&device))
                                .max_by_key(|m| m.priority)
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
                            // Avoid duplicate entries when one physical keyboard
                            // exposes multiple HID nodes (each fires Connected).
                            if !connected
                                .iter()
                                .any(|d| d.identifier.is_same_device(&device))
                            {
                                connected.push(ConnectedDevice {
                                    identifier: device.clone(),
                                    mapping: mapping.clone(),
                                });
                            }
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
                            // Use exact identity (not partial-name `matches`) so a
                            // disconnect only removes the device that actually
                            // disconnected, never a still-connected keyboard whose
                            // name is a substring of it (e.g. "Keychron K2" vs
                            // "Keychron K2 Pro").
                            connected.retain(|d| !d.identifier.is_same_device(&device));
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
            };

            let monitor = IoKitMonitor::new(usb_product_ids, bluetooth_names);
            monitor.start_monitoring(handler)
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = (usb_product_ids, bluetooth_names);
            Err(AppError::Monitor(
                "Device monitoring requires macOS (Karabiner-Elements is macOS-only)".to_string(),
            ))
        }
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

    #[test]
    fn test_device_identifier_empty_bluetooth_name_matches_nothing() {
        let empty = DeviceIdentifier::bluetooth("");
        let any_device = DeviceIdentifier::bluetooth("Magic Keyboard");

        // An empty configured name must NOT match every device.
        assert!(!empty.matches(&any_device));
        assert!(!any_device.matches(&empty));
    }

    #[test]
    fn test_is_same_device_requires_exact_identity() {
        // Partial-name `matches` is true both ways for overlapping names...
        let k2 = DeviceIdentifier::bluetooth("Keychron K2");
        let k2_pro = DeviceIdentifier::bluetooth("Keychron K2 Pro");
        assert!(k2.matches(&k2_pro));

        // ...but they are NOT the same physical device.
        assert!(!k2.is_same_device(&k2_pro));
        // Same name (case-insensitive) is the same device.
        assert!(k2.is_same_device(&DeviceIdentifier::bluetooth("keychron k2")));
        // USB exact match.
        assert!(DeviceIdentifier::usb(1234).is_same_device(&DeviceIdentifier::usb(1234)));
        assert!(!DeviceIdentifier::usb(1234).is_same_device(&DeviceIdentifier::usb(5678)));
    }

    #[test]
    fn test_disconnect_does_not_evict_overlapping_named_device() {
        // Two distinct BT keyboards with overlapping names are both tracked.
        let mut connected = vec![
            ConnectedDevice {
                identifier: DeviceIdentifier::bluetooth("Keychron K2"),
                mapping: KeyboardMapping::new(
                    "K2",
                    DeviceIdentifier::bluetooth("Keychron K2"),
                    "Profile K2",
                ),
            },
            ConnectedDevice {
                identifier: DeviceIdentifier::bluetooth("Keychron K2 Pro"),
                mapping: KeyboardMapping::new(
                    "K2 Pro",
                    DeviceIdentifier::bluetooth("Keychron K2 Pro"),
                    "Profile K2 Pro",
                ),
            },
        ];

        // Disconnecting "Keychron K2" must leave "Keychron K2 Pro" tracked.
        let disconnected = DeviceIdentifier::bluetooth("Keychron K2");
        connected.retain(|d| !d.identifier.is_same_device(&disconnected));

        assert_eq!(connected.len(), 1);
        assert_eq!(
            connected[0].identifier,
            DeviceIdentifier::bluetooth("Keychron K2 Pro")
        );
    }
}
