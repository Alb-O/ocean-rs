//! Bevy Screenshot Harness
//!
//! A headless screenshot capture system for Bevy examples.
//! Provides automated multi-angle screenshot capture with configurable presets.

#![allow(dead_code)]

mod cleanup;
mod config;
mod headless;
mod image_copy;
mod plugin;
mod presets;
mod state;
mod systems;

pub use cleanup::cleanup_old_sessions;
pub use config::{CliArgs, ScreenshotConfig};
pub use headless::{default_example_plugins, headless_runner};
pub use plugin::{HarnessCameraReady, ScreenshotHarnessPlugin};
pub use presets::{
	CameraPreset, DETAIL_PRESETS, MIN_CAMERA_HEIGHT, SIMPLE_PRESETS, STANDARD_PRESETS,
};
pub use state::{ScreenshotPhase, ScreenshotState};
pub use systems::{MainCamera, setup_camera};
