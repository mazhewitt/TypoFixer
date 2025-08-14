// Simple integration test for Qwen model loading
// Run with: rustc test_model_integration.rs -L target/debug/deps --extern typo_fixer=target/debug/libtypo_fixer.rlib

use std::path::PathBuf;

fn main() {
    println!("🧪 Testing Qwen CoreML Integration");
    println!("================================");
    
    // Test with existing GPT-2 model (should fail gracefully)
    test_model_loading("coreml-models/gpt-2-coreml", "GPT-2 CoreML (should fail gracefully)");
    
    // Test with non-existent path  
    test_model_loading("coreml-models/qwen-typo-fixer", "Qwen model directory (expected location)");
    
    // Test with completely invalid path
    test_model_loading("/non/existent/path", "Invalid path (should fail gracefully)");
    
    println!("\n💡 Next Steps:");
    println!("1. Place your converted Qwen CoreML model in: coreml-models/qwen-typo-fixer/");
    println!("2. Model should include:");
    println!("   - Model files (*.mlpackage or *.mlmodel)");
    println!("   - Tokenizer files (tokenizer.json, vocab files, etc.)");
    println!("3. Re-run this test to verify loading");
    println!("4. Test the full application with: cargo run");
}

fn test_model_loading(path: &str, description: &str) {
    println!("\n🔍 Testing: {}", description);
    println!("   Path: {}", path);
    
    let model_path = PathBuf::from(path);
    
    match typo_fixer::spell_check::LlamaModelWrapper::new(&model_path) {
        Ok(model) => {
            println!("   ✅ Model wrapper created successfully!");
            
            // Test if model is actually loaded
            // Note: This will likely fail for GPT-2 since it's not a Qwen model
            println!("   🧪 Testing text generation...");
            test_generation(model);
        }
        Err(e) => {
            println!("   ⚠️  Model loading failed: {}", e);
            println!("      This is expected if no compatible model is present");
        }
    }
}

fn test_generation(mut model: typo_fixer::spell_check::LlamaModelWrapper) {
    let test_cases = vec![
        "I have teh cat",
        "Their going to there house",
        "Its a beautifull day",
    ];
    
    for test_text in test_cases {
        match model.generate(test_text) {
            Ok(corrected) => {
                println!("      Input:  '{}'", test_text);
                println!("      Output: '{}'", corrected);
                
                if corrected != test_text {
                    println!("      🎯 Text was corrected!");
                } else {
                    println!("      ℹ️  Text unchanged (fallback mode or no corrections needed)");
                }
            }
            Err(e) => {
                println!("      ❌ Generation failed: {}", e);
                println!("         This is expected if model format is incompatible");
                break;
            }
        }
    }
}