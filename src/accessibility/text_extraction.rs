use std::ops::Range;

/// Text boundary detection and extraction utilities
pub struct TextExtractor;

impl TextExtractor {
    /// Extract text around a cursor position with smart boundary detection
    #[allow(dead_code)]
    pub fn extract_around_cursor(text: &str, cursor_pos: usize) -> (String, Range<usize>) {
        let cursor_pos = cursor_pos.min(text.len());
        let chars: Vec<char> = text.chars().collect();
        let (start, end) = Self::find_sentence_boundaries(&chars, cursor_pos);
        
        let extracted = chars[start..end].iter().collect();
        (extracted, start..end)
    }

    /// Extract the last sentence or reasonable chunk from text
    pub fn extract_last_sentence(text: &str) -> (String, Range<usize>) {
        let chars: Vec<char> = text.chars().collect();
        let end = chars.len();
        
        // Check if text ends with punctuation
        let ends_with_punctuation = chars.last()
            .map(|&c| Self::is_sentence_terminator(c))
            .unwrap_or(false);
        
        let start = if ends_with_punctuation {
            Self::find_sentence_start(&chars, end - 1)
        } else {
            Self::find_last_sentence_start(&chars)
        };
        
        let start = Self::skip_leading_whitespace(&chars, start);
        let extracted = chars[start..end].iter().collect();
        (extracted, start..end)
    }

    /// Find smart boundaries around a cursor position
    #[allow(dead_code)]
    fn find_sentence_boundaries(chars: &[char], cursor_pos: usize) -> (usize, usize) {
        let start = Self::find_sentence_start_from_cursor(chars, cursor_pos);
        let end = Self::find_sentence_end_from_cursor(chars, cursor_pos);
        (start, end)
    }

    /// Find sentence start working backwards from cursor
    #[allow(dead_code)]
    fn find_sentence_start_from_cursor(chars: &[char], cursor_pos: usize) -> usize {
        let mut start = 0;
        let max_lookback = 200;
        
        for i in (0..cursor_pos).rev() {
            if i < chars.len() && Self::is_sentence_terminator(chars[i]) {
                // Handle cursor right after punctuation
                if i + 1 == cursor_pos {
                    // Find start of the sentence ending with this punctuation
                    for j in (0..i).rev() {
                        if Self::is_sentence_terminator(chars[j]) {
                            start = (j + 1).min(chars.len());
                            break;
                        }
                        if i - j > max_lookback {
                            start = j;
                            break;
                        }
                    }
                } else {
                    // Start after this punctuation
                    start = (i + 1).min(chars.len());
                }
                break;
            }
            
            // Don't look back too far
            if cursor_pos - i > max_lookback {
                start = i;
                break;
            }
        }
        
        Self::skip_leading_whitespace(chars, start)
    }

    /// Find sentence end working forwards from cursor
    #[allow(dead_code)]
    fn find_sentence_end_from_cursor(chars: &[char], cursor_pos: usize) -> usize {
        let mut end = chars.len();
        let max_lookforward = 200;
        
        for i in cursor_pos..chars.len() {
            if Self::is_sentence_terminator(chars[i]) {
                end = (i + 1).min(chars.len());
                break;
            }
            
            // Don't look forward too far
            if i - cursor_pos > max_lookforward {
                end = i;
                break;
            }
        }
        
        Self::skip_trailing_whitespace(chars, end)
    }

    /// Find the start of the last sentence
    fn find_last_sentence_start(chars: &[char]) -> usize {
        let max_lookback = 300;
        
        for i in (0..chars.len()).rev() {
            if Self::is_sentence_terminator(chars[i]) {
                return (i + 1).min(chars.len());
            }
            
            // Don't look back too far
            if chars.len() - i > max_lookback {
                return i;
            }
        }
        
        0
    }

    /// Find sentence start from a given position
    fn find_sentence_start(chars: &[char], from_pos: usize) -> usize {
        let max_lookback = 300;
        
        for i in (0..from_pos).rev() {
            if Self::is_sentence_terminator(chars[i]) {
                return (i + 1).min(chars.len());
            }
            
            // Don't look back too far
            if from_pos - i > max_lookback {
                return i;
            }
        }
        
        0
    }

    /// Check if character is a sentence terminator
    fn is_sentence_terminator(c: char) -> bool {
        matches!(c, '.' | '!' | '?')
    }

    /// Skip leading whitespace and return new start position
    fn skip_leading_whitespace(chars: &[char], start: usize) -> usize {
        let mut new_start = start;
        while new_start < chars.len() && chars[new_start].is_whitespace() {
            new_start += 1;
        }
        new_start
    }

    /// Skip trailing whitespace and return new end position
    #[allow(dead_code)]
    fn skip_trailing_whitespace(chars: &[char], end: usize) -> usize {
        let mut new_end = end;
        while new_end > 0 && chars[new_end - 1].is_whitespace() {
            new_end -= 1;
        }
        new_end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_around_cursor() {
        let text = "First sentence. Second sentence. Third sentence.";
        let cursor_pos = 25; // In "Second sentence"
        let (extracted, range) = TextExtractor::extract_around_cursor(text, cursor_pos);
        
        assert_eq!(extracted, "Second sentence.");
        assert_eq!(range, 16..32);
    }

    #[test]
    fn test_extract_last_sentence() {
        let text = "First sentence. Second sentence.";
        let (extracted, range) = TextExtractor::extract_last_sentence(text);
        
        assert_eq!(extracted, "Second sentence.");
        assert_eq!(range, 16..32);
    }

    #[test]
    fn test_extract_last_sentence_no_punctuation() {
        let text = "First sentence. Second sentence without punctuation";
        let (extracted, range) = TextExtractor::extract_last_sentence(text);
        
        assert_eq!(extracted, "Second sentence without punctuation");
        assert_eq!(range, 16..51);
    }

    #[test]
    fn test_cursor_after_punctuation() {
        let text = "I have a question?";
        let cursor_pos = 18; // Right after the ?
        let (extracted, range) = TextExtractor::extract_around_cursor(text, cursor_pos);
        
        assert_eq!(extracted, "I have a question?");
        assert_eq!(range, 0..18);
    }

    #[test]
    fn test_long_text_boundary() {
        let long_text = "a".repeat(400);
        let cursor_pos = 200;
        let (extracted, _range) = TextExtractor::extract_around_cursor(&long_text, cursor_pos);
        
        // Should limit to reasonable length
        assert!(extracted.len() <= 400);
    }

    #[test]
    fn test_whitespace_handling() {
        let text = "First sentence.   Second sentence";
        let (extracted, range) = TextExtractor::extract_last_sentence(text);
        
        assert_eq!(extracted, "Second sentence");
        assert_eq!(range, 18..33); // Should skip leading whitespace
    }
}