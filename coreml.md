
Core ML Grammar Correction Implementation Progress

  Current Status

  We're implementing a Core ML-based grammar correction feature to replace Ollama with on-device inference using Apple's Neural Engine.

  What We've Accomplished

  1. ✅ Analyzed existing Ollama-based spell check system in src/spell_check/mod.rs
  2. ✅ Set up Python environment in coreml-setup/ folder with conversion scripts
  3. ✅ Downloaded pre-converted Core ML model - coreml-OpenELM-450M-Instruct from HuggingFace
  4. ✅ Added Core ML dependencies to Cargo.toml (switched from candle to objc2-core-ml)
  5. 🚧 Created Candle corrector wrapper in src/spell_check/candle_corrector.rs (needs fixes)
  6. 🚧 Started CorrectionEngine enum to support both Ollama and Core ML

  Current Issues to Fix

  1. Compilation errors in candle_corrector.rs:
    - Missing #[derive(Debug, Clone)] on CandleCorrector
    - anyhow context method not working with tokenizer Result type
    - Unused variables in tests
  2. Compilation errors in mod.rs:
    - LlamaModelWrapper missing Debug and Clone traits
    - CorrectionEngine enum can't derive traits due to missing implementations

  Next Steps

  1. Fix compilation errors by adding required trait derivations
  2. Switch from Candle to Core ML native APIs using objc2-core-ml
  3. Update CandleCorrector to CoreMLCorrector using the downloaded model
  4. Integrate with existing spell check system in main.rs
  5. Add configuration support for engine selection
  6. Add comprehensive tests

  Files Created/Modified

  - coreml-setup/ - Python conversion environment
  - src/spell_check/candle_corrector.rs - Core ML wrapper (needs Core ML API)
  - src/spell_check/mod.rs - Updated with CorrectionEngine enum
  - Cargo.toml - Added Core ML dependencies

  Model Location

  - Core ML model: coreml-setup/coreml-OpenELM-450M-Instruct/OpenELM-450M-Instruct-128-float32.mlpackage/

  Key Architecture Decision

  Using enum CorrectionEngine to support both Ollama (remote) and Core ML (local) engines, allowing runtime selection based on user preference or availability.