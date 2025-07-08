use crate::constants::{CONFIG_DIR_NAME, CONFIG_FILE_NAME, DEFAULT_PROFILE_NAME};
use crate::error::{AppError, Result};
use crate::usb_monitor::list_usb_devices;
use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    pub keyboard_id: u16,
    pub product_profile: String,
    pub default_profile: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        let contents = fs::read_to_string(config_path)
            .map_err(|e| AppError::Config(format!("Failed to read config file: {}", e)))?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
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

    pub fn create_interactively() -> Result<Self> {
        println!("Configuration file not found. Let's create one!");
        println!();
        
        // Show available USB devices
        println!("Available USB devices:");
        list_usb_devices();
        println!();

        // Get keyboard ID
        let keyboard_id = Self::prompt_for_keyboard_id()?;

        // Get product profile name
        let product_profile = Self::prompt_for_product_profile()?;

        // Get default profile name
        let default_profile = Self::prompt_for_default_profile()?;

        let config = Config {
            keyboard_id,
            product_profile,
            default_profile,
        };

        // Save the configuration
        config.save()?;
        println!();
        println!("Configuration saved successfully!");
        println!("Config file location: {:?}", Self::get_config_path()?);
        println!();

        Ok(config)
    }

    fn prompt_for_keyboard_id() -> Result<u16> {
        loop {
            print!("Enter the USB keyboard product ID: ");
            io::stdout().flush()
                .map_err(|e| AppError::Io(e))?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)
                .map_err(|e| AppError::Io(e))?;
            
            match input.trim().parse::<u16>() {
                Ok(id) => return Ok(id),
                Err(_) => {
                    println!("Please enter a valid number.");
                    continue;
                }
            }
        }
    }

    fn prompt_for_product_profile() -> Result<String> {
        loop {
            print!("Enter the Karabiner-Elements profile name for external keyboard: ");
            io::stdout().flush()
                .map_err(|e| AppError::Io(e))?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)
                .map_err(|e| AppError::Io(e))?;
            
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
        print!("Enter the default Karabiner-Elements profile name [{}]: ", DEFAULT_PROFILE_NAME);
        io::stdout().flush()
            .map_err(|e| AppError::Io(e))?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)
            .map_err(|e| AppError::Io(e))?;
        
        let default_profile = if input.trim().is_empty() {
            DEFAULT_PROFILE_NAME.to_string()
        } else {
            input.trim().to_string()
        };
        
        Ok(default_profile)
    }

    pub fn from_cli_args(
        keyboard_id: u16,
        product_profile: String,
        default_profile: Option<String>,
    ) -> Self {
        Config {
            keyboard_id,
            product_profile,
            default_profile: default_profile.unwrap_or_else(|| DEFAULT_PROFILE_NAME.to_string()),
        }
    }

    // Test helper methods
    #[cfg(test)]
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| AppError::Config(format!("Failed to read config file: {}", e)))?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    #[cfg(test)]
    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        let yaml_content = serde_yaml::to_string(self)?;
        fs::write(path, yaml_content)
            .map_err(|e| AppError::Config(format!("Failed to write config file: {}", e)))?;
        Ok(())
    }
}

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