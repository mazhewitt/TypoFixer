#!/usr/bin/env rust-script

//! Clean Core ML model cache
//! 
//! This script removes compiled Core ML models from the build cache.
//! It's automatically called by `cargo clean` when the cache directory
//! is in the target directory.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let target_dir = env::var("CARGO_TARGET_DIR")
        .or_else(|_| env::var("OUT_DIR").map(|out| {
            // Extract target dir from OUT_DIR path
            PathBuf::from(out)
                .ancestors()
                .find(|p| p.file_name().map_or(false, |name| name == "target"))
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "target".to_string())
        }))
        .unwrap_or_else(|_| "target".to_string());
    
    let cache_patterns = vec![
        PathBuf::from(&target_dir).join("**/coreml_models"),
        PathBuf::from(&target_dir).join("**/compile_model.swift"),
    ];
    
    println!("🧹 Cleaning Core ML cache...");
    
    let mut cleaned_count = 0;
    
    // Clean compiled models and build artifacts
    for pattern in cache_patterns {
        if let Some(parent) = pattern.parent() {
            if parent.exists() {
                if let Ok(entries) = fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let file_name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");
                        
                        let should_clean = file_name == "coreml_models" || 
                                         file_name == "compile_model.swift" ||
                                         file_name.ends_with(".mlmodelc");
                        
                        if should_clean {
                            if path.is_dir() {
                                if let Ok(()) = fs::remove_dir_all(&path) {
                                    println!("  🗑️  Removed directory: {}", path.display());
                                    cleaned_count += 1;
                                }
                            } else if let Ok(()) = fs::remove_file(&path) {
                                println!("  🗑️  Removed file: {}", path.display());
                                cleaned_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    
    if cleaned_count > 0 {
        println!("✅ Cleaned {} Core ML cache entries", cleaned_count);
    } else {
        println!("✨ Core ML cache already clean");
    }
}