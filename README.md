# TypoFixer

A macOS native menubar app that corrects typos in text fields using a local LLM.

## Features

- **Global hotkey**: Press `⌘⌥S` (Command + Option + S) to fix typos in any text field
- **Local LLM**: Uses your own quantized model file for privacy
- **Smart selection**: Automatically extends selection to sentence boundaries
- **Accessibility integration**: Works with any standard text field
- **Lightweight**: Runs in the menubar without cluttering your Dock

## Prerequisites

- macOS 10.15 or later
- Rust toolchain with `cargo-zigbuild`
- A quantized LLM model file (e.g., `llama3-8b-q4.gguf`)

## Installation

1. **Install Rust and dependencies**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   cargo install cargo-zigbuild
   ```

2. **Clone and build**:
   ```bash
   git clone <repository-url>
   cd typo-fixer
   ./build.sh
   ```

3. **Sign the app** (optional but recommended):
   ```bash
   export DEVELOPER_ID="Developer ID Application: Your Name (XXXXXXXXXX)"
   ./build.sh
   ```

## Setup

1. **First run**: Launch the app and it will prompt you for the path to your model file
   - Default location: `~/Models/llama3-8b-q4.gguf`
   - The app will create a config file at `~/Library/Application Support/TypoFixer/config.toml`

2. **Grant Accessibility permissions**:
   - Open **System Preferences** → **Security & Privacy** → **Privacy** → **Accessibility**
   - Click the lock icon and authenticate
   - Click the "+" button and add TypoFixer.app
   - Ensure the checkbox is checked

## Usage

1. **Start the app**: Double-click `TypoFixer.app` or run from the command line
2. **Fix typos**: 
   - Place cursor in any text field
   - Press `⌘⌥S` to correct the current sentence
   - A "Fixed ✓" notification will appear on success

## How it Works

- **Text Selection**: If no text is selected, the app extends backwards to the previous sentence boundary (`.`, `!`, `?`) up to 300 characters
- **LLM Processing**: Sends text to your local model with the prompt: *"Correct any spelling mistakes in the following sentence without re-phrasing: «sentence»"*
- **Safety**: Aborts if the corrected text is more than 1.5× the original length
- **Timeout**: Processing times out after 300ms to stay responsive

## Configuration

The app stores its configuration in `~/Library/Application Support/TypoFixer/config.toml`:

```toml
model_path = "/Users/username/Models/llama3-8b-q4.gguf"
```

## Logs

Error logs are written to `~/Library/Logs/TypoFixer.log` for debugging.

## Building from Source

```bash
# Install dependencies
cargo install cargo-zigbuild

# Build universal binary
cargo zigbuild --release --target universal2-apple-darwin

# Create signed app bundle
./build.sh
```

## Troubleshooting

- **"AccessibilityPermissionDenied"**: Grant Accessibility permissions in System Preferences
- **"Model not found"**: Ensure your model file exists at the configured path
- **No response**: Check that the model file is compatible with llama.cpp
- **Hotkey not working**: Verify no other app is using `⌘⌥S` hotkey combination

## Security Notes

- All text processing happens locally on your machine
- The app requires Accessibility permissions to read/write text fields
- No data is sent to external servers

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]