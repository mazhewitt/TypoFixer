# Model Setup Instructions

This document explains how to download and compile the CoreML model for TypoFixer.

## Quick Setup

### Option 1: Using the Setup Script (Recommended)

Run the automated setup script:

```bash
./setup_model.sh
```

Or using the Python version:

```bash
python3 setup_model.py
```

### Option 2: Using Make

```bash
make setup-model
```

## What the Script Does

The setup script will:

1. ✅ Check that you're running on macOS with Python 3.10+
2. 🔧 Create a Python virtual environment
3. 📦 Install required dependencies (PyTorch, CoreML Tools, Transformers)
4. 🤖 Download the `visheratin/t5-efficient-tiny-grammar-correction` model
5. 🔄 Convert it to CoreML format
6. 🗜️ Compress the model for efficiency
7. ⚙️ Compile it for the Apple Neural Engine
8. 🧹 Clean up temporary files

## Expected Output

After successful completion, you should have:

```
Models/
├── config.json
├── tokenizer.json
└── t5_tiny_grammar.mlmodel

ModelsCompiled/
└── t5_tiny_grammar.mlmodelc/
    ├── analytics/
    ├── coremldata.bin
    ├── metadata.json
    ├── model.mil
    └── weights/
```

## Requirements

- macOS 14+ on Apple Silicon
- Python 3.10 or higher
- Xcode Command Line Tools (`xcode-select --install`)
- ~2GB free disk space for model download and compilation

## Troubleshooting

### "xcrun: error: tool 'coremlc' requires Xcode"

Install Xcode Command Line Tools:
```bash
xcode-select --install
```

### "No module named 'torch'"

The script should handle dependency installation automatically. If it fails, try:
```bash
source .venv/bin/activate
pip install torch transformers coremltools tokenizers accelerate
```

### "Permission denied"

Make sure the script is executable:
```bash
chmod +x setup_model.sh
```

### Model files not found during build

Ensure the model setup completed successfully and the files exist:
```bash
ls -la Models/
ls -la ModelsCompiled/
```

## Manual Setup

If the automated script doesn't work, you can follow the manual steps in `Download instructions.md`.

## Performance

The CoreML model should provide:
- ~200 tokens/second on M1-class devices
- Low power consumption (uses Apple Neural Engine)
- No network dependency after initial setup
- Instant startup (no model loading time)
