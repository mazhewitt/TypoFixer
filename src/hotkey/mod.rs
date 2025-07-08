use global_hotkey::{GlobalHotKeyManager, HotKeyState, GlobalHotKeyEvent};
use global_hotkey::hotkey::{HotKey, Modifiers, Code};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{info, error};

pub struct HotkeyManager {
    manager: Option<GlobalHotKeyManager>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self { manager: None }
    }

    pub fn setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Setting up hotkey ⌘⌥S using global-hotkey");
        
        // Initialize the global hotkey manager
        let manager = GlobalHotKeyManager::new().map_err(|e| format!("Failed to create hotkey manager: {}", e))?;
        
        // Create the hotkey: Command + Option + S
        let hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyS);
        
        // Register the hotkey
        manager.register(hotkey).map_err(|e| format!("Failed to register hotkey: {}", e))?;
        
        // Store the manager
        self.manager = Some(manager);
        
        info!("✅ Hotkey ⌘⌥S registered successfully!");
        Ok(())
    }

    pub fn start_event_loop<F>(&self, callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn() + Send + 'static,
    {
        if self.manager.is_none() {
            return Err("Hotkey manager not initialized".into());
        }

        // Start the hotkey event handler thread
        thread::spawn(move || {
            let receiver = GlobalHotKeyEvent::receiver();
            loop {
                match receiver.try_recv() {
                    Ok(event) => {
                        if event.state == HotKeyState::Pressed {
                            info!("🔥 Hotkey ⌘⌥S pressed!");
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

        Ok(())
    }
}

// Global hotkey manager using global-hotkey crate
pub static HOTKEY_MANAGER: once_cell::sync::Lazy<Arc<Mutex<Option<GlobalHotKeyManager>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

pub fn setup_hotkey() -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting up hotkey ⌘⌥S using global-hotkey");
    
    // Initialize the global hotkey manager
    let manager = GlobalHotKeyManager::new().map_err(|e| format!("Failed to create hotkey manager: {}", e))?;
    
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
            return Err(format!("Failed to register hotkey: {}", e).into());
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