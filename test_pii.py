#!/usr/bin/env python3
"""
Simple test script to check if PII redaction is working through Tauri commands.
"""

import subprocess
import json
import time
import sys

def run_tauri_command(command, args=None):
    """Run a Tauri command and return the result."""
    if args is None:
        args = {}

    # This is a placeholder - in reality you'd need to communicate with the running Tauri app
    # For now, let's just check if the model files exist
    print(f"Would run command: {command} with args: {args}")

def main():
    print("Testing PII Redaction...")

    # Check if model files exist
    import os
    model_dir = "/home/aleks/PycharmProjects/pii_oss/Handy/TO_DO/onnx_models/gliner_multitask_large_v0_5"

    model_file = os.path.join(model_dir, "model_int8.onnx")
    tokenizer_file = os.path.join(model_dir, "tokenizer.json")

    print(f"Model file exists: {os.path.exists(model_file)}")
    print(f"Tokenizer file exists: {os.path.exists(tokenizer_file)}")

    if os.path.exists(model_file):
        print(f"Model file size: {os.path.getsize(model_file) / (1024*1024):.1f} MB")

    if os.path.exists(tokenizer_file):
        print(f"Tokenizer file size: {os.path.getsize(tokenizer_file) / 1024:.1f} KB")

    # Test text with PII
    test_text = "Hi, my name is John Smith and my email is john.smith@example.com. Call me at 555-123-4567."
    print(f"\nTest text: {test_text}")

    print("\nTo test PII redaction:")
    print("1. Make sure the Handy app is running")
    print("2. Enable PII redaction in settings")
    print("3. Use the transcription shortcut and say the test text above")
    print("4. Check if the output is redacted")

if __name__ == "__main__":
    main()