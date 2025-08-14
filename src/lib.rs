// Library interface for typo-fixer
// This exposes modules for testing and potential external use

pub mod accessibility;
pub mod config;
pub mod spell_check;

// Re-export commonly used types
pub use config::Config;
pub use spell_check::{
    generate_correction_sync, PromptTemplate, TypoFixer,
};
