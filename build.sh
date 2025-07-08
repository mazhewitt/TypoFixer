#!/bin/bash

# Build script for TypoFixer
set -e

echo "Building TypoFixer..."

# Install cargo-zigbuild if not present
if ! command -v cargo-zigbuild &> /dev/null; then
    echo "Installing cargo-zigbuild..."
    cargo install cargo-zigbuild
fi

# Build for universal binary
echo "Building universal binary..."
cargo zigbuild --release --target universal2-apple-darwin

# Create app bundle structure
APP_NAME="TypoFixer"
APP_BUNDLE="target/universal2-apple-darwin/release/$APP_NAME.app"
BINARY_PATH="target/universal2-apple-darwin/release/typo-fixer"

echo "Creating app bundle..."
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# Copy binary
cp "$BINARY_PATH" "$APP_BUNDLE/Contents/MacOS/$APP_NAME"

# Create Info.plist
cat > "$APP_BUNDLE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>com.typofixer.app</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# Sign the app bundle
echo "Signing app bundle..."
if [ -n "$DEVELOPER_ID" ]; then
    codesign --entitlements entitlements.plist --options runtime --sign "$DEVELOPER_ID" "$APP_BUNDLE"
    echo "App bundle signed with Developer ID: $DEVELOPER_ID"
else
    echo "Warning: DEVELOPER_ID not set. App will not be signed."
    echo "Set DEVELOPER_ID environment variable to your Developer ID Application certificate name"
fi

echo "Build complete! App bundle created at: $APP_BUNDLE"
echo ""
echo "To run the app:"
echo "  open $APP_BUNDLE"
echo ""
echo "To sign with your Developer ID:"
echo "  export DEVELOPER_ID=\"Developer ID Application: Your Name (XXXXXXXXXX)\""
echo "  ./build.sh"