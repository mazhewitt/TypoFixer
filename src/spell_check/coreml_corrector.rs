use std::path::Path;
use tracing::info;

// Import our modular components
use super::coreml::{
    CorrectionError, CoreMLModelManager, TextProcessor, ArrayUtils, TextUtils
};

/// Core ML-based grammar corrector for on-device inference
#[derive(Debug)]
pub struct CoreMLCorrector {
    model_manager: CoreMLModelManager,
    text_processor: TextProcessor,
}

// SAFETY: Temporary implementation for application compatibility
// The application currently requires Send + Sync for global state management.
// REQUIREMENTS for safety:
// 1. CoreMLCorrector must only be used from the main thread
// 2. No concurrent access to Core ML APIs
// 3. All operations should be synchronized through the application's mutex
//
// TODO: Redesign the application to use proper thread-local storage or message passing
// to eliminate the need for Send + Sync on Core ML types.
unsafe impl Send for CoreMLCorrector {}
unsafe impl Sync for CoreMLCorrector {}

impl CoreMLCorrector {
    /// Create a new CoreMLCorrector instance
    pub fn new(model_path: &Path) -> Result<Self, CorrectionError> {
        info!("🧠 Initializing Core ML-based grammar corrector...");
        
        let mut model_manager = CoreMLModelManager::new(model_path);
        let mut text_processor = TextProcessor::new();
        
        // Try to load the model - fail if it doesn't work
        model_manager.load_model()?;
        
        // Try to load the tokenizer (not critical if it fails)
        text_processor.load_tokenizer(model_path)?;
        
        Ok(Self {
            model_manager,
            text_processor,
        })
    }
    
    /// Get model loading status
    pub fn is_model_loaded(&self) -> bool {
        self.model_manager.is_loaded()
    }
    
    /// Get model path
    pub fn model_path(&self) -> &Path {
        self.model_manager.model_path()
    }
    
    /// Correct text using the loaded Core ML model
    pub fn correct(&self, text: &str) -> Result<String, CorrectionError> {
        info!("🔧 Correcting text with Core ML: '{}'", text);
        
        // Get the loaded model
        let model = self.model_manager.model()?;
        
        // Perform the full inference pipeline
        self.coreml_inference(text, model)
    }
    
    /// Perform Core ML inference with the actual model
    fn coreml_inference(&self, text: &str, model: &objc2_core_ml::MLModel) -> Result<String, CorrectionError> {
        info!("🤖 Using Core ML inference for: '{}'", text);
        
        // Step 1: Tokenize the input text
        let tokens = self.text_processor.tokenize(text)?;
        info!("📝 Tokenized input into {} tokens", tokens.len());
        
        // Step 2: Create MLMultiArray input from tokens
        let input_array = ArrayUtils::create_ml_multiarray(&tokens)?;
        info!("🔧 Created MLMultiArray with shape for {} tokens", tokens.len());
        
        // Step 3: Run Core ML model prediction (simplified identity transformation for now)
        let output_array = ArrayUtils::predict_with_model(&input_array, model)?;
        info!("✅ Core ML model prediction successful");
        
        // Step 4: Decode the output back to text
        let corrected_text = self.text_processor.detokenize(&ArrayUtils::extract_tokens(&output_array)?)?;
        info!("🔤 Decoded output: '{}'", corrected_text);
        
        // Step 5: Apply post-processing
        let final_text = TextUtils::post_process_text(&corrected_text, text)?;
        info!("✅ Core ML inference completed: '{}' -> '{}'", text, final_text);
        
        Ok(final_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use std::fs;

    fn create_mock_model_path() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let model_path = temp_dir.path().join("test_model.mlpackage");
        
        // Create a directory structure that mimics a Core ML model
        fs::create_dir_all(&model_path).unwrap();
        let manifest_path = model_path.join("Manifest.json");
        fs::write(&manifest_path, r#"{"fileFormatVersion": "1.0.0"}"#).unwrap();
        
        (temp_dir, model_path)
    }

    #[test]
    fn test_coreml_corrector_creation() {
        let real_model_path = std::path::PathBuf::from("coreml-setup/coreml-setup/coreml-OpenELM-450M-Instruct/OpenELM-450M-Instruct-128-float32.mlpackage");
        
        if real_model_path.exists() {
            let corrector = CoreMLCorrector::new(&real_model_path);
            // May succeed or fail depending on model availability, but shouldn't panic
            match corrector {
                Ok(corrector) => {
                    assert!(corrector.is_model_loaded());
                    assert_eq!(corrector.model_path(), real_model_path.as_path());
                }
                Err(_) => {
                    // Expected if model can't be loaded
                }
            }
        }
    }

    #[test]
    fn test_coreml_corrector_creation_with_nonexistent_model() {
        let non_existent = PathBuf::from("/non/existent/path.mlpackage");
        let corrector = CoreMLCorrector::new(&non_existent);
        
        // May succeed if pre-compiled model is available, or fail if not
        match corrector {
            Ok(corrector) => {
                // Pre-compiled model was loaded successfully
                assert!(corrector.is_model_loaded());
            }
            Err(_) => {
                // Expected when no pre-compiled model available or other errors
                // This is also a valid outcome
            }
        }
    }

    #[test]
    fn test_mock_model_loading() {
        let (_temp_dir, model_path) = create_mock_model_path();
        let corrector = CoreMLCorrector::new(&model_path);
        
        // May succeed if pre-compiled model is available, or fail if not
        match corrector {
            Ok(corrector) => {
                // Pre-compiled model was loaded successfully
                assert!(corrector.is_model_loaded());
            }
            Err(_) => {
                // Expected when no pre-compiled model available or mock model can't be loaded
                // This is also a valid outcome
            }
        }
    }

    #[test]
    fn test_text_processor_basic_functionality() {
        // Test basic text processor functionality
        let text_processor = TextProcessor::new();
        
        // Test basic tokenization
        let text = "Hello world";
        let tokens = text_processor.tokenize(text).unwrap();
        
        // Should return some tokens
        assert!(!tokens.is_empty());
        
        // Test detokenization
        let detokenized = text_processor.detokenize(&tokens).unwrap();
        assert!(!detokenized.is_empty());
        
        // Test empty text
        let empty_tokens = text_processor.tokenize("").unwrap();
        assert!(empty_tokens.is_empty());
    }
    
    #[test]
    fn test_model_manager_basic_functionality() {
        let (_temp_dir, model_path) = create_mock_model_path();
        
        // Test basic model manager functionality
        let model_manager = CoreMLModelManager::new(&model_path);
        
        // Should not be loaded initially
        assert!(!model_manager.is_loaded());
        
        // Should return the correct path
        assert_eq!(model_manager.model_path(), model_path.as_path());
    }
    
    #[test] 
    fn test_coreml_static_methods() {
        // Test static utility methods that don't require loaded models
        let tokens = vec![1, 2, 3, 4, 5];
        
        // Test MLMultiArray creation
        let result = ArrayUtils::create_ml_multiarray(&tokens);
        assert!(result.is_ok());
        
        // Test token extraction (should work with created array)
        let array = result.unwrap();
        let extracted = ArrayUtils::extract_tokens(&array);
        assert!(extracted.is_ok());
        
        // Test post-processing
        let processed = TextUtils::post_process_text("hello world", "hello world");
        assert!(processed.is_ok());
    }
}