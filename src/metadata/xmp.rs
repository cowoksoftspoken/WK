use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XmpData {
    pub title: Option<String>,
    pub description: Option<String>,
    pub creator: Vec<String>,
    pub subject: Vec<String>,
    pub rights: Option<String>,
    pub rating: Option<u8>,
    pub label: Option<String>,
    pub marked: Option<bool>,
    pub create_date: Option<String>,
    pub modify_date: Option<String>,
    pub creator_tool: Option<String>,
    pub custom: HashMap<String, String>,
}

impl XmpData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> XmpBuilder {
        XmpBuilder::new()
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
    }

    pub fn add_creator(&mut self, creator: impl Into<String>) {
        self.creator.push(creator.into());
    }

    pub fn add_subject(&mut self, subject: impl Into<String>) {
        self.subject.push(subject.into());
    }

    pub fn set_rating(&mut self, rating: u8) {
        self.rating = Some(rating.clamp(0, 5));
    }

    pub fn set_rights(&mut self, rights: impl Into<String>) {
        self.rights = Some(rights.into());
    }

    pub fn set_custom(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.custom.insert(key.into(), value.into());
    }

    pub fn get_custom(&self, key: &str) -> Option<&str> {
        self.custom.get(key).map(|s| s.as_str())
    }
}

pub struct XmpBuilder {
    data: XmpData,
}

impl XmpBuilder {
    pub fn new() -> Self {
        Self {
            data: XmpData::new(),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.data.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.data.description = Some(description.into());
        self
    }

    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        self.data.creator.push(creator.into());
        self
    }

    pub fn creators(mut self, creators: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for c in creators {
            self.data.creator.push(c.into());
        }
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.data.subject.push(subject.into());
        self
    }

    pub fn subjects(mut self, subjects: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for s in subjects {
            self.data.subject.push(s.into());
        }
        self
    }

    pub fn rights(mut self, rights: impl Into<String>) -> Self {
        self.data.rights = Some(rights.into());
        self
    }

    pub fn rating(mut self, rating: u8) -> Self {
        self.data.rating = Some(rating.clamp(0, 5));
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.data.label = Some(label.into());
        self
    }

    pub fn marked(mut self, marked: bool) -> Self {
        self.data.marked = Some(marked);
        self
    }

    pub fn create_date(mut self, date: impl Into<String>) -> Self {
        self.data.create_date = Some(date.into());
        self
    }

    pub fn modify_date(mut self, date: impl Into<String>) -> Self {
        self.data.modify_date = Some(date.into());
        self
    }

    pub fn creator_tool(mut self, tool: impl Into<String>) -> Self {
        self.data.creator_tool = Some(tool.into());
        self
    }

    pub fn custom(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.custom.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> XmpData {
        self.data
    }
}

impl Default for XmpBuilder {
    fn default() -> Self {
        Self::new()
    }
}
