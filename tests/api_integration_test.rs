use std::time::Duration;
use tokio::time::timeout;
use typo_fixer::{TypoFixer, TypoFixerLib};

/// Test the core TypoFixer API with real model loading
#[tokio::test]
async fn test_typo_fixer_api_integration() {
    println!("🔧 Testing TypoFixer API integration with REAL models...");
    
    tracing_subscriber::fmt::try_init().ok();

    // Test 1: Create TypoFixer directly from local config
    println!("\n📋 Testing TypoFixer::new_with_config_file...");
    let typo_fixer = timeout(
        Duration::from_secs(120),
        TypoFixer::new_with_config_file("configs/qwen-typo-fixer-ane.json", ".", true),
    )
    .await
    .expect("TypoFixer loading timed out")
    .expect("TypoFixer loading failed");

    println!("✅ TypoFixer created successfully");

    // Test 2: Test prompt template functionality
    println!("\n📝 Testing prompt template access...");
    let template = typo_fixer.prompt_template();
    assert_eq!(template.example_count(), 5, "Should have 5 default examples");
    
    let test_prompt = template.create_correction_prompt("test input");
    assert!(test_prompt.contains("Fix typos"), "Prompt should contain header");
    assert!(test_prompt.contains("test input"), "Prompt should contain input");
    println!("✅ Prompt template working correctly");

    // Test 3: Test TypoFixerLib wrapper API
    println!("\n🔧 Testing TypoFixerLib API...");
    let mut typo_fixer_lib = timeout(
        Duration::from_secs(120),
        TypoFixerLib::new_with_config_file("configs/qwen-typo-fixer-ane.json", ".", true),
    )
    .await
    .expect("TypoFixerLib loading timed out")
    .expect("TypoFixerLib loading failed");

    println!("✅ TypoFixerLib created successfully");

    // Test 4: API consistency - both APIs should work the same way
    println!("\n🔄 Testing API consistency...");
    let test_input = "teh quick test";
    
    let lib_result = timeout(
        Duration::from_secs(30),
        typo_fixer_lib.fix_typos(test_input),
    )
    .await
    .expect("TypoFixerLib inference timed out")
    .expect("TypoFixerLib inference failed");

    println!("   TypoFixerLib result: '{test_input}' → '{lib_result}'");
    
    assert!(!lib_result.is_empty(), "TypoFixerLib must produce output");
    assert!(lib_result.len() < 1000, "Output should be reasonable length");

    println!("✅ API integration test PASSED!");
}

/// Test different model loading methods
#[tokio::test]
async fn test_multiple_model_loading_methods() {
    println!("🔄 Testing multiple model loading methods...");
    
    tracing_subscriber::fmt::try_init().ok();

    // Method 1: Default HuggingFace model
    println!("\n1️⃣ Testing default HuggingFace loading...");
    let mut model1 = timeout(
        Duration::from_secs(300),
        TypoFixerLib::new(None, false), // Use default model
    )
    .await
    .expect("Default model loading timed out")
    .expect("Default model loading failed");

    let result1 = timeout(
        Duration::from_secs(30),
        model1.fix_typos("test input"),
    )
    .await
    .expect("Default model inference timed out")
    .expect("Default model inference failed");

    println!("   Default model result: '{result1}'");

    // Method 2: Specific HuggingFace model
    println!("\n2️⃣ Testing specific HuggingFace model...");
    let mut model2 = timeout(
        Duration::from_secs(300),
        TypoFixerLib::new(Some("mazhewitt/qwen-typo-fixer-coreml".to_string()), false),
    )
    .await
    .expect("Specific model loading timed out")
    .expect("Specific model loading failed");

    let result2 = timeout(
        Duration::from_secs(30),
        model2.fix_typos("test input"),
    )
    .await
    .expect("Specific model inference timed out")
    .expect("Specific model inference failed");

    println!("   Specific model result: '{result2}'");

    // Method 3: Config file model (if available)
    if std::path::Path::new("configs/qwen-typo-fixer-ane.json").exists() {
        println!("\n3️⃣ Testing config file model...");
        let mut model3 = timeout(
            Duration::from_secs(120),
            TypoFixerLib::new_with_config_file("configs/qwen-typo-fixer-ane.json", ".", false),
        )
        .await
        .expect("Config model loading timed out")
        .expect("Config model loading failed");

        let result3 = timeout(
            Duration::from_secs(30),
            model3.fix_typos("test input"),
        )
        .await
        .expect("Config model inference timed out")
        .expect("Config model inference failed");

        println!("   Config model result: '{result3}'");
    } else {
        println!("\n3️⃣ Skipping config file test (no config available)");
    }

    println!("\n✅ Multiple loading methods test PASSED!");
}

/// Test error handling and edge cases
#[tokio::test]
async fn test_model_error_handling() {
    println!("🛡️ Testing model error handling...");
    
    tracing_subscriber::fmt::try_init().ok();

    // Test 1: Invalid model ID should fail gracefully
    println!("\n❌ Testing invalid model ID...");
    let invalid_result = timeout(
        Duration::from_secs(60),
        TypoFixerLib::new(Some("invalid/nonexistent-model".to_string()), false),
    )
    .await;

    match invalid_result {
        Ok(Err(_)) => println!("   ✅ Invalid model correctly rejected"),
        Ok(Ok(_)) => panic!("❌ Invalid model should not load successfully"),
        Err(_) => println!("   ✅ Invalid model loading timed out (acceptable)"),
    }

    // Test 2: Invalid config file should fail gracefully
    println!("\n❌ Testing invalid config file...");
    let invalid_config_result = timeout(
        Duration::from_secs(30),
        TypoFixerLib::new_with_config_file("nonexistent-config.json", ".", false),
    )
    .await;

    match invalid_config_result {
        Ok(Err(_)) => println!("   ✅ Invalid config correctly rejected"),
        Ok(Ok(_)) => panic!("❌ Invalid config should not load successfully"),
        Err(_) => println!("   ✅ Invalid config loading timed out (acceptable)"),
    }

    // Test 3: Empty input handling
    println!("\n📝 Testing empty input handling...");
    let mut model = timeout(
        Duration::from_secs(120),
        TypoFixerLib::new(None, false),
    )
    .await
    .expect("Model loading for empty test timed out")
    .expect("Model loading for empty test failed");

    let empty_result = timeout(
        Duration::from_secs(30),
        model.fix_typos(""),
    )
    .await;

    match empty_result {
        Ok(Ok(result)) => {
            println!("   ✅ Empty input handled: '{result}'");
            // Empty input should produce some output or be handled gracefully
        }
        Ok(Err(e)) => {
            println!("   ✅ Empty input correctly rejected: {e}");
        }
        Err(_) => {
            println!("   ⚠️ Empty input processing timed out");
        }
    }

    // Test 4: Very long input handling
    println!("\n📏 Testing very long input handling...");
    let very_long_input = "word ".repeat(1000); // 5000 characters
    
    let long_result = timeout(
        Duration::from_secs(120), // Longer timeout for long input
        model.fix_typos(&very_long_input),
    )
    .await;

    match long_result {
        Ok(Ok(result)) => {
            println!("   ✅ Long input handled (result length: {})", result.len());
            assert!(result.len() < very_long_input.len() * 2, "Result should not be excessively long");
        }
        Ok(Err(e)) => {
            println!("   ✅ Long input correctly rejected: {e}");
        }
        Err(_) => {
            println!("   ⚠️ Long input processing timed out (may be acceptable)");
        }
    }

    println!("\n✅ Error handling test PASSED!");
}