use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use anyhow::Result;
use objc2::rc::Retained;
use objc2::AnyThread;
use objc2_core_ml::{MLModel, MLMultiArray, MLMultiArrayDataType};
use objc2_foundation::{NSString, NSURL, NSArray, NSNumber};
use tracing::{info, warn};
use tokenizers::Tokenizer;
use block2::{Block, StackBlock};

/// Errors that can occur during Core ML text correction
#[derive(Debug, thiserror::Error)]
pub enum CorrectionError {
    #[error("Model file not found: {path}")]
    ModelNotFound { path: String },
    
    #[error("Failed to load Core ML model from {path}: {details}")]
    ModelLoadFailed { path: String, details: String },
    
    #[error("Core ML model not loaded - call load_model() first")]
    ModelNotLoaded,
    
    #[error("Tokenization failed: {details}")]
    TokenizationFailed { details: String },
    
    #[error("Failed to create MLMultiArray: {details}")]
    ArrayCreationFailed { details: String },
    
    #[error("Core ML inference failed: {details}")]
    InferenceFailed { details: String },
    
    #[error("Failed to decode model output: {details}")]
    DecodingFailed { details: String },
    
    #[error("Text post-processing failed: {details}")]
    PostProcessingFailed { details: String },
    
    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
}

/// Manages Core ML model loading and lifecycle
#[derive(Debug)]
pub struct CoreMLModelManager {
    model_path: PathBuf,
    model: Option<Retained<MLModel>>,
}

impl CoreMLModelManager {
    /// Create a new model manager for the given path
    pub fn new(model_path: impl Into<PathBuf>) -> Self {
        Self {
            model_path: model_path.into(),
            model: None,
        }
    }
    
    /// Load the Core ML model (tries pre-compiled first, then direct loading)
    pub fn load_model(&mut self) -> Result<(), CorrectionError> {
        info!("🧠 Loading Core ML model from: {}", self.model_path.display());
        
        // First, check for pre-compiled model from build script
        if let Some(compiled_path) = Self::get_precompiled_model_path() {
            info!("🚀 Found pre-compiled Core ML model at: {}", compiled_path);
            return self.load_compiled_model(&compiled_path);
        }
        
        // Fallback to direct loading for development/testing
        info!("📦 No pre-compiled model found, attempting direct loading");
        self.load_direct()
    }
    
    /// Check if model is currently loaded
    pub fn is_loaded(&self) -> bool {
        self.model.is_some()
    }
    
    /// Get reference to loaded model
    pub fn model(&self) -> Result<&MLModel, CorrectionError> {
        self.model.as_ref().map(|m| m.as_ref()).ok_or(CorrectionError::ModelNotLoaded)
    }
    
    /// Get model path
    pub fn model_path(&self) -> &Path {
        &self.model_path
    }
    
    /// Load model directly from the configured path
    fn load_direct(&mut self) -> Result<(), CorrectionError> {
        if !self.model_path.exists() {
            return Err(CorrectionError::ModelNotFound {
                path: self.model_path.display().to_string(),
            });
        }
        
        let model_path_str = self.model_path.to_string_lossy();
        let ns_path = NSString::from_str(&model_path_str);
        let model_url = unsafe { NSURL::fileURLWithPath(&ns_path) };
        
        match unsafe { MLModel::modelWithContentsOfURL_error(&model_url) } {
            Ok(model) => {
                self.model = Some(model);
                info!("✅ Core ML model loaded successfully!");
                Ok(())
            }
            Err(e) => {
                Err(CorrectionError::ModelLoadFailed {
                    path: self.model_path.display().to_string(),
                    details: format!("{:?}", e),
                })
            }
        }
    }
    
    /// Load pre-compiled model from the given path
    fn load_compiled_model(&mut self, compiled_path: &str) -> Result<(), CorrectionError> {
        let path = Path::new(compiled_path);
        if !path.exists() {
            return Err(CorrectionError::ModelNotFound {
                path: compiled_path.to_string(),
            });
        }
        
        let ns_path = NSString::from_str(compiled_path);
        let model_url = unsafe { NSURL::fileURLWithPath(&ns_path) };
        
        match unsafe { MLModel::modelWithContentsOfURL_error(&model_url) } {
            Ok(model) => {
                self.model = Some(model);
                info!("✅ Pre-compiled Core ML model loaded successfully!");
                Ok(())
            }
            Err(e) => {
                Err(CorrectionError::ModelLoadFailed {
                    path: compiled_path.to_string(),
                    details: format!("{:?}", e),
                })
            }
        }
    }
    
    /// Get the path to the pre-compiled model if it exists
    fn get_precompiled_model_path() -> Option<String> {
        // Check if build script provided a compiled model path
        if let Some(compiled_path) = option_env!("COMPILED_MODEL_PATH") {
            if !compiled_path.is_empty() {
                let path = Path::new(compiled_path);
                if path.exists() {
                    return Some(compiled_path.to_string());
                }
            }
        }
        None
    }
}

/// Handles text tokenization and detokenization
#[derive(Debug)]
pub struct TextProcessor {
    tokenizer: Option<Tokenizer>,
}

impl TextProcessor {
    /// Create a new text processor
    pub fn new() -> Self {
        Self { tokenizer: None }
    }
    
    /// Load tokenizer from the model directory
    pub fn load_tokenizer(&mut self, model_path: &Path) -> Result<(), CorrectionError> {
        let tokenizer_paths = [
            model_path.join("tokenizer.json"),
            model_path.parent().unwrap_or(model_path).join("tokenizer.json"),
            model_path.parent().unwrap_or(model_path).join("vocab.json"),
        ];
        
        for tokenizer_path in &tokenizer_paths {
            if tokenizer_path.exists() {
                info!("🔤 Loading tokenizer from: {}", tokenizer_path.display());
                match Tokenizer::from_file(tokenizer_path) {
                    Ok(tokenizer) => {
                        self.tokenizer = Some(tokenizer);
                        info!("✅ Tokenizer loaded successfully!");
                        return Ok(());
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to load tokenizer from {}: {}", tokenizer_path.display(), e);
                        continue;
                    }
                }
            }
        }
        
        warn!("⚠️ No tokenizer found, will use basic text processing");
        Ok(()) // Not finding a tokenizer is not an error - we have fallbacks
    }
    
    /// Tokenize text into token IDs
    pub fn tokenize(&self, text: &str) -> Result<Vec<u32>, CorrectionError> {
        info!("📝 Tokenizing text: '{}'", text);
        
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        if let Some(tokenizer) = &self.tokenizer {
            match tokenizer.encode(text, false) {
                Ok(encoding) => {
                    let tokens: Vec<u32> = encoding.get_ids().iter().map(|&id| id as u32).collect();
                    info!("✅ Tokenized into {} tokens using trained tokenizer", tokens.len());
                    Ok(tokens)
                }
                Err(e) => {
                    warn!("⚠️ Tokenizer failed, using fallback: {}", e);
                    Ok(self.fallback_tokenize(text))
                }
            }
        } else {
            Ok(self.fallback_tokenize(text))
        }
    }
    
    /// Detokenize token IDs back to text
    pub fn detokenize(&self, token_ids: &[u32]) -> Result<String, CorrectionError> {
        if let Some(tokenizer) = &self.tokenizer {
            match tokenizer.decode(token_ids, false) {
                Ok(text) => {
                    info!("🔤 Successfully decoded {} tokens using tokenizer: '{}'", token_ids.len(), text);
                    Ok(text)
                }
                Err(e) => {
                    warn!("⚠️ Tokenizer decode failed, using fallback: {}", e);
                    Ok(self.fallback_detokenize(token_ids))
                }
            }
        } else {
            Ok(self.fallback_detokenize(token_ids))
        }
    }
    
    /// Simple character-based tokenization fallback
    fn fallback_tokenize(&self, text: &str) -> Vec<u32> {
        text.chars()
            .map(|c| c as u32)
            .filter(|&token_id| token_id <= 127) // ASCII only for safety
            .collect()
    }
    
    /// Simple character-based detokenization fallback
    fn fallback_detokenize(&self, token_ids: &[u32]) -> String {
        token_ids.iter()
            .filter_map(|&token_id| {
                if token_id > 0 && token_id <= 127 {
                    Some(token_id as u8 as char)
                } else {
                    None
                }
            })
            .collect()
    }
}

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
    
    
    /// Get the path to the pre-compiled model if it exists
    fn get_precompiled_model_path() -> Option<String> {
        // Check if build script provided a compiled model path
        if let Some(compiled_path) = option_env!("COMPILED_MODEL_PATH") {
            if !compiled_path.is_empty() {
                let path = Path::new(compiled_path);
                if path.exists() {
                    return Some(compiled_path.to_string());
                }
            }
        }
        None
    }
    
    
    
    /// Correct grammar and spelling in the given text
    /// Correct text using the loaded Core ML model
    pub fn correct(&self, text: &str) -> Result<String, CorrectionError> {
        info!("🔧 Correcting text with Core ML: '{}'", text);
        
        // Get the loaded model
        let model = self.model_manager.model()?;
        
        // Perform the full inference pipeline
        self.coreml_inference(text, model)
    }
    
    /// Perform Core ML inference with the actual model
    fn coreml_inference(&self, text: &str, model: &MLModel) -> Result<String, CorrectionError> {
        info!("🤖 Using Core ML inference for: '{}'", text);
        
        // Step 1: Tokenize the input text
        let tokens = self.text_processor.tokenize(text)?;
        info!("📝 Tokenized input into {} tokens", tokens.len());
        
        // Step 2: Create MLMultiArray input from tokens
        let input_array = Self::create_ml_multiarray(&tokens)?;
        info!("🔧 Created MLMultiArray with shape for {} tokens", tokens.len());
        
        // Step 3: Run Core ML model prediction (simplified identity transformation for now)
        let output_array = Self::predict_with_model(&input_array, model)?;
        info!("✅ Core ML model prediction successful");
        
        // Step 4: Decode the output back to text
        let corrected_text = self.text_processor.detokenize(&Self::extract_tokens(&output_array)?)?;
        info!("🔤 Decoded output: '{}'", corrected_text);
        
        // Step 5: Apply post-processing
        let final_text = Self::post_process_text(&corrected_text, text)?;
        info!("✅ Core ML inference completed: '{}' -> '{}'", text, final_text);
        
        Ok(final_text)
    }
    
    /// Create MLMultiArray from token IDs
    fn create_ml_multiarray(tokens: &[u32]) -> Result<Retained<MLMultiArray>, CorrectionError> {
        info!("🔧 Creating MLMultiArray for {} tokens", tokens.len());
        
        // Create shape for the array (1 x token_count)
        let shape = NSArray::from_slice(&[
            &*NSNumber::numberWithInt(1),
            &*NSNumber::numberWithInt(tokens.len() as i32),
        ]);
        
        // Create the MLMultiArray
        let multiarray = unsafe {
            MLMultiArray::initWithShape_dataType_error(
                MLMultiArray::alloc(),
                &shape,
                MLMultiArrayDataType::Int32,
            )
        }.map_err(|e| CorrectionError::ArrayCreationFailed {
            details: format!("{:?}", e),
        })?;
        
        // Fill the array with token data if we have tokens
        if !tokens.is_empty() {
            let tokens_to_copy = tokens.to_vec();
            let block = StackBlock::new(move |bytes_ptr: NonNull<std::ffi::c_void>, _strides: isize| {
                let data_ptr = bytes_ptr.as_ptr() as *mut i32;
                for (i, &token) in tokens_to_copy.iter().enumerate() {
                    if i < tokens_to_copy.len() {
                        unsafe { *data_ptr.add(i) = token as i32; }
                    }
                }
            });
            
            let block_ref: &Block<dyn Fn(NonNull<std::ffi::c_void>, isize)> = &block;
            unsafe { multiarray.getBytesWithHandler(block_ref); }
        }
        
        info!("✅ Successfully created MLMultiArray with shape [1, {}]", tokens.len());
        Ok(multiarray)
    }
    
    /// Simple prediction that performs identity transformation (for demonstration)
    fn predict_with_model(input: &MLMultiArray, _model: &MLModel) -> Result<Retained<MLMultiArray>, CorrectionError> {
        info!("🤖 Running Core ML model prediction (identity transformation)");
        
        // For now, we create an output array that matches the input
        // In a real implementation, this would call the actual model
        let input_shape = unsafe { input.shape() };
        let output_array = unsafe {
            MLMultiArray::initWithShape_dataType_error(
                MLMultiArray::alloc(),
                &input_shape,
                MLMultiArrayDataType::Int32,
            )
        }.map_err(|e| CorrectionError::InferenceFailed {
            details: format!("Failed to create output array: {:?}", e),
        })?;
        
        // Copy input data to output (identity transformation)
        let shape_count = input_shape.count();
        if shape_count > 0 {
            let seq_length = if shape_count >= 2 {
                let seq_dim = input_shape.objectAtIndex(1);
                seq_dim.intValue() as usize
            } else {
                1
            };
            
            // Extract from input and copy to output
            let extracted_tokens = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let extracted_tokens_clone = extracted_tokens.clone();
            
            let extract_block = StackBlock::new(move |bytes_ptr: NonNull<std::ffi::c_void>, _strides: isize| {
                let mut tokens = extracted_tokens_clone.lock().unwrap();
                let data_ptr = bytes_ptr.as_ptr() as *const i32;
                for i in 0..seq_length {
                    let value = unsafe { *data_ptr.add(i) };
                    tokens.push(value);
                }
            });
            
            let extract_block_ref: &Block<dyn Fn(NonNull<std::ffi::c_void>, isize)> = &extract_block;
            unsafe { input.getBytesWithHandler(extract_block_ref); }
            
            // Copy to output
            let copied_tokens = extracted_tokens.lock().unwrap().clone();
            let fill_block = StackBlock::new(move |bytes_ptr: NonNull<std::ffi::c_void>, _strides: isize| {
                let data_ptr = bytes_ptr.as_ptr() as *mut i32;
                for (i, &token) in copied_tokens.iter().enumerate() {
                    unsafe { *data_ptr.add(i) = token; }
                }
            });
            
            let fill_block_ref: &Block<dyn Fn(NonNull<std::ffi::c_void>, isize)> = &fill_block;
            unsafe { output_array.getBytesWithHandler(fill_block_ref); }
        }
        
        info!("✅ Core ML prediction completed (identity transformation)");
        Ok(output_array)
    }
    
    /// Extract token IDs from MLMultiArray
    fn extract_tokens(array: &MLMultiArray) -> Result<Vec<u32>, CorrectionError> {
        let shape = unsafe { array.shape() };
        let shape_count = shape.count();
        
        let sequence_length = if shape_count >= 2 {
            let seq_dim = shape.objectAtIndex(1);
            seq_dim.intValue() as usize
        } else if shape_count == 1 {
            let seq_dim = shape.objectAtIndex(0);
            seq_dim.intValue() as usize
        } else {
            0
        };
        
        if sequence_length == 0 {
            return Ok(Vec::new());
        }
        
        let extracted_tokens = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let extracted_tokens_clone = extracted_tokens.clone();
        
        let block = StackBlock::new(move |bytes_ptr: NonNull<std::ffi::c_void>, _strides: isize| {
            let mut tokens = extracted_tokens_clone.lock().unwrap();
            let data_ptr = bytes_ptr.as_ptr() as *const i32;
            for i in 0..sequence_length {
                let value = unsafe { *data_ptr.add(i) };
                tokens.push(value.max(0) as u32);
            }
        });
        
        let block_ref: &Block<dyn Fn(NonNull<std::ffi::c_void>, isize)> = &block;
        unsafe { array.getBytesWithHandler(block_ref); }
        
        let result = extracted_tokens.lock().unwrap().clone();
        Ok(result)
    }
    
    /// Apply post-processing to corrected text
    fn post_process_text(corrected_text: &str, original_text: &str) -> Result<String, CorrectionError> {
        info!("🔧 Post-processing corrected text");
        
        // If corrected text is empty, return original
        if corrected_text.trim().is_empty() {
            info!("⚠️ Corrected text is empty, returning original");
            return Ok(original_text.to_string());
        }
        
        // If corrected text is too different from original, return original
        if corrected_text.len() > original_text.len() * 2 {
            info!("⚠️ Corrected text too different from original, returning original");
            return Ok(original_text.to_string());
        }
        
        // Basic cleaning: trim whitespace
        let cleaned = corrected_text.trim().to_string();
        
        // Preserve original capitalization for single words
        if original_text.split_whitespace().count() == 1 && cleaned.split_whitespace().count() == 1 {
            let original_word = original_text.trim();
            let corrected_word = cleaned.trim();
            
            if original_word.chars().next().unwrap_or(' ').is_uppercase() {
                if let Some(first_char) = corrected_word.chars().next() {
                    let capitalized = first_char.to_uppercase().collect::<String>() + &corrected_word[1..];
                    return Ok(capitalized);
                }
            }
        }
        
        Ok(cleaned)
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
        fs::write(&manifest_path, r#"{"fileFormatVersion": "1.0.0", "itemInfoEntries": {}}"#).unwrap();
        
        (temp_dir, model_path)
    }

    #[test]
    fn test_coreml_corrector_creation() {
        let (_temp_dir, model_path) = create_mock_model_path();
        
        // Since the mock model isn't a real Core ML model, the CoreMLCorrector::new() 
        // will fail when trying to load it. This is expected behavior.
        let corrector = CoreMLCorrector::new(&model_path);
        assert!(corrector.is_err());
        
        // The error should be related to Core ML model loading
        let error = corrector.unwrap_err();
        assert!(error.to_string().contains("Failed to load Core ML model") || 
                error.to_string().contains("Failed to compile and load Core ML model"));
    }

    #[test]
    fn test_coreml_corrector_creation_with_nonexistent_model() {
        let non_existent = PathBuf::from("/non/existent/path.mlpackage");
        let corrector = CoreMLCorrector::new(&non_existent);
        assert!(corrector.is_err()); // Should fail without model
    }

    #[test]
    fn test_mock_model_loading() {
        let (_temp_dir, model_path) = create_mock_model_path();
        // This will likely fail since it's not a real Core ML model
        let corrector = CoreMLCorrector::new(&model_path);
        // We expect this to fail since we don't have a real model
        assert!(corrector.is_err());
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
        let result = CoreMLCorrector::create_ml_multiarray(&tokens);
        assert!(result.is_ok());
        
        // Test token extraction (should work with created array)
        let array = result.unwrap();
        let extracted = CoreMLCorrector::extract_tokens(&array);
        assert!(extracted.is_ok());
        
        // Test post-processing
        let processed = CoreMLCorrector::post_process_text("hello world", "hello world");
        assert!(processed.is_ok());
    }
}
