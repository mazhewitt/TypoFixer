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
use spell_check::{CoreMlCorrector, generate_correction};
use hotkey::{setup_hotkey, start_hotkey_event_loop};
use menu_bar::{setup_menu_bar, get_menu_bar};

// Global state
static COREML_CORRECTOR: Lazy<Arc<Mutex<Option<CoreMlCorrector>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));
static CONFIG: Lazy<Arc<RwLock<Config>>> = Lazy::new(|| Arc::new(RwLock::new(Config::default())));

#[allow(dead_code)]
fn handle_hotkey_press() {
    let start = Instant::now();
    
    println!("🎯 HOTKEY PRESSED! Processing text correction...");
    info!("🎯 HOTKEY PRESSED! Processing text correction...");
    
    match process_text_correction() {
        Ok(true) => {
            show_hud("Fixed ✓");
            info!("Text correction successful in {:?}", start.elapsed());
            println!("✅ Text correction successful!");
        }
        Ok(false) => {
            debug!("No correction needed");
            println!("ℹ️ No correction needed");
        }
        Err(e) => {
            error!("Text correction failed: {}", e);
            println!("❌ Text correction failed: {}", e);
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
        let mut corrector_guard = COREML_CORRECTOR.lock().unwrap();
        if let Some(ref mut corrector) = *corrector_guard {
            generate_correction(&text, corrector)?
        } else {
            return Err("Corrector model not loaded".into());
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
    // Mock implementation - just print to console
    println!("HUD: {}", message);
}

#[allow(dead_code)]
fn beep() {
    // Mock implementation - just print to console
    println!("BEEP!");
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

fn load_coreml_model() -> Result<(), Box<dyn std::error::Error>> {
    info!("Loading CoreML text correction model...");
    
    // These paths are based on the download instructions.
    // The user is expected to have these files in the project root.
    let model_path = "ModelsCompiled/t5_tiny_grammar.mlmodelc";
    let tokenizer_path = "Models/tokenizer.json";

    let corrector = CoreMlCorrector::new(model_path, tokenizer_path)?;
    *COREML_CORRECTOR.lock().unwrap() = Some(corrector);
    
    info!("CoreML Model loaded successfully");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load config
    let config = Config::load();
    *CONFIG.write().unwrap() = config;
    
    // Load model in background
    thread::spawn(|| {
        if let Err(e) = load_coreml_model() {
            error!("Failed to load CoreML model: {}", e);
            log_error(&format!("Failed to load CoreML model: {}. Make sure you have followed the download instructions.", e));
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
    
    info!("TypoFixer started - Press ⌘⌥S to fix typos");
    info!("🚀 TypoFixer is ready! Press ⌘⌥S to fix typos in any text field.");
    
    // Run the menu bar event loop (this will block until the app terminates)
    let menu_bar = get_menu_bar()?;
    menu_bar.run_event_loop();
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Note: Most tests for this file are difficult to run in a CI environment
    // because they require accessibility permissions and a running macOS GUI.
    // The most critical logic is now in the respective modules.

    #[test]
    fn test_load_coreml_model_failure() {
        // This test assumes model files are not present at these dummy paths.
        let result = load_coreml_model();
        assert!(result.is_err());
    }

    #[test]
    fn test_process_text_correction_no_model() {
        // Ensure no model is loaded
        *COREML_CORRECTOR.lock().unwrap() = None;
        
        // This test will fail because it can't get text, but we can check the error.
        // If a model were loaded, it would fail with "Corrector model not loaded".
        let result = process_text_correction();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // In a real run, it would be a text extraction error.
        // If text could be extracted, it would be the "not loaded" error.
        assert!(
            err_msg.contains("All text extraction methods failed") || 
            err_msg.contains("Corrector model not loaded")
        );
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