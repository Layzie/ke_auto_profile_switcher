use clap::Parser;
use ke_auto_profile_switcher::{
    cli::{Action, Args, DeviceType},
    config::{resolve_config, Config},
    monitor::{
        bluetooth::list_bluetooth_devices, combined::CombinedMonitor, usb::list_usb_devices,
    },
    Result,
};

fn main() -> Result<()> {
    let cli = Args::parse();

    match cli.action {
        Action::Watch {
            keyboard_id,
            product_profile,
            default_profile,
        } => {
            let config = resolve_config(keyboard_id, product_profile, default_profile)?;
            start_monitoring(config)?;
        }
        Action::Check { device_type } => {
            list_devices(device_type)?;
        }
    }

    Ok(())
}

/// Start monitoring for keyboard connections
fn start_monitoring(config: Config) -> Result<()> {
    let monitor = CombinedMonitor::new(config.keyboards, config.default_profile);
    monitor.start_monitoring()
}

/// List available devices
fn list_devices(device_type: DeviceType) -> Result<()> {
    match device_type {
        DeviceType::All => {
            println!("=== USB Devices ===");
            list_usb_devices();
            println!();
            println!("=== Bluetooth Devices ===");
            if let Err(e) = list_bluetooth_devices() {
                println!("  Could not list Bluetooth devices: {}", e);
            }
        }
        DeviceType::Usb => {
            list_usb_devices();
        }
        DeviceType::Bluetooth => {
            list_bluetooth_devices()?;
        }
    }
    Ok(())
}
