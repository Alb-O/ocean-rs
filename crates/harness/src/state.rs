//! Screenshot capture state machine.

use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;

/// Number of frames to wait before capturing (allows scene to fully render)
pub const PRE_ROLL_FRAMES: u32 = 60;

/// Number of frames to wait between shots for scene to settle
pub const SETTLE_FRAMES: u32 = 30;

/// Current state of the screenshot sequence
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenshotPhase {
    /// Waiting for initial scene render
    Init(u32),
    /// Settling before capture
    Settling(u32),
    /// Ready to capture
    Capturing,
    /// All done
    Done,
}

impl Default for ScreenshotPhase {
    fn default() -> Self {
        Self::Init(PRE_ROLL_FRAMES)
    }
}

/// Screenshot state resource
#[derive(Resource)]
pub struct ScreenshotState {
    pub phase: ScreenshotPhase,
    pub current_preset: usize,
    pub session_dir: String,
    pub captured_paths: Vec<String>,
    pub render_target: Option<Handle<Image>>,
}

impl Default for ScreenshotState {
    fn default() -> Self {
        let session_dir = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .to_string();

        Self {
            phase: ScreenshotPhase::default(),
            current_preset: 0,
            session_dir,
            captured_paths: Vec::new(),
            render_target: None,
        }
    }
}
