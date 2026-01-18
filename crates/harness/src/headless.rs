//! Rendering configuration for headless and interactive modes.

use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::log::{Level, LogPlugin, tracing_subscriber};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{InstanceFlags, RenderCreation, WgpuSettings};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::config::CliArgs;

/// Absolute path to workspace root, computed at compile time.
pub const WORKSPACE_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// Sets `BEVY_ASSET_ROOT` to the workspace root.
///
/// Call this before creating your App to ensure assets are loaded from the
/// workspace's `assets/` directory regardless of which crate the example is in.
///
/// # Safety
/// This modifies environment variables, which is safe when called at the start
/// of main() before any other threads are spawned.
pub fn set_workspace_asset_root() {
	// SAFETY: Called at start of main before any threads are spawned
	unsafe {
		std::env::set_var("BEVY_ASSET_ROOT", WORKSPACE_ROOT);
	}
}

fn log_plugin(filter: &str) -> LogPlugin {
	LogPlugin {
		filter: filter.to_string(),
		level: Level::INFO,
		custom_layer: |_| None,
		fmt_layer: |_| {
			Some(Box::new(
				tracing_subscriber::fmt::layer()
					.compact()
					.without_time()
					.with_target(false)
					.with_span_events(FmtSpan::NONE),
			))
		},
	}
}

/// Returns true if `--interactive` flag was passed.
pub fn is_interactive() -> bool {
	CliArgs::get().interactive
}

/// Creates plugins for headless screenshot rendering (no window).
pub fn headless_plugins(log_filter: Option<&str>) -> bevy::app::PluginGroupBuilder {
	let filter = log_filter.unwrap_or(
		"wgpu=off,wgpu_hal=off,naga=off,bevy_render=off,bevy_diagnostic=off,bevy_winit=off",
	);

	DefaultPlugins
		.set(WindowPlugin {
			primary_window: None,
			exit_condition: ExitCondition::DontExit,
			..default()
		})
		.set(log_plugin(filter))
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

/// Creates plugins for interactive windowed rendering.
pub fn interactive_plugins(log_filter: Option<&str>) -> bevy::app::PluginGroupBuilder {
	let filter = log_filter.unwrap_or(
		"wgpu=off,wgpu_hal=off,naga=off,bevy_render=off,bevy_diagnostic=off,bevy_winit=off",
	);

	DefaultPlugins.set(log_plugin(filter))
}

/// Creates a ScheduleRunnerPlugin for headless operation.
pub fn headless_runner() -> ScheduleRunnerPlugin {
	ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0))
}
