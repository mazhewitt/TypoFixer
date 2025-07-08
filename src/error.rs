use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    Accessibility(String),
    SpellCheck(String),
    Config(String),
    Hotkey(String),
    IO(std::io::Error),
    Other(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Accessibility(msg) => write!(f, "Accessibility error: {}", msg),
            AppError::SpellCheck(msg) => write!(f, "Spell check error: {}", msg),
            AppError::Config(msg) => write!(f, "Configuration error: {}", msg),
            AppError::Hotkey(msg) => write!(f, "Hotkey error: {}", msg),
            AppError::IO(err) => write!(f, "IO error: {}", err),
            AppError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::IO(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IO(err)
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Other(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Other(msg.to_string())
    }
}

// Note: From<AppError> for Box<dyn std::error::Error> is automatically implemented
// because AppError implements std::error::Error

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, AppError>;