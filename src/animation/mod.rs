pub mod frame;
pub mod motion;

pub use frame::{AnimationConfig, AnimationFrame, BlendMode, DisposeMode};
pub use motion::{apply_motion_compensation, MotionEstimator, MotionVector, SearchPattern};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameType {
    IFrame,
    PFrame,
}

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

    pub fn add_keyframe(&mut self, frame: AnimationFrame) {
        let mut f = frame;
        f.is_keyframe = true;
        self.frames.push(f);
    }

    pub fn add_delta_frame(&mut self, frame: AnimationFrame) {
        let mut f = frame;
        f.is_keyframe = false;
        self.frames.push(f);
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

    pub fn keyframe_count(&self) -> usize {
        self.frames.iter().filter(|f| f.is_keyframe).count()
    }

    pub fn get_keyframe_before(&self, index: usize) -> Option<usize> {
        (0..=index)
            .rev()
            .find(|&i| self.frames.get(i).map_or(false, |f| f.is_keyframe))
    }
}

impl Default for Animation {
    fn default() -> Self {
        Self::new()
    }
}
