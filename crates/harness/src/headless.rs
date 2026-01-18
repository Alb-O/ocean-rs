//! Headless rendering configuration.

use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::settings::{InstanceFlags, RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;

/// Creates the default plugins for headless rendering (no window spawned).
///
/// Usage:
/// ```ignore
/// App::new()
///     .add_plugins(default_example_plugins(None))
///     // ...
/// ```
pub fn default_example_plugins(log_filter: Option<&str>) -> bevy::app::PluginGroupBuilder {
    let filter = log_filter.unwrap_or(
        "wgpu=off,wgpu_hal=off,naga=off,bevy_render=off,bevy_diagnostic=off,bevy_winit=off",
    );

    DefaultPlugins
        .set(WindowPlugin {
            primary_window: None,
            exit_condition: ExitCondition::DontExit,
            ..default()
        })
        .set(LogPlugin {
            filter: filter.to_string(),
            ..default()
        })
        .set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                instance_flags: InstanceFlags::empty(),
                ..default()
            }),
            ..default()
        })
        .set(ImagePlugin::default_nearest())
        .disable::<WinitPlugin>()
}

/// Creates a ScheduleRunnerPlugin for headless operation
pub fn headless_runner() -> ScheduleRunnerPlugin {
    ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0))
}
