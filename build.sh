#!/bin/bash

# Build script for TypoFixer
set -e

echo "Building TypoFixer..."

APP_NAME="TypoFixer"

# Check if we should build universal binary or use existing build
if [ "$SKIP_BUILD" = "true" ]; then
    echo "Skipping build (SKIP_BUILD=true)..."
    # Use standard release build path (for GitHub Actions)
    APP_BUNDLE="target/release/$APP_NAME.app"
    BINARY_PATH="target/release/typo-fixer"
else
    # Install cargo-zigbuild if not present
    if ! command -v cargo-zigbuild &> /dev/null; then
        echo "Installing cargo-zigbuild..."
        cargo install cargo-zigbuild
    fi

    # Build for universal binary
    echo "Building universal binary..."
    cargo zigbuild --release --target universal2-apple-darwin
    
    # Use universal binary paths
    APP_BUNDLE="target/universal2-apple-darwin/release/$APP_NAME.app"
    BINARY_PATH="target/universal2-apple-darwin/release/typo-fixer"
fi

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    echo "Make sure to run 'cargo build --release' first if using SKIP_BUILD=true"
    exit 1
fi

echo "Creating app bundle..."
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# Copy binary
cp "$BINARY_PATH" "$APP_BUNDLE/Contents/MacOS/$APP_NAME"

# Get version from environment or use default
VERSION=${VERSION:-"1.0.0"}

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
    <key>CFBundleDisplayName</key>
    <string>TypoFixer</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSAppleEventsUsageDescription</key>
    <string>TypoFixer needs to send AppleScript events to interact with other applications for text correction.</string>
    <key>NSAccessibilityUsageDescription</key>
    <string>TypoFixer needs accessibility permissions to read and correct text in other applications.</string>
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
echo "  open \"$APP_BUNDLE\""
echo ""
if [ "$SKIP_BUILD" != "true" ]; then
    echo "To build for GitHub Actions (using standard cargo build):"
    echo "  SKIP_BUILD=true VERSION=\$VERSION ./build.sh"
    echo ""
fi
echo "To sign with your Developer ID:"
echo "  export DEVELOPER_ID=\"Developer ID Application: Your Name (XXXXXXXXXX)\""
echo "  ./build.sh"