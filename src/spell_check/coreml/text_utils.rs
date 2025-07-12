use tracing::info;
use super::errors::CorrectionError;

/// Utilities for text post-processing and cleanup
pub struct TextUtils;

impl TextUtils {
    /// Apply post-processing to corrected text
    pub fn post_process_text(corrected_text: &str, original_text: &str) -> Result<String, CorrectionError> {
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
    
    /// Clean and normalize text before processing
    pub fn normalize_text(text: &str) -> String {
        // Basic text normalization
        text.trim()
            .replace('\r', "")
            .replace('\t', " ")
            .chars()
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }
    
    /// Check if text needs correction based on basic heuristics
    pub fn needs_correction(text: &str) -> bool {
        let normalized = Self::normalize_text(text);
        
        // Basic checks for obvious errors
        if normalized.is_empty() {
            return false;
        }
        
        // Check for common patterns that might indicate errors
        let has_repeated_chars = normalized.contains("  ") || 
                                normalized.contains("..") ||
                                normalized.contains("??") ||
                                normalized.contains("!!");
        
        let has_mixed_case = normalized.chars().any(|c| c.is_lowercase()) && 
                           normalized.chars().any(|c| c.is_uppercase());
        
        // Very basic heuristic - in a real implementation, you'd use more sophisticated checks
        has_repeated_chars || (has_mixed_case && normalized.len() > 1)
    }
    
    /// Calculate similarity score between two texts (0.0 = completely different, 1.0 = identical)
    pub fn similarity_score(text1: &str, text2: &str) -> f32 {
        if text1 == text2 {
            return 1.0;
        }
        
        if text1.is_empty() || text2.is_empty() {
            return 0.0;
        }
        
        // Simple character-based similarity
        let chars1: Vec<char> = text1.chars().collect();
        let chars2: Vec<char> = text2.chars().collect();
        
        let max_len = chars1.len().max(chars2.len());
        let min_len = chars1.len().min(chars2.len());
        
        if max_len == 0 {
            return 1.0;
        }
        
        let mut matches = 0;
        for i in 0..min_len {
            if chars1[i] == chars2[i] {
                matches += 1;
            }
        }
        
        matches as f32 / max_len as f32
    }
    
    /// Validate that a correction is reasonable
    pub fn is_reasonable_correction(original: &str, corrected: &str) -> bool {
        // Basic validation rules
        
        // Don't allow completely empty corrections
        if corrected.trim().is_empty() && !original.trim().is_empty() {
            return false;
        }
        
        // Don't allow corrections that are too different in length
        let length_ratio = corrected.len() as f32 / original.len().max(1) as f32;
        if length_ratio > 3.0 || length_ratio < 0.3 {
            return false;
        }
        
        // Don't allow corrections that are too dissimilar
        let similarity = Self::similarity_score(original, corrected);
        if similarity < 0.2 {
            return false;
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_process_empty_corrected_text() {
        let result = TextUtils::post_process_text("", "original").unwrap();
        assert_eq!(result, "original");
    }

    #[test]
    fn test_post_process_whitespace_only_corrected() {
        let result = TextUtils::post_process_text("   ", "original").unwrap();
        assert_eq!(result, "original");
    }

    #[test]
    fn test_post_process_too_long_correction() {
        let original = "hello";
        let corrected = "this is a very long correction that should be rejected";
        let result = TextUtils::post_process_text(corrected, original).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_post_process_basic_cleaning() {
        let result = TextUtils::post_process_text("  hello world  ", "hello world").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_post_process_preserve_capitalization() {
        let result = TextUtils::post_process_text("hello", "Hello").unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_post_process_preserve_lowercase() {
        let result = TextUtils::post_process_text("Hello", "hello").unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_post_process_multi_word_no_capitalization_change() {
        let result = TextUtils::post_process_text("hello world", "Hello World").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_normalize_text_basic() {
        let result = TextUtils::normalize_text("  hello   world  ");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_normalize_text_tabs_and_returns() {
        let result = TextUtils::normalize_text("hello\tworld\r\n");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_normalize_text_multiple_spaces() {
        let result = TextUtils::normalize_text("hello     world");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_needs_correction_empty() {
        assert!(!TextUtils::needs_correction(""));
    }

    #[test]
    fn test_needs_correction_repeated_chars() {
        assert!(TextUtils::needs_correction("hello  world"));
        assert!(TextUtils::needs_correction("hello..world"));
        assert!(TextUtils::needs_correction("hello??world"));
    }

    #[test]
    fn test_needs_correction_mixed_case() {
        assert!(TextUtils::needs_correction("HeLLo"));
        assert!(!TextUtils::needs_correction("Hello")); // This is normal
        assert!(!TextUtils::needs_correction("HELLO")); // This is normal
        assert!(!TextUtils::needs_correction("hello")); // This is normal
    }

    #[test]
    fn test_similarity_score_identical() {
        assert_eq!(TextUtils::similarity_score("hello", "hello"), 1.0);
    }

    #[test]
    fn test_similarity_score_empty() {
        assert_eq!(TextUtils::similarity_score("", ""), 1.0);
        assert_eq!(TextUtils::similarity_score("hello", ""), 0.0);
        assert_eq!(TextUtils::similarity_score("", "hello"), 0.0);
    }

    #[test]
    fn test_similarity_score_partial() {
        let score = TextUtils::similarity_score("hello", "hallo");
        assert!(score > 0.5); // Should be somewhat similar
        assert!(score < 1.0); // But not identical
    }

    #[test]
    fn test_similarity_score_completely_different() {
        let score = TextUtils::similarity_score("hello", "xyz");
        assert!(score < 0.5); // Should be low similarity
    }

    #[test]
    fn test_is_reasonable_correction_empty_output() {
        assert!(!TextUtils::is_reasonable_correction("hello", ""));
        assert!(TextUtils::is_reasonable_correction("", ""));
    }

    #[test]
    fn test_is_reasonable_correction_length_ratio() {
        // Too long
        assert!(!TextUtils::is_reasonable_correction("hi", "this is way too long for a reasonable correction"));
        
        // Too short (but this passes the length test)
        assert!(TextUtils::is_reasonable_correction("hello world", "hi"));
    }

    #[test]
    fn test_is_reasonable_correction_similarity() {
        // Similar corrections should pass
        assert!(TextUtils::is_reasonable_correction("hello", "hallo"));
        
        // Very different corrections should fail
        assert!(!TextUtils::is_reasonable_correction("hello", "xyz"));
    }

    #[test]
    fn test_is_reasonable_correction_normal_cases() {
        assert!(TextUtils::is_reasonable_correction("teh", "the"));
        assert!(TextUtils::is_reasonable_correction("recieve", "receive"));
        assert!(TextUtils::is_reasonable_correction("seperate", "separate"));
    }
}