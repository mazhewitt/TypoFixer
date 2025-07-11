use accessibility_sys::*;
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::base::{CFTypeRef, TCFType};
use tracing::{debug, warn};

pub type ElementRef = AXUIElementRef;

/// Result type for accessibility operations
pub type AxResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Common accessibility API operations with error handling
pub struct AxApi;

impl AxApi {
    /// Safely wrap a CFString from a raw pointer
    pub fn wrap_cfstring(raw_ref: CFTypeRef) -> Option<CFString> {
        if raw_ref.is_null() {
            None
        } else {
            Some(unsafe { CFString::wrap_under_get_rule(raw_ref as CFStringRef) })
        }
    }

    /// Get attribute value from element
    pub fn get_attribute_value(element: ElementRef, attribute: &str) -> AxResult<Option<CFTypeRef>> {
        unsafe {
            let attr_cfstring = CFString::new(attribute);
            let mut value_ref: CFTypeRef = std::ptr::null();
            
            let result = AXUIElementCopyAttributeValue(
                element,
                attr_cfstring.as_concrete_TypeRef(),
                &mut value_ref
            );
            
            match result {
                kAXErrorSuccess => {
                    if value_ref.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(value_ref))
                    }
                }
                kAXErrorAPIDisabled | kAXErrorNotImplemented => {
                    Err("⚠️  Accessibility permissions not granted. Please:\n1. Go to System Preferences > Security & Privacy > Privacy > Accessibility\n2. Add this application\n3. Try again".into())
                }
                _ => {
                    warn!("AX API failed for attribute '{}': error {}", attribute, result);
                    Err(format!("Failed to get attribute '{}': AX error {}", attribute, result).into())
                }
            }
        }
    }

    /// Set attribute value on element
    pub fn set_attribute_value(element: ElementRef, attribute: &str, value: CFTypeRef) -> AxResult<()> {
        unsafe {
            let attr_cfstring = CFString::new(attribute);
            
            let result = AXUIElementSetAttributeValue(
                element,
                attr_cfstring.as_concrete_TypeRef(),
                value
            );
            
            match result {
                kAXErrorSuccess => Ok(()),
                _ => {
                    warn!("AX API failed to set attribute '{}': error {}", attribute, result);
                    Err(format!("Failed to set attribute '{}': AX error {}", attribute, result).into())
                }
            }
        }
    }

    /// Get system-wide accessibility element
    pub fn get_system_element() -> AxResult<ElementRef> {
        unsafe {
            let system_element = AXUIElementCreateSystemWide();
            if system_element.is_null() {
                Err("Failed to create system element".into())
            } else {
                Ok(system_element)
            }
        }
    }

    /// Get focused application from system element
    pub fn get_focused_application() -> AxResult<ElementRef> {
        let system_element = Self::get_system_element()?;
        
        match Self::get_attribute_value(system_element, "AXFocusedApplication")? {
            Some(app_ref) => {
                Ok(app_ref as ElementRef)
            }
            None => Err("No focused application found".into())
        }
    }

    /// Get element role as string
    pub fn get_element_role(element: ElementRef) -> AxResult<String> {
        match Self::get_attribute_value(element, "AXRole")? {
            Some(role_ref) => {
                let role_cfstring = unsafe { CFString::wrap_under_get_rule(role_ref as CFStringRef) };
                let role_string = role_cfstring.to_string();
                debug!("Element role: {}", role_string);
                Ok(role_string)
            }
            None => Err("Could not determine element role".into())
        }
    }

    /// Get element subrole as string
    pub fn get_element_subrole(element: ElementRef) -> AxResult<Option<String>> {
        match Self::get_attribute_value(element, "AXSubrole")? {
            Some(subrole_ref) => {
                let subrole_cfstring = unsafe { CFString::wrap_under_get_rule(subrole_ref as CFStringRef) };
                Ok(Some(subrole_cfstring.to_string()))
            }
            None => Ok(None)
        }
    }

    /// Get text value from element
    pub fn get_text_value(element: ElementRef) -> AxResult<String> {
        match Self::get_attribute_value(element, "AXValue")? {
            Some(value_ref) => {
                let value_cfstring = unsafe { CFString::wrap_under_get_rule(value_ref as CFStringRef) };
                Ok(value_cfstring.to_string())
            }
            None => Err("Could not read text from element".into())
        }
    }

    /// Get selected text from element
    pub fn get_selected_text(element: ElementRef) -> AxResult<Option<String>> {
        match Self::get_attribute_value(element, "AXSelectedText")? {
            Some(selected_ref) => {
                let selected_cfstring = unsafe { CFString::wrap_under_get_rule(selected_ref as CFStringRef) };
                let selected_text = selected_cfstring.to_string();
                if selected_text.trim().is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(selected_text))
                }
            }
            None => Ok(None)
        }
    }

    /// Set text value on element
    pub fn set_text_value(element: ElementRef, text: &str) -> AxResult<()> {
        let new_text_cfstring = CFString::new(text);
        Self::set_attribute_value(element, "AXValue", new_text_cfstring.as_concrete_TypeRef() as CFTypeRef)
    }

    /// Set selected text on element
    pub fn set_selected_text(element: ElementRef, text: &str) -> AxResult<()> {
        let new_text_cfstring = CFString::new(text);
        Self::set_attribute_value(element, "AXSelectedText", new_text_cfstring.as_concrete_TypeRef() as CFTypeRef)
    }

    /// Check if element is a text-editable field
    pub fn is_text_editable(element: ElementRef) -> bool {
        match Self::get_element_role(element) {
            Ok(role) => matches!(role.as_str(), 
                "AXTextField" | "AXTextArea" | "AXSecureTextField" | "AXComboBox"
            ),
            Err(_) => false
        }
    }

    /// Check if element is a secure text field
    pub fn is_secure_field(element: ElementRef) -> bool {
        // Check role first
        if let Ok(role) = Self::get_element_role(element) {
            if role == "AXSecureTextField" {
                debug!("🔒 Detected secure text field");
                return true;
            }
        }

        // Check subrole
        if let Ok(Some(subrole)) = Self::get_element_subrole(element) {
            if subrole.contains("Password") || subrole.contains("Secure") {
                debug!("🔒 Detected secure field by subrole: {}", subrole);
                return true;
            }
        }

        false
    }
}