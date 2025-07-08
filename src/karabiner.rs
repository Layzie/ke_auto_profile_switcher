use crate::constants::KARABINER_CLI_PATH;
use crate::error::{AppError, Result};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct KarabinerController {
    cli_path: PathBuf,
}

impl KarabinerController {
    pub fn new() -> Self {
        KarabinerController {
            cli_path: PathBuf::from(KARABINER_CLI_PATH),
        }
    }

    pub fn switch_profile(&self, profile_name: &str) -> Result<()> {
        let output = Command::new(&self.cli_path)
            .arg("--select-profile")
            .arg(profile_name)
            .output()
            .map_err(|e| AppError::Karabiner(format!("Failed to execute karabiner_cli: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Karabiner(format!(
                "Karabiner CLI failed: {}",
                stderr
            )));
        }

        println!("Switched to profile: {}", profile_name);
        Ok(())
    }
}

impl Default for KarabinerController {
    fn default() -> Self {
        Self::new()
    }
}