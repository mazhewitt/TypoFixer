use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=coreml-setup/");
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let source_model = "coreml-setup/coreml-setup/coreml-OpenELM-450M-Instruct/OpenELM-450M-Instruct-128-float32.mlpackage";
    let compiled_model_dir = PathBuf::from(&out_dir).join("compiled_model");
    
    // Check if source model exists
    if !std::path::Path::new(source_model).exists() {
        println!("cargo:warning=Core ML model not found at {}, skipping compilation", source_model);
        println!("cargo:rustc-env=COMPILED_MODEL_PATH=");  // Set empty path
        return;
    }
    
    // Create output directory
    std::fs::create_dir_all(&compiled_model_dir).unwrap();
    
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
        std::fs::canonicalize(source_model).unwrap().display(),
        compiled_model_dir.join("compiled_model.mlmodelc").display()
    );
    
    // Write Swift script to temporary file
    let script_path = PathBuf::from(&out_dir).join("compile_model.swift");
    std::fs::write(&script_path, swift_script).unwrap();
    
    // Execute Swift script
    let output = Command::new("swift")
        .arg(&script_path)
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("cargo:rustc-env=COMPILED_MODEL_PATH={}", 
                    compiled_model_dir.join("compiled_model.mlmodelc").display());
                println!("{}", String::from_utf8_lossy(&result.stdout));
            } else {
                println!("cargo:warning=Failed to compile Core ML model at build time");
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