# Build-Time Core ML Model Compilation

This project supports **build-time compilation** of Core ML models to eliminate the 30+ second startup delay that would otherwise occur when compiling models at runtime.

## How It Works

### Build Script (`build.rs`)
- **Detects** Core ML models in the project during build
- **Compiles** them using Swift's `MLModel.compileModel(at:)` 
- **Embeds** the compiled model path as a compile-time environment variable
- **Falls back** gracefully if compilation fails

### Runtime Loading (`coreml_corrector.rs`)
1. **First**: Checks for pre-compiled model from build script
2. **If found**: Loads instantly (no compilation delay)
3. **If not found**: Falls back to runtime compilation with user feedback

## Benefits

✅ **Instant app startup** - no 30+ second model compilation wait  
✅ **Better user experience** - app is immediately ready to use  
✅ **Graceful fallback** - still works if build-time compilation fails  
✅ **Clear feedback** - users know when models are loading vs ready  

## Usage

### With a Working Core ML Model

When you have a valid `.mlpackage` file:

```bash
# Build will automatically compile the model
cargo build

# App launches instantly with pre-compiled model
cargo run
```

**Logs will show:**
```
🚀 Found pre-compiled Core ML model at: /path/to/compiled.mlmodelc
✅ Pre-compiled Core ML model loaded successfully!
🎉 Core ML correction engine is ready! You can now use ⌘⌥S to fix typos.
```

### Without a Working Core ML Model

If the model fails to compile at build time:

```bash
# Build warns about compilation failure but succeeds
cargo build
# warning: Failed to compile Core ML model at build time

# App falls back to runtime compilation
cargo run
```

**Logs will show:**
```
📦 No pre-compiled model found, attempting runtime loading/compilation
🔨 Model loading failed, attempting to compile model...
```

## Configuration

### Model Path
Edit `build.rs` to change the source model path:

```rust
let source_model = "path/to/your/model.mlpackage";
```

### Build Environment Variables

- `COMPILED_MODEL_PATH` - Set by build script when compilation succeeds
- Empty or unset when compilation fails

### Requirements

- **Swift** available at build time (for model compilation)
- **Valid Core ML model** in `.mlpackage` format
- **macOS** (Core ML is Apple-specific)

## Troubleshooting

### Build Warnings
```
warning: Core ML model not found at path/to/model.mlpackage, skipping compilation
```
**Solution**: Ensure the model file exists at the specified path

```
warning: Swift not available for Core ML compilation
```  
**Solution**: Install Xcode Command Line Tools: `xcode-select --install`

```
warning: Failed to compile Core ML model at build time
```
**Solution**: Check if your `.mlpackage` file is valid and compatible

### Runtime Fallbacks

Even if build-time compilation fails, the app will:
1. Attempt runtime compilation (with 30+ second delay)
2. Show clear error messages if that also fails
3. Provide guidance on fixing the issue

This ensures the app always works, just with potentially longer startup times.

## Implementation Details

### Build Script Architecture
```rust
// build.rs
if model_exists {
    let swift_script = create_compilation_script();
    let result = compile_with_swift(swift_script);
    
    if success {
        set_env_var("COMPILED_MODEL_PATH", compiled_path);
    } else {
        set_env_var("COMPILED_MODEL_PATH", ""); // Empty = fallback
    }
}
```

### Runtime Detection
```rust
// coreml_corrector.rs
fn load_model() {
    if let Some(compiled_path) = get_precompiled_model_path() {
        // ⚡ Fast path: Use pre-compiled model
        load_compiled_model(compiled_path)
    } else {
        // 🐌 Slow path: Runtime compilation
        compile_and_load_at_runtime()
    }
}
```

This design ensures optimal performance when possible while maintaining reliability as a fallback.