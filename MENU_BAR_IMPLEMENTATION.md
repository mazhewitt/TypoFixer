# macOS Menu Bar Implementation

This document describes the modern, thread-safe menu bar implementation for the TypoFixer application.

## Overview

The menu bar implementation provides:
- A macOS status bar icon (⌨️) that appears in the top-right menu bar
- A dropdown menu with "About TypoFixer" and "Quit TypoFixer" options
- Proper quit functionality that terminates the application
- Thread-safe storage for the status item
- Background application mode (NSApplicationActivationPolicyAccessory)

## Key Features

### Thread Safety
- Uses `Arc<Mutex<Option<id>>>` for thread-safe status item storage
- Implements `Send` and `Sync` traits for the MenuBar struct
- Uses `std::sync::Once` for one-time initialization

### Modern Architecture
- Separate `menu_bar.rs` module for clean code organization
- Uses the stable `cocoa` crate instead of the newer but less stable `objc2`
- Proper memory management with NSAutoreleasePool

### Menu Structure
1. **About TypoFixer** - Shows application information
2. **Separator** - Visual divider
3. **Quit TypoFixer** - Terminates the application (⌘Q shortcut)

## Usage

### Setup
```rust
use menu_bar::{setup_menu_bar, get_menu_bar};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the menu bar
    setup_menu_bar()?;
    
    // Get reference to menu bar and run event loop
    let menu_bar = get_menu_bar()?;
    menu_bar.run_event_loop(); // This blocks until app terminates
}
```

### API Functions

#### `setup_menu_bar()`
Initializes the menu bar with the status item and menu. Must be called on the main thread.

#### `get_menu_bar()`
Returns a reference to the initialized menu bar instance.

#### `MenuBar::run_event_loop()`
Starts the NSApplication event loop. This function never returns under normal circumstances.

## Implementation Details

### Status Item Creation
```rust
let status_bar = NSStatusBar::systemStatusBar(nil);
let status_item = status_bar.statusItemWithLength_(NSVariableStatusItemLength);
status_item.setTitle_(NSString::alloc(nil).init_str("⌨️"));
```

### Menu Item Actions
- **About**: Calls `aboutAction:` function (currently shows console output)
- **Quit**: Calls `terminate:` on the NSApplication instance

### Background Application Mode
The application runs as an accessory app, meaning:
- It doesn't appear in the Dock
- It doesn't have a main window
- It runs in the background
- It appears only in the menu bar

## Error Handling

The implementation includes proper error handling for:
- Menu bar initialization failures
- Thread safety violations
- Memory management issues

## Testing

The implementation includes basic tests that verify:
- Menu bar creation doesn't crash
- Types compile correctly
- Thread safety mechanisms work

## Future Enhancements

Potential improvements:
1. Add more menu items (preferences, help, etc.)
2. Implement proper About dialog instead of console output
3. Add menu item icons
4. Add keyboard shortcuts for menu items
5. Implement menu item enabling/disabling based on application state

## Dependencies

- `cocoa = "0.26.1"` - macOS Cocoa framework bindings
- `objc = "0.2.7"` - Objective-C runtime interface
- `tracing` - Logging and debugging

## Platform Support

This implementation is macOS-specific and requires:
- macOS 10.9 or later
- Xcode Command Line Tools
- Rust with macOS target support

## Thread Safety Notes

The implementation uses unsafe code for Objective-C interop, but wraps it in safe Rust abstractions:
- Global static with `std::sync::Once` for initialization
- Arc<Mutex<>> for shared state
- Proper Send/Sync implementations

The menu bar must be initialized on the main thread, but can be accessed from other threads safely through the provided API.