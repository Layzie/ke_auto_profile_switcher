use clap::{Parser, Subcommand};

// Auto Karabiner Element profile switcher, based on keyboard availability
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

// Available CLI actions
#[derive(Subcommand, Debug)]
pub enum Action {
    #[command(about = "Watching USB device's connection")]
    Watch {
        #[arg(short, long, help = "Number of external USB keyboard product id")]
        keyboard_id: Option<u16>,
        #[arg(
            short,
            long,
            help = "Name of Karabiner-Elements profile using external keyboard"
        )]
        product_profile: Option<String>,
        #[arg(
            short,
            long,
            help = "Name of default Karabiner-Elements profile"
        )]
        default_profile: Option<String>,
    },
    #[command(about = "Check USB device information")]
    Check {},
}

#[derive(Debug, Clone)]
pub struct WatchArgs {
    pub keyboard_id: u16,
    pub product_profile: String,
    pub default_profile: String,
}

impl WatchArgs {
    pub fn from_config(config: &crate::config::Config) -> Self {
        WatchArgs {
            keyboard_id: config.keyboard_id,
            product_profile: config.product_profile.clone(),
            default_profile: config.default_profile.clone(),
        }
    }
}