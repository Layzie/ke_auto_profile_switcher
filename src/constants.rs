// Application constants

pub const DEFAULT_PROFILE_NAME: &str = "Default";
pub const KARABINER_CLI_PATH: &str =
    "/Library/Application Support/org.pqrs/Karabiner-Elements/bin/karabiner_cli";
pub const CONFIG_DIR_NAME: &str = "ke_auto_profile_switcher";
pub const CONFIG_FILE_NAME: &str = "config.yml";

/// Highest configuration schema version this build understands. A config file
/// declaring a higher version is rejected rather than silently misread.
pub const CURRENT_CONFIG_VERSION: u8 = 2;
