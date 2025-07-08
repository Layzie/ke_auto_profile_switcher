use clap::Parser;
use ke_auto_profile_switcher::{
    cli::{Action, Args, WatchArgs},
    config::resolve_config,
    karabiner::KarabinerController,
    usb_monitor::{list_usb_devices, UsbMonitor},
    Result,
};

fn main() -> Result<()> {
    let cli = Args::parse();

    match cli.action {
        Action::Watch {
            keyboard_id,
            product_profile,
            default_profile,
        } => {
            let config = resolve_config(keyboard_id, product_profile, default_profile)?;
            let watch_args = WatchArgs::from_config(&config);
            start_monitoring(watch_args)?;
        }
        Action::Check {} => {
            list_usb_devices();
        }
    }

    Ok(())
}

fn start_monitoring(watch_args: WatchArgs) -> Result<()> {
    let karabiner = KarabinerController::new();
    let monitor = UsbMonitor::new(watch_args.keyboard_id);

    let product_profile = watch_args.product_profile.clone();
    let default_profile = watch_args.default_profile.clone();

    let on_connect = {
        let karabiner = karabiner.clone();
        let profile = product_profile.clone();
        move || karabiner.switch_profile(&profile)
    };

    let on_disconnect = {
        let karabiner = karabiner.clone();
        let profile = default_profile.clone();
        move || karabiner.switch_profile(&profile)
    };

    monitor.start_monitoring(on_connect, on_disconnect)
}
