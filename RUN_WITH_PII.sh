#!/bin/bash

# Run Handy with PII Redaction Support
# This script ensures the app runs with the correct configuration

echo "Starting Handy with PII Redaction support..."
echo ""

# Check if models exist
if [ ! -f "TO_DO/onnx_models/gliner_multitask_large_v0_5/model_int8.onnx" ]; then
    echo "⚠️  GLiNER model not found!"
    echo "Please ensure model files are at:"
    echo "  - TO_DO/onnx_models/gliner_multitask_large_v0_5/model_int8.onnx"
    echo "  - TO_DO/onnx_models/gliner_multitask_large_v0_5/tokenizer.json"
    exit 1
fi

echo "✓ GLiNER model found"
echo ""

# Check glslc version
if command -v glslc &> /dev/null; then
    GLSLC_VERSION=$(glslc --version 2>&1 | head -n1)
    echo "glslc version: $GLSLC_VERSION"
    echo "Note: Version 15.3.0 or higher recommended for Vulkan support"
    echo ""
fi

# Main command - with Vulkan support (recommended)
echo "Starting development server with Vulkan support..."
echo "If you see Vulkan errors, press Ctrl+C and uncomment the alternative command below"
echo ""

# Primary command with full Vulkan support
bun run tauri dev

# Alternative commands if the above fails:

# Option 1: Skip Vulkan (if glslc issues persist)
# WHISPER_NO_VULKAN=1 bun run tauri dev

# Option 2: Direct cargo run (if bun has issues)
# cd src-tauri && cargo run

# Option 3: Force rebuild
# cd src-tauri && cargo clean && cd .. && bun run tauri dev