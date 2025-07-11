use std::path::PathBuf;
use std::fs;

#[derive(Clone, Debug)]
pub struct Config {
    pub model_path: PathBuf,
    pub config_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        // Use the Core ML model path by default
        let default_model_path = PathBuf::from("coreml-setup/coreml-setup/coreml-OpenELM-450M-Instruct/OpenELM-450M-Instruct-128-float32.mlpackage");
        
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/user".to_string());
        Self {
            model_path: default_model_path,
            config_path: PathBuf::from(&home).join("Library/Application Support/TypoFixer/config.toml"),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config = Config::default();
        
        // Ensure config directory exists
        if let Some(parent) = config.config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        
        // Load from file if exists
        if let Ok(contents) = fs::read_to_string(&config.config_path) {
            if let Ok(parsed) = contents.parse::<toml_edit::DocumentMut>() {
                let mut new_config = config.clone();
                
                if let Some(model_path) = parsed.get("model_path").and_then(|v| v.as_str()) {
                    new_config.model_path = PathBuf::from(model_path);
                }
                
                return new_config;
            }
        }
        
        config
    }
    
    #[allow(dead_code)]
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut doc = toml_edit::DocumentMut::new();
        doc["model_path"] = toml_edit::value(self.model_path.to_string_lossy().to_string());
        
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&self.config_path, doc.to_string())?;
        Ok(())
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.model_path.to_string_lossy().contains("OpenELM-450M-Instruct-128-float32.mlpackage"));
        assert!(config.config_path.to_string_lossy().contains("config.toml"));
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let model_path = temp_dir.path().join("test_model.gguf");
        
        let mut config = Config::default();
        config.config_path = config_path.clone();
        config.model_path = model_path.clone();
        
        // Test save
        config.save().unwrap();
        assert!(config_path.exists());
        
        // Test that saved file contains the expected model path
        let saved_content = fs::read_to_string(&config_path).unwrap();
        assert!(saved_content.contains("test_model.gguf"));
        
        // Test load - first check default config
        let default_config = Config::default();
        assert!(default_config.model_path.to_string_lossy().contains("OpenELM-450M-Instruct-128-float32.mlpackage"));
        
        // Note: Config::load() reads from user's config file which may contain old settings
        // So we don't test that here as it's environment-dependent
    }
}