// Quick test for tokenizer loading fix
use std::path::PathBuf;

fn main() {
    println!("🧪 Testing tokenizer fix for Qwen CoreML...");
    
    let cache_path = PathBuf::from("/Users/mazdahewitt/Library/Caches/candle-coreml/clean-mazhewitt--qwen-typo-fixer");
    
    println!("Cache path: {:?}", cache_path);
    println!("Cache exists: {}", cache_path.exists());
    
    let tokenizer_root = cache_path.join("tokenizer.json");
    let coreml_dir = cache_path.join("coreml");
    let tokenizer_coreml = coreml_dir.join("tokenizer.json");
    
    println!("Tokenizer at root: {} (exists: {})", tokenizer_root.display(), tokenizer_root.exists());
    println!("CoreML directory: {} (exists: {})", coreml_dir.display(), coreml_dir.exists());
    println!("Tokenizer in CoreML: {} (exists: {})", tokenizer_coreml.display(), tokenizer_coreml.exists());
    
    if tokenizer_root.exists() && coreml_dir.exists() && !tokenizer_coreml.exists() {
        println!("🔧 Copying tokenizer to CoreML directory...");
        match std::fs::copy(&tokenizer_root, &tokenizer_coreml) {
            Ok(_) => println!("✅ Tokenizer copied successfully!"),
            Err(e) => println!("❌ Failed to copy tokenizer: {}", e),
        }
    }
    
    // Now test model loading
    println!("\n🧪 Testing model loading...");
    match typo_fixer::spell_check::LlamaModelWrapper::new(&cache_path) {
        Ok(mut model) => {
            println!("✅ Model wrapper created!");
            
            let test_text = "I have teh cat";
            match model.generate(test_text) {
                Ok(result) => println!("✅ Generated: '{}' -> '{}'", test_text, result),
                Err(e) => println!("⚠️ Generation failed: {}", e),
            }
        }
        Err(e) => println!("❌ Model loading failed: {}", e),
    }
}