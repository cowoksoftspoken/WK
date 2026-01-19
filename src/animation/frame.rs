use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    Source,
    Over,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Source
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisposeMode {
    None,
    Background,
    Previous,
}

impl Default for DisposeMode {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    pub loop_count: u32,
    pub background_color: [u8; 4],
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            loop_count: 0,
            background_color: [0, 0, 0, 0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationFrame {
    pub delay_ms: u32,
    pub x_offset: u32,
    pub y_offset: u32,
    pub width: u32,
    pub height: u32,
    pub blend_mode: BlendMode,
    pub dispose_mode: DisposeMode,
    pub data: Vec<u8>,
}

impl AnimationFrame {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            delay_ms: 100,
            x_offset: 0,
            y_offset: 0,
            width,
            height,
            blend_mode: BlendMode::default(),
            dispose_mode: DisposeMode::default(),
            data,
        }
    }

    pub fn with_delay(mut self, delay_ms: u32) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    pub fn with_offset(mut self, x: u32, y: u32) -> Self {
        self.x_offset = x;
        self.y_offset = y;
        self
    }

    pub fn with_blend_mode(mut self, mode: BlendMode) -> Self {
        self.blend_mode = mode;
        self
    }

    pub fn with_dispose_mode(mut self, mode: DisposeMode) -> Self {
        self.dispose_mode = mode;
        self
    }
}
