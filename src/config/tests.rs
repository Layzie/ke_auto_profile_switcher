use super::*;
use tempfile::tempdir;

#[test]
fn test_config_serialization() {
    let config = Config {
        keyboard_id: 1234,
        product_profile: "External".to_string(),
        default_profile: "Default".to_string(),
    };

    let yaml = serde_yaml::to_string(&config).unwrap();
    let deserialized: Config = serde_yaml::from_str(&yaml).unwrap();
    
    assert_eq!(config, deserialized);
}

#[test]
fn test_config_save_and_load() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.yml");
    
    let config = Config {
        keyboard_id: 5678,
        product_profile: "Test Profile".to_string(),
        default_profile: "Test Default".to_string(),
    };

    // Save config
    config.save_to_path(&config_path).unwrap();
    
    // Load config
    let loaded_config = Config::load_from_path(&config_path).unwrap();
    
    assert_eq!(config, loaded_config);
}

#[test]
fn test_config_from_cli_args() {
    let config = Config::from_cli_args(
        1111,
        "CLI Profile".to_string(),
        Some("CLI Default".to_string()),
    );
    
    assert_eq!(config.keyboard_id, 1111);
    assert_eq!(config.product_profile, "CLI Profile");
    assert_eq!(config.default_profile, "CLI Default");
}

#[test]
fn test_config_from_cli_args_with_default() {
    let config = Config::from_cli_args(
        2222,
        "CLI Profile".to_string(),
        None,
    );
    
    assert_eq!(config.keyboard_id, 2222);
    assert_eq!(config.product_profile, "CLI Profile");
    assert_eq!(config.default_profile, DEFAULT_PROFILE_NAME);
}