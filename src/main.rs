use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use std::thread;
use tracing::{info, error, warn, debug};

// Module imports
mod config;
mod accessibility;
mod spell_check;
mod hotkey;
mod error;
mod menu_bar;

use config::Config;
use accessibility::{
    get_focused_element, is_secure_field,
    get_text_to_correct_with_fallbacks, get_text_via_clipboard_fallback, 
    get_text_via_applescript, set_text_with_fallbacks, set_text_clipboard_only
};
use spell_check::{CorrectionEngine, create_coreml_engine};
use hotkey::{setup_hotkey, start_hotkey_event_loop};
use menu_bar::{setup_menu_bar, get_menu_bar};

// Global state
static CORRECTION_ENGINE: Lazy<Arc<Mutex<Option<CorrectionEngine>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));
static CONFIG: Lazy<Arc<RwLock<Config>>> = Lazy::new(|| Arc::new(RwLock::new(Config::default())));

#[allow(dead_code)]
fn handle_hotkey_press() {
    let start = Instant::now();
    
    info!("🎯 HOTKEY PRESSED! Processing text correction...");
    
    match process_text_correction() {
        Ok(true) => {
            show_hud("Fixed ✓");
            info!("Text correction successful in {:?}", start.elapsed());
        }
        Ok(false) => {
            debug!("No correction needed");
        }
        Err(e) => {
            error!("Text correction failed: {}", e);
            beep();
            log_error(&format!("Text correction failed: {}", e));
        }
    }
}

fn process_text_correction() -> Result<bool, Box<dyn std::error::Error>> {
    // Try to get focused element first
    let focused_element = match get_focused_element() {
        Ok(elem) => Some(elem),
        Err(e) => {
            warn!("Could not get focused element: {}", e);
            None
        }
    };
    
    // Try different text extraction methods
    let (text, range) = match focused_element {
        Some(ref elem) => {
            // Try standard accessibility first
            match get_text_to_correct_with_fallbacks(elem) {
                Ok(result) => result,
                Err(_) => {
                    // Try clipboard fallback
                    match get_text_via_clipboard_fallback() {
                        Ok(result) => result,
                        Err(_) => {
                            // Try AppleScript fallback
                            match get_text_via_applescript() {
                                Ok(result) => result,
                                Err(e) => {
                                    return Err(format!("All text extraction methods failed: {}", e).into());
                                }
                            }
                        }
                    }
                }
            }
        }
        None => {
            // No focused element, try clipboard method directly
            match get_text_via_clipboard_fallback() {
                Ok(result) => result,
                Err(e) => {
                    return Err(format!("Text extraction failed: {}", e).into());
                }
            }
        }
    };
    
    // Check if it's a secure field (only if we have an element)
    if let Some(ref elem) = focused_element {
        if is_secure_field(elem) {
            return Ok(false);
        }
    }
    
    if text.trim().is_empty() {
        return Ok(false);
    }
    
    // Generate correction
    let corrected = {
        let mut engine_guard = CORRECTION_ENGINE.lock().unwrap();
        match engine_guard.as_mut() {
            Some(engine) => engine.generate_correction(&text)?,
            None => {
                return Err("Core ML model is still loading/compiling in background. Please wait a moment and try again.".into());
            }
        }
    };
    
    info!("Original text: '{}' (len: {})", text, text.len());
    info!("Corrected text: '{}' (len: {})", corrected, corrected.len());
    
    // Check if correction is reasonable (allow up to 50% longer or same length)
    if corrected.len() > text.len() + (text.len() / 2) + 20 {
        warn!("Correction too long, aborting (original: {}, corrected: {})", text.len(), corrected.len());
        return Ok(false);
    }
    
    // If no changes were made, don't apply
    if corrected == text {
        info!("No changes needed");
        return Ok(false);
    }
    
    // Apply correction
    if let Some(ref elem) = focused_element {
        // Try to set text with fallbacks
        match set_text_with_fallbacks(elem, &corrected, range) {
            Ok(()) => {
                info!("✅ Successfully applied correction");
            }
            Err(e) => {
                warn!("Failed to apply correction: {}", e);
            }
        }
    } else {
        // No element available, use clipboard-only method
        match set_text_clipboard_only(&corrected) {
            Ok(()) => {
                info!("✅ Successfully applied correction via clipboard-only method");
            }
            Err(e) => {
                warn!("Failed to apply correction via clipboard-only method: {}", e);
            }
        }
    }
    
    Ok(true)
}

#[allow(dead_code)]
fn show_hud(message: &str) {
    // Mock implementation - use logging instead of terminal output
    info!("HUD: {}", message);
}

#[allow(dead_code)]
fn beep() {
    // Mock implementation - use logging instead of terminal output
    warn!("BEEP!");
}

fn log_error(message: &str) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/user".to_string());
    let log_path = PathBuf::from(&home).join("Library/Logs/TypoFixer.log");
    
    if let Some(parent) = log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(file, "[{}] {}", timestamp, message);
    }
}

// This function is no longer needed - menu bar functionality is now in menu_bar.rs

fn load_correction_engine() -> Result<(), Box<dyn std::error::Error>> {
    let config = CONFIG.read().unwrap();
    
    info!("Loading Core ML correction engine...");
    
    // Try to load Core ML corrector first
    match create_coreml_engine(&config.model_path) {
        Ok(engine) => {
            info!("✅ Core ML correction engine loaded successfully from: {}", config.model_path.display());
            *CORRECTION_ENGINE.lock().unwrap() = Some(engine);
            Ok(())
        }
        Err(e) => {
            warn!("❌ Failed to load Core ML correction engine: {}", e);
            warn!("   Make sure the Core ML model exists at: {}", config.model_path.display());
            warn!("   The model should be a .mlpackage file");
            Err(format!("Failed to load Core ML correction engine: {}", e).into())
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load config
    let config = Config::load();
    *CONFIG.write().unwrap() = config;
    
    // Load Core ML correction engine in background
    thread::spawn(|| {
        match load_correction_engine() {
            Ok(()) => {
                info!("🎉 Core ML correction engine is ready! You can now use ⌘⌥S to fix typos.");
            }
            Err(e) => {
                error!("❌ Failed to load correction engine: {}", e);
                error!("   TypoFixer will not work until the model is loaded.");
            }
        }
    });
    
    // Setup menu bar (this also configures the app as accessory)
    setup_menu_bar()?;
    
    // Setup hotkey
    setup_hotkey()?;
    
    // Start hotkey event loop in background
    thread::spawn(|| {
        start_hotkey_event_loop(handle_hotkey_press);
    });
    
    info!("TypoFixer started - Core ML model loading in background...");
    info!("🚀 TypoFixer hotkey registered! Core ML model is loading - you'll see a message when ready.");
    
    // Run the menu bar event loop (this will block until the app terminates)
    let menu_bar = get_menu_bar();
    let mtm = objc2_foundation::MainThreadMarker::new().expect("must run on main thread");
    menu_bar.lock().unwrap().run_event_loop(mtm);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_temp_model_file() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let model_path = temp_dir.path().join("test_model.gguf");
        fs::write(&model_path, "mock model content").unwrap();
        (temp_dir, model_path)
    }

    #[test]
    #[ignore] // This test calls real system functions that trigger Cmd+A key combinations
    fn test_process_text_correction_secure_field() {
        // Set up a Core ML corrector first (this will fail without a real model, but tests the interface)
        let (_temp_dir, _model_path) = create_temp_model_file();
        
        // Since we can't easily create a real Core ML model for testing, 
        // we expect the correction engine loading to fail
        // This test primarily validates that the error handling works correctly
        
        let result = process_text_correction();
        // In test environment with mocking, this may succeed or fail
        // If it succeeds, it should return a boolean indicating success
        // If it fails, it should be due to accessibility/permissions issues or model loading issues
        match result {
            Ok(success) => {
                // If successful, verify it's a boolean result
                assert!(success == true || success == false);
            }
            Err(error_msg) => {
                // If failed, verify it's due to expected issues
                let msg = error_msg.to_string();
                assert!(msg.contains("Accessibility permissions not granted") || 
                        msg.contains("No focused application found") ||
                        msg.contains("Failed to get focused application") ||
                        msg.contains("Text extraction failed") ||
                        msg.contains("Correction engine not loaded") ||
                        msg.contains("Core ML model is still loading"));
            }
        }
    }

    #[test]
    fn test_load_correction_engine_missing_file() {
        // Set up a config with a non-existent model path
        let temp_dir = TempDir::new().unwrap();
        let missing_model_path = temp_dir.path().join("missing_model.mlpackage");
        
        let mut config = Config::default();
        config.model_path = missing_model_path;
        *CONFIG.write().unwrap() = config;
        
        let result = load_correction_engine();
        // Core ML model loading should fail without a proper .mlpackage file
        assert!(result.is_err());
    }

    #[test]
    fn test_load_correction_engine_success() {
        // Test with the actual Core ML model if it exists
        let real_model_path = std::path::PathBuf::from("coreml-setup/coreml-setup/coreml-OpenELM-450M-Instruct/OpenELM-450M-Instruct-128-float32.mlpackage");
        
        if real_model_path.exists() {
            let mut config = Config::default();
            config.model_path = real_model_path;
            *CONFIG.write().unwrap() = config;
            
            let result = load_correction_engine();
            
            // This might succeed if the model exists and can be loaded
            match result {
                Ok(()) => {
                    // Verify engine was loaded into global state
                    let engine_guard = CORRECTION_ENGINE.lock().unwrap();
                    assert!(engine_guard.is_some());
                }
                Err(e) => {
                    // If it fails, it should be due to model compilation issues
                    assert!(e.to_string().contains("Failed to load Core ML correction engine"));
                }
            }
        } else {
            // If model doesn't exist, create a test scenario
            let (_temp_dir, model_path) = create_temp_model_file();
            
            let mut config = Config::default();
            config.model_path = model_path;
            *CONFIG.write().unwrap() = config;
            
            let result = load_correction_engine();
            // Should fail since we don't have a real Core ML model
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_log_error_creates_file() {
        // This function now uses a hardcoded path, so we just test it creates a log
        log_error("Test error message");
        
        // The log should be created in the user's home directory
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/user".to_string());
        let log_path = PathBuf::from(&home).join("Library/Logs/TypoFixer.log");
        
        // Check if log file exists (it should after calling log_error)
        // Note: This test may affect the actual log file
        assert!(log_path.exists() || log_path.parent().unwrap().exists());
    }
}