# TypoFixer 🔧

**Intelligent macOS Text Correction with LLM-Powered Spell & Grammar Checking**

A powerful macOS application that provides instant, context-aware text correction using local language models. With robust fallback mechanisms, it works seamlessly across all macOS applications, including challenging Electron-based apps like VS Code.

## ✨ Features

- **Global hotkey**: Press `⌘⌥S` (Command + Option + S) to fix typos in any text field
- **LLM-powered correction**: Uses local Qwen CoreML models for intelligent spell and grammar checking
- **Universal compatibility**: Works with native macOS apps and Electron-based applications (VS Code, Discord, etc.)
- **Smart text extraction**: Automatically detects and extracts relevant text context for correction
- **Robust fallback methods**: Accessibility API with clipboard and AppleScript fallbacks
- **Secure field detection**: Automatically skips password fields for security
- **Menu bar integration**: Convenient access from the macOS menu bar

## 🚨 **IMPORTANT: Qwen CoreML Model Required**

**TypoFixer requires a Qwen CoreML model to be placed in the correct directory.**

### Quick Model Setup:
1. Convert your Qwen model to CoreML format (if not already done)
2. Place the model directory in `coreml-models/qwen-typo-fixer/`
3. Ensure the model includes tokenizer files

## Prerequisites

- **macOS 10.15+** (Intel or Apple Silicon)
- **Qwen CoreML model** placed in correct directory
- **Accessibility permissions** (granted during setup)

## 📦 Installation

### Option 1: Download Release (Recommended for Users)

1. **Set up Qwen CoreML model**:
   - Place your converted Qwen model in `coreml-models/qwen-typo-fixer/`

2. **Download TypoFixer**:
   - Get the latest `TypoFixer-v1.0.0.dmg` from [Releases](https://github.com/yourusername/typofixer/releases)
   - Open the DMG and drag TypoFixer to Applications
   - Launch TypoFixer from Applications or Spotlight

### Option 2: Build from Source (For Developers)

1. **Install Ollama** (see above)

2. **Install Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Clone and build**:
   ```bash
   git clone <repository-url>
   cd typo-fixer
   make install
   ```

## ⚙️ Setup

### 1. **Ensure Ollama is Running**
```bash
# Start Ollama (keep this running in background)
ollama serve
```

### 2. **Grant Accessibility Permissions**
- Go to **System Preferences** → **Security & Privacy** → **Privacy** → **Accessibility**
- Click the lock icon and authenticate
- Add TypoFixer and enable it

### 3. **Launch TypoFixer**
- TypoFixer will appear in your menu bar as ⌨️
- Ready to use! Press `⌘⌥S` in any text field

## 🎮 Usage

**Prerequisites: Make sure Ollama is running!**
```bash
ollama serve  # Keep this running in a terminal
```

1. **Position your cursor** in any text field (VS Code, TextEdit, browser, etc.)
2. **Press `⌘⌥S`** (Command + Option + S) to trigger correction
3. **Watch the magic** - your text gets intelligently corrected!

### Example Corrections:
- `"I recieve teh mesage"` → `"I received the message"`
- `"Their going to there house"` → `"They're going to their house"`
- `"Its a beautifull day"` → `"It's a beautiful day"`

## 🧪 Tested Applications

### ✅ Fully Supported
- **Development**: VS Code, Xcode, Terminal, iTerm2
- **Communication**: Mail, Messages, Discord, Slack, Telegram
- **Productivity**: Notes, TextEdit, Pages, Notion, Obsidian
- **Browsers**: Safari, Chrome, Firefox (text areas)

### 🔄 Automatic Fallback
Some Electron-based apps automatically use clipboard fallback (this is normal):
- VS Code, Atom, Discord, Slack, WhatsApp, Spotify

## 🔧 How It Works

1. **Text Detection**: Uses macOS accessibility APIs to find the focused text field
2. **Smart Extraction**: Intelligently extracts the current sentence or relevant text chunk
3. **LLM Processing**: Sends text to Ollama for intelligent correction
4. **Robust Replacement**: Uses accessibility API or falls back to clipboard method
5. **Fallback Methods**: Clipboard and AppleScript fallbacks for problematic apps

## ⚙️ Configuration

TypoFixer automatically detects and uses:
- **Ollama endpoint**: `http://localhost:11434` (default)
- **Model selection**: Uses the first available model from `ollama list`
- **Hotkey**: `Cmd+Option+S` (customizable in future versions)

## 🐛 Troubleshooting

### Common Issues

**❌ "No model found" or corrections not working**
```bash
# Check if Ollama is running
curl http://localhost:11434/api/version

# Install a model if needed
ollama pull llama3.2:1b

# Start Ollama
ollama serve
```

**❌ "Accessibility permissions not granted"**
- Go to System Preferences → Security & Privacy → Privacy → Accessibility
- Add TypoFixer and enable it
- Restart TypoFixer after granting permissions

**❌ Text not being corrected in some apps**
- This is normal for Electron-based apps (VS Code, Discord, etc.)
- TypoFixer automatically uses clipboard fallback method
- Make sure the text field is focused before pressing `⌘⌥S`

**❌ Hotkey not working**
- Verify no other app is using `⌘⌥S` hotkey combination
- Check that TypoFixer is running (should see ⌨️ in menu bar)
- Try restarting TypoFixer

### Debug Mode
Run from terminal to see detailed logs:
```bash
./TypoFixer.app/Contents/MacOS/TypoFixer
```

## 🛠️ Development

### Quick Commands
```bash
make build       # Build the application
make app         # Create app bundle  
make package     # Create all distribution packages
make install     # Install to /Applications
make test        # Run tests
```

### Building from Source
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone <repository-url>
cd typo-fixer
make install
```

## 🔒 Privacy & Security

- **Local Processing**: All text correction happens locally via Ollama
- **No Data Collection**: Your text never leaves your machine  
- **Secure Fields**: Password fields are automatically skipped
- **Accessibility**: Only reads/writes when explicitly triggered

## 📄 License

MIT License - see LICENSE file for details.

---

**Made with ❤️ for the macOS community**

- All text processing happens locally on your machine
- The app requires Accessibility permissions to read/write text fields
- No data is sent to external servers

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]