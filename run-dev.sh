#!/bin/bash

npm install

echo "🚀 Starting SDKMAN GUI Tauri Development Server..."
echo ""
echo "📝 Note: This will:"
echo "   1. Start Vite dev server on http://localhost:1420"
echo "   2. Compile Rust backend (first time may take 5-10 minutes)"
echo "   3. Launch the Tauri desktop application"
echo ""
echo "⏳ Please wait..."
echo ""

# Export proxy if needed (uncomment if you use proxy)
 export https_proxy=http://127.0.0.1:7897
 export http_proxy=http://127.0.0.1:7897

# Run Tauri dev
npm run tauri dev
