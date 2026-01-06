use clap::{Parser, Subcommand, ValueEnum};

/// Auto Karabiner Element profile switcher, based on keyboard availability
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

/// Device type for filtering
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum DeviceType {
    /// Show all device types
    #[default]
    All,
    /// USB devices only
    Usb,
    /// Bluetooth devices only
    Bluetooth,
}

/// Available CLI actions
#[derive(Subcommand, Debug)]
pub enum Action {
    /// Watch for keyboard connections and switch profiles automatically
    #[command(about = "Watch for keyboard connections and switch profiles")]
    Watch {
        /// USB keyboard product ID (legacy option, use config file for multiple keyboards)
        #[arg(short, long, help = "USB keyboard product ID")]
        keyboard_id: Option<u16>,

        /// Profile name when keyboard is connected (legacy option)
        #[arg(
            short,
            long,
            help = "Karabiner-Elements profile name for external keyboard"
        )]
        product_profile: Option<String>,

        /// Default profile name (legacy option)
        #[arg(short, long, help = "Default Karabiner-Elements profile name")]
        default_profile: Option<String>,
    },

    /// List available devices
    #[command(about = "List available USB and Bluetooth devices")]
    Check {
        /// Filter by device type
        #[arg(
            short = 't',
            long = "type",
            value_enum,
            default_value = "all",
            help = "Device type to list"
        )]
        device_type: DeviceType,
    },
}

/// Legacy watch arguments for backward compatibility
#[derive(Debug, Clone)]
#[deprecated(since = "0.3.0", note = "Use Config directly instead")]
pub struct WatchArgs {
    pub keyboard_id: u16,
    pub product_profile: String,
    pub default_profile: String,
}

#[allow(deprecated)]
impl WatchArgs {
    /// Create from legacy config (single USB keyboard)
    pub fn from_legacy_config(config: &crate::config::LegacyConfig) -> Self {
        WatchArgs {
            keyboard_id: config.keyboard_id,
            product_profile: config.product_profile.clone(),
            default_profile: config.default_profile.clone(),
        }
    }
}
