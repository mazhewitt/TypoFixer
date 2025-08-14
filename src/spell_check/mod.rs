//! Core spell checking functionality using candle-coreml

use anyhow::{Context, Result};
use candle_coreml::{
    get_builtin_config, list_builtin_models, model_downloader, ModelConfig, QwenConfig, QwenModel,
};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Template for typo fixing prompts with few-shot examples
/// Based on the typo-fixer-cli approach
pub struct PromptTemplate {
    examples: Vec<(String, String)>, // (input with typos, corrected output)
}

impl PromptTemplate {
    /// Create a new prompt template with default few-shot examples
    pub fn new() -> Self {
        let examples = vec![
            (
                "the quik brown fox".to_string(),
                "the quick brown fox".to_string(),
            ),
            (
                "i cant beleive it".to_string(),
                "i can't believe it".to_string(),
            ),
            (
                "recieve the package".to_string(),
                "receive the package".to_string(),
            ),
            (
                "seperate the items".to_string(),
                "separate the items".to_string(),
            ),
            (
                "occured yesterday".to_string(),
                "occurred yesterday".to_string(),
            ),
        ];

        Self { examples }
    }

    /// Create a typo correction prompt using the format that works with the model
    pub fn create_correction_prompt(&self, input_text: &str) -> String {
        self.create_few_shot_prompt(input_text)
    }

    /// Create a typo correction prompt with few-shot examples
    pub fn create_few_shot_prompt(&self, input_text: &str) -> String {
        let mut prompt = String::new();

        prompt.push_str("Fix typos in these sentences:\n\n");

        // Add few-shot examples
        for (typo_text, correct_text) in &self.examples {
            prompt.push_str(&format!("Input: {typo_text}\nOutput: {correct_text}\n\n"));
        }

        // Add the actual input to be corrected
        prompt.push_str(&format!("Input: {input_text}\nOutput:"));

        prompt
    }

    /// Add a custom example to the prompt template
    pub fn add_example(&mut self, input_with_typos: String, corrected_output: String) {
        self.examples.push((input_with_typos, corrected_output));
    }

    /// Clear all examples (for custom prompting)
    pub fn clear_examples(&mut self) {
        self.examples.clear();
    }

    /// Get the number of examples currently in the template
    pub fn example_count(&self) -> usize {
        self.examples.len()
    }
}

impl Default for PromptTemplate {
    fn default() -> Self {
        Self::new()
    }
}

/// Core typo fixer that uses QwenModel from candle-coreml
pub struct TypoFixer {
    model: QwenModel,
    prompt_template: PromptTemplate,
    verbose: bool,
}

impl TypoFixer {
    /// Create a new TypoFixer by loading the model from HuggingFace Hub
    pub async fn new(model_id: &str, verbose: bool) -> Result<Self> {
        Self::new_with_config(model_id, verbose, None).await
    }

    /// Create a new TypoFixer by loading the model from a local directory
    pub async fn new_from_local(model_path: &str, verbose: bool) -> Result<Self> {
        Self::load_local_model(model_path, verbose, None).await
    }

    /// Create a new TypoFixer by loading the model with a configuration file
    pub async fn new_with_config_file(
        config_path: &str,
        model_path: &str,
        verbose: bool,
    ) -> Result<Self> {
        if verbose {
            info!("🔧 Loading model configuration from: {}", config_path);
        }

        let model_config = ModelConfig::load_from_file(config_path)
            .context("Failed to load model configuration file")?;

        let qwen_config = QwenConfig::from_model_config(model_config);

        Self::load_local_model(model_path, verbose, Some(qwen_config)).await
    }

    /// Create a new TypoFixer with custom naming configuration
    pub async fn new_with_config(
        model_id: &str,
        verbose: bool,
        custom_config: Option<QwenConfig>,
    ) -> Result<Self> {
        if verbose {
            info!("🔄 Loading typo-fixing model: {}", model_id);
        }

        Self::load_model(model_id, verbose, custom_config).await
    }

    /// Load the model from HuggingFace Hub using candle-coreml
    async fn load_model(
        model_id: &str,
        verbose: bool,
        custom_config: Option<QwenConfig>,
    ) -> Result<Self> {
        #[cfg(target_os = "macos")]
        {
            if verbose {
                info!("📥 Downloading model from HuggingFace Hub...");
            }

            // Download the model from HuggingFace Hub
            let model_path = model_downloader::ensure_model_downloaded(model_id, verbose)
                .context("Failed to download model from HuggingFace Hub")?;

            if verbose {
                info!("📦 Model downloaded to: {:?}", model_path);
                info!("🔧 Loading QwenModel components...");
            }

            // Use custom config if provided, otherwise try to get config by model ID
            let config = custom_config.unwrap_or_else(|| {
                if verbose {
                    info!(
                        "🔍 Looking for built-in configuration for model: {}",
                        model_id
                    );
                }
                if let Some(mut model_config) = get_builtin_config(model_id) {
                    if verbose {
                        info!("🔧 Using built-in configuration for model: {}", model_id);
                    }
                    // Attempt to patch component file paths relative to downloaded directory
                    if let Some(path_str) = model_path.to_str() {
                        let base = std::path::Path::new(path_str);
                        Self::adjust_component_paths(&mut model_config, base, verbose);
                    }
                    QwenConfig::from_model_config(model_config)
                } else {
                    if verbose {
                        warn!("⚠️ No built-in configuration found for model: {}", model_id);
                        let available = list_builtin_models();
                        info!("📋 Available built-in models: {:?}", available);
                        info!("🔧 Using default configuration (may fail)");
                    }
                    QwenConfig::default()
                }
            });

            // Load the QwenModel from the downloaded components
            let model = QwenModel::load_from_directory(&model_path, Some(config))
                .context("Failed to load QwenModel from downloaded components")?;

            if verbose {
                info!("✅ QwenModel loaded successfully");
            }

            // Create the prompt template with examples suitable for typo fixing
            let prompt_template = PromptTemplate::new();

            Ok(Self {
                model,
                prompt_template,
                verbose,
            })
        }

        #[cfg(not(target_os = "macos"))]
        {
            // On non-macOS platforms, return an appropriate error
            let _ = (model_id, verbose, custom_config);
            Err(anyhow::anyhow!(
                "Model loading is only supported on macOS with CoreML. Current platform is not supported."
            ))
        }
    }

    /// Load the model from a local directory using candle-coreml
    async fn load_local_model(
        model_path: &str,
        verbose: bool,
        custom_config: Option<QwenConfig>,
    ) -> Result<Self> {
        #[cfg(target_os = "macos")]
        {
            if verbose {
                info!("📁 Loading model from local path: {}", model_path);
            }

            let path = Path::new(model_path);
            if !path.exists() {
                return Err(anyhow::anyhow!("Model path does not exist: {}", model_path));
            }

            // Use custom config if provided, otherwise try to auto-detect from known CLI configs
            let config = match custom_config {
                Some(cfg) => cfg,
                None => {
                    // 1. Look for explicit candle ModelConfig in model directory
                    if let Some(cfg) = Self::try_load_model_dir_config(path, verbose) {
                        cfg
                    } else if let Some(cfg) = Self::try_discover_cli_config(path, verbose) {
                        cfg
                    } else {
                        if verbose {
                            warn!("⚠️ No explicit config found. Falling back to default QwenConfig (may lack component file paths)");
                        }
                        QwenConfig::default()
                    }
                }
            };

            // Load the QwenModel from the local directory
            let model = QwenModel::load_from_directory(path, Some(config))
                .context("Failed to load QwenModel from local directory")?;

            if verbose {
                info!("✅ QwenModel loaded successfully from local path");
            }

            // Create the prompt template with examples suitable for typo fixing
            let prompt_template = PromptTemplate::new();

            Ok(Self {
                model,
                prompt_template,
                verbose,
            })
        }

        #[cfg(not(target_os = "macos"))]
        {
            // On non-macOS platforms, return an appropriate error
            let _ = (model_path, verbose, custom_config);
            Err(anyhow::anyhow!(
                "Model loading is only supported on macOS with CoreML. Current platform is not supported."
            ))
        }
    }

    #[cfg(target_os = "macos")]
    fn try_load_model_dir_config(model_dir: &Path, verbose: bool) -> Option<QwenConfig> {
        let candidate = model_dir.join("config.json");
        if candidate.exists() {
            if verbose {
                info!("🔧 Found config.json in model directory, attempting to load...");
            }
            match ModelConfig::load_from_file(&candidate) {
                Ok(mut mc) => {
                    if verbose {
                        info!("✅ Loaded configuration from model directory");
                    }
                    Self::adjust_component_paths(&mut mc, model_dir, verbose);
                    Some(QwenConfig::from_model_config(mc))
                }
                Err(e) => {
                    if verbose {
                        warn!("⚠️ Failed to parse model directory config: {e}");
                    }
                    None
                }
            }
        } else {
            None
        }
    }

    #[cfg(target_os = "macos")]
    fn try_discover_cli_config(model_dir: &Path, verbose: bool) -> Option<QwenConfig> {
        use std::env;
        use std::fs;
        let cwd = env::current_dir().ok();
        let mut search_roots = vec![];
        if let Some(c) = &cwd {
            search_roots.push(c.clone());
        }
        // If running from workspace root, also add typo-fixer-cli subfolder
        if let Some(c) = &cwd {
            search_roots.push(c.join("typo-fixer-cli"));
        }

        let model_hint = model_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let name_candidates = ["qwen-typo-fixer-ane", "qwen-typo-fixer", "typo-fixer"];

        for root in search_roots {
            let configs_dir = root.join("configs");
            if !configs_dir.is_dir() {
                continue;
            }
            if verbose {
                info!("🔍 Searching for CLI config in {}", configs_dir.display());
            }
            if let Ok(entries) = fs::read_dir(&configs_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) != Some("json") {
                        continue;
                    }
                    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    // Must match candidate names and hint
                    if name_candidates
                        .iter()
                        .any(|cand| file_name.contains(cand) || model_hint.contains(cand))
                    {
                        if verbose {
                            info!("🔧 Found candidate CLI config: {}", path.display());
                        }
                        match ModelConfig::load_from_file(&path) {
                            Ok(mut mc) => {
                                if verbose {
                                    info!("✅ Loaded CLI configuration: {}", path.display());
                                }
                                Self::adjust_component_paths(&mut mc, model_dir, verbose);
                                return Some(QwenConfig::from_model_config(mc));
                            }
                            Err(e) => {
                                if verbose {
                                    warn!("⚠️ Failed to parse CLI config {}: {e}", path.display());
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    #[cfg(target_os = "macos")]
    fn adjust_component_paths(model_config: &mut ModelConfig, base_dir: &Path, verbose: bool) {
        for (name, comp) in model_config.components.iter_mut() {
            if let Some(fp) = comp.file_path.clone() {
                let original = PathBuf::from(&fp);
                if original.exists() {
                    continue;
                }
                if let Some(fname) = original.file_name() {
                    let candidate = base_dir.join(fname);
                    if candidate.exists() {
                        comp.file_path = Some(candidate.to_string_lossy().to_string());
                        if verbose {
                            info!(
                                "🔄 Updated {} component path -> {}",
                                name,
                                candidate.display()
                            );
                        }
                    }
                }
            }
        }
    }

    /// Fix typos in the given text using default settings
    pub async fn fix_text(&mut self, text: &str) -> Result<String> {
        self.fix_text_with_options(text, 0.1, None).await
    }

    /// Fix typos with custom temperature and max tokens
    pub async fn fix_text_with_options(
        &mut self,
        text: &str,
        temperature: f32,
        max_tokens: Option<usize>,
    ) -> Result<String> {
        if self.verbose {
            info!("🔧 Fixing typos in: '{}'", text);
            info!(
                "   Temperature: {}, Max tokens: {:?}",
                temperature, max_tokens
            );
        }

        // Create the prompt using few-shot examples
        let prompt = self.prompt_template.create_correction_prompt(text);

        if self.verbose {
            info!("📝 Generated prompt:\n{}", prompt);
        }

        // Generate the correction using the model
        let max_tokens = max_tokens.unwrap_or(50);

        let corrected_text = if temperature == 0.0 {
            // Use deterministic generation (greedy)
            self.generate_deterministic(&prompt, max_tokens).await?
        } else {
            // Use temperature sampling
            self.generate_with_temperature(&prompt, temperature, max_tokens)
                .await?
        };

        // Post-process the output to extract just the corrected text
        let cleaned = self.extract_correction(&corrected_text)?;

        if self.verbose {
            info!("✅ Corrected: '{}' -> '{}'", text, cleaned);
        }

        Ok(cleaned)
    }

    /// Generate text using deterministic (greedy) sampling
    async fn generate_deterministic(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        if self.verbose {
            info!("🎯 Using deterministic (greedy) generation");
        }

        // Use candle-coreml's generate_text method with temperature 0.0
        let generated_text = self
            .model
            .generate_text(prompt, max_tokens, 0.0)
            .context("Failed to generate tokens deterministically")?;

        Ok(generated_text)
    }

    /// Generate text using temperature sampling
    async fn generate_with_temperature(
        &mut self,
        prompt: &str,
        temperature: f32,
        max_tokens: usize,
    ) -> Result<String> {
        if self.verbose {
            info!(
                "🎲 Using temperature sampling (temperature: {})",
                temperature
            );
        }

        // Use candle-coreml's generate_text method with specified temperature
        let generated_text = self
            .model
            .generate_text(prompt, max_tokens, temperature)
            .context("Failed to generate tokens with temperature")?;

        Ok(generated_text)
    }

    /// Extract the corrected text from the model's output
    fn extract_correction(&self, raw_output: &str) -> Result<String> {
        if self.verbose {
            info!("🔍 Raw model output: {:?}", raw_output);
        }

        // First, stop at the first <|endoftext|> token
        let cleaned_output = if let Some(end_pos) = raw_output.find("<|endoftext|>") {
            &raw_output[..end_pos]
        } else {
            raw_output
        };

        // Also handle other potential stopping patterns
        let cleaned_output = cleaned_output
            .split("Input:")
            .next()
            .unwrap_or(cleaned_output);
        let cleaned_output = cleaned_output
            .split("Output:")
            .next()
            .unwrap_or(cleaned_output);

        // Split into lines and find the actual correction
        let lines: Vec<&str> = cleaned_output.lines().collect();

        // Look for the first meaningful line that looks like corrected text
        for line in lines {
            let trimmed = line.trim();

            // Skip empty lines and prompt-like text
            if trimmed.is_empty()
                || trimmed.starts_with("Input:")
                || trimmed.starts_with("Output:")
                || trimmed.starts_with("Fix typos")
                || trimmed.starts_with("the quick brown fox") // Skip template examples
                || trimmed.starts_with("i can't believe it")
                || trimmed.starts_with("receive the package")
            {
                continue;
            }

            // If this looks like actual content, use it
            if self.verbose {
                info!("✅ Extracted correction: {:?}", trimmed);
            }
            return Ok(trimmed.to_string());
        }

        // Fallback: clean up the entire output
        let fallback = cleaned_output.trim();
        if fallback.is_empty() {
            return Err(anyhow::anyhow!("Model produced empty output"));
        }

        if self.verbose {
            warn!("⚠️ Using fallback extraction: {:?}", fallback);
        }
        Ok(fallback.to_string())
    }

    /// Get mutable reference to the prompt template for customization
    pub fn prompt_template_mut(&mut self) -> &mut PromptTemplate {
        &mut self.prompt_template
    }

    /// Get reference to the prompt template
    pub fn prompt_template(&self) -> &PromptTemplate {
        &self.prompt_template
    }
}

/// Main library interface for fixing typos using candle-coreml.
/// Wrapper around TypoFixer to match typo-fixer-cli patterns
pub struct TypoFixerLib {
    typo_fixer: TypoFixer,
}

impl TypoFixerLib {
    /// Create a new TypoFixerLib instance by downloading a model from HuggingFace Hub.
    pub async fn new(model_id: Option<String>, verbose: bool) -> Result<Self> {
        let model_id = model_id.unwrap_or_else(|| "mazhewitt/qwen-typo-fixer".to_string());

        let typo_fixer = TypoFixer::new(&model_id, verbose).await?;

        Ok(Self { typo_fixer })
    }

    /// Create a new TypoFixerLib instance from a local model directory.
    pub async fn new_from_local(model_path: String, verbose: bool) -> Result<Self> {
        let typo_fixer = TypoFixer::new_from_local(&model_path, verbose).await?;

        Ok(Self { typo_fixer })
    }

    /// Create a new TypoFixerLib instance with an explicit configuration file.
    pub async fn new_with_config_file(
        config_path: &str,
        model_path: &str,
        verbose: bool,
    ) -> Result<Self> {
        let typo_fixer = TypoFixer::new_with_config_file(config_path, model_path, verbose).await?;

        Ok(Self { typo_fixer })
    }

    /// Fix typos in the given text using default settings.
    pub async fn fix_typos(&mut self, text: &str) -> Result<String> {
        self.typo_fixer.fix_text(text).await
    }

    /// Fix typos with custom generation parameters.
    pub async fn fix_typos_with_options(
        &mut self,
        text: &str,
        temperature: f32,
        max_tokens: Option<usize>,
    ) -> Result<String> {
        self.typo_fixer
            .fix_text_with_options(text, temperature, max_tokens)
            .await
    }
}

/// Generate correction using the model wrapper (for backwards compatibility)
pub async fn generate_correction(text: &str, model: &mut Option<TypoFixerLib>) -> Result<String> {
    info!("Generating correction for: '{}'", text);

    if let Some(ref mut model) = model {
        let result = model.fix_typos(text).await;
        match &result {
            Ok(corrected) => info!("Generated correction: '{}'", corrected),
            Err(e) => {
                info!("Typo correction failed: {}", e);
                // Return original text if correction fails
                return Ok(text.to_string());
            }
        }
        result
    } else {
        Err(anyhow::anyhow!("Model not loaded"))
    }
}

/// Generate correction synchronously (for backward compatibility)
pub fn generate_correction_sync(text: &str, model: &mut Option<TypoFixerLib>) -> Result<String> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(generate_correction(text, model))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_typo_fixer_creation_with_invalid_model() {
        // Test that TypoFixer creation fails gracefully with an invalid model
        let result = TypoFixer::new("nonexistent/invalid-model", false).await;
        assert!(
            result.is_err(),
            "Expected TypoFixer::new to fail with invalid model"
        );

        // Test that the error indicates download or model loading failure
        if let Err(e) = result {
            let error_msg = e.to_string();
            // Should contain information about download failure
            assert!(
                error_msg.contains("Failed to download")
                    || error_msg.contains("Failed to load")
                    || error_msg.contains("not supported"),
                "Error should indicate download or loading failure. Got: {error_msg}"
            );
        }
    }

    #[test]
    fn test_prompt_template_creation() {
        let template = PromptTemplate::new();
        assert_eq!(template.example_count(), 5);
    }

    #[test]
    fn test_create_correction_prompt() {
        let template = PromptTemplate::new();
        let prompt = template.create_correction_prompt("this sentance has typoos");

        // Check that prompt contains few-shot examples
        assert!(prompt.contains("Fix typos in these sentences:"));
        assert!(prompt.contains("Input: the quik brown fox"));
        assert!(prompt.contains("Output: the quick brown fox"));

        // Check that it contains our input
        assert!(prompt.contains("Input: this sentance has typoos"));
        assert!(prompt.ends_with("Output:"));
    }

    #[test]
    fn test_extract_correction_logic_standalone() {
        // Test the extraction logic without needing a TypoFixer instance

        // Test simple case
        let raw_output = "this sentence is corrected";
        let result = extract_correction_standalone(raw_output);
        assert_eq!(result, "this sentence is corrected");

        // Test case with noise
        let raw_output = "Input: test\nOutput: \nthis sentence has corrections\n";
        let result = extract_correction_standalone(raw_output);
        assert_eq!(result, "this sentence has corrections");

        // Test case with <|endoftext|> token
        let raw_output =
            "this sentence has typos<|endoftext|>This sentence has typos<|endoftext|>This sentence";
        let result = extract_correction_standalone(raw_output);
        assert_eq!(result, "this sentence has typos");
    }

    // Standalone function to test extraction logic
    fn extract_correction_standalone(raw_output: &str) -> String {
        // First, stop at the first <|endoftext|> token
        let cleaned_output = if let Some(end_pos) = raw_output.find("<|endoftext|>") {
            &raw_output[..end_pos]
        } else {
            raw_output
        };

        // Split into lines and find the actual correction
        let lines: Vec<&str> = cleaned_output.lines().collect();

        // Look for the first meaningful line that looks like corrected text
        for line in lines {
            let trimmed = line.trim();

            // Skip empty lines and prompt-like text
            if trimmed.is_empty()
                || trimmed.starts_with("Input:")
                || trimmed.starts_with("Output:")
                || trimmed.starts_with("Fix typos")
            {
                continue;
            }

            // Skip template examples from our prompt
            if trimmed.starts_with("the quick brown fox")
                || trimmed.starts_with("i can't believe it")
                || trimmed.starts_with("receive the package")
                || trimmed.starts_with("separate the items")
                || trimmed.starts_with("occurred yesterday")
            {
                continue;
            }

            // If this looks like actual content, use it
            return trimmed.to_string();
        }

        // Fallback: clean up the entire output
        cleaned_output.trim().to_string()
    }
}
