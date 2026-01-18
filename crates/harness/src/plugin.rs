//! Screenshot harness plugin.

use bevy::prelude::*;

use crate::config::ScreenshotConfig;
use crate::image_copy::ImageCopyPlugin;
use crate::state::ScreenshotState;
use crate::systems::{prepare_screenshot_dir, screenshot_sequence, setup_camera};

/// Marker resource indicating the harness camera setup is complete.
/// Use with `run_if(resource_exists::<HarnessCameraReady>)` to order systems after camera setup.
#[derive(Resource, Default)]
pub struct HarnessCameraReady;

/// Plugin that provides the screenshot harness for examples.
pub struct ScreenshotHarnessPlugin {
    config: ScreenshotConfig,
}

impl ScreenshotHarnessPlugin {
    pub fn new(example_name: &str) -> Self {
        Self {
            config: ScreenshotConfig::from_cli(example_name),
        }
    }

    pub fn with_config(config: ScreenshotConfig) -> Self {
        Self { config }
    }
}

impl Plugin for ScreenshotHarnessPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone())
            .init_resource::<ScreenshotState>()
            .add_plugins(ImageCopyPlugin)
            .add_systems(Startup, (setup_camera, prepare_screenshot_dir))
            .add_systems(PostUpdate, screenshot_sequence);
    }
}
