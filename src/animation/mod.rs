pub mod frame;

pub use frame::{AnimationConfig, AnimationFrame, BlendMode, DisposeMode};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    pub config: AnimationConfig,
    pub frames: Vec<AnimationFrame>,
}

impl Animation {
    pub fn new() -> Self {
        Self {
            config: AnimationConfig::default(),
            frames: Vec::new(),
        }
    }

    pub fn with_loop_count(mut self, count: u32) -> Self {
        self.config.loop_count = count;
        self
    }

    pub fn infinite_loop(mut self) -> Self {
        self.config.loop_count = 0;
        self
    }

    pub fn add_frame(&mut self, frame: AnimationFrame) {
        self.frames.push(frame);
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn total_duration_ms(&self) -> u32 {
        self.frames.iter().map(|f| f.delay_ms).sum()
    }

    pub fn is_animated(&self) -> bool {
        self.frames.len() > 1
    }
}

impl Default for Animation {
    fn default() -> Self {
        Self::new()
    }
}
