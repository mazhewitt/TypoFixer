use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{error, info};

// Global hotkey manager using global-hotkey crate
pub static HOTKEY_MANAGER: once_cell::sync::Lazy<Arc<Mutex<Option<GlobalHotKeyManager>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

pub fn setup_hotkey() -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting up hotkey ⌘⌥S using global-hotkey");

    // Initialize the global hotkey manager
    let manager =
        GlobalHotKeyManager::new().map_err(|e| format!("Failed to create hotkey manager: {e}"))?;

    // Create the hotkey: Command + Option + S
    let hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyS);
    info!("Created hotkey: {:?} (⌘⌥S)", hotkey);

    // Register the hotkey
    match manager.register(hotkey) {
        Ok(_) => {
            info!("✅ Hotkey ⌘⌥S registered successfully!");
        }
        Err(e) => {
            error!("❌ Failed to register hotkey: {}", e);
            return Err(format!("Failed to register hotkey: {e}").into());
        }
    }

    // Store the manager in global state
    *HOTKEY_MANAGER.lock().unwrap() = Some(manager);

    info!("Hotkey manager stored in global state");
    Ok(())
}

pub fn start_hotkey_event_loop<F>(callback: F)
where
    F: Fn() + Send + 'static,
{
    info!("Starting hotkey event loop thread...");

    // Start the hotkey event handler thread
    thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        info!("Hotkey event loop thread started, listening for events...");

        loop {
            match receiver.try_recv() {
                Ok(event) => {
                    info!("📡 Received hotkey event: {:?}", event);
                    if event.state == HotKeyState::Pressed {
                        info!("🔥 Hotkey ⌘⌥S pressed!");
                        println!("🔥 Hotkey ⌘⌥S pressed!"); // Also print to stdout
                        callback();
                    }
                }
                Err(_) => {
                    // No events, sleep briefly
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
    });
}

pub fn start_hotkey_event_loop_with_init<I, F>(init: I, callback: F)
where
    I: FnOnce() + Send + 'static,
    F: Fn() + Send + 'static,
{
    info!("Starting hotkey event loop thread (with init)...");

    thread::spawn(move || {
        // Run initialization (e.g., load model) on this thread so thread_local state is shared
        init();

        let receiver = GlobalHotKeyEvent::receiver();
        info!("Hotkey event loop thread started, listening for events...");

        loop {
            match receiver.try_recv() {
                Ok(event) => {
                    info!("📡 Received hotkey event: {:?}", event);
                    if event.state == HotKeyState::Pressed {
                        info!("🔥 Hotkey ⌘⌥S pressed!");
                        println!("🔥 Hotkey ⌘⌥S pressed!");
                        callback();
                    }
                }
                Err(_) => {
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
    });
}
