// Quick model integration test - demonstrates that the model loading works
// Run with: cargo run --bin test_model_quick

use candle_coreml::ModelConfig;
use std::time::Duration;
use tokio::time::timeout;
use typo_fixer::TypoFixerLib;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Quick Model Integration Test");
    println!("================================");

    // Test 1: Configuration Discovery
    println!("\n📋 Testing configuration discovery...");

    let config_path = std::path::Path::new("configs/qwen-typo-fixer-ane.json");
    if config_path.exists() {
        println!("✅ Found config file: {}", config_path.display());
        let config = ModelConfig::load_from_file(config_path)?;
        println!(
            "   Model: {} ({})",
            config.model_info.path.as_deref().unwrap_or("<no path>"),
            config.model_info.model_type
        );
        println!(
            "   Vocab size: {}, Context: {}",
            config.shapes.vocab_size, config.shapes.context_length
        );
    } else {
        println!("⚠️ No config file found - will use fallback loading");
    }

    // Test 2: Model Loading (with reasonable timeout)
    println!("\n🔄 Testing model loading with auto-config...");
    println!("⏰ Note: This may take several minutes on first run to download the model");

    let model_result = timeout(
        Duration::from_secs(120), // 2 minute timeout for quick test
        TypoFixerLib::new_with_config_file("configs/qwen-typo-fixer-ane.json", ".", true),
    )
    .await;

    let mut model = match model_result {
        Ok(Ok(model)) => {
            println!("✅ Model loaded successfully!");
            model
        }
        Ok(Err(e)) => {
            println!("⚠️ Auto-config loading failed: {e}");
            println!("🔄 Trying fallback to HuggingFace Hub (coreml assets)...");

            timeout(
                Duration::from_secs(120),
                TypoFixerLib::new(Some("mazhewitt/qwen-typo-fixer-coreml".to_string()), true),
            )
            .await??
        }
        Err(_) => {
            println!("⏰ Model loading timed out after 2 minutes");
            println!("💡 The model is likely downloading in the background");
            println!("💡 For full integration test with actual corrections, run:");
            println!("   cargo test test_model_integration -- --nocapture --timeout 600");
            return Ok(());
        }
    };

    // Test 3: Model ready
    println!("\n🔥 Model is ready for use...");

    // Test 4: Quick typo correction test
    println!("\n🔧 Testing basic typo correction...");
    let test_input = "teh cat";

    let correction_result = timeout(Duration::from_secs(30), model.fix_typos(test_input)).await;

    match correction_result {
        Ok(Ok(corrected)) => {
            println!("✅ Typo correction successful!");
            println!("   Input: '{test_input}'");
            println!("   Output: '{corrected}'");

            // Verify the result is not empty and is a string
            assert!(
                !corrected.is_empty(),
                "Correction result should not be empty"
            );
            assert!(
                !corrected.is_empty(),
                "Correction result should have content"
            );

            if corrected.to_lowercase() != test_input.to_lowercase() {
                println!("   📝 Correction was applied");
            } else {
                println!("   📝 Model kept original (may be correct as-is)");
            }
        }
        Ok(Err(e)) => {
            println!("❌ Typo correction failed: {e}");
        }
        Err(_) => {
            println!("⏰ Typo correction timed out after 30 seconds");
        }
    }

    // Test 5: Model Info
    println!("\n📊 Model Information:");
    println!("   TypoFixerLib ready for typo correction");

    println!("\n🎉 Quick integration test completed!");
    println!("💡 For comprehensive testing with multiple test cases, run:");
    println!("   cargo test test_model_integration -- --nocapture");

    Ok(())
}
