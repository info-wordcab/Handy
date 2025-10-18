use anyhow::{anyhow, Result};
use freedesktop_desktop_entry::DesktopEntry;
use log::{error, info};
use nucleo::{Config, Matcher, Utf32String};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

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
    pub fn new(_app: &AppHandle) -> Result<Self> {
        let manager = Self {
            apps: Arc::new(Mutex::new(HashMap::new())),
        };

        // Load apps on initialization
        manager.load_apps()?;
        Ok(manager)
    }

    /// Scans all desktop entry directories and loads application information
    pub fn load_apps(&self) -> Result<()> {
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
                        if let Ok(desktop_entry) = DesktopEntry::from_path::<&str>(&file_path, None) {
                            // Only add visible applications
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

                            apps.insert(app_info.name.to_lowercase(), app_info);
                            app_count += 1;
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

        // First try exact match with cleaned query
        if let Some(app) = apps.get(&cleaned_query) {
            return Some(app.clone());
        }

        // Try matching just the first word
        let first_word = cleaned_query.split_whitespace().next().unwrap_or(&cleaned_query);
        if let Some(app) = apps.get(first_word) {
            return Some(app.clone());
        }

        // Then try fuzzy matching with nucleo
        let mut matcher = Matcher::new(Config::DEFAULT);
        let mut best_match: Option<(AppInfo, u16)> = None;

        for (_, app_info) in apps.iter() {
            // Create a search space including name, generic name, and keywords
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

        best_match.map(|(app, _)| app)
    }

    /// Launches an application by its exec command
    pub fn launch_app(&self, app_info: &AppInfo) -> Result<()> {
        info!("Launching application: {}", app_info.name);

        // Parse the exec command, removing field codes like %u, %U, %f, %F
        let exec_cleaned = app_info
            .exec
            .split_whitespace()
            .filter(|part| !part.starts_with('%'))
            .collect::<Vec<_>>()
            .join(" ");

        // Split into command and arguments
        let parts: Vec<&str> = exec_cleaned.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow!("Empty exec command"));
        }

        let command = parts[0];
        let args = &parts[1..];

        // Launch the command in the background
        // Use /bin/sh -c to ensure the command runs in a fresh shell context
        // This helps avoid X11 threading issues from the parent process
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

    /// Returns a list of all application names
    pub fn get_app_names(&self) -> Vec<String> {
        let apps = self.apps.lock().unwrap();
        apps.values().map(|app| app.name.clone()).collect()
    }

    /// Refreshes the app list
    pub fn refresh(&self) -> Result<()> {
        self.load_apps()
    }
}