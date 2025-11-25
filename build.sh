#!/bin/bash

# SDKMAN GUI - Build Script
# This script builds the Tauri application for the current platform
# Supports: macOS, Linux, Windows (via Git Bash or WSL)

set -e  # Exit on error

echo "=========================================="
echo "Building SDKMAN GUI..."
echo "=========================================="
echo ""

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
    echo ""
fi

# Build the application
echo "🔨 Building application..."
npm run tauri build

echo ""
echo "=========================================="
echo "✅ Build completed successfully!"
echo "=========================================="
echo ""

# Rename build artifacts to use "sdkman-gui" instead of "SDKMAN GUI"
echo "📝 Renaming build artifacts..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # Rename macOS DMG files
    if [ -d "src-tauri/target/release/bundle/dmg" ]; then
        for file in src-tauri/target/release/bundle/dmg/SDKMAN\ GUI*.dmg; do
            if [ -f "$file" ]; then
                newname=$(echo "$file" | sed 's/SDKMAN GUI/sdkman-gui/g')
                mv "$file" "$newname"
                echo "   Renamed: $(basename "$newname")"
            fi
        done
    fi
    # Rename macOS .app bundle
    if [ -d "src-tauri/target/release/bundle/macos/SDKMAN GUI.app" ]; then
        mv "src-tauri/target/release/bundle/macos/SDKMAN GUI.app" "src-tauri/target/release/bundle/macos/sdkman-gui.app"
        echo "   Renamed: sdkman-gui.app"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Rename Linux DEB files
    if [ -d "src-tauri/target/release/bundle/deb" ]; then
        for file in src-tauri/target/release/bundle/deb/SDKMAN\ GUI*.deb; do
            if [ -f "$file" ]; then
                newname=$(echo "$file" | sed 's/SDKMAN GUI/sdkman-gui/g')
                mv "$file" "$newname"
                echo "   Renamed: $(basename "$newname")"
            fi
        done
    fi
    # Rename Linux RPM files
    if [ -d "src-tauri/target/release/bundle/rpm" ]; then
        for file in src-tauri/target/release/bundle/rpm/SDKMAN\ GUI*.rpm; do
            if [ -f "$file" ]; then
                newname=$(echo "$file" | sed 's/SDKMAN GUI/sdkman-gui/g')
                mv "$file" "$newname"
                echo "   Renamed: $(basename "$newname")"
            fi
        done
    fi
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
    # Rename Windows MSI files and remove language suffix
    if [ -d "src-tauri/target/release/bundle/msi" ]; then
        for file in src-tauri/target/release/bundle/msi/SDKMAN\ GUI*.msi; do
            if [ -f "$file" ]; then
                newname=$(echo "$file" | sed 's/SDKMAN GUI/sdkman-gui/g' | sed 's/_en-US//g')
                mv "$file" "$newname"
                echo "   Renamed: $(basename "$newname")"
            fi
        done
    fi
    # Rename Windows NSIS installer
    if [ -d "src-tauri/target/release/bundle/nsis" ]; then
        for file in src-tauri/target/release/bundle/nsis/SDKMAN\ GUI*.exe; do
            if [ -f "$file" ]; then
                newname=$(echo "$file" | sed 's/SDKMAN GUI/sdkman-gui/g')
                mv "$file" "$newname"
                echo "   Renamed: $(basename "$newname")"
            fi
        done
    fi
fi

echo ""

# Display build artifacts location based on OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "📦 macOS build artifacts:"
    echo "   DMG: src-tauri/target/release/bundle/dmg/"
    echo "   APP: src-tauri/target/release/bundle/macos/"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "📦 Linux build artifacts:"
    echo "   DEB: src-tauri/target/release/bundle/deb/"
    echo "   RPM: src-tauri/target/release/bundle/rpm/"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
    echo "📦 Windows build artifacts:"
    echo "   MSI: src-tauri/target/release/bundle/msi/"
    echo "   NSIS: src-tauri/target/release/bundle/nsis/"
else
    echo "📦 Build artifacts location:"
    echo "   src-tauri/target/release/bundle/"
fi

echo ""
