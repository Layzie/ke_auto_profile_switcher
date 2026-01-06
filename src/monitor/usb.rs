//! USB device monitoring module.
//!
//! Uses the `usb_enumeration` crate to monitor USB device connections.

use crate::constants::USB_POLL_INTERVAL_SECONDS;
use crate::error::{AppError, Result};
use crate::monitor::{DeviceEvent, DeviceIdentifier, DeviceInfo, DeviceMonitor};
use usb_enumeration::{Event, Observer};

/// Monitor for USB keyboard connections
pub struct UsbMonitor {
    /// USB product IDs to monitor
    product_ids: Vec<u16>,
}

impl UsbMonitor {
    /// Create a new USB monitor for the given product IDs
    pub fn new(product_ids: Vec<u16>) -> Self {
        UsbMonitor { product_ids }
    }

    /// Create a USB monitor for a single product ID
    pub fn single(product_id: u16) -> Self {
        UsbMonitor {
            product_ids: vec![product_id],
        }
    }
}

impl DeviceMonitor for UsbMonitor {
    fn start_monitoring<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(DeviceEvent) -> Result<()> + Send + Sync + 'static,
    {
        // If no product IDs to monitor, return early
        if self.product_ids.is_empty() {
            return Ok(());
        }

        // Create observer for all USB devices (we'll filter ourselves)
        let subscription = Observer::new()
            .with_poll_interval(USB_POLL_INTERVAL_SECONDS)
            .subscribe();

        let product_ids = self.product_ids.clone();

        for event in subscription.rx_event.iter() {
            match event {
                Event::Initial(devices) => {
                    let matched: Vec<DeviceIdentifier> = devices
                        .iter()
                        .filter(|d| product_ids.contains(&d.product_id))
                        .map(|d| DeviceIdentifier::usb(d.product_id))
                        .collect();

                    if !matched.is_empty() {
                        callback(DeviceEvent::Initial(matched)).map_err(|e| {
                            AppError::UsbDevice(format!("Failed to handle initial devices: {}", e))
                        })?;
                    }
                }
                Event::Connect(device) => {
                    if product_ids.contains(&device.product_id) {
                        let identifier = DeviceIdentifier::usb(device.product_id);
                        println!(
                            "USB device connected: {} (Product ID: {})",
                            device.description.as_deref().unwrap_or("Unknown"),
                            device.product_id
                        );
                        callback(DeviceEvent::Connected(identifier)).map_err(|e| {
                            AppError::UsbDevice(format!(
                                "Failed to handle device connection: {}",
                                e
                            ))
                        })?;
                    }
                }
                Event::Disconnect(device) => {
                    if product_ids.contains(&device.product_id) {
                        let identifier = DeviceIdentifier::usb(device.product_id);
                        println!(
                            "USB device disconnected: {} (Product ID: {})",
                            device.description.as_deref().unwrap_or("Unknown"),
                            device.product_id
                        );
                        callback(DeviceEvent::Disconnected(identifier)).map_err(|e| {
                            AppError::UsbDevice(format!(
                                "Failed to handle device disconnection: {}",
                                e
                            ))
                        })?;
                    }
                }
            }
        }

        Ok(())
    }

    fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
        let devices = usb_enumeration::enumerate(None, None);
        Ok(devices
            .into_iter()
            .map(|d| {
                let identifier = DeviceIdentifier::usb(d.product_id);
                let description = d
                    .description
                    .unwrap_or_else(|| "Unknown Device".to_string());
                DeviceInfo::new(identifier, description, true)
            })
            .collect())
    }
}

/// List all USB devices (for the check command)
pub fn list_usb_devices() {
    let devices = usb_enumeration::enumerate(None, None);
    if devices.is_empty() {
        println!("No USB devices found.");
        return;
    }

    println!("USB Devices:");
    for device in devices.iter() {
        let description = device.description.as_deref().unwrap_or("Unknown Device");
        println!("  Product ID: {}, Name: {}", device.product_id, description);
    }
}
