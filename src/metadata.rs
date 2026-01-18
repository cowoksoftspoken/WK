use crate::error::{WkError, WkResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WkMetadata {
    pub created_at: Option<String>,
    pub software: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub custom_fields: HashMap<String, String>,
}

impl WkMetadata {
    pub fn new() -> Self {
        Self {
            created_at: Some(format!("{:?}", std::time::SystemTime::now())),
            software: Some("Wk-image-format v0.1.0".to_string()),
            author: Some("Inggrit Setya Budi".to_string()),
            description: Some("Wk image file".to_string()),
            custom_fields: HashMap::new(),
        }
    }

    pub fn encode(&self) -> WkResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| WkError::EncodingError(format!("Metadata encoding failed: {}", e)))
    }

    pub fn decode(data: &[u8]) -> WkResult<Self> {
        bincode::deserialize(data)
            .map_err(|e| WkError::DecodingError(format!("Metadata decoding failed: {}", e)))
    }

    pub fn add_custom_field(&mut self, key: String, value: String) {
        self.custom_fields.insert(key, value);
    }
}
