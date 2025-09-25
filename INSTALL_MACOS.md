# macOS Installation Instructions

## First-Time Installation

Since Handy-PII is ad-hoc signed (not notarized by Apple), macOS Gatekeeper will show a warning on first launch.

### Method 1: System Preferences (Recommended)
1. Download the `.dmg` file from the [releases page](https://github.com/info-wordcab/Handy/releases)
2. Open the DMG and drag Handy-PII to Applications
3. Try to open Handy-PII from Applications (it will be blocked)
4. Go to **System Preferences → Security & Privacy → General**
5. Click **"Open Anyway"** next to the message about Handy-PII
6. Click **"Open"** in the dialog that appears

### Method 2: Terminal (For Developers)
After installing the app, run this command in Terminal:
```bash
xattr -cr /Applications/Handy-PII.app
```
This removes the quarantine flag and allows the app to run immediately.

### Method 3: Control+Click (Alternative)
1. Install the app as usual
2. Right-click (or Control+click) on Handy-PII in Applications
3. Select "Open" from the context menu
4. Click "Open" in the warning dialog

## Why This Happens

Handy-PII is an open-source fork that uses ad-hoc signing for transparency and cost-effectiveness. Full Apple notarization requires a $99/year Apple Developer Program membership.

The app is still signed and safe to use - it's just not notarized by Apple's servers, which means you need to explicitly allow it to run.

## Security Note

Ad-hoc signed apps are cryptographically signed to verify their integrity, but they haven't been reviewed by Apple. This is common for open-source software distributed outside the Mac App Store.

You can verify the app's signature by running:
```bash
codesign -dv /Applications/Handy-PII.app
```

## Troubleshooting

### "App is damaged and can't be opened"
This usually happens if the download was corrupted. Try:
1. Delete the app from Applications
2. Empty Trash
3. Re-download and install

### App still won't open after following instructions
1. Make sure you're running macOS 10.13 or later
2. Try the terminal command: `xattr -cr /Applications/Handy-PII.app`
3. If issues persist, please open an issue on GitHub

## Updates

Future updates through the app's auto-updater should work normally once you've done the initial approval. If you encounter issues with updates, you may need to repeat the approval process.