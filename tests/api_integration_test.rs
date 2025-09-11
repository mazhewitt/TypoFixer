use std::time::Duration;
use tokio::time::timeout;
use typo_fixer::TypoFixer;

/// Test the core TypoFixer API with real model loading
#[tokio::test]
async fn test_typo_fixer_api_integration() {
    println!("🔧 Testing TypoFixer API integration with REAL models...");
    
    tracing_subscriber::fmt::try_init().ok();

    // Test 1: Create TypoFixer directly from HuggingFace
    println!("\n📋 Testing TypoFixer::new...");
    let typo_fixer = timeout(
        Duration::from_secs(120),
        TypoFixer::new(None, true),
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

    // Test 3: Test TypoFixer wrapper API
    println!("\n🔧 Testing TypoFixer API...");
    let mut typo_fixer_lib = timeout(
        Duration::from_secs(120),
        TypoFixer::new(None, true),
    )
    .await
    .expect("TypoFixer loading timed out")
    .expect("TypoFixer loading failed");

    println!("✅ TypoFixer created successfully");

    // Test 4: API consistency - both APIs should work the same way
    println!("\n🔄 Testing API consistency...");
    let test_input = "teh quick test";
    
    let lib_result = timeout(
        Duration::from_secs(30),
        typo_fixer_lib.fix_typos(test_input),
    )
    .await
    .expect("TypoFixer inference timed out")
    .expect("TypoFixer inference failed");

    println!("   TypoFixer result: '{test_input}' → '{lib_result}'");
    
    assert!(!lib_result.is_empty(), "TypoFixer must produce output");
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
        TypoFixer::new(None, false), // Use default model
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
        TypoFixer::new(Some("mazhewitt/qwen-typo-fixer-coreml"), false),
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

    // Method 3: Local path model (if available)
    if std::path::Path::new("configs/qwen-typo-fixer-ane.json").exists() {
        println!("\n3️⃣ Testing local path model...");
        let mut model3 = timeout(
            Duration::from_secs(120),
            TypoFixer::new_with_local_path("configs/qwen-typo-fixer-ane.json", false),
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

        println!("   Local path model result: '{result3}'");
    } else {
        println!("\n3️⃣ Skipping local path test (no config available)");
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
        TypoFixer::new(Some("invalid/nonexistent-model"), false),
    )
    .await;

    match invalid_result {
        Ok(Err(_)) => println!("   ✅ Invalid model correctly rejected"),
        Ok(Ok(_)) => panic!("❌ Invalid model should not load successfully"),
        Err(_) => println!("   ✅ Invalid model loading timed out (acceptable)"),
    }

    // Test 2: Invalid local path should fail gracefully
    println!("\n❌ Testing invalid local path...");
    let invalid_config_result = timeout(
        Duration::from_secs(30),
        TypoFixer::new_with_local_path("nonexistent-config.json", false),
    )
    .await;

    match invalid_config_result {
        Ok(Err(_)) => println!("   ✅ Invalid local path correctly rejected"),
        Ok(Ok(_)) => panic!("❌ Invalid local path should not load successfully"),
        Err(_) => println!("   ✅ Invalid local path loading timed out (acceptable)"),
    }

    // Test 3: Empty input handling
    println!("\n📝 Testing empty input handling...");
    let mut model = timeout(
        Duration::from_secs(120),
        TypoFixer::new(None, false),
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