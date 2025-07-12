//! Core ML text correction modules
//! 
//! This module provides a modular architecture for Core ML-based text correction,
//! with separate components for different concerns:
//! 
//! - `errors`: Structured error types for Core ML operations
//! - `model_manager`: Core ML model loading and lifecycle management
//! - `text_processor`: Text tokenization and detokenization
//! - `array_utils`: MLMultiArray operations and utilities
//! - `text_utils`: Text post-processing and validation utilities

pub mod errors;
pub mod model_manager;
pub mod text_processor;
pub mod array_utils;
pub mod text_utils;

// Re-export commonly used types for convenience
pub use errors::CorrectionError;
pub use model_manager::CoreMLModelManager;
pub use text_processor::TextProcessor;
pub use array_utils::ArrayUtils;
pub use text_utils::TextUtils;