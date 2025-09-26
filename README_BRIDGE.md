# Handy-Claude PII Bridge

A Python script that connects to Handy's API server, processes PII-redacted transcriptions, and forwards them to Claude API with privacy protection.

## Features

- **Real-time transcription processing** via Handy's WebSocket API
- **Entity mapping** with numbered placeholders (e.g., `[NAME_0]`, `[NAME_1]`)
- **Privacy-aware Claude integration** with system prompts
- **Conversation history** with anonymized storage
- **Entity restoration** in responses for natural display

## Setup

1. **Install dependencies:**
   ```bash
   pip install -r requirements.txt
   ```

2. **Set up Claude API key:**
   ```bash
   export ANTHROPIC_API_KEY="your_api_key_here"
   ```

3. **Enable Handy API server:**
   - Open Handy settings → Advanced Settings
   - Toggle "Enable API Server" ON
   - Ensure it shows: `API running at: ws://localhost:7878/ws`

4. **Enable PII redaction (recommended):**
   - In Handy settings → Advanced Settings
   - Toggle "Enable PII Redaction" ON
   - Configure which entity types to redact

## Usage

```bash
python claude_pii_bridge.py
```

The script will:
1. Connect to Handy's WebSocket API
2. Listen for transcriptions
3. Process PII-redacted text with entity mapping
4. Send to Claude with privacy-aware system prompt
5. Display Claude's response with entities restored

## How It Works

### Example Flow:

1. **Original transcription:** `"Hi, my name is John Smith and my phone is 555-0123"`

2. **Handy PII redaction:** `"Hi, my name is [PERSON] and my phone is [PHONE_NUMBER]"`

3. **Entity mapping:**
   ```
   [PERSON_0] → John Smith
   [PHONE_NUMBER_0] → 555-0123
   ```

4. **Processed for Claude:** `"Hi, my name is [PERSON_0] and my phone is [PHONE_NUMBER_0]"`

5. **System prompt:**
   ```
   This user message has been processed for privacy protection.

   PRIVACY NOTICE: Personal information has been redacted:
   • [PERSON_0] represents a real person
   • [PHONE_NUMBER_0] represents a real phone number

   Treat each bracketed placeholder as if it were the real entity.
   ```

6. **Claude's response:** `"Nice to meet you, [PERSON_0]! I can help you with your inquiry."`

7. **Displayed to user:** `"Nice to meet you, John Smith! I can help you with your inquiry."`

## Privacy Benefits

- **Your PII never reaches Claude's servers** in original form
- **Numbered entities** allow referencing across conversation
- **Local restoration** shows natural responses
- **Conversation history** stays anonymized
- **Entity consistency** across multiple interactions

## Configuration

Edit the script to customize:
- `CLAUDE_API_URL` - Claude API endpoint
- `HANDY_WEBSOCKET_URL` - Handy WebSocket URL
- `max_tokens` - Claude response length
- Model selection (`claude-3-5-sonnet-20241022`)
- Conversation history length

## Troubleshooting

- **"Failed to connect to Handy"** - Ensure Handy is running with API server enabled
- **"ANTHROPIC_API_KEY not set"** - Set your Claude API key as environment variable
- **Entity mapping issues** - The word alignment algorithm is simplified; complex redactions may need manual adjustment

## Security Notes

- API key is loaded from environment variable (secure)
- PII entities are only mapped locally
- Original text never sent to external APIs
- Conversation history stored in memory only (not persistent)