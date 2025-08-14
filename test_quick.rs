fn main() { 
    println!("🧪 Testing Qwen CoreML integration...");
    
    // Test with GPT-2 CoreML model directory as placeholder
    let model_path = std::path::PathBuf::from("coreml-models/gpt-2-coreml");
    println!("Model path: {:?}", model_path);
    
    match typo_fixer::spell_check::LlamaModelWrapper::new(&model_path) {
        Ok(mut model) => {
            println!("✅ Model wrapper created successfully!");
            
            let test_text = "I have teh cat";
            println!("Input: '{}'", test_text);
            
            match model.generate(test_text) {
                Ok(corrected) => {
                    println!("✅ Generation successful!");
                    println!("Output: '{}'", corrected);
                }
                Err(e) => {
                    println!("⚠️  Generation failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Model loading failed: {}", e);
        }
    }
    
    println!("\n💡 For real Qwen model: place in 'coreml-models/qwen-typo-fixer/'");
}
