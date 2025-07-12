use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::fs;

fn main() {
    // Tell Cargo to rerun this build script if the model changes
    println!("cargo:rerun-if-changed=coreml-setup/");
    println!("cargo:rerun-if-changed=build.rs");
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let source_model = "coreml-models/SentimentPolarity.mlmodel";
    let compiled_model_dir = PathBuf::from(&out_dir).join("coreml_models");
    let compiled_model_path = compiled_model_dir.join("compiled_model.mlmodelc");
    
    println!("cargo:rustc-env=COREML_CACHE_DIR={}", compiled_model_dir.display());
    
    // Check if source model exists
    if !std::path::Path::new(source_model).exists() {
        println!("cargo:warning=Core ML model not found at {}, skipping compilation", source_model);
        println!("cargo:rustc-env=COMPILED_MODEL_PATH=");  // Set empty path
        return;
    }
    
    // Create output directory
    fs::create_dir_all(&compiled_model_dir).unwrap();
    
    // Check if model is already compiled and cached
    if compiled_model_path.exists() {
        // Check if source is newer than compiled model
        let source_metadata = fs::metadata(source_model).unwrap();
        let compiled_metadata = fs::metadata(&compiled_model_path).unwrap();
        
        if compiled_metadata.modified().unwrap() >= source_metadata.modified().unwrap() {
            println!("cargo:warning=Using cached Core ML model from: {}", compiled_model_path.display());
            println!("cargo:rustc-env=COMPILED_MODEL_PATH={}", compiled_model_path.display());
            return;
        } else {
            println!("cargo:warning=Source model newer than cache, recompiling...");
            // Remove old compiled model
            if compiled_model_path.exists() {
                fs::remove_dir_all(&compiled_model_path).ok();
            }
        }
    }
    
    // Use Swift to compile the Core ML model at build time
    let swift_script = format!(r#"
import Foundation
import CoreML

let sourceURL = URL(fileURLWithPath: "{}")
let outputURL = URL(fileURLWithPath: "{}")

do {{
    let compiledURL = try MLModel.compileModel(at: sourceURL)
    
    // Copy compiled model to output directory
    let fileManager = FileManager.default
    if fileManager.fileExists(atPath: outputURL.path) {{
        try fileManager.removeItem(at: outputURL)
    }}
    try fileManager.copyItem(at: compiledURL, to: outputURL)
    
    print("✅ Core ML model compiled successfully to: \(outputURL.path)")
    exit(0)
}} catch {{
    print("❌ Failed to compile Core ML model: \(error)")
    exit(1)
}}
"#, 
        fs::canonicalize(source_model).unwrap().display(),
        compiled_model_path.display()
    );
    
    // Write Swift script to temporary file  
    let script_path = PathBuf::from(&out_dir).join("compile_model.swift");
    fs::write(&script_path, swift_script).unwrap();
    
    println!("cargo:warning=Compiling Core ML model at build time...");
    
    // Execute Swift script
    let output = Command::new("swift")
        .arg(&script_path)
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("cargo:warning=✅ Core ML model compiled successfully!");
                println!("cargo:rustc-env=COMPILED_MODEL_PATH={}", compiled_model_path.display());
                println!("cargo:warning={}", String::from_utf8_lossy(&result.stdout));
            } else {
                println!("cargo:warning=❌ Failed to compile Core ML model at build time");
                println!("cargo:warning=stdout: {}", String::from_utf8_lossy(&result.stdout));
                println!("cargo:warning=stderr: {}", String::from_utf8_lossy(&result.stderr));
                println!("cargo:rustc-env=COMPILED_MODEL_PATH=");  // Set empty path on failure
            }
        }
        Err(e) => {
            println!("cargo:warning=Swift not available for Core ML compilation: {}", e);
            println!("cargo:warning=Model will be compiled at runtime instead");
            println!("cargo:rustc-env=COMPILED_MODEL_PATH=");  // Set empty path
        }
    }
}