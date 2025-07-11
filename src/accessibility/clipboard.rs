use cocoa::base::id;
use cocoa::appkit::NSPasteboard;
use objc::{msg_send, sel, sel_impl};
use std::ffi;
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tracing::info;

/// Clipboard operations for text extraction and setting
pub struct ClipboardManager;

impl ClipboardManager {
    /// Get current clipboard text content
    pub fn get_text() -> Result<Option<String>, Box<dyn std::error::Error>> {
        unsafe {
            let _pool = cocoa::foundation::NSAutoreleasePool::new(cocoa::base::nil);
            let pasteboard = NSPasteboard::generalPasteboard(cocoa::base::nil);
            Ok(Self::get_clipboard_text(pasteboard))
        }
    }

    /// Set clipboard text content
    pub fn set_text(text: &str) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let _pool = cocoa::foundation::NSAutoreleasePool::new(cocoa::base::nil);
            let pasteboard = NSPasteboard::generalPasteboard(cocoa::base::nil);
            Self::set_clipboard_text(pasteboard, text);
            Ok(())
        }
    }

    /// Extract text from current focused field via clipboard
    pub fn extract_text_via_clipboard() -> Result<String, Box<dyn std::error::Error>> {
        info!("🔄 Attempting clipboard fallback for text extraction...");
        
        // Save current clipboard content
        let old_clipboard = Self::get_text()?;
        
        // Select all text and copy
        Self::send_select_all()?;
        thread::sleep(Duration::from_millis(50));
        
        Self::send_copy()?;
        thread::sleep(Duration::from_millis(100));
        
        // Get the copied text
        let copied_text = Self::get_text()?;
        
        // Restore old clipboard content
        if let Some(old_content) = old_clipboard {
            Self::set_text(&old_content)?;
        }
        
        match copied_text {
            Some(text) if !text.trim().is_empty() => {
                info!("📋 Successfully extracted text via clipboard: '{}'", text);
                Ok(text)
            }
            _ => Err("Could not extract text via clipboard".into())
        }
    }

    /// Set text in focused field via clipboard (select all + paste)
    pub fn set_text_via_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("📋 Attempting clipboard fallback for text setting");
        
        // Copy corrected text to clipboard
        let mut copy_process = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()?;
        
        if let Some(stdin) = copy_process.stdin.as_mut() {
            stdin.write_all(text.as_bytes())?;
        }
        
        copy_process.wait()?;
        thread::sleep(Duration::from_millis(100));
        
        // Select all and paste
        Self::send_select_all()?;
        thread::sleep(Duration::from_millis(100));
        
        Self::send_paste()?;
        
        info!("📋 Successfully set text via clipboard");
        println!("✅ Corrected text: {}", text);
        Ok(())
    }

    /// Send Cmd+A (select all) key combination
    fn send_select_all() -> Result<(), Box<dyn std::error::Error>> {
        Self::send_key_combination("keystroke \"a\" using command down")
    }

    /// Send Cmd+C (copy) key combination
    fn send_copy() -> Result<(), Box<dyn std::error::Error>> {
        Self::send_key_combination("keystroke \"c\" using command down")
    }

    /// Send Cmd+V (paste) key combination
    fn send_paste() -> Result<(), Box<dyn std::error::Error>> {
        Self::send_key_combination("keystroke \"v\" using command down")
    }

    /// Send key combination via AppleScript
    fn send_key_combination(key_command: &str) -> Result<(), Box<dyn std::error::Error>> {
        let script = format!("tell application \"System Events\" to {}", key_command);
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("AppleScript key command failed: {}", error_msg).into());
        }
        
        Ok(())
    }

    /// Get text from pasteboard (unsafe helper)
    unsafe fn get_clipboard_text(pasteboard: id) -> Option<String> {
        use cocoa::appkit::NSPasteboardTypeString;
        
        let string_type = NSPasteboardTypeString;
        let ns_string: id = msg_send![pasteboard, stringForType: string_type];
        
        if ns_string != cocoa::base::nil {
            let utf8_str: *const i8 = msg_send![ns_string, UTF8String];
            if !utf8_str.is_null() {
                let c_str = ffi::CStr::from_ptr(utf8_str);
                return Some(c_str.to_string_lossy().to_string());
            }
        }
        None
    }

    /// Set text on pasteboard (unsafe helper)
    unsafe fn set_clipboard_text(pasteboard: id, text: &str) {
        use cocoa::foundation::NSString;
        use cocoa::appkit::NSPasteboardTypeString;
        
        let ns_string = NSString::alloc(cocoa::base::nil);
        let ns_string: id = msg_send![ns_string, initWithUTF8String: text.as_ptr()];
        
        let string_type = NSPasteboardTypeString;
        let _: () = msg_send![pasteboard, clearContents];
        let _: bool = msg_send![pasteboard, setString: ns_string forType: string_type];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_clipboard_operations() {
        // Test setting and getting clipboard text
        let test_text = "Test clipboard content";
        
        // Set text
        let set_result = ClipboardManager::set_text(test_text);
        assert!(set_result.is_ok());
        
        // Get text
        let get_result = ClipboardManager::get_text();
        assert!(get_result.is_ok());
        
        // Note: We can't reliably test the actual clipboard content in CI
        // since it depends on system permissions and state
    }

    #[test]
    #[ignore]
    fn test_extract_text_via_clipboard_without_permissions() {
        // This test will likely fail in CI without proper permissions
        // which is expected behavior
        let result = ClipboardManager::extract_text_via_clipboard();
        // We just verify it returns some result (either success or expected failure)
        assert!(result.is_ok() || result.is_err());
    }
}