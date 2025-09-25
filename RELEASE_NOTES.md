# Handy-PII v0.5.1 - PII Redaction Release

## What's New

### 🔐 PII Redaction Feature
This fork introduces advanced PII (Personally Identifiable Information) redaction capabilities not available in the original Handy application.

#### Features:
- **Real-time PII detection and redaction** during transcription
- **Configurable PII entity types** including:
  - Personal identifiers (names, DOB, age, gender)
  - Contact information (email, phone, addresses)
  - Financial data (credit cards, bank accounts)
  - Government IDs (SSN, licenses, passports)
  - Medical information (coming soon)
- **Optional entity labels** - show what type of PII was redacted
- **Toggle PII redaction** on/off from settings

#### Technology:
The PII detection is powered by GLiNER PII models via the **gliner-pii collection**, a collaboration between:
- **Wordcab** (wordcab.com) - Speech intelligence tools
- **Knowledgator** (knowledgator.com) - NLP solutions provider

Model Collection: https://huggingface.co/collections/knowledgator/gliner-pii-68d3fd140d4480852d1e37ba

### 🛠 Technical Improvements
- Heavily modified `gline-rs` (by Wordcab) to integrate GLiNER models with Handy
- Integrated GLiNER-based PII detection pipeline
- Optimized INT8 quantized model for efficient CPU inference
- Added PII settings UI with granular control
- Automatic PII model management within ModelManager

## Installation
Download the appropriate installer for your platform below. The PII model will be automatically downloaded on first use when PII redaction is enabled.

## Credits
- Original Handy by @cjpais
- PII redaction feature and gline-rs modifications by Wordcab team
- GLiNER PII models via Knowledgator & Wordcab collaboration