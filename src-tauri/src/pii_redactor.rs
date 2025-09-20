use anyhow::Result;
use gliner::model::{
    input::text::TextInput,
    params::Parameters,
    pipeline::token::TokenMode,
    GLiNER,
};
use gliner::orp::params::RuntimeParameters;
use log::debug;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

// Default PII entity labels
pub const DEFAULT_PII_ENTITIES: &[&str] = &[
    "person",
    "email",
    "phone_number",
    "social_security_number",
    "credit_card",
    "address",
    "date_of_birth",
    "passport_number",
    "driver_license",
    "bank_account",
];

pub struct PIIRedactor {
    model: Arc<Mutex<Option<GLiNER<TokenMode>>>>,
    app_handle: AppHandle,
    model_path: PathBuf,
    tokenizer_path: PathBuf,
}

impl PIIRedactor {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        // For development, use paths relative to the project
        // In production, these would be in the app's resource directory
        let model_base = PathBuf::from("TO_DO/onnx_models/gliner_multitask_large_v0_5");
        let tokenizer_path = model_base.join("tokenizer.json");
        let model_path = model_base.join("model_int8.onnx");

        Ok(Self {
            model: Arc::new(Mutex::new(None)),
            app_handle: app_handle.clone(),
            model_path,
            tokenizer_path,
        })
    }

    /// Initialize the GLiNER model lazily
    pub fn load_model(&self) -> Result<()> {
        let mut model_guard = self.model.lock().unwrap();

        if model_guard.is_some() {
            debug!("PII redaction model already loaded");
            return Ok(());
        }

        debug!("Loading PII redaction model from: {:?}", self.model_path);

        // Check if model files exist
        if !self.tokenizer_path.exists() {
            return Err(anyhow::anyhow!(
                "GLiNER tokenizer not found at: {:?}",
                self.tokenizer_path
            ));
        }
        if !self.model_path.exists() {
            return Err(anyhow::anyhow!(
                "GLiNER model not found at: {:?}",
                self.model_path
            ));
        }

        // Load the model
        let model = GLiNER::<TokenMode>::new(
            Parameters::default(),
            RuntimeParameters::default(),
            self.tokenizer_path.to_str().unwrap(),
            self.model_path.to_str().unwrap(),
        ).map_err(|e| anyhow::anyhow!("Failed to load GLiNER model: {:?}", e))?;

        *model_guard = Some(model);
        debug!("PII redaction model loaded successfully");

        Ok(())
    }

    /// Unload the model to free memory
    pub fn unload_model(&self) {
        let mut model_guard = self.model.lock().unwrap();
        if model_guard.is_some() {
            *model_guard = None;
            debug!("PII redaction model unloaded");
        }
    }

    /// Check if model is loaded
    pub fn is_model_loaded(&self) -> bool {
        let model_guard = self.model.lock().unwrap();
        model_guard.is_some()
    }

    /// Redact PII from text
    pub fn redact_text(&self, text: &str, entities_to_redact: &[String]) -> Result<String> {
        // Ensure model is loaded
        if !self.is_model_loaded() {
            self.load_model()?;
        }

        let model_guard = self.model.lock().unwrap();
        let model = model_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Model not loaded"))?;

        // If no entities to redact, return original text
        if entities_to_redact.is_empty() {
            return Ok(text.to_string());
        }

        // Prepare entity labels for detection
        let labels: Vec<&str> = entities_to_redact
            .iter()
            .map(|s| s.as_str())
            .collect();

        // Create input for GLiNER
        let input = TextInput::from_str(&[text], &labels)
            .map_err(|e| anyhow::anyhow!("Failed to create text input: {:?}", e))?;

        // Run inference
        let output = model.inference(input)
            .map_err(|e| anyhow::anyhow!("Failed to run inference: {:?}", e))?;

        // If no spans detected, return original text
        if output.spans.is_empty() || output.spans[0].is_empty() {
            return Ok(text.to_string());
        }

        // Sort spans by start position (reverse order for replacement)
        let mut spans_to_redact: Vec<_> = output.spans[0]
            .iter()
            .map(|span| {
                let (start, end) = span.offsets();
                (
                    start,
                    end,
                    span.class().to_string(),
                    span.text().to_string(),
                )
            })
            .collect();

        // Sort by start position in reverse order
        spans_to_redact.sort_by(|a, b| b.0.cmp(&a.0));

        // Replace detected entities with redacted versions
        let mut redacted_text = text.to_string();
        for (start, end, entity_type, _original_text) in spans_to_redact {
            // Create a redaction placeholder
            let redaction = format!("[{}_REDACTED]", entity_type.to_uppercase());

            // Calculate byte positions for proper replacement
            let byte_start = text.char_indices()
                .nth(start)
                .map(|(i, _)| i)
                .unwrap_or(0);
            let byte_end = text.char_indices()
                .nth(end)
                .map(|(i, _)| i)
                .unwrap_or(text.len());

            // Replace the text
            redacted_text.replace_range(byte_start..byte_end, &redaction);
        }

        Ok(redacted_text)
    }

    /// Redact multiple texts in batch
    pub fn redact_texts(&self, texts: &[String], entities_to_redact: &[String]) -> Result<Vec<String>> {
        texts
            .iter()
            .map(|text| self.redact_text(text, entities_to_redact))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction_placeholder_format() {
        // Test that redaction placeholders are correctly formatted
        let entity_type = "person";
        let redaction = format!("[{}_REDACTED]", entity_type.to_uppercase());
        assert_eq!(redaction, "[PERSON_REDACTED]");
    }
}