# ke-auto-profile-switcher

## What's this?

This CLI automatically switches [Karabiner-Elements](https://karabiner-elements.pqrs.org/ "Karabiner-Elements") profiles based on keyboard connection status. It supports both **USB** and **Bluetooth** keyboards, and can monitor multiple keyboards simultaneously with different profile mappings.

## Features

- **Automatic Profile Switching**: Seamlessly switches between Karabiner profiles based on keyboard connection
- **USB & Bluetooth Support**: Monitor both USB and Bluetooth keyboards
- **Multiple Keyboard Support**: Configure different profiles for different keyboards
- **Multiple Configuration Methods**: Support for configuration files, command-line arguments, or interactive setup
- **Interactive Setup**: First-time users are guided through an easy setup process
- **Device Detection**: Lists available USB and Bluetooth devices to help identify your keyboards

## Install
```bash
$ cargo install ke_auto_profile_switcher
```

## Quick Start

### Method 1: Interactive Setup (Recommended for first-time users)
```bash
# Simply run watch command - you'll be guided through setup if no config exists
$ kaps watch
```

### Method 2: Using Command-Line Arguments (Legacy - USB only)
```bash
# Check your USB keyboard's product ID first
$ kaps check

# Start watching with your keyboard's details
$ kaps watch --keyboard-id 1234 --product-profile "External Keyboard" --default-profile "Default"
```

### Method 3: Configuration File (Recommended for multiple keyboards)
Create `~/.config/ke_auto_profile_switcher/config.yml`:

#### Simple Configuration (Single USB Keyboard)
```yaml
keyboard_id: 1234
product_profile: "External Keyboard"
default_profile: "Default"
```

#### Advanced Configuration (Multiple Keyboards)
```yaml
version: 2
default_profile: "Default"
keyboards:
  - name: "Work USB Keyboard"
    device:
      type: usb
      product_id: 1234
    profile: "Work Profile"
  - name: "Magic Keyboard"
    device:
      type: bluetooth
      device_name: "Magic Keyboard"
    profile: "Bluetooth Profile"
  - name: "Home Mechanical"
    device:
      type: usb
      product_id: 5678
    profile: "Gaming Profile"
```

Then simply run:
```bash
$ kaps watch
```

## Usage

### Commands

#### `kaps check`
Lists all connected devices. Use this to identify your keyboards.

```bash
# List all devices (USB and Bluetooth)
$ kaps check

# List only USB devices
$ kaps check -t usb

# List only Bluetooth devices
$ kaps check -t bluetooth
```

#### `kaps watch`
Starts monitoring device connections and automatically switches Karabiner profiles.

```bash
# Interactive mode (will prompt for configuration if needed)
$ kaps watch

# With command-line arguments (legacy USB-only mode)
$ kaps watch --keyboard-id <PRODUCT_ID> --product-profile <EXTERNAL_PROFILE>

# With all options
$ kaps watch --keyboard-id <PRODUCT_ID> --product-profile <EXTERNAL_PROFILE> --default-profile <DEFAULT_PROFILE>
```

### Options

- `--keyboard-id` (`-k`): USB product ID of your external keyboard (legacy option)
- `--product-profile` (`-p`): Karabiner profile name to use when external keyboard is connected (legacy option)
- `--default-profile` (`-d`): Karabiner profile name to use when external keyboard is disconnected (defaults to "Default")

### Configuration Priority

The application uses configuration in the following priority order:
1. **Configuration file** (if exists): `~/.config/ke_auto_profile_switcher/config.yml`
2. **Command-line arguments** (if keyboard-id and product-profile are provided)
3. **Interactive setup** (if neither config file nor complete arguments are provided)

## Configuration File Format

### Version 2 (New - Recommended)

The new configuration format supports multiple keyboards and both USB and Bluetooth devices:

```yaml
version: 2
default_profile: "Default"  # Profile when no keyboards are connected
keyboards:
  - name: "My USB Keyboard"      # Human-readable name
    device:
      type: usb                  # Device type: "usb" or "bluetooth"
      product_id: 1234           # USB product ID (for USB devices)
    profile: "USB Profile"       # Profile to switch to when connected
  - name: "My Bluetooth Keyboard"
    device:
      type: bluetooth
      device_name: "Magic Keyboard"  # Bluetooth device name
    profile: "Bluetooth Profile"
```

### Version 1 (Legacy - Still Supported)

The legacy format is still supported for backward compatibility:

```yaml
keyboard_id: 1234
product_profile: "External Keyboard"
default_profile: "Default"
```

## Example Workflow

1. **Find your keyboard's device info:**
   ```bash
   $ kaps check
   === USB Devices ===
     Product ID: 1452, Name: Apple Internal Keyboard / Trackpad
     Product ID: 1234, Name: My External Keyboard
   
   === Bluetooth Devices ===
     Name: Magic Keyboard, Status: Connected, Type: Keyboard
     Name: AirPods Pro, Status: Connected, Type: Other
   ```

2. **Start monitoring (interactive setup):**
   ```bash
   $ kaps watch
   Configuration file not found. Let's create one!
   
   === Available USB devices ===
     Product ID: 1452, Name: Apple Internal Keyboard / Trackpad
     Product ID: 1234, Name: My External Keyboard
   
   === Available Bluetooth devices ===
     Name: Magic Keyboard, Status: Connected, Type: Keyboard
   
   Enter the default Karabiner-Elements profile name [Default]: 
   
   Device type:
     1. USB keyboard
     2. Bluetooth keyboard
   Select (1 or 2): 2
   Enter the Bluetooth device name: Magic Keyboard
   Enter a name for this keyboard (e.g., 'Work Keyboard'): Magic Keyboard
   Enter the Karabiner-Elements profile name for this keyboard: Bluetooth Profile
   
   Add another keyboard? (y/N): n
   
   Configuration saved successfully!
   ```

3. **Connect/disconnect your keyboards and watch the automatic profile switching!**

## How It Works

- **USB Monitoring**: Uses the `usb_enumeration` crate for real-time USB device monitoring
- **Bluetooth Monitoring**: Polls macOS `system_profiler` to detect Bluetooth device connections
- **Combined Monitoring**: Both USB and Bluetooth devices are monitored simultaneously
- **Profile Priority**: When multiple keyboards are connected, the first matching keyboard's profile is used

## Requirements

- macOS with [Karabiner-Elements](https://karabiner-elements.pqrs.org/) installed
- The application assumes Karabiner-Elements is installed in the default location

## LICENSE

```
MIT License

Copyright (c) 2022 HIRAKI Satoru

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```
