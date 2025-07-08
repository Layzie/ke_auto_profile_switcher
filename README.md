# ke-auto-profile-switcher

## What's this?

This CLI automatically switches [Karabiner-Elements](https://karabiner-elements.pqrs.org/ "Karabiner-Elements") profiles based on USB keyboard connection status. When an external USB keyboard is connected, it switches to your designated profile for external keyboards. When disconnected, it switches back to your default profile.

## Features

- **Automatic Profile Switching**: Seamlessly switches between Karabiner profiles based on USB keyboard connection
- **Multiple Configuration Methods**: Support for configuration files, command-line arguments, or interactive setup
- **Interactive Setup**: First-time users are guided through an easy setup process
- **USB Device Detection**: Lists available USB devices to help identify your keyboard's product ID

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

### Method 2: Using Command-Line Arguments
```bash
# Check your USB keyboard's product ID first
$ kaps check

# Start watching with your keyboard's details
$ kaps watch --keyboard-id 1234 --product-profile "External Keyboard" --default-profile "Default"
```

### Method 3: Configuration File
Create `~/.config/ke_auto_profile_switcher/config.yml`:
```yaml
keyboard_id: 1234
product_profile: "External Keyboard"
default_profile: "Default"
```

Then simply run:
```bash
$ kaps watch
```

## Usage

### Commands

#### `kaps check`
Lists all connected USB devices with their product IDs. Use this to identify your external keyboard's product ID.

```bash
$ kaps check
```

#### `kaps watch`
Starts monitoring USB device connections and automatically switches Karabiner profiles.

```bash
# Interactive mode (will prompt for configuration if needed)
$ kaps watch

# With command-line arguments
$ kaps watch --keyboard-id <PRODUCT_ID> --product-profile <EXTERNAL_PROFILE>

# With all options
$ kaps watch --keyboard-id <PRODUCT_ID> --product-profile <EXTERNAL_PROFILE> --default-profile <DEFAULT_PROFILE>
```

### Options

- `--keyboard-id` (`-k`): USB product ID of your external keyboard
- `--product-profile` (`-p`): Karabiner profile name to use when external keyboard is connected
- `--default-profile` (`-d`): Karabiner profile name to use when external keyboard is disconnected (defaults to "Default")

### Configuration Priority

The application uses configuration in the following priority order:
1. **Configuration file** (if exists): `~/.config/ke_auto_profile_switcher/config.yml`
2. **Command-line arguments** (if keyboard-id and product-profile are provided)
3. **Interactive setup** (if neither config file nor complete arguments are provided)

## Example Workflow

1. **Find your keyboard's product ID:**
   ```bash
   $ kaps check
   Available USB devices:
     ID: 1452, Product: Apple Internal Keyboard / Trackpad
     ID: 1234, Product: My External Keyboard
   ```

2. **Start monitoring (interactive setup):**
   ```bash
   $ kaps watch
   Configuration file not found. Let's create one!
   
   Available USB devices:
     ID: 1452, Product: Apple Internal Keyboard / Trackpad
     ID: 1234, Product: My External Keyboard
   
   Enter the USB keyboard product ID: 1234
   Enter the Karabiner-Elements profile name for external keyboard: External Keyboard
   Enter the default Karabiner-Elements profile name [Default]: 
   
   Configuration saved successfully!
   ```

3. **Connect/disconnect your external keyboard and watch the automatic profile switching!**

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
