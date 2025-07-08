use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};
use toml_edit;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    // The model_path is no longer needed as we use bundled CoreML models.
    // We keep the config file for future settings.
    pub config_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/user".to_string());
        Self {
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
        
        // Config loading is simplified as there are no fields to load yet.
        // This structure is kept for future expansion.
        if !config.config_path.exists() {
            let _ = config.save();
        }
        
        config
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut doc = toml_edit::DocumentMut::new();
        // Add a placeholder or comment to indicate the file's purpose.
        doc.insert("settings", toml_edit::Item::Table(toml_edit::Table::new()));
        doc["settings"].as_table_mut().unwrap().set_implicit(true);
        doc.to_string();

        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&self.config_path, "# TypoFixer configuration file.\n# Settings will be added here in future versions.\n")?;
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
        assert!(config.config_path.to_string_lossy().contains("config.toml"));
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let mut config = Config::default();
        config.config_path = config_path.clone();
        
        // Test save
        config.save().unwrap();
        assert!(config_path.exists());
        
        // Test load
        let loaded_config = Config::load();
        assert_eq!(loaded_config.config_path, Config::default().config_path);
    }
}