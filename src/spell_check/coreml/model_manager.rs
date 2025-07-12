use std::path::{Path, PathBuf};
use objc2::rc::Retained;
use objc2_core_ml::MLModel;
use objc2_foundation::{NSString, NSURL};
use tracing::info;

use super::errors::CorrectionError;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_model_path() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let model_path = temp_dir.path().join("test_model.mlpackage");
        fs::create_dir_all(&model_path).unwrap();
        (temp_dir, model_path)
    }

    #[test]
    fn test_new_model_manager() {
        let (_temp_dir, model_path) = create_temp_model_path();
        let manager = CoreMLModelManager::new(&model_path);
        
        assert_eq!(manager.model_path(), model_path.as_path());
        assert!(!manager.is_loaded());
    }

    #[test]
    fn test_model_path_getter() {
        let (_temp_dir, model_path) = create_temp_model_path();
        let manager = CoreMLModelManager::new(&model_path);
        
        assert_eq!(manager.model_path(), model_path.as_path());
    }

    #[test]
    fn test_is_loaded_initially_false() {
        let (_temp_dir, model_path) = create_temp_model_path();
        let manager = CoreMLModelManager::new(&model_path);
        
        assert!(!manager.is_loaded());
    }

    #[test]
    fn test_model_access_when_not_loaded() {
        let (_temp_dir, model_path) = create_temp_model_path();
        let manager = CoreMLModelManager::new(&model_path);
        
        let result = manager.model();
        assert!(result.is_err());
        match result.unwrap_err() {
            CorrectionError::ModelNotLoaded => {},
            _ => panic!("Expected ModelNotLoaded error"),
        }
    }

    #[test]
    fn test_load_model_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent.mlpackage");
        let mut manager = CoreMLModelManager::new(&nonexistent_path);
        
        let result = manager.load_model();
        // This test may succeed if pre-compiled model is available, or fail if not
        // Both outcomes are valid depending on build environment
        match result {
            Ok(_) => {
                // Pre-compiled model was loaded successfully
                assert!(manager.is_loaded());
            }
            Err(CorrectionError::ModelNotFound { .. }) => {
                // Expected when no pre-compiled model available
                assert!(!manager.is_loaded());
            }
            Err(_) => {
                // Other errors are acceptable (model compilation issues, etc.)
                assert!(!manager.is_loaded());
            }
        }
    }

    #[test]
    fn test_get_precompiled_model_path() {
        // This will return None in test environment unless COMPILED_MODEL_PATH is set
        let result = CoreMLModelManager::get_precompiled_model_path();
        // Should not panic and should return either None or a valid path
        if let Some(path) = result {
            assert!(!path.is_empty());
        }
    }

    #[test]
    fn test_load_compiled_model_nonexistent() {
        let (_temp_dir, model_path) = create_temp_model_path();
        let mut manager = CoreMLModelManager::new(&model_path);
        
        let result = manager.load_compiled_model("/nonexistent/path");
        assert!(result.is_err());
        match result.unwrap_err() {
            CorrectionError::ModelNotFound { .. } => {},
            _ => panic!("Expected ModelNotFound error"),
        }
    }
}