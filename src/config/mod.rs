use crate::constants::{CONFIG_DIR_NAME, CONFIG_FILE_NAME, DEFAULT_PROFILE_NAME};
use crate::error::{AppError, Result};
use crate::monitor::bluetooth::list_bluetooth_devices;
use crate::monitor::usb::list_usb_devices;
use crate::monitor::{DeviceIdentifier, KeyboardMapping};
use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Legacy configuration format (v1) for backward compatibility
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LegacyConfig {
    pub keyboard_id: u16,
    pub product_profile: String,
    pub default_profile: String,
}

/// New configuration format (v2) supporting multiple keyboards and device types
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    /// Version of the configuration format
    #[serde(default = "default_version")]
    pub version: u8,
    /// Default profile when no keyboards are connected
    pub default_profile: String,
    /// List of keyboard mappings
    pub keyboards: Vec<KeyboardMapping>,
}

fn default_version() -> u8 {
    2
}

impl Config {
    /// Create a new configuration
    pub fn new(default_profile: impl Into<String>, keyboards: Vec<KeyboardMapping>) -> Self {
        Config {
            version: 2,
            default_profile: default_profile.into(),
            keyboards,
        }
    }

    /// Load configuration from file, supporting both legacy and new formats
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        let contents = fs::read_to_string(&config_path)
            .map_err(|e| AppError::Config(format!("Failed to read config file: {}", e)))?;

        // Try to parse as new format first
        if let Ok(config) = serde_yaml::from_str::<Config>(&contents) {
            return Ok(config);
        }

        // Try to parse as legacy format
        if let Ok(legacy) = serde_yaml::from_str::<LegacyConfig>(&contents) {
            return Ok(Self::from_legacy(legacy));
        }

        Err(AppError::Config(
            "Failed to parse config file. Invalid format.".to_string(),
        ))
    }

    /// Convert legacy config to new format
    pub fn from_legacy(legacy: LegacyConfig) -> Self {
        let mapping = KeyboardMapping::new(
            "USB Keyboard",
            DeviceIdentifier::usb(legacy.keyboard_id),
            legacy.product_profile,
        );

        Config {
            version: 2,
            default_profile: legacy.default_profile,
            keyboards: vec![mapping],
        }
    }

    /// Convert to legacy config if possible (single USB keyboard)
    pub fn to_legacy(&self) -> Option<LegacyConfig> {
        if self.keyboards.len() == 1 {
            if let DeviceIdentifier::Usb { product_id } = &self.keyboards[0].device {
                return Some(LegacyConfig {
                    keyboard_id: *product_id,
                    product_profile: self.keyboards[0].profile.clone(),
                    default_profile: self.default_profile.clone(),
                });
            }
        }
        None
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let yaml_content = serde_yaml::to_string(self)?;
        fs::write(config_path, yaml_content)
            .map_err(|e| AppError::Config(format!("Failed to write config file: {}", e)))?;
        Ok(())
    }

    pub fn get_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().ok_or(AppError::HomeDirectoryNotFound)?;
        let mut path = home_dir.join(".config").join(CONFIG_DIR_NAME);
        fs::create_dir_all(&path)
            .map_err(|e| AppError::Config(format!("Failed to create config directory: {}", e)))?;
        path.push(CONFIG_FILE_NAME);
        Ok(path)
    }

    /// Create configuration interactively
    pub fn create_interactively() -> Result<Self> {
        println!("Configuration file not found. Let's create one!");
        println!();

        // Show available devices
        println!("=== Available USB devices ===");
        list_usb_devices();
        println!();

        println!("=== Available Bluetooth devices ===");
        if let Err(e) = list_bluetooth_devices() {
            println!("  Could not list Bluetooth devices: {}", e);
        }
        println!();

        // Get default profile name first
        let default_profile = Self::prompt_for_default_profile()?;

        // Collect keyboard mappings
        let mut keyboards = Vec::new();
        loop {
            println!();
            let mapping = Self::prompt_for_keyboard_mapping()?;
            keyboards.push(mapping);

            print!("Add another keyboard? (y/N): ");
            io::stdout().flush().map_err(AppError::Io)?;

            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(AppError::Io)?;

            if !input.trim().to_lowercase().starts_with('y') {
                break;
            }
        }

        let config = Config::new(default_profile, keyboards);

        // Save the configuration
        config.save()?;
        println!();
        println!("Configuration saved successfully!");
        println!("Config file location: {:?}", Self::get_config_path()?);
        println!();

        Ok(config)
    }

    fn prompt_for_keyboard_mapping() -> Result<KeyboardMapping> {
        // Ask for device type
        println!("Device type:");
        println!("  1. USB keyboard");
        println!("  2. Bluetooth keyboard");
        print!("Select (1 or 2): ");
        io::stdout().flush().map_err(AppError::Io)?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(AppError::Io)?;

        let device = match input.trim() {
            "1" => {
                let product_id = Self::prompt_for_usb_product_id()?;
                DeviceIdentifier::usb(product_id)
            }
            "2" => {
                let device_name = Self::prompt_for_bluetooth_name()?;
                DeviceIdentifier::bluetooth(device_name)
            }
            _ => {
                println!("Invalid selection, defaulting to USB.");
                let product_id = Self::prompt_for_usb_product_id()?;
                DeviceIdentifier::usb(product_id)
            }
        };

        // Get a name for this mapping
        print!("Enter a name for this keyboard (e.g., 'Work Keyboard'): ");
        io::stdout().flush().map_err(AppError::Io)?;

        let mut name_input = String::new();
        io::stdin()
            .read_line(&mut name_input)
            .map_err(AppError::Io)?;
        let name = if name_input.trim().is_empty() {
            "Keyboard".to_string()
        } else {
            name_input.trim().to_string()
        };

        // Get profile name
        let profile = Self::prompt_for_product_profile()?;

        Ok(KeyboardMapping::new(name, device, profile))
    }

    fn prompt_for_usb_product_id() -> Result<u16> {
        loop {
            print!("Enter the USB keyboard product ID: ");
            io::stdout().flush().map_err(AppError::Io)?;

            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(AppError::Io)?;

            match input.trim().parse::<u16>() {
                Ok(id) => return Ok(id),
                Err(_) => {
                    println!("Please enter a valid number.");
                    continue;
                }
            }
        }
    }

    fn prompt_for_bluetooth_name() -> Result<String> {
        loop {
            print!("Enter the Bluetooth device name: ");
            io::stdout().flush().map_err(AppError::Io)?;

            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(AppError::Io)?;

            let trimmed = input.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            } else {
                println!("Device name cannot be empty.");
                continue;
            }
        }
    }

    fn prompt_for_product_profile() -> Result<String> {
        loop {
            print!("Enter the Karabiner-Elements profile name for this keyboard: ");
            io::stdout().flush().map_err(AppError::Io)?;

            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(AppError::Io)?;

            let trimmed = input.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            } else {
                println!("Profile name cannot be empty.");
                continue;
            }
        }
    }

    fn prompt_for_default_profile() -> Result<String> {
        print!(
            "Enter the default Karabiner-Elements profile name [{}]: ",
            DEFAULT_PROFILE_NAME
        );
        io::stdout().flush().map_err(AppError::Io)?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(AppError::Io)?;

        let default_profile = if input.trim().is_empty() {
            DEFAULT_PROFILE_NAME.to_string()
        } else {
            input.trim().to_string()
        };

        Ok(default_profile)
    }

    /// Create config from CLI arguments (legacy compatibility)
    pub fn from_cli_args(
        keyboard_id: u16,
        product_profile: String,
        default_profile: Option<String>,
    ) -> Self {
        let mapping = KeyboardMapping::new(
            "USB Keyboard",
            DeviceIdentifier::usb(keyboard_id),
            product_profile,
        );

        Config::new(
            default_profile.unwrap_or_else(|| DEFAULT_PROFILE_NAME.to_string()),
            vec![mapping],
        )
    }

    // Test helper methods
    #[cfg(test)]
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| AppError::Config(format!("Failed to read config file: {}", e)))?;

        // Try new format first
        if let Ok(config) = serde_yaml::from_str::<Config>(&contents) {
            return Ok(config);
        }

        // Try legacy format
        if let Ok(legacy) = serde_yaml::from_str::<LegacyConfig>(&contents) {
            return Ok(Self::from_legacy(legacy));
        }

        Err(AppError::Config("Failed to parse config file".to_string()))
    }

    #[cfg(test)]
    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        let yaml_content = serde_yaml::to_string(self)?;
        fs::write(path, yaml_content)
            .map_err(|e| AppError::Config(format!("Failed to write config file: {}", e)))?;
        Ok(())
    }
}

/// Resolve configuration from various sources
pub fn resolve_config(
    keyboard_id: Option<u16>,
    product_profile: Option<String>,
    default_profile: Option<String>,
) -> Result<Config> {
    // Try to load from config file first
    match Config::load() {
        Ok(config) => Ok(config),
        Err(_) => {
            // Config file doesn't exist, check for command line arguments
            if let (Some(id), Some(profile)) = (keyboard_id, product_profile) {
                // Use command line arguments
                Ok(Config::from_cli_args(id, profile, default_profile))
            } else {
                // Neither config file nor complete command line arguments available
                // Create configuration interactively
                Config::create_interactively()
            }
        }
    }
}

#[cfg(test)]
mod tests;
