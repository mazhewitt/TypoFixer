use std::path::Path;
use tokenizers::Tokenizer;
use tracing::{info, warn};

use super::errors::CorrectionError;

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
    
    /// Check if a tokenizer is loaded
    pub fn has_tokenizer(&self) -> bool {
        self.tokenizer.is_some()
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

impl Default for TextProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use std::path::PathBuf;

    fn create_temp_dir_with_tokenizer() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let model_path = temp_dir.path().join("model");
        fs::create_dir_all(&model_path).unwrap();
        
        // Create a very basic tokenizer config (this won't actually work but tests the loading path)
        let tokenizer_path = model_path.join("tokenizer.json");
        let basic_config = r#"{"version": "1.0", "model": {"type": "BPE", "vocab": {}, "merges": []}}"#;
        fs::write(&tokenizer_path, basic_config).unwrap();
        
        (temp_dir, model_path)
    }

    #[test]
    fn test_new_text_processor() {
        let processor = TextProcessor::new();
        assert!(!processor.has_tokenizer());
    }

    #[test]
    fn test_default_text_processor() {
        let processor = TextProcessor::default();
        assert!(!processor.has_tokenizer());
    }

    #[test]
    fn test_tokenize_empty_text() {
        let processor = TextProcessor::new();
        let result = processor.tokenize("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_tokenize_whitespace_only() {
        let processor = TextProcessor::new();
        let result = processor.tokenize("   ").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_fallback_tokenize() {
        let processor = TextProcessor::new();
        let tokens = processor.fallback_tokenize("Hi");
        
        // Should convert each character to its ASCII value
        assert_eq!(tokens, vec![72, 105]); // 'H' = 72, 'i' = 105
    }

    #[test]
    fn test_fallback_detokenize() {
        let processor = TextProcessor::new();
        let text = processor.fallback_detokenize(&[72, 105]); // 'H' = 72, 'i' = 105
        
        assert_eq!(text, "Hi");
    }

    #[test]
    fn test_fallback_tokenize_non_ascii() {
        let processor = TextProcessor::new();
        let tokens = processor.fallback_tokenize("Hi🚀");
        
        // Should filter out non-ASCII characters (🚀 is > 127)
        assert_eq!(tokens, vec![72, 105]); // Only 'H' and 'i'
    }

    #[test]
    fn test_fallback_detokenize_invalid_tokens() {
        let processor = TextProcessor::new();
        let text = processor.fallback_detokenize(&[72, 0, 200, 105]); // 0 and 200 are invalid
        
        // Should filter out invalid tokens (0 and > 127)
        assert_eq!(text, "Hi");
    }

    #[test]
    fn test_tokenize_with_fallback() {
        let processor = TextProcessor::new();
        let result = processor.tokenize("Hello").unwrap();
        
        // Should use fallback tokenization (character-based)
        assert!(!result.is_empty());
        assert_eq!(result.len(), 5); // 5 characters
    }

    #[test]
    fn test_detokenize_with_fallback() {
        let processor = TextProcessor::new();
        let tokens = vec![72, 101, 108, 108, 111]; // "Hello"
        let result = processor.detokenize(&tokens).unwrap();
        
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_roundtrip_tokenization() {
        let processor = TextProcessor::new();
        let original_text = "Hello World";
        
        let tokens = processor.tokenize(original_text).unwrap();
        let reconstructed = processor.detokenize(&tokens).unwrap();
        
        // Note: This will only work for ASCII text with fallback tokenization
        assert_eq!(reconstructed, "Hello World");
    }

    #[test]
    fn test_load_tokenizer_nonexistent_paths() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent");
        
        let mut processor = TextProcessor::new();
        let result = processor.load_tokenizer(&nonexistent_path);
        
        // Should succeed (not finding tokenizer is not an error)
        assert!(result.is_ok());
        assert!(!processor.has_tokenizer());
    }

    #[test]
    fn test_load_tokenizer_invalid_file() {
        let temp_dir = TempDir::new().unwrap();
        let model_path = temp_dir.path().join("model");
        fs::create_dir_all(&model_path).unwrap();
        
        // Create invalid tokenizer file
        let tokenizer_path = model_path.join("tokenizer.json");
        fs::write(&tokenizer_path, "invalid json").unwrap();
        
        let mut processor = TextProcessor::new();
        let result = processor.load_tokenizer(&model_path);
        
        // Should succeed even with invalid tokenizer (fallback behavior)
        assert!(result.is_ok());
        assert!(!processor.has_tokenizer());
    }

    #[test]
    fn test_has_tokenizer_initially_false() {
        let processor = TextProcessor::new();
        assert!(!processor.has_tokenizer());
    }
}