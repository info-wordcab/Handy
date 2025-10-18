# App Launcher Feature Implementation Guide for Handy

**Date**: October 17, 2025
**Feature**: Voice-activated application launcher with custom shortcut
**Target Branch**: main

## Overview

This document provides a complete implementation guide for adding a voice-activated application launcher feature to the Handy speech-to-text application. The feature allows users to launch desktop applications by speaking their names after pressing CTRL+SHIFT+SPACE.

## Architecture Summary

The app launcher integrates into Handy's existing architecture:
- **New Manager**: `AppLauncherManager` for desktop app discovery and launching
- **New Action**: `OpenAppAction` in the ACTION_MAP for handling the shortcut
- **Settings Integration**: New "open_app" shortcut binding in settings
- **Linux-specific**: X11 threading considerations for GUI app launching

## Implementation Steps

### 1. Add Dependencies to Cargo.toml

Add these dependencies to `src-tauri/Cargo.toml`:

```toml
nucleo = "0.5.0"                    # Fuzzy string matching
freedesktop-desktop-entry = "0.7.19" # Parse .desktop files
shellexpand = "3.1"                  # Expand ~ in paths

[target.'cfg(target_os = "linux")'.dependencies]
x11 = { version = "2.21", features = ["xlib"] }  # X11 threading
```

### 2. Create App Launcher Manager

Create `src-tauri/src/managers/app_launcher.rs`:

```rust
use anyhow::{anyhow, Result};
use freedesktop_desktop_entry::DesktopEntry;
use log::{debug, error, info};
use nucleo::{Config, Matcher, Utf32String};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::App;

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub exec: String,
    pub description: Option<String>,
    pub generic_name: Option<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
}

#[derive(Clone)]
pub struct AppLauncherManager {
    apps: Arc<Mutex<HashMap<String, AppInfo>>>,
}

impl AppLauncherManager {
    pub fn new(_app: &App) -> Result<Self> {
        let manager = Self {
            apps: Arc::new(Mutex::new(HashMap::new())),
        };
        manager.load_apps()?;
        Ok(manager)
    }

    /// Scans desktop entry directories and loads application information
    pub fn load_apps(&self) -> Result<()> {
        info!("Loading desktop applications...");
        let mut apps = self.apps.lock().unwrap();
        apps.clear();

        let desktop_paths = vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from(shellexpand::tilde("~/.local/share/applications").to_string()),
            PathBuf::from("/var/lib/flatpak/exports/share/applications"),
            PathBuf::from("/var/lib/snapd/desktop/applications"),
        ];

        let mut app_count = 0;

        for path in desktop_paths {
            if !path.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let file_path = entry.path();
                    if file_path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                        match DesktopEntry::from_path::<&str>(&file_path, None) {
                            Ok(desktop_entry) => {
                                // Skip hidden apps
                                if desktop_entry.no_display() {
                                    continue;
                                }

                                let locales: &[&str] = &[];
                                let name = desktop_entry.name(locales).unwrap_or_default();
                                if name.is_empty() {
                                    continue;
                                }

                                let exec = desktop_entry.exec().unwrap_or_default();
                                if exec.is_empty() {
                                    continue;
                                }

                                let app_info = AppInfo {
                                    name: name.to_string(),
                                    exec: exec.to_string(),
                                    description: desktop_entry
                                        .comment(locales)
                                        .map(|s| s.to_string()),
                                    generic_name: desktop_entry
                                        .generic_name(locales)
                                        .map(|s| s.to_string()),
                                    categories: desktop_entry
                                        .categories()
                                        .map(|cats| {
                                            cats.iter()
                                                .map(|c| c.to_string())
                                                .collect()
                                        })
                                        .unwrap_or_default(),
                                    keywords: desktop_entry
                                        .keywords(locales)
                                        .map(|kws| {
                                            kws.iter()
                                                .map(|k| k.to_string())
                                                .collect()
                                        })
                                        .unwrap_or_default(),
                                };

                                // Store with lowercase key for case-insensitive lookup
                                apps.insert(app_info.name.to_lowercase(), app_info);
                                app_count += 1;
                            }
                            Err(e) => {
                                debug!("Failed to parse {:?}: {}", file_path, e);
                            }
                        }
                    }
                }
            }
        }

        info!("Loaded {} desktop applications", app_count);
        Ok(())
    }

    /// Finds the best matching application using fuzzy matching
    pub fn find_app(&self, query: &str) -> Option<AppInfo> {
        if query.is_empty() {
            return None;
        }

        let apps = self.apps.lock().unwrap();
        if apps.is_empty() {
            error!("No applications loaded");
            return None;
        }

        // Clean up the query: remove punctuation and normalize
        let cleaned_query = query
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .trim()
            .to_lowercase();

        debug!("Searching for app matching: '{}', cleaned: '{}'", query, cleaned_query);

        // First try exact match with cleaned query
        if let Some(app) = apps.get(&cleaned_query) {
            debug!("Found exact match: {}", app.name);
            return Some(app.clone());
        }

        // Try matching just the first word
        let first_word = cleaned_query.split_whitespace().next().unwrap_or(&cleaned_query);
        if let Some(app) = apps.get(first_word) {
            debug!("Found exact match on first word: {}", app.name);
            return Some(app.clone());
        }

        // Fuzzy matching with nucleo
        let mut matcher = Matcher::new(Config::DEFAULT);
        let mut best_match: Option<(AppInfo, u16)> = None;

        for (_, app_info) in apps.iter() {
            let search_texts = vec![
                Some(&app_info.name),
                app_info.generic_name.as_ref(),
            ]
            .into_iter()
            .flatten()
            .chain(app_info.keywords.iter());

            for text in search_texts {
                let haystack = Utf32String::from(text.to_lowercase().as_str());
                let needle = Utf32String::from(cleaned_query.as_str());

                if let Some(score) = matcher.fuzzy_match(haystack.slice(..), needle.slice(..)) {
                    // Boost score if the app name starts with the query
                    let boosted_score = if text.to_lowercase().starts_with(&cleaned_query) {
                        score + 1000
                    } else {
                        score
                    };

                    if best_match
                        .as_ref()
                        .map_or(true, |(_, current_score)| boosted_score > *current_score)
                    {
                        best_match = Some((app_info.clone(), boosted_score));
                    }
                }
            }
        }

        if let Some((app, score)) = best_match {
            debug!("Found fuzzy match: {} (score: {})", app.name, score);
            Some(app)
        } else {
            debug!("No match found for: '{}'", query);
            None
        }
    }

    /// Launches an application by its exec command
    pub fn launch_app(&self, app_info: &AppInfo) -> Result<()> {
        info!("Launching application: {} ({})", app_info.name, app_info.exec);

        // Parse the exec command, removing field codes like %u, %U, %f, %F
        let exec_cleaned = app_info
            .exec
            .split_whitespace()
            .filter(|part| !part.starts_with('%'))
            .collect::<Vec<_>>()
            .join(" ");

        let parts: Vec<&str> = exec_cleaned.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow!("Empty exec command"));
        }

        let command = parts[0];
        let args = &parts[1..];

        debug!("Executing: {} {:?}", command, args);

        // CRITICAL: Use /bin/sh -c to avoid X11 threading issues
        #[cfg(target_os = "linux")]
        {
            let full_command = if args.is_empty() {
                command.to_string()
            } else {
                format!("{} {}", command, args.join(" "))
            };

            Command::new("/bin/sh")
                .arg("-c")
                .arg(&full_command)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| anyhow!("Failed to launch {}: {}", app_info.name, e))?;
        }

        #[cfg(not(target_os = "linux"))]
        {
            Command::new(command)
                .args(args)
                .spawn()
                .map_err(|e| anyhow!("Failed to launch {}: {}", app_info.name, e))?;
        }

        info!("Successfully launched: {}", app_info.name);
        Ok(())
    }
}
```

### 3. Update managers/mod.rs

Add the new module to `src-tauri/src/managers/mod.rs`:

```rust
pub mod app_launcher;  // Add this line
pub mod audio;
pub mod history;
pub mod model;
pub mod transcription;
```

### 4. Add the Open App Action

Update `src-tauri/src/actions.rs` to include the new action:

```rust
use crate::managers::app_launcher::AppLauncherManager;
use std::sync::Arc;

// Add this struct
pub struct OpenAppAction;

impl Action for OpenAppAction {
    fn start(&self, app: &AppHandle, id: &str, shortcut: &str) {
        debug!("OpenAppAction start called for id: {} shortcut: {}", id, shortcut);

        let app_launcher_manager = app.state::<Arc<AppLauncherManager>>().clone();
        let audio_manager = app.state::<Arc<AudioRecordingManager>>().clone();
        let transcription_manager = app.state::<Arc<TranscriptionManager>>().clone();
        let app_handle = app.clone();
        let shortcut_owned = shortcut.to_string();

        tauri::async_runtime::spawn(async move {
            // Record audio
            let audio_result = audio_manager.record_audio(true).await;

            match audio_result {
                Ok(audio) => {
                    debug!("Audio recording completed");

                    // Transcribe
                    match transcription_manager.transcribe_audio(&audio, None).await {
                        Ok(text) => {
                            info!("Transcription for app launcher: '{}'", text);

                            // Find and launch app
                            if let Some(app_info) = app_launcher_manager.find_app(&text) {
                                info!("Found app: {} for query '{}'", app_info.name, text);

                                if let Err(e) = app_launcher_manager.launch_app(&app_info) {
                                    error!("Failed to launch app: {}", e);
                                }
                            } else {
                                info!("No app found for: '{}'", text);
                            }
                        }
                        Err(e) => {
                            error!("Transcription failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Recording failed: {}", e);
                }
            }
        });
    }

    fn stop(&self, _app: &AppHandle, _id: &str, _shortcut: &str) {
        // Stop recording if active
    }
}

// Update ACTION_MAP
lazy_static! {
    pub static ref ACTION_MAP: HashMap<&'static str, Box<dyn Action + Send + Sync>> = {
        let mut m: HashMap<&'static str, Box<dyn Action + Send + Sync>> = HashMap::new();
        m.insert("transcribe", Box::new(TranscribeAction));
        m.insert("open_app", Box::new(OpenAppAction));  // Add this line
        m
    };
}
```

### 5. Update Settings

Update `src-tauri/src/settings.rs` to include the new shortcut:

```rust
pub fn get_default_settings() -> AppSettings {
    // ... existing code ...

    let mut bindings = HashMap::new();

    // Existing transcribe binding
    bindings.insert(
        "transcribe".to_string(),
        ShortcutBinding {
            id: "transcribe".to_string(),
            name: "Transcribe".to_string(),
            description: "Converts your speech into text.".to_string(),
            default_binding: default_shortcut.to_string(),
            current_binding: default_shortcut.to_string(),
        },
    );

    // Add open_app binding
    bindings.insert(
        "open_app".to_string(),
        ShortcutBinding {
            id: "open_app".to_string(),
            name: "Open Application".to_string(),
            description: "Opens an application by voice command.".to_string(),
            default_binding: "ctrl+shift+space".to_string(),
            current_binding: "ctrl+shift+space".to_string(),
        },
    );

    AppSettings {
        bindings,
        // ... rest of settings ...
    }
}
```

### 6. Initialize the Manager in lib.rs

Update `src-tauri/src/lib.rs`:

```rust
use managers::app_launcher::AppLauncherManager;

// In the run() function, after other manager initialization:
pub fn run() {
    // Initialize X11 threading on Linux (CRITICAL!)
    #[cfg(target_os = "linux")]
    {
        unsafe {
            x11::xlib::XInitThreads();
        }
    }

    tauri::Builder::default()
        // ... existing plugins ...
        .setup(move |app| {
            // ... existing managers ...

            // Add app launcher manager
            let app_launcher_manager = Arc::new(
                AppLauncherManager::new(&app).expect("Failed to initialize app launcher manager"),
            );

            app.manage(app_launcher_manager.clone());

            // ... rest of setup ...
        })
        // ... rest of builder ...
}
```

### 7. Important Considerations

#### X11 Threading Issues on Linux

**Problem**: Direct process spawning from Tauri can cause XCB threading errors:
```
[xcb] Unknown sequence number while processing queue
[xcb] Most likely this is a multi-threaded client and XInitThreads has not been called
```

**Solution**:
1. Initialize X11 threads at startup: `x11::xlib::XInitThreads()`
2. Spawn GUI apps through `/bin/sh -c` with stdio redirected to null
3. This isolates child processes from parent's X11 context

#### Fuzzy Matching Improvements

The implementation handles common speech-to-text issues:
- **Punctuation removal**: "Firefox." → "firefox"
- **Case insensitive**: "CHROME" → "chrome"
- **First word matching**: "Google Chrome" can match with just "Google"
- **Score boosting**: Apps starting with the query get higher priority

#### Desktop Entry Scanning

The app scans multiple locations for .desktop files:
- `/usr/share/applications` - System apps
- `~/.local/share/applications` - User apps
- `/var/lib/flatpak/exports/share/applications` - Flatpak apps
- `/var/lib/snapd/desktop/applications` - Snap apps

### 8. Testing the Feature

1. **Build and run**:
   ```bash
   bun install
   bun run tauri dev
   ```

2. **Test the shortcut**:
   - Press CTRL+SHIFT+SPACE
   - Say an application name (e.g., "Firefox", "Calculator", "Terminal")
   - The application should launch

3. **Debug logging**:
   ```bash
   RUST_LOG=debug bun run tauri dev
   ```

### 9. Common Issues and Solutions

| Issue | Solution |
|-------|----------|
| XCB threading crash | Ensure XInitThreads is called and use shell spawning |
| Apps not found | Check if .desktop files exist and are readable |
| Fuzzy matching fails | Verify punctuation stripping and case normalization |
| No apps loaded | Check desktop entry paths and permissions |

### 10. File Changes Summary

**Modified files**:
- `src-tauri/Cargo.toml` - New dependencies
- `src-tauri/src/lib.rs` - Manager initialization and X11 setup
- `src-tauri/src/actions.rs` - OpenAppAction implementation
- `src-tauri/src/settings.rs` - Default shortcut binding
- `src-tauri/src/managers/mod.rs` - Module declaration

**New files**:
- `src-tauri/src/managers/app_launcher.rs` - Complete app launcher implementation

## Conclusion

This implementation adds a robust, voice-activated application launcher to Handy. The feature handles common edge cases like transcription punctuation, case sensitivity, and Linux-specific X11 threading issues. The fuzzy matching ensures applications can be found even with imperfect speech recognition.

The architecture follows Handy's existing patterns with a dedicated manager, action handler, and settings integration, making it maintainable and consistent with the codebase.