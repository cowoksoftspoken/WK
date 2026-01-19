use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomMetadata {
    pub created_at: Option<String>,
    pub software: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub fields: HashMap<String, MetadataValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetadataValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Bytes(Vec<u8>),
    Array(Vec<MetadataValue>),
}

impl From<String> for MetadataValue {
    fn from(s: String) -> Self {
        MetadataValue::String(s)
    }
}

impl From<&str> for MetadataValue {
    fn from(s: &str) -> Self {
        MetadataValue::String(s.to_string())
    }
}

impl From<i64> for MetadataValue {
    fn from(v: i64) -> Self {
        MetadataValue::Int(v)
    }
}

impl From<f64> for MetadataValue {
    fn from(v: f64) -> Self {
        MetadataValue::Float(v)
    }
}

impl From<bool> for MetadataValue {
    fn from(v: bool) -> Self {
        MetadataValue::Bool(v)
    }
}

impl CustomMetadata {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        Self {
            created_at: Some(format!("{}", now)),
            software: Some("WK Image Format v2.0".into()),
            author: None,
            description: None,
            fields: HashMap::new(),
        }
    }
    
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<MetadataValue>) {
        self.fields.insert(key.into(), value.into());
    }
    
    pub fn get(&self, key: &str) -> Option<&MetadataValue> {
        self.fields.get(key)
    }
    
    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.fields.get(key) {
            Some(MetadataValue::String(s)) => Some(s),
            _ => None,
        }
    }
    
    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.fields.get(key) {
            Some(MetadataValue::Int(v)) => Some(*v),
            _ => None,
        }
    }
    
    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.fields.get(key) {
            Some(MetadataValue::Float(v)) => Some(*v),
            _ => None,
        }
    }
    
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.fields.get(key) {
            Some(MetadataValue::Bool(v)) => Some(*v),
            _ => None,
        }
    }
    
    pub fn remove(&mut self, key: &str) -> Option<MetadataValue> {
        self.fields.remove(key)
    }
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }
    
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.fields.keys()
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&String, &MetadataValue)> {
        self.fields.iter()
    }
}
