use super::*;
use crate::monitor::DeviceIdentifier;
use tempfile::tempdir;

#[test]
fn test_config_serialization() {
    let mapping = KeyboardMapping::new("Test Keyboard", DeviceIdentifier::usb(1234), "External");

    let config = Config::new("Default", vec![mapping]);

    let yaml = serde_yaml::to_string(&config).unwrap();
    let deserialized: Config = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(config, deserialized);
}

#[test]
fn test_config_with_multiple_keyboards() {
    let usb_mapping =
        KeyboardMapping::new("USB Keyboard", DeviceIdentifier::usb(1234), "USB Profile");

    let bluetooth_mapping = KeyboardMapping::new(
        "Bluetooth Keyboard",
        DeviceIdentifier::bluetooth("Magic Keyboard"),
        "Bluetooth Profile",
    );

    let config = Config::new(
        "Default",
        vec![usb_mapping.clone(), bluetooth_mapping.clone()],
    );

    let yaml = serde_yaml::to_string(&config).unwrap();
    let deserialized: Config = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(config.keyboards.len(), 2);
    assert_eq!(deserialized.keyboards.len(), 2);
    assert_eq!(config, deserialized);
}

#[test]
fn test_config_save_and_load() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.yml");

    let mapping =
        KeyboardMapping::new("Test Keyboard", DeviceIdentifier::usb(5678), "Test Profile");

    let config = Config::new("Test Default", vec![mapping]);

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

    assert_eq!(config.keyboards.len(), 1);
    assert_eq!(config.keyboards[0].profile, "CLI Profile");
    assert_eq!(config.default_profile, "CLI Default");

    // Verify the device identifier
    match &config.keyboards[0].device {
        DeviceIdentifier::Usb { product_id } => assert_eq!(*product_id, 1111),
        _ => panic!("Expected USB device identifier"),
    }
}

#[test]
fn test_config_from_cli_args_with_default() {
    let config = Config::from_cli_args(2222, "CLI Profile".to_string(), None);

    assert_eq!(config.keyboards.len(), 1);
    assert_eq!(config.keyboards[0].profile, "CLI Profile");
    assert_eq!(config.default_profile, DEFAULT_PROFILE_NAME);
}

#[test]
fn test_legacy_config_compatibility() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.yml");

    // Create a legacy config file
    let legacy_yaml = r#"
keyboard_id: 1234
product_profile: "External"
default_profile: "Default"
"#;
    fs::write(&config_path, legacy_yaml).unwrap();

    // Load it with the new Config
    let config = Config::load_from_path(&config_path).unwrap();

    assert_eq!(config.keyboards.len(), 1);
    assert_eq!(config.default_profile, "Default");
    assert_eq!(config.keyboards[0].profile, "External");

    match &config.keyboards[0].device {
        DeviceIdentifier::Usb { product_id } => assert_eq!(*product_id, 1234),
        _ => panic!("Expected USB device identifier"),
    }
}

#[test]
fn test_device_identifier_display() {
    let usb = DeviceIdentifier::usb(1234);
    let bt = DeviceIdentifier::bluetooth("Magic Keyboard");

    assert!(usb.display_name().contains("1234"));
    assert!(bt.display_name().contains("Magic Keyboard"));
}

#[test]
fn test_to_legacy_single_usb() {
    let mapping = KeyboardMapping::new("USB Keyboard", DeviceIdentifier::usb(1234), "External");
    let config = Config::new("Default", vec![mapping]);

    let legacy = config.to_legacy().unwrap();
    assert_eq!(legacy.keyboard_id, 1234);
    assert_eq!(legacy.product_profile, "External");
    assert_eq!(legacy.default_profile, "Default");
}

#[test]
fn test_to_legacy_not_possible_for_bluetooth() {
    let mapping = KeyboardMapping::new(
        "BT Keyboard",
        DeviceIdentifier::bluetooth("Magic Keyboard"),
        "External",
    );
    let config = Config::new("Default", vec![mapping]);

    assert!(config.to_legacy().is_none());
}

#[test]
fn test_to_legacy_not_possible_for_multiple() {
    let mapping1 = KeyboardMapping::new("KB1", DeviceIdentifier::usb(1234), "Profile1");
    let mapping2 = KeyboardMapping::new("KB2", DeviceIdentifier::usb(5678), "Profile2");
    let config = Config::new("Default", vec![mapping1, mapping2]);

    assert!(config.to_legacy().is_none());
}
