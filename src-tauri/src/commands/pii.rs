use crate::pii_redactor::{PIIRedactor, DEFAULT_PII_ENTITIES};
use crate::settings::{get_settings, write_settings};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn get_pii_redaction_enabled(app: AppHandle) -> bool {
    let settings = get_settings(&app);
    settings.pii_redaction_enabled
}

#[tauri::command]
pub fn set_pii_redaction_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.pii_redaction_enabled = enabled;
    write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
pub fn get_pii_entities(app: AppHandle) -> Vec<String> {
    let settings = get_settings(&app);
    settings.pii_entities
}

#[tauri::command]
pub fn set_pii_entities(app: AppHandle, entities: Vec<String>) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.pii_entities = entities;
    write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
pub fn get_default_pii_entities() -> Vec<String> {
    DEFAULT_PII_ENTITIES.iter().map(|s| s.to_string()).collect()
}

#[tauri::command]
pub fn is_pii_model_loaded(app: AppHandle) -> bool {
    let pii_redactor = app.state::<Arc<PIIRedactor>>();
    pii_redactor.is_model_loaded()
}

#[tauri::command]
pub fn load_pii_model(app: AppHandle) -> Result<(), String> {
    let pii_redactor = app.state::<Arc<PIIRedactor>>();
    pii_redactor.load_model()
        .map_err(|e| format!("Failed to load PII model: {}", e))
}

#[tauri::command]
pub fn unload_pii_model(app: AppHandle) -> Result<(), String> {
    let pii_redactor = app.state::<Arc<PIIRedactor>>();
    pii_redactor.unload_model();
    Ok(())
}

#[tauri::command]
pub fn get_show_pii_entity_labels(app: AppHandle) -> bool {
    let settings = get_settings(&app);
    settings.show_pii_entity_labels
}

#[tauri::command]
pub fn set_show_pii_entity_labels(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.show_pii_entity_labels = enabled;
    write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
pub fn test_pii_redaction(app: AppHandle, text: String) -> Result<String, String> {
    let settings = get_settings(&app);
    let pii_redactor = app.state::<Arc<PIIRedactor>>();

    pii_redactor.redact_text(&text, &settings.pii_entities, settings.show_pii_entity_labels)
        .map_err(|e| format!("Failed to redact PII: {}", e))
}