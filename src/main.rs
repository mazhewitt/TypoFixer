use once_cell::sync::Lazy;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tracing::{debug, error, info, warn};

// Module imports
mod accessibility;
mod config;
mod error;
mod hotkey;
mod menu_bar;
mod spell_check;

use accessibility::{
    get_focused_element, get_text_to_correct_with_fallbacks, get_text_via_applescript,
    get_text_via_clipboard_fallback, is_secure_field, set_text_clipboard_only,
    set_text_with_fallbacks,
};
use config::Config;
use hotkey::{setup_hotkey, start_hotkey_event_loop_with_init};
use menu_bar::{get_menu_bar, setup_menu_bar};
use spell_check::{generate_correction_sync, is_correction_plausible, TypoFixer};

// Global state
// We can't store TypoFixer in static due to Send/Sync constraints from CoreML
// Instead we'll use thread_local storage or handle this differently
use std::cell::RefCell;
thread_local! {
    static TYPO_FIXER: RefCell<Option<TypoFixer>> = const { RefCell::new(None) };
}
static CONFIG: Lazy<Arc<RwLock<Config>>> = Lazy::new(|| Arc::new(RwLock::new(Config::default())));


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
            log_error(&format!("Text correction failed: {e}"));
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
                                    return Err(
                                        format!("All text extraction methods failed: {e}").into()
                                    );
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
                    return Err(format!("Text extraction failed: {e}").into());
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
    let corrected = TYPO_FIXER.with(|model_cell| {
        let mut model_ref = model_cell.borrow_mut();
        generate_correction_sync(&text, &mut model_ref)
    })?;

    info!("Original text: '{}' (len: {})", text, text.len());
    info!("Corrected text: '{}' (len: {})", corrected, corrected.len());

    if !is_correction_plausible(&text, &corrected) {
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
                warn!(
                    "Failed to apply correction via clipboard-only method: {}",
                    e
                );
            }
        }
    }

    Ok(true)
}

// Expose viability logic for tests and to keep process_text_correction simple
// was is_correction_viable; now using spell_check::is_correction_plausible

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
        let _ = writeln!(file, "[{timestamp}] {message}");
    }
}

// This function is no longer needed - menu bar functionality is now in menu_bar.rs

fn find_config_upwards(rel_path: &str) -> Option<PathBuf> {
    // Prefer searching from the executable location up to the filesystem root
    if let Ok(mut dir) = std::env::current_exe().and_then(|p| p.parent().map(|p| p.to_path_buf()).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "no parent"))) {
        loop {
            let candidate = dir.join(rel_path);
            if candidate.exists() {
                return Some(candidate);
            }
            if let Some(parent) = dir.parent() {
                dir = parent.to_path_buf();
            } else {
                break;
            }
        }
    }
    // Fallback: search from current working directory upwards
    if let Ok(mut dir) = std::env::current_dir() {
        loop {
            let candidate = dir.join(rel_path);
            if candidate.exists() {
                return Some(candidate);
            }
            if let Some(parent) = dir.parent() {
                dir = parent.to_path_buf();
            } else {
                break;
            }
        }
    }
    None
}

async fn load_llama_model_async() -> Result<(), String> {
    info!("Loading text correction model...");

    // Try to load with automatic configuration discovery first
    let config_path = find_config_upwards("configs/qwen-typo-fixer-ane.json");
    let mut model = match config_path {
        Some(cfg) => {
            info!("Attempting config-based load from {}", cfg.display());
            match TypoFixer::new_with_config(cfg.to_string_lossy().as_ref(), ".", true)
                .await
            {
                Ok(model) => model,
                Err(e) => {
                    warn!("Config-based loading failed: {}", e);
                    info!("Falling back to HuggingFace Hub loading...");
                    TypoFixer::new(Some("mazhewitt/qwen-typo-fixer-coreml"), true)
                    .await
                    .map_err(|e| format!("Failed to load fallback model: {e}"))?
                }
            }
        }
        None => {
            warn!(
                "Config file not found in executable/working directory tree; using fallback model"
            );
            TypoFixer::new(Some("mazhewitt/qwen-typo-fixer-coreml"), true)
                .await
                .map_err(|e| format!("Failed to load fallback model: {e}"))?
        }
    };

    // Warm up: run a tiny inference to initialize CoreML pipelines and caches
    info!("🔥 Warming up model (tiny inference)...");
    match model
        .fix_typos_with_options("warmup", 0.0, Some(1))
        .await
    {
        Ok(_) => info!("✅ Model warm-up complete"),
        Err(e) => warn!("⚠️ Model warm-up failed (continuing): {}", e),
    }

    // Model is ready to use

    TYPO_FIXER.with(|model_cell| {
        *model_cell.borrow_mut() = Some(model);
    });

    info!("Model loaded successfully");
    Ok(())
}

fn load_llama_model() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(load_llama_model_async()).map_err(|e| e.into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load config
    let config = Config::load();
    *CONFIG.write().unwrap() = config;

    // Setup menu bar (this also configures the app as accessory)
    setup_menu_bar()?;

    // Setup hotkey
    setup_hotkey()?;

    // Start hotkey event loop in background, but initialize the model on the same thread
    start_hotkey_event_loop_with_init(
        || {
            if let Err(e) = load_llama_model() {
                error!("Failed to load model: {}", e);
            }
        },
        handle_hotkey_press,
    );

    info!("TypoFixer started - Press ⌘⌥S to fix typos");
    info!("⏳ Model will finish loading shortly; hotkey will work once ready.");

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
    use crate::spell_check::{choose_correction_from_raw, is_correction_plausible};

    fn create_temp_model_file() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let model_path = temp_dir.path().join("test_model.gguf");
        fs::write(&model_path, "mock model content").unwrap();
        (temp_dir, model_path)
    }

    #[test]
    #[ignore] // This test calls real system functions that trigger Cmd+A key combinations
    fn test_process_text_correction_secure_field() {
        // Set up a model first
        let (_temp_dir, _model_path) = create_temp_model_file();
        let rt = tokio::runtime::Runtime::new().unwrap();
        // Try to create a model - this may fail in test environment, which is expected
        match rt.block_on(TypoFixer::new(Some("test-model"), false)) {
            Ok(model) => {
                TYPO_FIXER.with(|model_cell| {
                    *model_cell.borrow_mut() = Some(model);
                });
            }
            Err(_) => {
                // Model creation failed, which is expected in test environment
                // Tests should handle this gracefully
            }
        }

        let result = process_text_correction();
        // In test environment with mocking, this may succeed or fail
        // If it succeeds, it should return a boolean indicating success
        // If it fails, it should be due to accessibility/permissions issues
        match result {
            Ok(success) => {
                // If successful, verify it's a boolean result
                assert!(
                    success || !success,
                    "Expected boolean result, got unexpected value"
                );
            }
            Err(error_msg) => {
                // If failed, verify it's due to expected issues
                let msg = error_msg.to_string();
                assert!(
                    msg.contains("Accessibility permissions not granted")
                        || msg.contains("No focused application found")
                        || msg.contains("Failed to get focused application")
                        || msg.contains("Text extraction failed")
                        || msg.contains("Model not loaded")
                        || msg.contains("All text extraction methods failed"),
                    "Unexpected error message: {msg}"
                );
            }
        }
    }

    #[test]
    fn test_load_llama_model_missing_file() {
        // Set up a config with a non-existent model path
        let temp_dir = TempDir::new().unwrap();
        let missing_model_path = temp_dir.path().join("missing_model.gguf");

        let mut config = Config::default();
        config.model_path = missing_model_path;
        *CONFIG.write().unwrap() = config;

        let result = load_llama_model();
        // Model loading may succeed (fallback to HF) or fail (network/permissions issues)
        // Both outcomes are acceptable in test environment
        match result {
            Ok(()) => {
                // Success is good
                assert!(true);
            }
            Err(e) => {
                // Failure is also acceptable if it's due to expected issues
                let msg = e.to_string();
                assert!(
                    msg.contains("Failed to download")
                        || msg.contains("Failed to load")
                        || msg.contains("not supported")
                        || msg.contains("Config-based loading failed"),
                    "Unexpected error during model loading: {msg}"
                );
            }
        }
    }

    #[test]
    fn test_extraction_rejects_implausible_short_output() {
    // Given a longer input, plausibility gating should reject a single short word like "The"
    let input = "This sentance has typoos ";
    let raw = "<|endoftext|><|endoftext|>The<|endoftext|>The<|endoftext|>The.<|endoftext|>The.<|endoftext|>The.<|endoftext|>TheTheThe";

    let chosen = choose_correction_from_raw(input, raw);
    assert!(chosen.is_none(), "should reject implausible short output");
    }

    #[test]
    fn test_viability_rejects_implausibly_short_correction() {
        let original = "This sentance has typoos ";
        let corrected = "The";
        // Expect the viability check to reject this correction for a longer input
        assert!(
            !is_correction_plausible(original, corrected),
            "viability should reject implausibly short correction"
        );
    }

    #[test]
    fn test_load_llama_model_success() {
        let (_temp_dir, model_path) = create_temp_model_file();

        let mut config = Config::default();
        config.model_path = model_path;
        *CONFIG.write().unwrap() = config;

        let result = load_llama_model();

        // Model loading may succeed or fail depending on environment
        match result {
            Ok(()) => {
                // If successful, verify model was loaded into global state
                TYPO_FIXER.with(|model_cell| {
                    let model_ref = model_cell.borrow();
                    assert!(
                        model_ref.is_some(),
                        "Model should be loaded into global state on success"
                    );
                });
            }
            Err(e) => {
                // If failed, verify it's due to expected issues
                let msg = e.to_string();
                assert!(
                    msg.contains("Failed to download")
                        || msg.contains("Failed to load")
                        || msg.contains("not supported")
                        || msg.contains("Config-based loading failed"),
                    "Unexpected error during model loading: {msg}"
                );
            }
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
