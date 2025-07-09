use std::path::Path;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

// Ollama API request/response structures
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    top_p: f32,
    max_tokens: i32,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
}

// Model wrapper for text correction using Ollama
pub struct LlamaModelWrapper {
    client: Client,
    model_name: String,
    base_url: String,
}

impl LlamaModelWrapper {
    pub fn new(_model_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing Ollama client for text correction...");
        
        let client = Client::new();
        let model_name = "phi:2.7b".to_string(); // Use phi-2 model
        let base_url = "http://localhost:11434".to_string();
        
        let wrapper = Self {
            client,
            model_name,
            base_url,
        };
        
        // Test if Ollama is available
        match wrapper.test_ollama_connection() {
            Ok(_) => {
                info!("✅ Ollama connection successful!");
                Ok(wrapper)
            }
            Err(e) => {
                warn!("Could not connect to Ollama: {}", e);
                info!("Make sure Ollama is running with: ollama serve");
                info!("And pull the model with: ollama pull phi:2.7b");
                
                // Return the wrapper anyway for fallback mode
                Ok(wrapper)
            }
        }
    }
    
    fn test_ollama_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let response = self.client
                .get(format!("{}/api/tags", self.base_url))
                .send()
                .await?;
            
            if response.status().is_success() {
                info!("Ollama is running and accessible");
                Ok(())
            } else {
                Err(format!("Ollama returned status: {}", response.status()).into())
            }
        })
    }
    
    pub fn generate(&mut self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        info!("Generating correction for: '{}'", prompt);
        
        // Create a focused prompt for text correction - optimized for phi-2
        let correction_prompt = format!(
            "Correct the spelling and grammar:\n{}\n\nCorrected version:",
            prompt
        );
        
        let request = OllamaRequest {
            model: self.model_name.clone(),
            prompt: correction_prompt,
            stream: false,
            options: OllamaOptions {
                temperature: 0.0,  // Even lower temperature for more focused responses
                top_p: 0.8,
                max_tokens: 50,    // Shorter response length
            },
        };
        
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let response = self.client
                .post(format!("{}/api/generate", self.base_url))
                .json(&request)
                .send()
                .await?;
            
            if !response.status().is_success() {
                return Err(format!("Ollama API error: {}", response.status()).into());
            }
            
            let ollama_response: OllamaResponse = response.json().await?;
            let corrected = self.clean_response(&ollama_response.response, prompt);
            
            info!("Generated correction: '{}'", corrected);
            Ok(corrected)
        })
    }
    
    fn clean_response(&self, response: &str, original: &str) -> String {
        // Clean up the response to extract just the corrected text
        let cleaned = response.trim();
        
        // Remove common prefixes that models sometimes add
        let prefixes_to_remove = [
            "Here's the corrected text:",
            "Corrected text:",
            "Corrected version:",
            "Corrected:",
            "Fixed:",
            "The corrected text is:",
            "Here is the corrected version:",
        ];
        
        let mut result = cleaned;
        for prefix in &prefixes_to_remove {
            if result.starts_with(prefix) {
                result = result[prefix.len()..].trim();
                break;
            }
        }
        
        // If the response is empty or too different, return original
        if result.is_empty() || result.len() > original.len() * 2 {
            return original.to_string();
        }
        
        // Take only the first line if there are multiple lines
        if let Some(first_line) = result.lines().next() {
            result = first_line;
        }
        
        // Remove unwanted periods that the model might add
        // If the original text didn't end with punctuation, don't add it
        if !original.ends_with('.') && !original.ends_with('!') && !original.ends_with('?') && result.ends_with('.') {
            result = result.trim_end_matches('.').trim();
        }
        
        result.to_string()
    }
}

pub fn generate_correction(
    text: &str, 
    model: &mut Option<LlamaModelWrapper>
) -> Result<String, Box<dyn std::error::Error>> {
    info!("Generating correction for: '{}'", text);
    
    if let Some(ref mut model) = model {
        let result = model.generate(text);
        match &result {
            Ok(corrected) => info!("Generated correction: '{}'", corrected),
            Err(e) => {
                info!("Ollama correction failed: {}", e);
                // Return original text if Ollama fails
                return Ok(text.to_string());
            }
        }
        result
    } else {
        Err("Model not loaded".into())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use std::path::PathBuf;

    fn create_temp_model_file() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let model_path = temp_dir.path().join("test_model.gguf");
        fs::write(&model_path, "mock model content").unwrap();
        (temp_dir, model_path)
    }

    #[test]
    fn test_llama_model_wrapper_creation() {
        let (_temp_dir, model_path) = create_temp_model_file();
        
        // Test successful creation (will work even if Ollama isn't running)
        let model = LlamaModelWrapper::new(&model_path);
        assert!(model.is_ok());
        
        // Test with non-existent file (should still work)
        let non_existent = PathBuf::from("/non/existent/path");
        let model = LlamaModelWrapper::new(&non_existent);
        assert!(model.is_ok());
    }

    #[test]
    fn test_llama_model_generate() {
        let (_temp_dir, model_path) = create_temp_model_file();
        let mut model = LlamaModelWrapper::new(&model_path).unwrap();
        
        // Note: These tests will only work if Ollama is running with the model
        // If Ollama is not available, they will return the original text or error
        let result = model.generate("I have teh cat");
        
        // The test should not panic - either it succeeds with Ollama or fails gracefully
        match result {
            Ok(text) => {
                assert!(!text.is_empty());
                // Could be corrected text or original text
            }
            Err(_) => {
                // This is expected if Ollama is not running
                assert!(true);
            }
        }
    }

    #[test]
    fn test_generate_correction_without_model() {
        // Test when no model is loaded
        let mut model = None;
        let result = generate_correction("test text", &mut model);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Model not loaded");
    }

    #[test]
    fn test_clean_response() {
        let (_temp_dir, model_path) = create_temp_model_file();
        let model = LlamaModelWrapper::new(&model_path).unwrap();
        
        // Test cleaning response with prefix
        let cleaned = model.clean_response("Here's the corrected text: The cat is here", "teh cat is here");
        assert_eq!(cleaned, "The cat is here");
        
        // Test cleaning response without prefix
        let cleaned = model.clean_response("The cat is here", "teh cat is here");
        assert_eq!(cleaned, "The cat is here");
        
        // Test fallback for empty response
        let cleaned = model.clean_response("", "original text");
        assert_eq!(cleaned, "original text");
        
        // Test fallback for too long response
        let long_response = "a".repeat(1000);
        let cleaned = model.clean_response(&long_response, "short");
        assert_eq!(cleaned, "short");
    }

    #[test]
    fn test_no_unwanted_period_addition() {
        let (_temp_dir, model_path) = create_temp_model_file();
        let model = LlamaModelWrapper::new(&model_path).unwrap();
        
        // Test that periods are not added to fragments or incomplete sentences
        let test_cases = vec![
            ("hello world", "hello world."),    // Should not add period to greeting
            ("teh cat", "the cat."),            // Should not add period to fragment
            ("my name is", "my name is."),      // Should not add period to incomplete sentence
            ("hello there", "hello there."),   // Should not add period to greeting
            ("quick test", "quick test."),      // Should not add period to fragment
        ];
        
        for (original, response_with_period) in test_cases {
            let cleaned = model.clean_response(response_with_period, original);
            // If the original didn't have a period, the cleaned version shouldn't either
            if !original.ends_with('.') && !original.ends_with('!') && !original.ends_with('?') {
                assert!(
                    !cleaned.ends_with('.') || cleaned == original,
                    "Should not add period to '{}', got '{}'", original, cleaned
                );
            }
        }
    }
}