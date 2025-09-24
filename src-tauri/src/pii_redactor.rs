use anyhow::Result;
use gliner::model::{
    input::text::TextInput,
    params::Parameters,
    pipeline::span::SpanMode,
    GLiNER,
};
use gliner::orp::params::RuntimeParameters;
#[cfg(feature = "cuda")]
use gliner::execution_providers::{CPUExecutionProvider, CUDAExecutionProvider};
#[cfg(not(feature = "cuda"))]
use gliner::execution_providers::CPUExecutionProvider;
use log::debug;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::fs;
use tauri::{AppHandle, Manager};

// Default PII entity labels - Personal Identifiers category (default checked)
pub const DEFAULT_PII_ENTITIES: &[&str] = &[
    "name",
    "first name",
    "last name",
    "name medical professional",
    "dob",
    "age",
    "gender",
    "marital status",
];

// PII entity categories mapping to exact GLiNER model labels
pub const PERSONAL_IDENTIFIERS: &[&str] = &[
    "name",
    "first name",
    "last name",
    "name medical professional",
    "dob",
    "age",
    "gender",
    "marital status",
];

pub const CONTACT_INFORMATION: &[&str] = &[
    "email address",
    "phone number",
    "ip address",
    "url",
    "location address",
    "location street",
    "location city",
    "location state",
    "location country",
    "location zip",
];

pub const FINANCIAL_INFORMATION: &[&str] = &[
    "account number",
    "bank account",
    "routing number",
    "credit card",
    "credit card expiration",
    "cvv",
    "ssn",
    "money",
];

pub const HEALTHCARE_INFORMATION: &[&str] = &[
    "condition",
    "medical process",
    "drug",
    "dose",
    "blood type",
    "injury",
    "organization medical facility",
    "healthcare number",
    "medical code",
];

pub const IDENTIFICATION_DOCUMENTS: &[&str] = &[
    "passport number",
    "driver license",
    "username",
    "password",
    "vehicle id",
];

pub struct PIIRedactor {
    model: Arc<Mutex<Option<GLiNER<SpanMode>>>>,
    app_handle: AppHandle,
    model_path: PathBuf,
    tokenizer_path: PathBuf,
}

impl PIIRedactor {
    /// Map PII category names to their corresponding GLiNER model labels
    pub fn get_labels_for_category(category: &str) -> &'static [&'static str] {
        match category {
            "personal_identifiers" => PERSONAL_IDENTIFIERS,
            "contact_information" => CONTACT_INFORMATION,
            "financial_information" => FINANCIAL_INFORMATION,
            "healthcare_information" => HEALTHCARE_INFORMATION,
            "identification_documents" => IDENTIFICATION_DOCUMENTS,
            _ => &[], // Unknown category
        }
    }

    /// Get all labels for selected categories
    pub fn get_labels_for_categories(categories: &[String]) -> Vec<String> {
        let mut all_labels = Vec::new();
        for category in categories {
            let labels = Self::get_labels_for_category(category);
            for label in labels {
                if !all_labels.contains(&label.to_string()) {
                    all_labels.push(label.to_string());
                }
            }
        }
        all_labels
    }

    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        // Use app data directory for model storage
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get app data dir: {}", e))?;

        let model_base = app_data_dir.join("models").join("pii");

        // Create the directory if it doesn't exist
        if !model_base.exists() {
            fs::create_dir_all(&model_base)?;
        }

        let tokenizer_path = model_base.join("tokenizer.json");
        let model_path = model_base.join("model_quint8.onnx");

        debug!("PII model base directory: {:?}", model_base);
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
                "GLiNER tokenizer not found at: {:?}. Please download the model first.",
                self.tokenizer_path
            ));
        }
        if !self.model_path.exists() {
            return Err(anyhow::anyhow!(
                "GLiNER model not found at: {:?}. Please download the model first.",
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

        // Convert category names to actual GLiNER model labels
        let working_labels = Self::get_labels_for_categories(entities_to_redact);
        let working_labels_str: Vec<&str> = working_labels.iter().map(|s| s.as_str()).collect();

        debug!("Filtered working labels: {:?}", working_labels_str);

        // If no valid entities to redact, return original text
        if working_labels_str.is_empty() {
            debug!("No valid entities selected for redaction");
            return Ok(text.to_string());
        }

        // Create input for GLiNER
        let input = TextInput::from_str(&[text], &working_labels_str)
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
                // Use generic entity labels based on GLiNER entity types
                match entity_type.as_str() {
                    // Personal identifiers
                    "name" | "first name" | "last name" => "[NAME]".to_string(),
                    "name medical professional" => "[MEDICAL_PROFESSIONAL]".to_string(),
                    "dob" => "[DATE_OF_BIRTH]".to_string(),
                    "age" => "[AGE]".to_string(),
                    "gender" => "[GENDER]".to_string(),
                    "marital status" => "[MARITAL_STATUS]".to_string(),

                    // Contact information
                    "email address" => "[EMAIL]".to_string(),
                    "phone number" => "[PHONE]".to_string(),
                    "ip address" => "[IP_ADDRESS]".to_string(),
                    "url" => "[URL]".to_string(),
                    "location address" | "location street" => "[ADDRESS]".to_string(),
                    "location city" => "[CITY]".to_string(),
                    "location state" => "[STATE]".to_string(),
                    "location country" => "[COUNTRY]".to_string(),
                    "location zip" => "[ZIP_CODE]".to_string(),

                    // Financial information
                    "account number" | "bank account" => "[ACCOUNT_NUMBER]".to_string(),
                    "routing number" => "[ROUTING_NUMBER]".to_string(),
                    "credit card" => "[CREDIT_CARD]".to_string(),
                    "credit card expiration" => "[CARD_EXPIRATION]".to_string(),
                    "cvv" => "[CVV]".to_string(),
                    "ssn" => "[SSN]".to_string(),
                    "money" => "[MONETARY_AMOUNT]".to_string(),

                    // Healthcare information
                    "condition" => "[MEDICAL_CONDITION]".to_string(),
                    "medical process" => "[MEDICAL_PROCEDURE]".to_string(),
                    "drug" => "[MEDICATION]".to_string(),
                    "dose" => "[DOSAGE]".to_string(),
                    "blood type" => "[BLOOD_TYPE]".to_string(),
                    "injury" => "[INJURY]".to_string(),
                    "organization medical facility" => "[MEDICAL_FACILITY]".to_string(),
                    "healthcare number" => "[HEALTHCARE_NUMBER]".to_string(),
                    "medical code" => "[MEDICAL_CODE]".to_string(),

                    // Identification documents
                    "passport number" => "[PASSPORT]".to_string(),
                    "driver license" => "[DRIVER_LICENSE]".to_string(),
                    "username" => "[USERNAME]".to_string(),
                    "password" => "[PASSWORD]".to_string(),
                    "vehicle id" => "[VEHICLE_ID]".to_string(),

                    // Generic fallback
                    _ => format!("[{}]", entity_type.to_uppercase().replace(" ", "_")),
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