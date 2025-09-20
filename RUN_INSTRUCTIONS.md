# How to Run Handy with PII Redaction

## Quick Start

To run the application in development mode:

```bash
# From the Handy directory
bun run tauri dev
```

Or if you encounter Vulkan/build issues:

```bash
# Skip Vulkan compilation
WHISPER_NO_VULKAN=1 bun run tauri dev
```

## Testing PII Redaction

1. **Launch the app** using the command above

2. **Open Settings**:
   - Click the Handy icon in the system tray
   - Select "Settings" from the menu

3. **Enable PII Redaction**:
   - Go to the "Advanced" tab in settings
   - Look for "PII Redaction" section
   - Toggle "Enable PII Redaction" ON
   - The app will load the GLiNER model (takes 2-3 seconds)

4. **Select PII Types to Redact** (all enabled by default):
   - Person names
   - Email addresses
   - Phone numbers
   - Social Security Numbers
   - Credit cards
   - Addresses
   - Dates of birth
   - Passport numbers
   - Driver licenses
   - Bank accounts

5. **Test Transcription**:
   - Use your configured hotkey to start recording
   - Say something with PII like: "My name is John Smith and my email is john@example.com"
   - Release the hotkey to stop recording
   - The transcription will be processed and PII will be redacted
   - Check your clipboard - it should contain: "My name is [PERSON_REDACTED] and my email is [EMAIL_REDACTED]"

6. **Check History**:
   - The original (unredacted) text is saved in history
   - Only the clipboard receives the redacted version

## Troubleshooting

### If the model doesn't load:
- Check that the model files exist at:
  - `TO_DO/onnx_models/gliner_multitask_large_v0_5/model_int8.onnx`
  - `TO_DO/onnx_models/gliner_multitask_large_v0_5/tokenizer.json`

### If build fails with Vulkan errors:
```bash
WHISPER_NO_VULKAN=1 bun run tauri dev
```

### If you see Rust compilation errors:
```bash
cd src-tauri
cargo clean
cd ..
bun run tauri dev
```

## Expected Behavior

When PII Redaction is enabled:
1. The GLiNER model loads on first enable (2-3 seconds)
2. Each transcription is processed for PII detection (~50-100ms overhead)
3. Detected PII is replaced with `[TYPE_REDACTED]` placeholders
4. Original text is preserved in history
5. Only redacted text goes to clipboard

## Performance Notes

- Model loading: One-time 2-3 second delay when first enabling
- Memory usage: ~266MB additional when model is loaded
- Processing time: Adds 50-100ms to transcription processing
- The model uses INT8 quantization for efficiency