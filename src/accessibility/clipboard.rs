use cocoa::base::id;
use cocoa::appkit::NSPasteboard;
use objc::{msg_send, sel, sel_impl};
use std::ffi;
use std::process::Command;
use std::thread;
use std::time::Duration;
use tracing::info;

#[cfg(test)]
use mockall::automock;

/// Trait for clipboard backend operations
#[cfg_attr(test, automock)]
pub trait ClipboardBackend {
    /// Get current clipboard text content
    fn get_text(&self) -> Result<Option<String>, Box<dyn std::error::Error>>;
    
    /// Set clipboard text content
    fn set_text(&self, text: &str) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Send a key command (e.g., "keystroke \"c\" using command down")
    fn send_key(&self, key_command: &str) -> Result<(), Box<dyn std::error::Error>>;
}

/// System clipboard implementation using Cocoa/AppleScript
pub struct SystemClipboard;

impl ClipboardBackend for SystemClipboard {
    /// Get current clipboard text content
    fn get_text(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        unsafe {
            let _pool = cocoa::foundation::NSAutoreleasePool::new(cocoa::base::nil);
            let pasteboard = NSPasteboard::generalPasteboard(cocoa::base::nil);
            Ok(Self::get_clipboard_text(pasteboard))
        }
    }

    /// Set clipboard text content
    fn set_text(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let _pool = cocoa::foundation::NSAutoreleasePool::new(cocoa::base::nil);
            let pasteboard = NSPasteboard::generalPasteboard(cocoa::base::nil);
            Self::set_clipboard_text(pasteboard, text);
            Ok(())
        }
    }

    /// Send key combination via AppleScript
    fn send_key(&self, key_command: &str) -> Result<(), Box<dyn std::error::Error>> {
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
}

impl SystemClipboard {
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

/// Clipboard operations manager that works with any backend
pub struct ClipboardManager<B: ClipboardBackend> {
    backend: B,
}

impl<B: ClipboardBackend> ClipboardManager<B> {
    /// Create a new ClipboardManager with the given backend
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Get current clipboard text content
    pub fn get_text(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        self.backend.get_text()
    }

    /// Set clipboard text content
    pub fn set_text(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.backend.set_text(text)
    }

    /// Extract text from current focused field via clipboard
    pub fn extract_text_via_clipboard(&self) -> Result<String, Box<dyn std::error::Error>> {
        info!("🔄 Attempting clipboard fallback for text extraction...");
        
        // Save current clipboard content
        let old_clipboard = self.get_text()?;
        
        // Select all text and copy
        self.send_select_all()?;
        thread::sleep(Duration::from_millis(50));
        
        self.send_copy()?;
        thread::sleep(Duration::from_millis(100));
        
        // Get the copied text
        let copied_text = self.get_text()?;
        
        // Restore old clipboard content
        if let Some(old_content) = old_clipboard {
            self.set_text(&old_content)?;
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
    pub fn set_text_via_clipboard(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("📋 Attempting clipboard fallback for text setting");
        
        // Copy corrected text to clipboard using the backend
        self.set_text(text)?;
        thread::sleep(Duration::from_millis(100));
        
        // Select all and paste
        self.send_select_all()?;
        thread::sleep(Duration::from_millis(100));
        
        self.send_paste()?;
        
        info!("📋 Successfully set text via clipboard");
        Ok(())
    }

    /// Send Cmd+A (select all) key combination
    fn send_select_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.backend.send_key("keystroke \"a\" using command down")
    }

    /// Send Cmd+C (copy) key combination
    fn send_copy(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.backend.send_key("keystroke \"c\" using command down")
    }

    /// Send Cmd+V (paste) key combination
    fn send_paste(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.backend.send_key("keystroke \"v\" using command down")
    }
}

/// Type alias for the default system clipboard manager
#[allow(dead_code)]
pub type DefaultClipboardManager = ClipboardManager<SystemClipboard>;

impl DefaultClipboardManager {
    /// Create a new default clipboard manager with system clipboard
    #[allow(dead_code)]
    pub fn new_system() -> Self {
        Self::new(SystemClipboard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_operations() {
        let mut mock_backend = MockClipboardBackend::new();
        
        // Set up expectations
        mock_backend
            .expect_get_text()
            .returning(|| Ok(Some("test content".to_string())));
        
        mock_backend
            .expect_set_text()
            .with(mockall::predicate::eq("hello"))
            .returning(|_| Ok(()));
        
        let manager = ClipboardManager::new(mock_backend);
        
        // Test get_text
        let result = manager.get_text().unwrap();
        assert_eq!(result, Some("test content".to_string()));
        
        // Test set_text
        assert!(manager.set_text("hello").is_ok());
    }

    #[test]
    fn test_extract_text_via_clipboard_without_permissions() {
        let mut mock_backend = MockClipboardBackend::new();
        
        // Set up expectations for the clipboard extraction sequence
        mock_backend
            .expect_get_text()
            .times(2)
            .returning(|| Ok(Some("hello".to_string())));
        
        mock_backend
            .expect_send_key()
            .with(mockall::predicate::eq("keystroke \"a\" using command down"))
            .times(1)
            .returning(|_| Ok(()));
        
        mock_backend
            .expect_send_key()
            .with(mockall::predicate::eq("keystroke \"c\" using command down"))
            .times(1)
            .returning(|_| Ok(()));
        
        mock_backend
            .expect_set_text()
            .with(mockall::predicate::eq("hello"))
            .times(1)
            .returning(|_| Ok(()));
        
        let manager = ClipboardManager::new(mock_backend);
        
        let result = manager.extract_text_via_clipboard();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_set_text_clipboard_only() {
        let mut mock_backend = MockClipboardBackend::new();
        
        // Expect set_text to be called first to copy text to clipboard
        mock_backend
            .expect_set_text()
            .with(mockall::predicate::eq("test text"))
            .times(1)
            .returning(|_| Ok(()));
        
        // Expect send_key to be called for select all (Cmd+A)
        mock_backend
            .expect_send_key()
            .with(mockall::predicate::eq("keystroke \"a\" using command down"))
            .times(1)
            .returning(|_| Ok(()));
        
        // Expect send_key to be called for paste (Cmd+V)
        mock_backend
            .expect_send_key()
            .with(mockall::predicate::eq("keystroke \"v\" using command down"))
            .times(1)
            .returning(|_| Ok(()));
        
        let manager = ClipboardManager::new(mock_backend);
        
        let result = manager.set_text_via_clipboard("test text");
        assert!(result.is_ok());
    }
}