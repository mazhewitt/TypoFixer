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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display() {
        let error = CorrectionError::ModelNotFound {
            path: "/test/path".to_string(),
        };
        assert!(error.to_string().contains("Model file not found"));
        assert!(error.to_string().contains("/test/path"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let correction_error = CorrectionError::from(io_error);
        
        match correction_error {
            CorrectionError::IoError { .. } => {
                assert!(correction_error.to_string().contains("IO error"));
            }
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_all_error_variants() {
        let errors = vec![
            CorrectionError::ModelNotFound { path: "test".to_string() },
            CorrectionError::ModelLoadFailed { 
                path: "test".to_string(), 
                details: "failed".to_string() 
            },
            CorrectionError::ModelNotLoaded,
            CorrectionError::TokenizationFailed { details: "failed".to_string() },
            CorrectionError::ArrayCreationFailed { details: "failed".to_string() },
            CorrectionError::InferenceFailed { details: "failed".to_string() },
            CorrectionError::DecodingFailed { details: "failed".to_string() },
            CorrectionError::PostProcessingFailed { details: "failed".to_string() },
        ];

        // Ensure all errors can be formatted and have proper error messages
        for error in errors {
            let error_string = error.to_string();
            assert!(!error_string.is_empty());
            assert!(!error_string.contains("Error"));  // Should have descriptive messages, not just "Error"
        }
    }
}