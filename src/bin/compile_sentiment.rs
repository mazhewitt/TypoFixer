use std::path::Path;
use objc2_core_ml::MLModel;
use objc2_foundation::{NSString, NSURL};

fn main() {
    println!("Testing SentimentPolarity model loading...");
    
    let sentiment_path = Path::new("coreml-models/SentimentPolarity.mlmodel");
    
    if !sentiment_path.exists() {
        println!("❌ SentimentPolarity model not found at: {}", sentiment_path.display());
        return;
    }
    
    let ns_path = NSString::from_str(&sentiment_path.to_string_lossy());
    let model_url = unsafe { NSURL::fileURLWithPath(&ns_path) };
    
    println!("🔄 Loading model directly...");
    match unsafe { MLModel::modelWithContentsOfURL_error(&model_url) } {
        Ok(_model) => {
            println!("✅ Model loaded successfully!");
            println!("📝 The SentimentPolarity model is ready to use");
            println!("💡 For production use, models are pre-compiled at build time");
        }
        Err(e) => {
            println!("❌ Failed to load model: {:?}", e);
            let error_desc = e.localizedDescription();
            println!("   Error: {}", error_desc.to_string());
            println!("💡 Note: This model may need to be compiled first.");
            println!("   The build script handles this automatically for production builds.");
        }
    }
}