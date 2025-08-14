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

    prompt.push_str("Fix typos in these sentences. Respond only with the corrected sentence, no extra words.\n\n");

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
        #[cfg(target_os = "macos")]
        {
            if verbose {
                info!("🔧 Loading model configuration from: {}", config_path);
            }
            let (qwen_config, effective_base) = Self::find_and_load_qwen_config(
                Some(config_path),
                Some(model_path),
                verbose,
            )?;
            if verbose {
                info!("📁 Loading model from base directory: {}", effective_base.display());
            }
            let model = QwenModel::load_from_directory(&effective_base, Some(qwen_config))
                .context("Failed to load QwenModel from local directory")?;
            if verbose {
                info!("✅ QwenModel loaded successfully from config");
            }
            let prompt_template = PromptTemplate::new();
            return Ok(Self { model, prompt_template, verbose });
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (config_path, model_path, verbose);
            return Err(anyhow::anyhow!(
                "Model loading is only supported on macOS with CoreML. Current platform is not supported."
            ));
        }
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
            let model_path = model_downloader::ensure_model_downloaded(model_id, verbose)
                .context("Failed to download model from HuggingFace Hub")?;
            let (qwen_config, effective_base) = if let Some(cfg) = custom_config {
                (cfg, model_path.clone())
            } else {
                Self::find_and_load_qwen_config(None, Some(model_path.to_str().unwrap()), verbose)?
            };
            if verbose {
                info!("📦 Model downloaded to: {:?}", model_path);
                info!("🔧 Loading QwenModel components...");
            }
            let model = QwenModel::load_from_directory(&effective_base, Some(qwen_config))
                .context("Failed to load QwenModel from downloaded components")?;
            if verbose {
                info!("✅ QwenModel loaded successfully");
            }
            let prompt_template = PromptTemplate::new();
            Ok(Self {
                model,
                prompt_template,
                verbose,
            })
        }
        #[cfg(not(target_os = "macos"))]
        {
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
            let (qwen_config, effective_base) = if let Some(cfg) = custom_config {
                (cfg, path.to_path_buf())
            } else {
                Self::find_and_load_qwen_config(None, Some(model_path), verbose)?
            };
            let model = QwenModel::load_from_directory(&effective_base, Some(qwen_config))
                .context("Failed to load QwenModel from local directory")?;
            if verbose {
                info!("✅ QwenModel loaded successfully from local path");
            }
            let prompt_template = PromptTemplate::new();
            Ok(Self {
                model,
                prompt_template,
                verbose,
            })
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (model_path, verbose, custom_config);
            Err(anyhow::anyhow!(
                "Model loading is only supported on macOS with CoreML. Current platform is not supported."
            ))
        }
    }
    #[cfg(target_os = "macos")]
    fn find_and_load_qwen_config(
        config_path: Option<&str>,
        model_path: Option<&str>,
        verbose: bool,
    ) -> Result<(QwenConfig, PathBuf)> {
        use std::collections::HashMap;
        let mut model_config = if let Some(config_path) = config_path {
            if verbose {
                info!("🔧 Loading model configuration from: {}", config_path);
            }
            ModelConfig::load_from_file(config_path)
                .context("Failed to load model configuration file")?
        } else if let Some(model_path) = model_path {
            let path = Path::new(model_path);
            if let Some(mut mc) = Self::try_discover_cli_model_config(path, verbose) {
                if verbose {
                    info!("🔧 Using discovered CLI configuration");
                }
                Self::adjust_component_paths(&mut mc, path, verbose);
                return Ok((QwenConfig::from_model_config(mc), path.to_path_buf()));
            } else if let Some(mut mc) = get_builtin_config(model_path) {
                if verbose {
                    info!("🔧 Using built-in configuration for model: {}", model_path);
                }
                Self::adjust_component_paths(&mut mc, path, verbose);
                return Ok((QwenConfig::from_model_config(mc), path.to_path_buf()));
            } else {
                if verbose {
                    warn!("⚠️ No built-in configuration found for model: {}", model_path);
                    let available = list_builtin_models();
                    info!("📋 Available built-in models: {:?}", available);
                    info!("🔧 Using default configuration (may fail)");
                }
                return Ok((QwenConfig::default(), path.to_path_buf()));
            }
        } else {
            return Err(anyhow::anyhow!("No config_path or model_path provided to find_and_load_qwen_config"));
        };

        // Try to determine a base directory from component file paths that actually exist.
        let mut candidates: Vec<PathBuf> = Vec::new();
        for comp in model_config.components.values() {
            if let Some(fp) = &comp.file_path {
                let p = PathBuf::from(fp);
                if p.exists() {
                    if let Some(parent) = p.parent() {
                        candidates.push(parent.to_path_buf());
                    }
                }
            }
        }
        let effective_base = if candidates.is_empty() {
            if let Some(model_path) = model_path {
                PathBuf::from(model_path)
            } else {
                PathBuf::new()
            }
        } else {
            let mut counts: HashMap<PathBuf, usize> = HashMap::new();
            for c in candidates {
                *counts.entry(c).or_insert(0) += 1;
            }
            let (best_dir, _) = counts
                .into_iter()
                .max_by_key(|(_, n)| *n)
                .unwrap();
            Self::adjust_component_paths(&mut model_config, &best_dir, verbose);
            best_dir
        };
        Ok((QwenConfig::from_model_config(model_config), effective_base))
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
    fn try_discover_cli_model_config(model_dir: &Path, verbose: bool) -> Option<ModelConfig> {
        use std::env;
        use std::fs;
        let cwd = env::current_dir().ok();
        let mut search_roots = vec![];
        if let Some(c) = &cwd {
            // Walk up the directory tree to find a configs folder from any ancestor
            let mut dir = Some(c.as_path());
            while let Some(d) = dir {
                search_roots.push(d.to_path_buf());
                search_roots.push(d.join("typo-fixer-cli"));
                dir = d.parent();
            }
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
                    if name_candidates
                        .iter()
                        .any(|cand| file_name.contains(cand) || model_hint.contains(cand))
                    {
                        if verbose {
                            info!("🔧 Found candidate CLI config: {}", path.display());
                        }
                        match ModelConfig::load_from_file(&path) {
                            Ok(mc) => {
                                if verbose {
                                    info!("✅ Loaded CLI configuration: {}", path.display());
                                }
                                return Some(mc);
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
    fn extract_correction(&self, raw_output: &str) -> Result<String> {
        extract_correction_from_raw(raw_output, self.verbose)
    }

    /// Generate a correction for the given text (deterministic by default)
    pub async fn fix_text(&mut self, text: &str) -> Result<String> {
        self.fix_text_with_options(text, 0.0, Some(16)).await
    }

    /// Generate a correction with custom temperature and token cap
    pub async fn fix_text_with_options(
        &mut self,
        text: &str,
        temperature: f32,
        max_tokens: Option<usize>,
    ) -> Result<String> {
        // Build prompt
        let prompt = self.prompt_template.create_correction_prompt(text);
        let max_toks = max_tokens.unwrap_or(16);

        // Run generation via CoreML
        let raw = self
            .model
            .generate_text(prompt.as_str(), max_toks, temperature)
            .map_err(|e| anyhow::anyhow!("Generation failed: {e}"))?;

        // Extract + plausibility-check. If implausible, fall back to original text.
        if let Some(chosen) = choose_correction_from_raw(text, &raw) {
            if self.verbose {
                info!("✅ Accepted plausible correction: {:?}", chosen);
            }
            Ok(chosen)
        } else {
            if self.verbose {
                warn!("⚠️ Rejected implausible correction; returning original text");
            }
            Ok(text.to_string())
        }
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

/// Shared plausibility check for corrections used by the UI layer and tests
pub(crate) fn is_correction_plausible(original: &str, corrected: &str) -> bool {
    let orig = original.trim();
    let corr = corrected.trim();

    // Length guard: no more than +50% + 20 chars
    if corr.len() > orig.len() + (orig.len() / 2) + 20 {
        return false;
    }

    // If original has spaces, require corrected to also contain at least one space
    if orig.contains(' ') && !corr.contains(' ') {
        return false;
    }

    // Repetition/low-diversity guard
    let words: Vec<&str> = corr.split_whitespace().collect();
    if words.len() >= 6 {
        let uniq: std::collections::HashSet<&str> = words.iter().copied().collect();
        if uniq.len() <= 2 {
            return false;
        }
    }

    true
}

/// Free function to extract the correction from raw model output (testable without a model)
pub(crate) fn extract_correction_from_raw(raw_output: &str, verbose: bool) -> Result<String> {
    if verbose {
        info!("🔍 Raw model output: {:?}", raw_output);
    }

    // Strategy 0: First non-empty segment before <|endoftext|>
    if let Some(seg) = raw_output
        .split("<|endoftext|>")
        .map(|s| s.trim())
        .find(|s| !s.is_empty())
    {
        if let Some(line) = seg.lines().find(|l| !l.trim().is_empty()) {
            let t = first_sentence(line.trim());
            if !t.starts_with("Input:") && !t.starts_with("Output:") && !t.starts_with("Fix typos") {
                if verbose {
                    info!("✅ Extracted correction (first <|endoftext|> segment): {:?}", t);
                }
                return Ok(t.to_string());
            }
        }
    }

    // Strategy 1: Text after last Output:
    let normalized = raw_output;
    if let Some(pos) = normalized.rfind("Output:") {
        let after = &normalized[pos + "Output:".len()..];
        if let Some(text) = after.lines().find_map(|l| {
            let t = first_sentence(l.trim());
            if !t.is_empty() && !t.starts_with("Input:") && !t.starts_with("Fix typos") {
                Some(t)
            } else {
                None
            }
        }) {
            if verbose {
                info!("✅ Extracted correction: {:?}", text);
            }
            return Ok(text.to_string());
        }
    }

    // Strategy 2: First meaningful line
    let cleaned_output = raw_output;
    let lines: Vec<&str> = cleaned_output.lines().collect();
    for line in &lines {
    let trimmed = first_sentence(line.trim());
        if trimmed.is_empty()
            || trimmed.starts_with("Input:")
            || trimmed.starts_with("Output:")
            || trimmed.starts_with("Fix typos")
            || trimmed.starts_with("the quick brown fox")
            || trimmed.starts_with("i can't believe it")
            || trimmed.starts_with("receive the package")
        {
            continue;
        }
        if verbose {
            info!("✅ Extracted correction: {:?}", trimmed);
        }
        return Ok(trimmed.to_string());
    }

    // Strategy 3: Reverse scan for last meaningful non-empty line
    for line in lines.iter().rev() {
    let trimmed = first_sentence(line.trim());
        if trimmed.is_empty()
            || trimmed.starts_with("Input:")
            || trimmed.starts_with("Output:")
            || trimmed.starts_with("Fix typos")
        {
            continue;
        }
        return Ok(trimmed.to_string());
    }

    // Fallback
    let fallback = first_sentence(cleaned_output.trim());
    if fallback.is_empty() {
        return Err(anyhow::anyhow!("Model produced empty output"));
    }
    Ok(fallback.to_string())
}

/// Keep only the first complete sentence from a string.
fn first_sentence(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if matches!(b as char, '.' | '!' | '?') {
            // Include the terminator
            return &s[..=i];
        }
    }
    s
}

/// Combine extraction and plausibility for testing end-to-end (without UI)
pub(crate) fn choose_correction_from_raw(original: &str, raw_output: &str) -> Option<String> {
    let extracted = extract_correction_from_raw(raw_output, true).ok()?;
    if is_correction_plausible(original, &extracted) {
        Some(extracted)
    } else {
        None
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
        // Prefer the CoreML-packaged model by default for macOS tests
        let model_id = model_id
            .unwrap_or_else(|| "mazhewitt/qwen-typo-fixer-coreml".to_string());

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
        match result {
            Ok(corrected) => {
                // Defense in depth: apply plausibility gating here, too
                if !is_correction_plausible(text, &corrected) {
                    warn!("⚠️ Wrapper rejected implausible correction, falling back to original");
                    return Ok(text.to_string());
                }
                info!("Generated correction: '{}'", corrected);
                Ok(corrected)
            }
            Err(e) => {
                info!("Typo correction failed: {}", e);
                // Return original text if correction fails
                Ok(text.to_string())
            }
        }
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
    assert!(prompt.contains("Fix typos in these sentences. Respond only with the corrected sentence, no extra words."));
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

    #[test]
    fn test_extraction_logged_raw_prefers_sentence_segment() {
        // From logs: model produced multiple segments; we expect the first meaningful sentence
        let raw = " This sentance has typos.<|endoftext|>This sentance has typoos.<|endoftext|>This sentance has typoos.<|endoftext|>This sentance has typoos.<|endoftext|>This sent sent sent sent";
        let got = extract_correction_from_raw(raw, true).expect("should extract");
        assert_eq!(got, "This sentance has typos.");
    }

    #[test]
    fn test_choose_correction_rejects_the_only_output() {
        let raw = "<|endoftext|><|endoftext|>The<|endoftext|>The<|endoftext|>The.<|endoftext|>The.";
        let original = "This sentance has typoos ";
        // Currently extraction would pick "The", but plausibility should reject it
        let chosen = choose_correction_from_raw(original, raw);
        assert!(chosen.is_none(), "Should reject implausible 'The' correction");
    }

    #[test]
    fn test_plausibility_rejects_single_word_when_original_has_space() {
        let original = "This sentance has typoos ";
        let candidate = "The";
        assert!(!is_correction_plausible(original, candidate));
    }
}
