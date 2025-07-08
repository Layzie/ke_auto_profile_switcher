use crate::constants::USB_POLL_INTERVAL_SECONDS;
use crate::error::{AppError, Result};
use usb_enumeration::{Event, Observer};

pub struct UsbMonitor {
    keyboard_id: u16,
}

impl UsbMonitor {
    pub fn new(keyboard_id: u16) -> Self {
        UsbMonitor { keyboard_id }
    }

    pub fn start_monitoring<F1, F2>(&self, on_connect: F1, on_disconnect: F2) -> Result<()>
    where
        F1: Fn() -> Result<()>,
        F2: Fn() -> Result<()>,
    {
        let keyboard = Observer::new()
            .with_poll_interval(USB_POLL_INTERVAL_SECONDS)
            .with_product_id(self.keyboard_id)
            .subscribe();

        for event in keyboard.rx_event.iter() {
            match event {
                Event::Initial(devices) => {
                    println!("Initial devices: {:?}", devices);
                    // Check if the target keyboard is already connected
                    if devices.iter().any(|d| d.product_id == self.keyboard_id) {
                        on_connect().map_err(|e| {
                            AppError::UsbDevice(format!("Failed to handle initial connection: {}", e))
                        })?;
                    }
                }
                Event::Connect(device) => {
                    println!("Connected device: {:?}", device);
                    on_connect().map_err(|e| {
                        AppError::UsbDevice(format!("Failed to handle device connection: {}", e))
                    })?;
                }
                Event::Disconnect(device) => {
                    println!("Disconnected device: {:?}", device);
                    on_disconnect().map_err(|e| {
                        AppError::UsbDevice(format!("Failed to handle device disconnection: {}", e))
                    })?;
                }
            }
        }

        Ok(())
    }
}

pub fn list_usb_devices() {
    let devices = usb_enumeration::enumerate(None, None);
    if devices.is_empty() {
        println!("No USB devices found.");
        return;
    }

    for device in devices.iter() {
        let description = device.description.as_deref().unwrap_or("Unknown Device");
        println!("  ID: {}, Product: {}", device.product_id, description);
    }
}