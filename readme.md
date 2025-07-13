# TMWT - Total Management for Windows Terminal

A powerful Terminal User Interface (TUI) application for managing Windows system settings directly from the command line. TMWT allows you to view and modify Windows settings without leaving your terminal, perfect for embedded systems, remote administration, or users who prefer keyboard-driven interfaces.

## Features

### ✨ Core Features
- **Native Terminal UI**: Full keyboard navigation with an intuitive interface
- **Direct Settings Editing**: Modify system settings without opening Windows Settings app
- **Real-time Search**: Quickly find settings with fuzzy search
- **Category Organization**: Settings grouped by system categories
- **Admin Privilege Handling**: Clear indicators and elevation when needed

### 🔧 Editable Settings
TMWT can directly modify the following settings in-terminal:

#### System & Display
- **Display Resolution**: Change screen resolution and refresh rate
- **Power Plans**: Switch between power plans (Balanced, High Performance, Power Saver)
- **Audio Devices**: Select default audio output device

#### Network & Internet  
- **Network Adapters**: Enable/disable Wi-Fi and Ethernet adapters
- **DNS Configuration**: Set DNS servers (Automatic, Google, Cloudflare, etc.)
- **Wi-Fi Power Management**: Adjust wireless adapter power saving modes

More settings are being added continuously!

### 📋 Additional Features
- Settings marked with ✏ can be edited inline
- Settings marked with [Admin] require administrator privileges
- Fallback to Windows Settings app for non-editable items
- Status messages and error handling
- Responsive layout that adapts to terminal size

## Installation

### Prerequisites
- Windows 10/11
- Rust toolchain (for building from source)
- Administrator privileges (for certain settings)

### Building from Source
```bash
# Clone the repository
git clone https://github.com/yourusername/tmwt.git
cd tmwt

# Build with the included script
.\build.bat

# Or build manually
cargo build --release
```

The executable will be created at `target/release/TMWT.exe`

## Usage

### Basic Navigation
```
tmwt.exe
```

#### Keyboard Shortcuts
- **Arrow Keys**: Navigate between categories and items
- **Tab**: Switch focus between categories and items panels
- **Enter**: Open setting (edit inline if available, otherwise launch Windows Settings)
- **e**: Quick edit for editable settings
- **/**: Enter search mode
- **q**: Quit application

### Edit Mode Controls
When editing a setting:

#### Toggle Settings
- **Space/Enter**: Toggle between enabled/disabled
- **S**: Save changes
- **Esc**: Cancel without saving

#### Dropdown Settings
- **↑/↓**: Navigate options
- **Enter**: Select option
- **S**: Save selection
- **Esc**: Cancel

#### Resolution Picker
- **↑/↓**: Navigate resolutions
- **Enter**: Select resolution
- **S**: Apply changes
- **Esc**: Cancel

## Examples

### Change Display Resolution
1. Navigate to "System & Display" category
2. Select "Display Resolution" (marked with ✏)
3. Press Enter or 'e' to edit
4. Use arrow keys to select desired resolution
5. Press 'S' to save

### Configure DNS Servers
1. Navigate to "Network & Internet"
2. Select "Wi-Fi DNS Settings"
3. Press Enter to edit (requires admin)
4. Choose from preset DNS providers
5. Press 'S' to apply

### Quick Search
1. Press '/' from anywhere
2. Type part of the setting name
3. Press Enter to exit search
4. Navigate to filtered results

## Architecture

TMWT uses a modular architecture that makes it easy to add new settings:

- **settings.rs**: Defines all available settings and categories
- **settings_editor.rs**: Core trait system for setting editors
- **edit_ui.rs**: UI components for the edit interface
- **launcher.rs**: Fallback system for launching Windows Settings

### Adding New Settings

To add a new editable setting:

1. Create an editor implementing the `SettingEditor` trait
2. Add the editor to the factory function in `settings_editor.rs`
3. Update the setting definition in `settings.rs` with `.with_editor("key")`

## Security Considerations

- TMWT requires administrator privileges for system-level changes
- All changes are validated before applying
- Original Windows security model is preserved
- No settings are cached or stored by TMWT

## Limitations

- Some settings require Windows Settings app (no API available)
- Changes may require restart to take full effect
- Network changes may briefly disconnect active connections
- Some advanced settings not yet implemented