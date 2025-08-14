use std::path::PathBuf;
use typo_fixer::spell_check::LlamaModelWrapper;

fn main() {
    // Test with the existing GPT-2 CoreML model directory as a placeholder
    let model_path = PathBuf::from("coreml-models/gpt-2-coreml");
    
    println!("🧪 Testing Qwen integration...");
    println!("Model path: {:?}", model_path);
    
    // Test model loading
    match LlamaModelWrapper::new(&model_path) {
        Ok(mut model) => {
            println!("✅ Model wrapper created successfully!");
            
            // Test text generation
            let test_text = "I have teh cat";
            println!("Input: '{}'", test_text);
            
            match model.generate(test_text) {
                Ok(corrected) => {
                    println!("✅ Generation successful!");
                    println!("Output: '{}'", corrected);
                    
                    if corrected != test_text {
                        println!("🎯 Text was modified - correction working!");
                    } else {
                        println!("ℹ️  Text unchanged - model may not be Qwen or fallback active");
                    }
                }
                Err(e) => {
                    println!("⚠️  Generation failed: {}", e);
                    println!("This is expected if the GPT-2 model isn't compatible with Qwen interface");
                }
            }
        }
        Err(e) => {
            println!("❌ Model loading failed: {}", e);
        }
    }
    
    println!("\n💡 To test with your actual Qwen model:");
    println!("   1. Place your Qwen CoreML model in 'coreml-models/qwen-typo-fixer/'");
    println!("   2. Update the model_path above to point to your Qwen directory");
    println!("   3. Run this test again");
}