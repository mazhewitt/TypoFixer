# TypoFixer 🔧

**Intelligent macOS Text Correction with LLM-Powered Spell & Grammar Checking**

TypoFixer is a powerful macOS application that provides instant, context-aware text correction using local language models. With robust fallback mechanisms, it works seamlessly across all macOS applications, including challenging Electron-based apps like VS Code.

[![Release](https://img.shields.io/github/v/release/yourusername/typofixer)](https://github.com/yourusername/typofixer/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![macOS](https://img.shields.io/badge/macOS-10.15+-green.svg)](https://www.apple.com/macos/)

## ✨ Features

### 🎯 Core Functionality
- **Global Hotkey**: Press `Cmd+Option+S` anywhere to trigger text correction
- **Smart AI Correction**: Uses local LLM models via Ollama for intelligent spell and grammar checking
- **Universal Compatibility**: Works with native macOS apps and Electron-based applications
- **Menu Bar Integration**: Convenient access from the macOS menu bar

### 🛡️ Robust Technology
- **Accessibility API**: Primary method for seamless text interaction
- **Clipboard Fallback**: Automatic fallback for problematic applications (VS Code, Discord, etc.)
- **AppleScript Integration**: Additional compatibility layer for maximum coverage
- **Smart Text Selection**: Automatically extracts relevant text context for correction

### 🧠 Intelligent Features
- **Context-Aware Corrections**: Understands sentence structure and context
- **Secure Field Detection**: Automatically skips password fields for security
- **App-Specific Optimization**: Recognizes and adapts to different application types
- **Comprehensive Error Handling**: Graceful fallbacks with detailed logging

## 🚀 Quick Start

### Prerequisites
1. **macOS 10.15+** (Intel or Apple Silicon)
2. **Ollama** - Install from [ollama.ai](https://ollama.ai)
3. **Language Model** - Download with: `ollama pull llama3.2:1b`

### Installation

#### Option 1: Download Release (Recommended)
1. Download the latest `TypoFixer-v1.0.0.dmg` from [Releases](https://github.com/yourusername/typofixer/releases)
2. Open the DMG and drag TypoFixer to Applications
3. Launch TypoFixer from Applications or Spotlight

#### Option 2: Build from Source
```bash
# Clone the repository
git clone https://github.com/yourusername/typofixer.git
cd typofixer

# Build and install
make install

# Or just build the app bundle
make app
```

### Setup
1. **Grant Accessibility Permissions**:
   - Go to System Preferences → Security & Privacy → Privacy → Accessibility
   - Add TypoFixer and enable it

2. **Start Ollama** (if not running):
   ```bash
   ollama serve
   ```

3. **Launch TypoFixer** - It will appear in your menu bar

## 🎮 Usage

1. **Position your cursor** in any text field
2. **Press `Cmd+Option+S`** to trigger correction
3. **Watch the magic** - your text gets intelligently corrected!

### Example Corrections
- `"I recieve teh mesage"` → `"I received the message"`
- `"Their going to there house"` → `"They're going to their house"`
- `"Its a beautifull day"` → `"It's a beautiful day"`

## 🧪 Tested Applications

### ✅ Fully Supported
- **Development**: VS Code, Xcode, Terminal, iTerm2
- **Communication**: Mail, Messages, Discord, Slack, Telegram
- **Productivity**: Notes, TextEdit, Pages, Notion, Obsidian
- **Browsers**: Safari, Chrome, Firefox (text areas)
- **Design**: Figma (text layers)

### 🔄 Automatic Fallback Apps
Some Electron-based apps automatically use clipboard fallback:
- VS Code, Atom, Discord, Slack, WhatsApp, Spotify

## 🛠️ Development

### Building
```bash
# Development build
make build

# Create app bundle
make app

# Create distribution packages
make package

# Run tests
make test

# Format code
make format
```

### Project Structure
```
src/
├── main.rs              # Application entry point
├── accessibility/       # Text extraction and setting
├── spell_check/         # LLM integration
├── hotkey/             # Global hotkey handling
├── menu_bar/           # macOS menu bar integration
└── config/             # Configuration management
```

## 📦 Packaging

The project includes comprehensive packaging options:

```bash
# Create all distribution formats
./package.sh
```

Creates:
- **DMG**: `TypoFixer-v1.0.0.dmg` (drag-to-install)
- **ZIP**: `TypoFixer-v1.0.0.zip` (portable)
- **PKG**: `TypoFixer-v1.0.0-Installer.pkg` (installer)

## ⚙️ Configuration

TypoFixer automatically detects and configures:
- Ollama endpoint (default: `http://localhost:11434`)
- Model selection (default: first available model)
- Hotkey (Cmd+Option+S, customizable in future versions)

## 🔒 Privacy & Security

- **Local Processing**: All text correction happens locally via Ollama
- **No Data Collection**: Your text never leaves your machine
- **Secure Fields**: Password fields are automatically skipped
- **Accessibility**: Only reads/writes when explicitly triggered

## 🐛 Troubleshooting

### Common Issues

**"Accessibility permissions not granted"**
- Grant permissions in System Preferences → Security & Privacy → Privacy → Accessibility

**"No model found"**
- Install a model: `ollama pull llama3.2:1b`
- Ensure Ollama is running: `ollama serve`

**Text not being corrected in some apps**
- TypoFixer automatically falls back to clipboard method for problematic apps
- This is normal behavior for Electron-based applications

### Debug Mode
Run from terminal to see detailed logs:
```bash
./TypoFixer.app/Contents/MacOS/TypoFixer
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup
1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Install Ollama: `curl -fsSL https://ollama.ai/install.sh | sh`
3. Clone and build: `git clone ... && cd typofixer && make dev-deps && make build`

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Ollama** for providing excellent local LLM infrastructure
- **Rust Community** for incredible accessibility and GUI libraries
- **macOS Accessibility Framework** for enabling seamless text interaction

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/typofixer/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/typofixer/discussions)
- **Email**: support@typofixer.com

---

**Made with ❤️ for the macOS community**
