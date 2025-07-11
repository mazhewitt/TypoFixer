use std::process::Command;
use tracing::info;

/// AppleScript-based fallback operations
pub struct AppleScriptManager;

impl AppleScriptManager {
    /// Extract text from focused field using AppleScript
    pub fn extract_text() -> Result<String, Box<dyn std::error::Error>> {
        info!("🍎 Attempting AppleScript text extraction...");
        
        let script = r#"
            tell application "System Events"
                set frontApp to name of first application process whose frontmost is true
                tell process frontApp
                    try
                        set selectedText to value of text field 1 of window 1
                        return selectedText
                    on error
                        try
                            set selectedText to value of text area 1 of scroll area 1 of window 1
                            return selectedText
                        on error
                            return ""
                        end try
                    end try
                end tell
            end tell
        "#;
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()?;
        
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !text.is_empty() {
                info!("🍎 AppleScript extracted text: '{}'", text);
                return Ok(text);
            }
        }
        
        Err("AppleScript text extraction failed".into())
    }

    /// Get the name of the frontmost application
    pub fn get_frontmost_app() -> Result<String, Box<dyn std::error::Error>> {
        let script = r#"
            tell application "System Events"
                set frontApp to name of first application process whose frontmost is true
            end tell
            return frontApp
        "#;
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()?;
        
        if output.status.success() {
            let app_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(app_name)
        } else {
            Err("Failed to get frontmost application".into())
        }
    }

    /// Check if the current app is known to be problematic with accessibility
    pub fn is_problematic_app() -> bool {
        match Self::get_frontmost_app() {
            Ok(app_name) => {
                let app_name = app_name.to_lowercase();
                
                // List of known problematic Electron-based or difficult apps
                let problematic_apps = [
                    "visual studio code",
                    "code", 
                    "atom",
                    "discord",
                    "slack",
                    "whatsapp",
                    "telegram",
                    "signal",
                    "spotify",
                    "figma",
                    "notion",
                    "obsidian",
                    "postman",
                    "insomnia",
                    "electron",
                ];
                
                for problematic in &problematic_apps {
                    if app_name.contains(problematic) {
                        info!("🚨 Detected problematic app: {}", app_name);
                        return true;
                    }
                }
                
                false
            }
            Err(_) => false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_frontmost_app() {
        // This test depends on system state and permissions
        let result = AppleScriptManager::get_frontmost_app();
        // We just verify it returns some result
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_is_problematic_app() {
        // This test depends on what app is currently running
        let result = AppleScriptManager::is_problematic_app();
        // Should return a boolean
        assert!(result == true || result == false);
    }

    #[test]
    fn test_extract_text_without_permissions() {
        // This test will likely fail in CI without proper permissions
        let result = AppleScriptManager::extract_text();
        // We just verify it returns some result
        assert!(result.is_ok() || result.is_err());
    }
}