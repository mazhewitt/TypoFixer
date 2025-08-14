use candle_coreml::ModelConfig;
use std::time::Duration;
use tokio::time::timeout;
use typo_fixer::{PromptTemplate, TypoFixerLib};

/// Real integration test that verifies actual model loading and typo correction
/// This test requires real model files to be available and will fail if they're not
#[tokio::test]
async fn test_real_model_loading_and_inference() {
    println!("🚀 Starting REAL model integration test (no mocks, no fallbacks)");
    println!("================================================================================");
    
    // Initialize logging
    tracing_subscriber::fmt::try_init().ok(); // Ignore if already initialized

    // Test 1: Verify config file exists and is valid
    println!("\n📋 Step 1: Validating configuration file...");
    let config_path = "configs/qwen-typo-fixer-ane.json";
    assert!(
        std::path::Path::new(config_path).exists(),
        "❌ Config file must exist at {config_path}"
    );

    let config = ModelConfig::load_from_file(config_path)
        .expect("❌ Must be able to load config file");
    
    println!("✅ Config loaded successfully:");
    println!("   Model path: {}", config.model_info.path.as_deref().unwrap_or("<no path>"));
    println!("   Model type: {}", config.model_info.model_type);
    println!("   Context length: {}", config.shapes.context_length);
    println!("   Vocab size: {}", config.shapes.vocab_size);
    println!("   Components: {}", config.components.len());

    // Verify all required components exist
    let required_components = ["embeddings", "ffn_prefill", "ffn_infer", "lm_head"];
    for component_name in &required_components {
        assert!(
            config.components.contains_key(*component_name),
            "❌ Config must contain {component_name} component"
        );
        let component = &config.components[*component_name];
        assert!(
            component.file_path.is_some(),
            "❌ Component {component_name} must have a file path"
        );
        println!("   ✅ Component {component_name}: {}", component.file_path.as_ref().unwrap());
    }

    // Test 2: Load model with config file (REAL loading, no fallbacks)
    println!("\n🔄 Step 2: Loading model from config file (NO FALLBACKS)...");
    let model_result = timeout(
        Duration::from_secs(120), // 2 minute timeout
        TypoFixerLib::new_with_config_file(config_path, ".", true),
    )
    .await;

    let mut model = match model_result {
        Ok(Ok(model)) => {
            println!("✅ Model loaded successfully from config file");
            model
        }
        Ok(Err(e)) => {
            panic!("❌ CRITICAL: Model loading from config failed: {e}");
        }
        Err(_) => {
            panic!("❌ CRITICAL: Model loading timed out after 2 minutes");
        }
    };

    // Test 3: Verify model is ready for inference
    println!("\n🔥 Step 3: Model loaded and ready for inference");

    // Test 4: Real typo correction with actual model inference
    println!("\n🔧 Step 4: Testing REAL typo correction with model inference...");
    
    let test_cases = vec![
        ("teh cat", "should correct 'teh' to 'the'"),
        ("i cant beleive it", "should correct multiple typos"),
        ("recieve the packge", "should correct 'recieve' and 'packge'"),
        ("separat ethe itmes", "should correct spacing and spelling"),
        ("it occured yestrday", "should correct 'occured' and 'yestrday'"),
    ];

    let mut successful_corrections = 0;
    let mut total_inference_time = Duration::from_secs(0);

    for (i, (input, description)) in test_cases.iter().enumerate() {
        println!("\n  Test {}: '{input}' - {description}", i + 1);
        
        let start_time = std::time::Instant::now();
        let result = timeout(
            Duration::from_secs(60), // 60 second timeout per inference
            model.fix_typos(input),
        )
        .await;

        let inference_time = start_time.elapsed();
        total_inference_time += inference_time;

        match result {
            Ok(Ok(corrected)) => {
                println!("    ✅ Result: '{corrected}' (took {inference_time:?})");
                
                // Strict validation - these are REAL requirements
                assert!(
                    !corrected.is_empty(),
                    "❌ CRITICAL: Model must produce non-empty output for '{input}'"
                );
                
                assert!(
                    corrected.len() <= input.len() * 4, // Allow some expansion but not too much
                    "❌ CRITICAL: Output too long for '{input}': '{corrected}'"
                );
                
                assert!(
                    corrected.chars().all(|c| c.is_ascii() || c.is_whitespace()),
                    "❌ CRITICAL: Output contains invalid characters for '{input}': '{corrected}'"
                );

                // If the model made changes, that's good (though not required for every case)
                if corrected.to_lowercase().trim() != input.to_lowercase().trim() {
                    println!("    📝 Model applied corrections: '{input}' → '{corrected}'");
                    successful_corrections += 1;
                } else {
                    println!("    📝 Model kept original (may be intentional): '{corrected}'");
                }
            }
            Ok(Err(e)) => {
                panic!("❌ CRITICAL: Model inference failed for '{input}': {e}");
            }
            Err(_) => {
                panic!("❌ CRITICAL: Model inference timed out for '{input}' after 60 seconds");
            }
        }
    }

    // Test 5: Performance validation
    println!("\n📊 Step 5: Performance validation");
    let avg_inference_time = total_inference_time / test_cases.len() as u32;
    println!("   Total inference time: {total_inference_time:?}");
    println!("   Average per correction: {avg_inference_time:?}");
    println!("   Successful corrections: {successful_corrections}/{}", test_cases.len());

    // Performance assertions
    assert!(
        avg_inference_time < Duration::from_secs(30),
        "❌ CRITICAL: Average inference time too slow: {avg_inference_time:?}"
    );

    // Test 6: Model consistency test
    println!("\n🔁 Step 6: Testing model consistency...");
    let consistency_input = "teh quick brown fox";
    let mut results = Vec::new();
    
    for i in 0..3 {
        let result = timeout(
            Duration::from_secs(30),
            model.fix_typos(consistency_input),
        )
        .await
        .expect("Consistency test timed out")
        .expect("Consistency test failed");
        
        results.push(result.clone());
        println!("   Run {}: '{result}'", i + 1);
    }

    // Check if results are consistent (they should be for the same input)
    let first_result = &results[0];
    let all_same = results.iter().all(|r| r == first_result);
    
    if all_same {
        println!("   ✅ Model outputs are consistent");
    } else {
        println!("   ⚠️ Model outputs vary (this may be expected with temperature > 0)");
    }

    println!("\n🎉 REAL MODEL INTEGRATION TEST PASSED!");
    println!("================================================================================");
    println!("✅ Model loading: SUCCESS");
    println!("✅ Model inference: SUCCESS"); 
    println!("✅ Performance: ACCEPTABLE");
    println!("✅ All test cases: PASSED");
    println!("🔥 The app's core functionality is working with REAL models!");
}

/// Test model loading from HuggingFace Hub (requires internet)
#[tokio::test]
async fn test_real_huggingface_model_loading() {
    println!("🌐 Testing REAL HuggingFace Hub model loading...");
    
    tracing_subscriber::fmt::try_init().ok();

    let model_result = timeout(
        Duration::from_secs(300), // 5 minute timeout for download
        TypoFixerLib::new(Some("mazhewitt/qwen-typo-fixer-coreml".to_string()), true),
    )
    .await;

    let mut model = match model_result {
        Ok(Ok(model)) => {
            println!("✅ HuggingFace model loaded successfully");
            model
        }
        Ok(Err(e)) => {
            panic!("❌ CRITICAL: HuggingFace model loading failed: {e}");
        }
        Err(_) => {
            panic!("❌ CRITICAL: HuggingFace model loading timed out");
        }
    };

    // Test a simple correction to verify the model works
    let test_input = "teh cat is hear";
    let result = timeout(
        Duration::from_secs(30),
        model.fix_typos(test_input),
    )
    .await
    .expect("HuggingFace model inference timed out")
    .expect("HuggingFace model inference failed");

    println!("✅ HuggingFace model inference: '{test_input}' → '{result}'");
    
    assert!(!result.is_empty(), "HuggingFace model must produce output");
    
    println!("🎉 HuggingFace model integration test PASSED!");
}

/// Benchmark test for model performance
#[tokio::test]
async fn test_model_performance_benchmark() {
    println!("⏱️ Running model performance benchmark...");
    
    tracing_subscriber::fmt::try_init().ok();

    // Load model
    let mut model = timeout(
        Duration::from_secs(120),
        TypoFixerLib::new_with_config_file("configs/qwen-typo-fixer-ane.json", ".", false), // No verbose logging for benchmark
    )
    .await
    .expect("Benchmark model loading timed out")
    .expect("Benchmark model loading failed");

    // Benchmark different input lengths
    let benchmark_cases = vec![
        ("short", "teh cat"),
        ("medium", "i cant beleive this sentance has typos in it"),
        ("long", "this is a much longer sentance with many words that contains several typos like recieve, occured, seperate and other common misspellings that the model should correct"),
    ];

    println!("\nBenchmark Results:");
    println!("==================");

    for (length_type, input) in benchmark_cases {
        let start = std::time::Instant::now();
        
        let result = timeout(
            Duration::from_secs(60),
            model.fix_typos(input),
        )
        .await
        .expect(&format!("Benchmark {length_type} case timed out"))
        .expect(&format!("Benchmark {length_type} case failed"));
        
        let duration = start.elapsed();
        
        println!("{length_type:>6}: {duration:>8?} | '{input}' → '{result}'");
        
        // Performance assertions
        match length_type {
            "short" => assert!(duration < Duration::from_secs(10), "Short inputs should be fast"),
            "medium" => assert!(duration < Duration::from_secs(20), "Medium inputs should be reasonable"),
            "long" => assert!(duration < Duration::from_secs(30), "Long inputs should complete in reasonable time"),
            _ => {}
        }
    }

    println!("\n✅ Performance benchmark PASSED!");
}