use std::path::Path;
use objc2_core_ml::MLModel;
use objc2_foundation::{NSString, NSURL};

fn main() {
    println!("Compiling SentimentPolarity model...");
    
    let sentiment_path = Path::new("coreml-models/SentimentPolarity.mlmodel");
    
    if !sentiment_path.exists() {
        println!("❌ SentimentPolarity model not found at: {}", sentiment_path.display());
        return;
    }
    
    let ns_path = NSString::from_str(&sentiment_path.to_string_lossy());
    let model_url = unsafe { NSURL::fileURLWithPath(&ns_path) };
    
    println!("🔄 Compiling model...");
    match unsafe { MLModel::compileModelAtURL_error(&model_url) } {
        Ok(compiled_url) => {
            println!("✅ Model compiled successfully!");
            let compiled_path_str = unsafe { compiled_url.path() }.unwrap().to_string();
            println!("   Compiled to: {}", compiled_path_str);
            
            // Test loading the compiled model
            println!("🔄 Testing compiled model...");
            match unsafe { MLModel::modelWithContentsOfURL_error(&compiled_url) } {
                Ok(_model) => {
                    println!("✅ Compiled model loads successfully!");
                    println!("📝 The SentimentPolarity model is now ready to use");
                }
                Err(e) => {
                    println!("❌ Failed to load compiled model: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Compilation failed: {:?}", e);
            let error_desc = e.localizedDescription();
            println!("   Error: {}", error_desc.to_string());
        }
    }
}