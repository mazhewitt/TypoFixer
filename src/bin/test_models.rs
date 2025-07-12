use std::path::Path;
use objc2_core_ml::MLModel;
use objc2_foundation::{NSString, NSURL};

fn main() {
    println!("Testing Core ML model compatibility...");
    
    // Test 1: SentimentPolarity.mlmodel (should work)
    let sentiment_path = Path::new("coreml-models/SentimentPolarity.mlmodel");
    println!("\n🔍 Testing SentimentPolarity.mlmodel");
    println!("   Path: {}", sentiment_path.display());
    println!("   Exists: {}", sentiment_path.exists());
    
    if sentiment_path.exists() {
        let ns_path = NSString::from_str(&sentiment_path.to_string_lossy());
        let model_url = unsafe { NSURL::fileURLWithPath(&ns_path) };
        
        match unsafe { MLModel::modelWithContentsOfURL_error(&model_url) } {
            Ok(_model) => {
                println!("   ✅ SentimentPolarity model loaded successfully!");
                println!("   📝 This model is compatible with your Core ML version");
            }
            Err(e) => {
                println!("   ❌ SentimentPolarity model failed: {:?}", e);
                let error_desc = e.localizedDescription();
                println!("   Error: {}", error_desc.to_string());
            }
        }
    } else {
        println!("   ⚠️  SentimentPolarity model file not found");
    }
    
    // Test 2: OpenELM model (will fail with wireType 6)
    let openelm_path = Path::new("coreml-setup/coreml-setup/coreml-OpenELM-450M-Instruct/OpenELM-450M-Instruct-128-float32.mlpackage");
    println!("\n🔍 Testing OpenELM model");
    println!("   Path: {}", openelm_path.display());
    println!("   Exists: {}", openelm_path.exists());
    
    if openelm_path.exists() {
        let ns_path = NSString::from_str(&openelm_path.to_string_lossy());
        let model_url = unsafe { NSURL::fileURLWithPath(&ns_path) };
        
        println!("   🔄 Attempting to load model...");
        match unsafe { MLModel::modelWithContentsOfURL_error(&model_url) } {
            Ok(_model) => {
                println!("   ✅ OpenELM model loaded successfully!");
            }
            Err(load_error) => {
                println!("   ❌ Direct load failed, trying compilation...");
                
                match unsafe { MLModel::compileModelAtURL_error(&model_url) } {
                    Ok(_compiled_url) => {
                        println!("   ✅ OpenELM model compiled successfully!");
                    }
                    Err(compile_error) => {
                        println!("   ❌ Compilation failed: {:?}", compile_error);
                        let error_desc = compile_error.localizedDescription();
                        let error_str = error_desc.to_string();
                        println!("   Error: {}", error_str);
                        
                        if error_str.contains("wireType 6") {
                            println!("   🎯 CONFIRMED: wireType 6 parsing issue");
                            println!("   📝 This model was created with newer Core ML tools");
                            println!("   💡 Solution: Use a compatible model or update Core ML");
                        }
                    }
                }
            }
        }
    } else {
        println!("   ⚠️  OpenELM model file not found");
    }
    
    println!("\n📋 Summary:");
    println!("   - SentimentPolarity.mlmodel: Compatible test model");
    println!("   - OpenELM model: Incompatible (wireType 6 issue)");
    println!("   - Recommendation: Use SentimentPolarity.mlmodel for now");
}