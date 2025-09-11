//! Core spell checking functionality using candle-coreml

use anyhow::{Context, Result};
use candle_coreml::{UnifiedModelLoader, QwenModel};
use std::path::Path;
use tracing::{info, warn};

/// Template for typo fixing prompts with few-shot examples
/// Based on the typo-fixer-cli approach
pub struct PromptTemplate {
    pub examples: Vec<(String, String)>, // (input with typos, corrected output)
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

    prompt.push_str("Fix typos. Respond with only the corrected text.\n\n");

        // Add only 2 examples to reduce repetition
        let examples = &self.examples[0..2];
        for (typo_text, correct_text) in examples {
            prompt.push_str(&format!("{typo_text} -> {correct_text}\n"));
        }

        // Add the actual input to be corrected
        prompt.push_str(&format!("\n{input_text} ->"));

        prompt
    }

    /// Get the number of examples in the template
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
    pub async fn new(model_id: Option<&str>, verbose: bool) -> Result<Self> {
        let model_id = model_id.unwrap_or("mazhewitt/qwen-typo-fixer-coreml");
        
        if verbose {
            info!("📥 Loading model from HuggingFace Hub: {}", model_id);
        }
        
        let loader = UnifiedModelLoader::new()
            .context("Failed to create UnifiedModelLoader")?;
        let model = loader.load_model(model_id)
            .context("Failed to load model from HuggingFace Hub")?;
        
        if verbose {
            info!("✅ Model loaded successfully");
        }
        
        let prompt_template = PromptTemplate::new();
        Ok(Self {
            model,
            prompt_template,
            verbose,
        })
    }


    /// Create a new TypoFixer with a local model directory
    /// Note: The new API prefers using model IDs, but this method supports local paths
    pub async fn new_with_local_path(
        model_path: &str,
        verbose: bool,
    ) -> Result<Self> {
        #[cfg(target_os = "macos")]
        {
            if verbose {
                info!("📁 Loading model from local directory: {}", model_path);
            }
            
            let _loader = UnifiedModelLoader::new()
                .context("Failed to create UnifiedModelLoader")?;
            
            // For local paths, we could try to use the directory name as a pseudo model-id
            let model_dir = Path::new(model_path);
            let _model_name = model_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("local-model");
            
            // Try to load using the loader, which will handle config generation
            let model = match QwenModel::load_from_directory(&model_dir.to_path_buf(), None) {
                Ok(m) => m,
                Err(e) => {
                    warn!("Failed to load from directory directly: {}. Using fallback method.", e);
                    return Err(anyhow::anyhow!("Local model loading not fully supported with new API. Consider using a HuggingFace model ID instead."));
                }
            };
            
            if verbose {
                info!("✅ Model loaded successfully from local path");
            }
            
            let prompt_template = PromptTemplate::new();
            Ok(Self { model, prompt_template, verbose })
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (model_path, verbose);
            Err(anyhow::anyhow!(
                "Model loading is only supported on macOS with CoreML. Current platform is not supported."
            ))
        }
    }



    /// Fix typos in the given text (deterministic by default)
    pub async fn fix_typos(&mut self, text: &str) -> Result<String> {
        self.fix_typos_with_options(text, 0.0, Some(20)).await
    }

    /// Fix typos with custom temperature and max tokens
    pub async fn fix_typos_with_options(
        &mut self,
        text: &str,
        temperature: f32,
        max_tokens: Option<usize>,
    ) -> Result<String> {
        // Preprocess input: strip trailing whitespace to avoid generation issues
        let cleaned_text = text.trim();
        
        // Build prompt
        let prompt = self.prompt_template.create_correction_prompt(cleaned_text);
        let max_toks = max_tokens.unwrap_or(48);
        
        if self.verbose {
            info!("🔍 Input text: {:?}", text);
            info!("🧹 Cleaned text: {:?}", cleaned_text);
            info!("📝 Prompt length: {} chars", prompt.len());
        }

        // Run generation via CoreML using the new API
        let raw = if temperature < 0.1 {
            // For very low temperature, use the simpler complete_text API
            self.model
                .complete_text(prompt.as_str(), max_toks)
                .map_err(|e| anyhow::anyhow!("Generation failed: {e}"))?
        } else {
            self.model
                .generate_text_with_params(prompt.as_str(), max_toks, temperature, None)
                .map_err(|e| anyhow::anyhow!("Generation failed: {e}"))?
        };

        // Extract + plausibility-check. If implausible, fall back to original text.
        if let Some(chosen) = choose_correction_from_raw(cleaned_text, &raw) {
            if self.verbose {
                info!("✅ Accepted plausible correction: {:?}", chosen);
            }
            Ok(chosen)
        } else {
            if self.verbose {
                warn!("⚠️ Rejected implausible correction; returning original text");
            }
            // Return the original text as provided (not cleaned)
            Ok(text.to_string())
        }
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

    // Hard block obvious corruption artifacts
    let lc = corr.to_lowercase();
    let corruption_patterns = [
        "the the the", // repeated common word
        "sent sent sent",
        "<|endoftext|><|endoftext|>",
        "thethethethe",
    ];
    if corruption_patterns.iter().any(|p| lc.contains(p)) {
        return false;
    }

    // Reject if any word repeats 4+ times consecutively
    let mut last = "";
    let mut run = 0;
    for w in words.iter().copied() {
        if w.eq_ignore_ascii_case(last) {
            run += 1;
            if run >= 3 { // means word appeared 4 times total
                return false;
            }
        } else {
            last = w;
            run = 0;
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

/// Collect multiple candidate corrections from raw output instead of only one.
fn extract_candidate_list(raw_output: &str, verbose: bool) -> Vec<String> {
    let mut candidates: Vec<String> = Vec::new();

    // Strategy A: split by <|endoftext|> and take first meaningful line of each segment
    for seg in raw_output.split("<|endoftext|>") {
        let seg = seg.trim();
        if seg.is_empty() { continue; }
        if let Some(line) = seg.lines().find(|l| !l.trim().is_empty()) {
            let t = line.trim();
            // Stop at first sentence terminator or newline
            let t = if let Some(pos) = t.find(&['.', '!', '?', '\n'][..]) {
                &t[..pos]
            } else {
                t
            };
            if !(t.starts_with("Input:") || t.starts_with("Output:") || t.starts_with("Fix typos") || t.starts_with("->")) {
                candidates.push(t.to_string());
                break; // Take only the first good candidate
            }
        }
    }

    // Strategy B: text after last Output:
    if let Some(pos) = raw_output.rfind("Output:") {
        let after = &raw_output[pos + "Output:".len()..];
        for line in after.lines() {
            let t = first_sentence(line.trim());
            if t.is_empty() { continue; }
            if t.starts_with("Input:") || t.starts_with("Fix typos") { continue; }
            candidates.push(t.to_string());
            break;
        }
    }

    // Strategy C: forward scan lines
    for line in raw_output.lines() {
        let t = first_sentence(line.trim());
        if t.is_empty() { continue; }
        if t.starts_with("Input:") || t.starts_with("Output:") || t.starts_with("Fix typos") { continue; }
        if t.starts_with("the quick brown fox")
            || t.starts_with("i can't believe it")
            || t.starts_with("receive the package") {
            continue;
        }
        candidates.push(t.to_string());
        break;
    }

    // Strategy D: reverse scan lines
    for line in raw_output.lines().rev() {
        let t = first_sentence(line.trim());
        if t.is_empty() { continue; }
        if t.starts_with("Input:") || t.starts_with("Output:") || t.starts_with("Fix typos") { continue; }
        candidates.push(t.to_string());
        break;
    }

    // Strategy E: fallback entire first sentence
    let fallback = first_sentence(raw_output.trim());
    if !fallback.is_empty() {
        candidates.push(fallback.to_string());
    }

    // Deduplicate preserving order
    let mut seen = std::collections::HashSet::new();
    candidates.retain(|c| seen.insert(c.clone()));

    if verbose {
        info!("🔎 Candidate corrections extracted: {:?}", candidates);
    }

    candidates
}

/// Combine extraction and plausibility for testing end-to-end (without UI)
pub(crate) fn choose_correction_from_raw(original: &str, raw_output: &str) -> Option<String> {
    // Try multiple candidates in order and return first plausible one
    let candidates = extract_candidate_list(raw_output, true);
    for cand in candidates {
        if is_correction_plausible(original, &cand) {
            return Some(cand);
        }
    }
    None
}


/// Generate correction synchronously
pub fn generate_correction_sync(text: &str, model: &mut Option<TypoFixer>) -> Result<String> {
    info!("Generating correction for: '{}'", text);

    if let Some(ref mut model) = model {
        let rt = tokio::runtime::Runtime::new()?;
        let result = rt.block_on(model.fix_typos(text));
        match result {
            Ok(corrected) => {
                if !is_correction_plausible(text, &corrected) {
                    warn!("⚠️ Rejected implausible correction, falling back to original");
                    return Ok(text.to_string());
                }
                info!("Generated correction: '{}'", corrected);
                Ok(corrected)
            }
            Err(e) => {
                info!("Typo correction failed: {}", e);
                Ok(text.to_string())
            }
        }
    } else {
        Err(anyhow::anyhow!("Model not loaded"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_typo_fixer_creation_with_invalid_model() {
        // Test that TypoFixer creation fails gracefully with an invalid model
        let result = TypoFixer::new(Some("nonexistent/invalid-model"), false).await;
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

    #[test]
    fn test_choose_correction_multi_candidate_skips_bad_leading() {
        let raw = "<|endoftext|><|endoftext|>The<|endoftext|>This sentance has typos.<|endoftext|>This sent sent sent";
        let original = "This sentance has typoos ";
        let chosen = choose_correction_from_raw(original, raw).expect("should find plausible candidate");
        assert_eq!(chosen, "This sentance has typos.");
    }
}
