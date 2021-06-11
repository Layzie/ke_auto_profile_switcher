use clap::{Parser, Subcommand};
use std::process::Command;
use usb_enumeration::{Event, Observer};

// Auto Karabiner Element profile swithcer, based on keyboard availability
#[derive(Parser, Debug)]
#[clap (author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

// Check connected USB diveces
#[derive(Subcommand, Debug)]
enum Action {
    #[clap(help = "Watching USB device's connection")]
    Watch(WatchArgs),
    #[clap(help = "Check USB device information")]
    Check {},
}

#[derive(clap::Args, Debug)]
#[clap (author, version, about, long_about = None)]
struct WatchArgs {
    #[clap(short, long, help = "Number of external USB keyboard product id")]
    keyboard_id: u16,
    #[clap(
        short,
        long,
        help = "Name of Karabiner-Elements profile using external keyboard"
    )]
    product_profile: String,
    #[clap(
        short,
        long,
        default_value = "Default",
        help = "Name of default Karbiner-Elements profile"
    )]
    default_profile: String,
}

fn check_keyboard_id() {
    let devices = usb_enumeration::enumerate(None, None);
    for device in devices.iter() {
        println!("{:?}", device);
    }
}

fn change_karabiner_profile(watch_args: WatchArgs) {
    let mut karabiner =
        Command::new("/Library/Application Support/org.pqrs/Karabiner-Elements/bin/karabiner_cli");
    let keyboard = Observer::new()
        .with_poll_interval(2)
        .with_product_id(watch_args.keyboard_id)
        .subscribe();
    for event in keyboard.rx_event.iter() {
        match event {
            Event::Initial(d) => println!("Initial devices: {:?}", d),
            Event::Connect(d) => {
                println!("Connected device: {:?}", d);
                karabiner
                    .arg("--select-profile")
                    .arg(&watch_args.product_profile)
                    .output()
                    .expect("select profile process failed to execute");
            }
            Event::Disconnect(d) => {
                println!("Disconnected device: {:?}", d);
                karabiner
                    .arg("--select-profile")
                    .arg(&watch_args.default_profile)
                    .output()
                    .expect("select profile process failed to execute");
            }
        }
    }
}

fn main() {
    let cli = Args::parse();
    match cli.action {
        Action::Watch(watch_args) => change_karabiner_profile(watch_args),
        Action::Check {} => check_keyboard_id(),
    }
}
