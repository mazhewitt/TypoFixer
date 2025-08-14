use std::time::Duration;
use tokio::time::timeout;
use typo_fixer::TypoFixerLib;

/// Debug test to understand model generation issues
#[tokio::test]
async fn test_debug_model_generation() {
    println!("🔍 Debug test for model generation issues");
    println!("=========================================");
    
    tracing_subscriber::fmt::try_init().ok();

    // Use the working model that loaded successfully  
    let model_id = "anemll/anemll-Qwen-Qwen3-0.6B-LUT888-ctx512_0.3.4";
    println!("📦 Loading model: {model_id}");

    let mut model = timeout(
        Duration::from_secs(300), // Shorter timeout since we know it loads
        TypoFixerLib::new(Some(model_id.to_string()), true),
    )
    .await
    .expect("Model loading timeout")
    .expect("Model loading failed");

    println!("✅ Model loaded successfully");

    // Test different generation approaches
    println!("\n🔧 Testing different generation approaches...");

    // Test 1: Try with explicit options
    println!("\n1️⃣ Testing with explicit temperature and max_tokens...");
    let result1 = timeout(
        Duration::from_secs(30),
        model.fix_typos_with_options("teh cat", 0.0, Some(10)), // Deterministic, short
    )
    .await;

    match result1 {
        Ok(Ok(output)) => {
            println!("   ✅ Success with explicit options: '{output}'");
        }
        Ok(Err(e)) => {
            println!("   ❌ Failed with explicit options: {e}");
        }
        Err(_) => {
            println!("   ⏰ Timed out with explicit options");
        }
    }

    // Test 2: Try with default options
    println!("\n2️⃣ Testing with default options...");
    let result2 = timeout(
        Duration::from_secs(30),
        model.fix_typos("hello"), // Simple input
    )
    .await;

    match result2 {
        Ok(Ok(output)) => {
            println!("   ✅ Success with default options: '{output}'");
        }
        Ok(Err(e)) => {
            println!("   ❌ Failed with default options: {e}");
        }
        Err(_) => {
            println!("   ⏰ Timed out with default options");
        }
    }

    // Test 3: Try with very simple input
    println!("\n3️⃣ Testing with very simple input...");
    let result3 = timeout(
        Duration::from_secs(60),
        model.fix_typos_with_options("cat", 0.0, Some(5)), // Simplest possible
    )
    .await;

    match result3 {
        Ok(Ok(output)) => {
            println!("   ✅ Success with simple input: '{output}'");
        }
        Ok(Err(e)) => {
            println!("   ❌ Failed with simple input: {e}");
        }
        Err(_) => {
            println!("   ⏰ Timed out with simple input");
        }
    }

    println!("\n🔍 Debug test completed");
}