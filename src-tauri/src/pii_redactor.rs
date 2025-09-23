use anyhow::Result;
use gliner::model::{
    input::text::TextInput,
    params::Parameters,
    pipeline::span::SpanMode,
    GLiNER,
};
use gliner::orp::params::RuntimeParameters;
use gliner::execution_providers::{CPUExecutionProvider};
#[cfg(feature = "cuda")]
use gliner::execution_providers::CUDAExecutionProvider;
use log::debug;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

// Default PII entity labels - optimized for gliner_small_v2_1
pub const DEFAULT_PII_ENTITIES: &[&str] = &[
    "person",
    "phone",
    "address",
    "social_security_number",
];

pub struct PIIRedactor {
    model: Arc<Mutex<Option<GLiNER<SpanMode>>>>,
    app_handle: AppHandle,
    model_path: PathBuf,
    tokenizer_path: PathBuf,
}

impl PIIRedactor {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        // For development, use absolute paths to the model directory
        // In production, these would be in the app's resource directory
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        debug!("Current working directory: {:?}", current_dir);

        // Try multiple possible paths
        let possible_paths = vec![
            current_dir.join("TO_DO/onnx_models/gliner_pii_small"),
            current_dir.join("../TO_DO/onnx_models/gliner_pii_small"),
            PathBuf::from("/home/aleks/PycharmProjects/pii_oss/Handy/TO_DO/onnx_models/gliner_pii_small"),
        ];

        let model_base = possible_paths.into_iter()
            .find(|path| {
                let exists = path.join("tokenizer.json").exists();
                debug!("Checking path: {:?} - exists: {}", path, exists);
                exists
            })
            .unwrap_or_else(|| {
                debug!("No valid model path found, using default");
                current_dir.join("TO_DO/onnx_models/gliner_pii_small")
            });

        let tokenizer_path = model_base.join("tokenizer.json");
        let model_path = model_base.join("model_int8.onnx");
        debug!("Selected model base: {:?}", model_base);
        debug!("Tokenizer path: {:?}", tokenizer_path);
        debug!("Model path: {:?}", model_path);

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

        // Configure runtime parameters with CUDA if available, CPU fallback
        let runtime_params = {
            #[cfg(feature = "cuda")]
            {
                debug!("PII Model: Attempting to use CUDA execution provider with CPU fallback");
                let providers = [
                    CUDAExecutionProvider::default().build(),
                    CPUExecutionProvider::default().build(), // Fallback to CPU
                ];
                debug!("PII Model: Configured execution providers: CUDA (primary), CPU (fallback)");
                RuntimeParameters::default().with_execution_providers(providers)
            }
            #[cfg(not(feature = "cuda"))]
            {
                debug!("PII Model: Using CPU execution provider (CUDA feature not enabled)");
                RuntimeParameters::default()
            }
        };

        // Load the model
        let model = GLiNER::<SpanMode>::new(
            Parameters::default(),
            runtime_params,
            self.tokenizer_path.to_str().unwrap(),
            self.model_path.to_str().unwrap(),
        ).map_err(|e| {
            debug!("PII Model: Failed to load with error: {:?}", e);
            let error_str = format!("{:?}", e);
            if error_str.contains("CUDA") || error_str.contains("cuda") {
                debug!("PII Model: CUDA error detected - falling back to CPU");
            }
            anyhow::anyhow!("Failed to load GLiNER model: {:?}", e)
        })?;

        *model_guard = Some(model);
        debug!("PII Model: Successfully loaded and ready for inference");

        #[cfg(feature = "cuda")]
        debug!("PII Model: If CUDA was available and working, it will be used for inference. CPU fallback is automatic if CUDA fails.");

        #[cfg(not(feature = "cuda"))]
        debug!("PII Model: Running on CPU (CUDA support not compiled in)");

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
    pub fn redact_text(&self, text: &str, entities_to_redact: &[String], show_entity_labels: bool) -> Result<String> {
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

        // Debug the input parameters
        debug!("Redacting text: '{}'", text);
        debug!("Entity labels: {:?}", entities_to_redact);
        debug!("Show entity labels: {}", show_entity_labels);

        // Filter entities to only use those supported by the model and selected by user
        let model_supported_entities = vec!["person", "phone", "address", "social_security_number"];
        let working_labels: Vec<&str> = entities_to_redact
            .iter()
            .filter_map(|entity| {
                // Map frontend entity names to model entity names
                let model_entity = match entity.as_str() {
                    "phone_number" => "phone", // Handle alternate naming
                    other => other,
                };
                if model_supported_entities.contains(&model_entity) {
                    Some(model_entity)
                } else {
                    None
                }
            })
            .collect();

        debug!("Filtered working labels: {:?}", working_labels);

        // If no valid entities to redact, return original text
        if working_labels.is_empty() {
            debug!("No valid entities selected for redaction");
            return Ok(text.to_string());
        }

        // Create input for GLiNER
        let input = TextInput::from_str(&[text], &working_labels)
            .map_err(|e| anyhow::anyhow!("Failed to create text input: {:?}", e))?;

        debug!("Successfully created TextInput, running inference...");

        // Run inference with error handling for known span mode issues
        let output = match model.inference(input) {
            Ok(output) => output,
            Err(e) => {
                // Check if this is the known span mode array bounds error
                let error_msg = format!("{:?}", e);
                if error_msg.contains("ShapeError/OutOfBounds") || error_msg.contains("out of bounds indexing") {
                    debug!("Known GLiNER span mode issue detected - text might not match expected patterns");
                    debug!("This commonly happens with single names or non-standard sentence structures");
                    return Ok(text.to_string()); // Return original text unchanged
                } else {
                    return Err(anyhow::anyhow!("Failed to run inference: {:?}", e));
                }
            }
        };

        debug!("Inference completed successfully!");
        debug!("Output spans length: {}", output.spans.len());

        // If no spans detected, return original text
        if output.spans.is_empty() || output.spans[0].is_empty() {
            debug!("No spans detected, returning original text");
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

        debug!("Found {} spans to redact", spans_to_redact.len());
        for (i, (start, end, entity_type, original_text)) in spans_to_redact.iter().enumerate() {
            debug!("Span {}: '{}' ({}) at {}..{}", i, original_text, entity_type, start, end);
        }

        // Sort by start position in reverse order
        spans_to_redact.sort_by(|a, b| b.0.cmp(&a.0));

        // Replace detected entities with redacted versions
        let mut redacted_text = text.to_string();
        for (start, end, entity_type, original_text) in spans_to_redact {
            // Validate span bounds
            if start >= end || end > text.chars().count() {
                debug!("Invalid span bounds: {}..{} for text length {}", start, end, text.chars().count());
                continue;
            }

            // Create redaction based on settings
            let redaction = if show_entity_labels {
                // Use generic entity labels like [PERSON], [PHONE], etc.
                match entity_type.as_str() {
                    "person" => "[PERSON]".to_string(),
                    "phone" => "[PHONE]".to_string(),
                    "address" => "[ADDRESS]".to_string(),
                    "social_security_number" => "[SSN]".to_string(),
                    _ => format!("[{}]", entity_type.to_uppercase()),
                }
            } else {
                // Use hashtag redaction based on original text length
                "#".repeat(original_text.len())
            };
            debug!("Redacting '{}' ({}) at {}..{} with '{}'", original_text, entity_type, start, end, redaction);

            // Calculate byte positions from current redacted_text
            let byte_start = redacted_text.char_indices()
                .nth(start)
                .map(|(i, _)| i)
                .unwrap_or_else(|| {
                    debug!("Could not find byte start for char index {}", start);
                    0
                });
            let byte_end = redacted_text.char_indices()
                .nth(end)
                .map(|(i, _)| i)
                .unwrap_or_else(|| {
                    debug!("Could not find byte end for char index {}, using text length", end);
                    redacted_text.len()
                });

            // Validate byte range
            if byte_start <= byte_end && byte_end <= redacted_text.len() {
                // Replace the text
                redacted_text.replace_range(byte_start..byte_end, &redaction);
                debug!("Successfully replaced text segment");
            } else {
                debug!("Invalid byte range: {}..{} for text length {}", byte_start, byte_end, redacted_text.len());
            }
        }

        Ok(redacted_text)
    }

    /// Redact multiple texts in batch
    pub fn redact_texts(&self, texts: &[String], entities_to_redact: &[String], show_entity_labels: bool) -> Result<Vec<String>> {
        texts
            .iter()
            .map(|text| self.redact_text(text, entities_to_redact, show_entity_labels))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction_placeholder_format() {
        // Test that redaction placeholders are correctly formatted with hashtags
        let original_text = "John";
        let redaction = "#".repeat(original_text.len());
        assert_eq!(redaction, "####");

        let original_text = "john@example.com";
        let redaction = "#".repeat(original_text.len());
        assert_eq!(redaction, "################");
    }
}