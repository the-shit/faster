#!/bin/bash
set -e

echo "🦀 Building Faster Zellij Plugin..."

# Add WASM target if not already added
rustup target add wasm32-wasip1 2>/dev/null || true

# Build the plugin
echo "📦 Compiling to WASM..."
cargo build --release --target wasm32-wasip1 --features plugin

# Create plugin directory
PLUGIN_DIR="$HOME/.config/zellij/plugins"
mkdir -p "$PLUGIN_DIR"

# Copy the WASM file
echo "📂 Installing to $PLUGIN_DIR..."
cp target/wasm32-wasip1/release/faster.wasm "$PLUGIN_DIR/"
cp plugin.yaml "$PLUGIN_DIR/faster.yaml"

echo "✅ Plugin installed!"
echo ""
echo "Add to your Zellij layout:"
echo ""
echo "  pane {
    plugin location=\"file:$PLUGIN_DIR/faster.wasm\"
  }"
echo ""
echo "Or run: zellij action new-pane --plugin file:$PLUGIN_DIR/faster.wasm"
