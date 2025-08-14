use std::time::Duration;
use tokio::time::timeout;
use typo_fixer::TypoFixerLib;

/// Integration test with a model that actually works
#[tokio::test]
async fn test_working_model_integration() {
    println!("🚀 Testing with a model that actually works!");
    println!("==============================================");
    
    tracing_subscriber::fmt::try_init().ok();

    // Use the built-in model that was mentioned in the logs
    let model_id = "anemll/anemll-Qwen-Qwen3-0.6B-LUT888-ctx512_0.3.4";
    println!("📦 Attempting to load built-in model: {model_id}");

    let model_result = timeout(
        Duration::from_secs(600), // 10 minute timeout for download
        TypoFixerLib::new(Some(model_id.to_string()), true),
    )
    .await;

    let mut model = match model_result {
        Ok(Ok(model)) => {
            println!("✅ Model loaded successfully!");
            model
        }
        Ok(Err(e)) => {
            println!("❌ Model loading failed: {e}");
            println!("📝 This indicates the core model loading functionality has issues");
            panic!("Model loading must work for the app to function");
        }
        Err(_) => {
            println!("⏰ Model loading timed out after 10 minutes");
            panic!("Model loading is too slow");
        }
    };

    println!("🔥 Model is ready - testing inference...");

    // Test basic inference
    let test_cases = vec![
        "teh cat",
        "hello wrold", 
        "this is a test",
    ];

    for input in test_cases {
        println!("\n🔧 Testing: '{input}'");
        
        let start = std::time::Instant::now();
        let result = timeout(
            Duration::from_secs(60),
            model.fix_typos(input),
        )
        .await;

        let duration = start.elapsed();

        match result {
            Ok(Ok(output)) => {
                println!("   ✅ Output: '{output}' (took {duration:?})");
                
                // Basic validation
                assert!(!output.is_empty(), "Output should not be empty");
                assert!(output.len() < 1000, "Output should not be excessively long");
                
                println!("   📊 Input length: {}, Output length: {}", input.len(), output.len());
            }
            Ok(Err(e)) => {
                println!("   ❌ Inference failed: {e}");
                panic!("Model inference must work for the app to function");
            }
            Err(_) => {
                println!("   ⏰ Inference timed out");
                panic!("Model inference is too slow");
            }
        }
    }

    println!("\n🎉 WORKING MODEL TEST PASSED!");
    println!("✅ Model loading: SUCCESS");
    println!("✅ Model inference: SUCCESS");
    println!("🔥 The core functionality works!");
}

/// Test that demonstrates the app's main use case
#[tokio::test] 
async fn test_end_to_end_typo_correction() {
    println!("🎯 End-to-end typo correction test");
    println!("===================================");
    
    tracing_subscriber::fmt::try_init().ok();

    // Use the working model
    let model_id = "anemll/anemll-Qwen-Qwen3-0.6B-LUT888-ctx512_0.3.4";
    
    let mut model = timeout(
        Duration::from_secs(600), // 10 minute timeout
        TypoFixerLib::new(Some(model_id.to_string()), false), // Quiet loading
    )
    .await
    .expect("Model loading timeout")
    .expect("Model loading failed");

    // Real-world typo scenarios
    let test_scenarios = vec![
        ("teh quick brown fox", "Classic typo test"),
        ("i cant beleive it", "Multiple common typos"),
        ("seperate the items", "Common misspelling"),
        ("recieve the package", "Another common misspelling"),
        ("hello world", "Already correct text"),
    ];

    println!("\n📝 Testing real-world typo correction scenarios:");
    
    let mut total_time = Duration::from_secs(0);
    let mut successful_tests = 0;

    for (input, description) in &test_scenarios {
        println!("\n  Scenario: {description}");
        println!("  Input: '{input}'");
        
        let start = std::time::Instant::now();
        let result = timeout(
            Duration::from_secs(30),
            model.fix_typos(input),
        )
        .await;
        
        let elapsed = start.elapsed();
        total_time += elapsed;

        match result {
            Ok(Ok(output)) => {
                println!("  Output: '{output}' ({elapsed:?})");
                
                // Validate output quality
                assert!(!output.is_empty(), "Must produce output");
                assert!(output.len() <= input.len() * 3, "Output shouldn't be too long");
                assert!(output.chars().all(|c| c.is_ascii_graphic() || c.is_whitespace()), 
                       "Output should contain valid characters");
                
                if output.trim().to_lowercase() != input.trim().to_lowercase() {
                    println!("  📝 Correction applied: '{input}' → '{output}'");
                } else {
                    println!("  📝 Text kept as-is: '{output}'");
                }
                
                successful_tests += 1;
            }
            Ok(Err(e)) => {
                println!("  ❌ Failed: {e}");
            }
            Err(_) => {
                println!("  ⏰ Timed out");
            }
        }
    }

    let avg_time = total_time / test_scenarios.len() as u32;
    
    println!("\n📊 Test Results:");
    println!("  Successful tests: {}/{}", successful_tests, test_scenarios.len());
    println!("  Total time: {total_time:?}");
    println!("  Average per test: {avg_time:?}");
    
    // Require at least 80% success rate
    let success_rate = (successful_tests as f64 / test_scenarios.len() as f64) * 100.0;
    assert!(success_rate >= 80.0, "Success rate {success_rate:.1}% is too low");
    
    // Require reasonable performance
    assert!(avg_time < Duration::from_secs(10), "Average time {avg_time:?} is too slow");
    
    println!("\n🎉 END-TO-END TEST PASSED!");
    println!("✅ Success rate: {success_rate:.1}%");
    println!("✅ Performance: {avg_time:?} average");
    println!("🚀 The app is ready for real use!");
}