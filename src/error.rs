use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("USB device error: {0}")]
    UsbDevice(String),
    
    #[error("Karabiner-Elements error: {0}")]
    Karabiner(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("YAML serialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("Home directory not found")]
    HomeDirectoryNotFound,
    
    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;