//! Shared utilities for examples.
//!
//! This module provides reusable infrastructure for all development examples:
//! - Automated screenshot capture with multiple camera presets (headless, no window)
//! - Debug overlay with performance stats
//! - Standard camera setup and controls
//! - Consistent logging configuration

#![allow(dead_code, unused_imports, unused_variables)]

use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy::app::ScheduleRunnerPlugin;
use bevy::camera::RenderTarget;
use bevy::image::TextureFormatPixelInfo;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{
	self, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel,
};
use bevy::render::render_resource::{
	Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, MapMode, PollType,
	TexelCopyBufferInfo, TexelCopyBufferLayout, TextureFormat, TextureUsages,
};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::settings::{InstanceFlags, RenderCreation, WgpuSettings};
use bevy::render::{Extract, Render, RenderApp, RenderPlugin, RenderSystems};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use crossbeam_channel::{Receiver, Sender};

/// Default screenshot output directory (relative to example folder)
pub const SCREENSHOT_SUBDIR: &str = "screenshots";

/// Minimum camera height
pub const MIN_CAMERA_HEIGHT: f32 = 2.0;

/// Screenshot image width
pub const SCREENSHOT_WIDTH: u32 = 1920;

/// Screenshot image height
pub const SCREENSHOT_HEIGHT: u32 = 1080;

/// Number of frames to wait before capturing (allows scene to fully render)
pub const PRE_ROLL_FRAMES: u32 = 60;

/// Number of frames to wait between shots for scene to settle
pub const SETTLE_FRAMES: u32 = 30;

/// A camera position preset for screenshots
#[derive(Clone, Copy, Debug)]
pub struct CameraPreset {
	pub name: &'static str,
	pub radius: f32,
	pub height: f32,
	pub angle: f32,
	pub look_offset: Vec3,
}

impl CameraPreset {
	pub fn to_position(self) -> Vec3 {
		let x = self.radius * self.angle.sin();
		let z = self.radius * self.angle.cos();
		let y = self.height.max(MIN_CAMERA_HEIGHT);
		Vec3::new(x, y, z)
	}
}

/// Standard camera presets for visualization
pub const STANDARD_PRESETS: &[CameraPreset] = &[
	CameraPreset {
		name: "wide",
		radius: 120.0,
		height: 35.0,
		angle: 0.0,
		look_offset: Vec3::ZERO,
	},
	CameraPreset {
		name: "close",
		radius: 40.0,
		height: 15.0,
		angle: 2.5,
		look_offset: Vec3::ZERO,
	},
	CameraPreset {
		name: "dramatic",
		radius: 70.0,
		height: 20.0,
		angle: 5.5,
		look_offset: Vec3::new(10.0, 0.0, 10.0),
	},
];

/// Close-up presets for detail inspection
pub const DETAIL_PRESETS: &[CameraPreset] = &[
	CameraPreset {
		name: "detail_top",
		radius: 20.0,
		height: 30.0,
		angle: 0.0,
		look_offset: Vec3::ZERO,
	},
	CameraPreset {
		name: "detail_angle",
		radius: 15.0,
		height: 8.0,
		angle: 0.8,
		look_offset: Vec3::ZERO,
	},
	CameraPreset {
		name: "detail_low",
		radius: 12.0,
		height: 3.0,
		angle: 1.2,
		look_offset: Vec3::ZERO,
	},
];

/// Simple presets for basic examples (cube, etc.)
pub const SIMPLE_PRESETS: &[CameraPreset] = &[
	CameraPreset {
		name: "front",
		radius: 5.0,
		height: 3.0,
		angle: 0.5,
		look_offset: Vec3::ZERO,
	},
	CameraPreset {
		name: "angle",
		radius: 6.0,
		height: 4.0,
		angle: 2.0,
		look_offset: Vec3::ZERO,
	},
	CameraPreset {
		name: "top",
		radius: 4.0,
		height: 6.0,
		angle: 0.0,
		look_offset: Vec3::ZERO,
	},
];

/// Configuration for the screenshot system
#[derive(Resource, Clone)]
pub struct ScreenshotConfig {
	/// Exit application after capturing all screenshots
	pub exit_after: bool,
	/// Capture multiple presets (vs just one)
	pub multi_shot: bool,
	/// Camera presets to use
	pub presets: Vec<CameraPreset>,
	/// Example name (determines screenshot location: examples/{example_name}/screenshots/)
	pub example_name: String,
	/// Screenshot width
	pub width: u32,
	/// Screenshot height
	pub height: u32,
}

impl ScreenshotConfig {
	/// Returns the base directory for screenshots: examples/{example_name}/screenshots
	pub fn screenshot_dir(&self) -> String {
		format!("examples/{}/{}", self.example_name, SCREENSHOT_SUBDIR)
	}
}

impl Default for ScreenshotConfig {
	fn default() -> Self {
		Self {
			exit_after: true,
			multi_shot: true,
			presets: STANDARD_PRESETS.to_vec(),
			example_name: "default".to_string(),
			width: SCREENSHOT_WIDTH,
			height: SCREENSHOT_HEIGHT,
		}
	}
}

impl ScreenshotConfig {
	/// Create config from environment variables
	pub fn from_env(example_name: &str) -> Self {
		let exit_after = std::env::var("SCREENSHOT_EXIT")
			.map(|v| v != "0")
			.unwrap_or(true);
		let multi_shot = std::env::var("SCREENSHOT_MULTI")
			.map(|v| v != "0")
			.unwrap_or(true);

		Self {
			exit_after,
			multi_shot,
			presets: STANDARD_PRESETS.to_vec(),
			example_name: example_name.to_string(),
			width: SCREENSHOT_WIDTH,
			height: SCREENSHOT_HEIGHT,
		}
	}

	/// Use detail presets instead of standard ones
	pub fn with_detail_presets(mut self) -> Self {
		self.presets = DETAIL_PRESETS.to_vec();
		self
	}

	/// Use simple presets for basic examples
	pub fn with_simple_presets(mut self) -> Self {
		self.presets = SIMPLE_PRESETS.to_vec();
		self
	}

	/// Use custom presets
	pub fn with_presets(mut self, presets: Vec<CameraPreset>) -> Self {
		self.presets = presets;
		self
	}

	/// Don't exit after screenshots
	pub fn no_exit(mut self) -> Self {
		self.exit_after = false;
		self
	}

	/// Set custom resolution
	pub fn with_resolution(mut self, width: u32, height: u32) -> Self {
		self.width = width;
		self.height = height;
		self
	}
}

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
		let timestamp = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis();
		Self {
			phase: ScreenshotPhase::default(),
			current_preset: 0,
			session_dir: format!("{}", timestamp),
			captured_paths: Vec::new(),
			render_target: None,
		}
	}
}

/// Marker component for the main camera
#[derive(Component)]
pub struct MainCamera;

/// Channel receiver for image data from render world
#[derive(Resource, Deref)]
struct MainWorldReceiver(Receiver<Vec<u8>>);

/// Channel sender for image data to main world
#[derive(Resource, Deref)]
struct RenderWorldSender(Sender<Vec<u8>>);

/// Plugin that provides the screenshot harness for examples
pub struct ExampleHarnessPlugin {
	config: ScreenshotConfig,
}

impl ExampleHarnessPlugin {
	pub fn new(example_name: &str) -> Self {
		Self {
			config: ScreenshotConfig::from_env(example_name),
		}
	}

	pub fn with_config(config: ScreenshotConfig) -> Self {
		Self { config }
	}
}

impl Plugin for ExampleHarnessPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(self.config.clone())
			.init_resource::<ScreenshotState>()
			.add_plugins(ImageCopyPlugin)
			.add_systems(Startup, (setup_camera, prepare_screenshot_dir))
			.add_systems(PostUpdate, screenshot_sequence);
	}
}

/// Creates the default plugins for headless rendering (no window spawned).
///
/// Usage:
/// ```ignore
/// App::new()
///     .add_plugins(common::default_example_plugins(None))
///     // ...
/// ```
pub fn default_example_plugins(log_filter: Option<&str>) -> bevy::app::PluginGroupBuilder {
	let filter = log_filter.unwrap_or(
		"wgpu=off,wgpu_hal=off,naga=off,bevy_render=off,bevy_diagnostic=off,bevy_winit=off",
	);

	DefaultPlugins
		.set(WindowPlugin {
			primary_window: None, // No window - headless rendering
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

fn setup_camera(
	mut commands: Commands,
	mut images: ResMut<Assets<Image>>,
	config: Res<ScreenshotConfig>,
	mut state: ResMut<ScreenshotState>,
	render_device: Res<RenderDevice>,
) {
	let preset = config.presets.first().copied().unwrap_or(CameraPreset {
		name: "default",
		radius: 50.0,
		height: 20.0,
		angle: 0.0,
		look_offset: Vec3::ZERO,
	});

	let pos = preset.to_position();

	commands.insert_resource(ClearColor(Color::BLACK));

	let size = Extent3d {
		width: config.width,
		height: config.height,
		..default()
	};

	let mut render_target_image =
		Image::new_target_texture(size.width, size.height, TextureFormat::bevy_default(), None);
	render_target_image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
	let render_target_handle = images.add(render_target_image);

	let cpu_image =
		Image::new_target_texture(size.width, size.height, TextureFormat::bevy_default(), None);
	let cpu_image_handle = images.add(cpu_image);

	commands.spawn(ImageCopier::new(
		render_target_handle.clone(),
		size,
		&render_device,
	));

	commands.spawn(ImageToSave(cpu_image_handle));
	state.render_target = Some(render_target_handle.clone());

	// Spawn camera (no skybox for simple examples)
	commands.spawn((
		Camera3d::default(),
		Camera {
			clear_color: ClearColorConfig::Custom(Color::BLACK),
			..default()
		},
		RenderTarget::Image(render_target_handle.into()),
		Transform::from_translation(pos).looking_at(preset.look_offset, Vec3::Y),
		MainCamera,
	));

	commands.insert_resource(GlobalAmbientLight {
		color: Color::WHITE,
		brightness: 300.0,
		..default()
	});

	commands.spawn((
		DirectionalLight {
			illuminance: 10000.0,
			shadows_enabled: true,
			..default()
		},
		Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.0)),
	));
}

fn prepare_screenshot_dir(config: Res<ScreenshotConfig>, state: Res<ScreenshotState>) {
	let session_path = format!("{}/{}", config.screenshot_dir(), state.session_dir);
	let _ = std::fs::create_dir_all(&session_path);
}

fn screenshot_sequence(
	config: Res<ScreenshotConfig>,
	mut state: ResMut<ScreenshotState>,
	mut camera: Query<&mut Transform, With<MainCamera>>,
	receiver: Res<MainWorldReceiver>,
	images_to_save: Query<&ImageToSave>,
	mut images: ResMut<Assets<Image>>,
	mut app_exit: MessageWriter<AppExit>,
) {
	match &state.phase {
		ScreenshotPhase::Init(frames_remaining) => {
			while receiver.try_recv().is_ok() {}

			if *frames_remaining == 0 {
				state.phase = ScreenshotPhase::Capturing;
			} else {
				state.phase = ScreenshotPhase::Init(frames_remaining - 1);
			}
		}

		ScreenshotPhase::Settling(frames_remaining) => {
			while receiver.try_recv().is_ok() {}

			if *frames_remaining == 0 {
				state.phase = ScreenshotPhase::Capturing;
			} else {
				state.phase = ScreenshotPhase::Settling(frames_remaining - 1);
			}
		}

		ScreenshotPhase::Capturing => {
			let mut image_data = Vec::new();
			while let Ok(data) = receiver.try_recv() {
				image_data = data;
			}

			if image_data.is_empty() {
				return;
			}

			for image_to_save in images_to_save.iter() {
				let img_bytes = images.get_mut(image_to_save.id()).unwrap();
				let row_bytes = img_bytes.width() as usize
					* img_bytes.texture_descriptor.format.pixel_size().unwrap();
				let aligned_row_bytes = RenderDevice::align_copy_bytes_per_row(row_bytes);

				if row_bytes == aligned_row_bytes {
					img_bytes.data.as_mut().unwrap().clone_from(&image_data);
				} else {
					img_bytes.data = Some(
						image_data
							.chunks(aligned_row_bytes)
							.take(img_bytes.height() as usize)
							.flat_map(|row| &row[..row_bytes.min(row.len())])
							.cloned()
							.collect(),
					);
				}

				let img = match img_bytes.clone().try_into_dynamic() {
					Ok(img) => img.to_rgba8(),
					Err(e) => {
						error!("Failed to create image buffer: {e:?}");
						continue;
					}
				};

				let preset = &config.presets[state.current_preset];
				let path = format!(
					"{}/{}/{}.png",
					config.screenshot_dir(),
					state.session_dir,
					preset.name
				);

				if let Some(parent) = PathBuf::from(&path).parent() {
					let _ = std::fs::create_dir_all(parent);
				}

				if let Err(e) = img.save(&path) {
					error!("Failed to save image: {e}");
				} else {
					info!("Screenshot saved to {}", path);
					state.captured_paths.push(path);
				}
			}

			if config.multi_shot && state.current_preset + 1 < config.presets.len() {
				state.current_preset += 1;
				let preset = &config.presets[state.current_preset];
				if let Ok(mut transform) = camera.single_mut() {
					let pos = preset.to_position();
					transform.translation = pos;
					transform.look_at(preset.look_offset, Vec3::Y);
				}

				state.phase = ScreenshotPhase::Settling(SETTLE_FRAMES);
			} else {
				state.phase = ScreenshotPhase::Done;
			}
		}

		ScreenshotPhase::Done => {
			if config.exit_after {
				print_agent_instructions(&state.captured_paths);
				app_exit.write(AppExit::Success);
			}
		}
	}
}

fn print_agent_instructions(paths: &[String]) {
	println!();
	println!(
		"Agents: use PARALLEL READS on the {} images like so:",
		paths.len()
	);
	println!();

	let tool_uses: Vec<String> = paths
		.iter()
		.map(|p| {
			format!(
				r#"      {{
        "recipient_name": "functions.read",
        "parameters": {{
          "filePath": "{}"
        }}
      }}"#,
				p
			)
		})
		.collect();

	let json_block = format!(
		r#"{{
  "recipient_name": "multi_tool_use.parallel",
  "parameters": {{
    "tool_uses": [
{}
    ]
  }}
}}"#,
		tool_uses.join(",\n")
	);

	println!("{}", json_block);
}

/// Plugin for copying rendered images from GPU to CPU via render graph.
struct ImageCopyPlugin;

impl Plugin for ImageCopyPlugin {
	fn build(&self, app: &mut App) {
		let (sender, receiver) = crossbeam_channel::unbounded();

		let render_app = app
			.insert_resource(MainWorldReceiver(receiver))
			.sub_app_mut(RenderApp);

		let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
		graph.add_node(ImageCopyLabel, ImageCopyDriver);
		graph.add_node_edge(bevy::render::graph::CameraDriverLabel, ImageCopyLabel);

		render_app
			.insert_resource(RenderWorldSender(sender))
			.add_systems(ExtractSchedule, image_copy_extract)
			.add_systems(
				Render,
				receive_image_from_buffer.after(RenderSystems::Render),
			);
	}
}

/// Aggregator for image copiers in render world
#[derive(Clone, Default, Resource, Deref, DerefMut)]
struct ImageCopiers(pub Vec<ImageCopier>);

/// Component for copying from render target to buffer
#[derive(Clone, Component)]
struct ImageCopier {
	buffer: Buffer,
	enabled: Arc<AtomicBool>,
	src_image: Handle<Image>,
}

impl ImageCopier {
	pub fn new(src_image: Handle<Image>, size: Extent3d, render_device: &RenderDevice) -> Self {
		let padded_bytes_per_row =
			RenderDevice::align_copy_bytes_per_row((size.width) as usize) * 4;

		let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
			label: Some("screenshot_buffer"),
			size: padded_bytes_per_row as u64 * size.height as u64,
			usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		Self {
			buffer: cpu_buffer,
			src_image,
			enabled: Arc::new(AtomicBool::new(true)),
		}
	}

	pub fn enabled(&self) -> bool {
		self.enabled.load(Ordering::Relaxed)
	}
}

/// CPU-side image handle for saving
#[derive(Component, Deref, DerefMut)]
struct ImageToSave(Handle<Image>);

/// Extract image copiers into render world
fn image_copy_extract(mut commands: Commands, image_copiers: Extract<Query<&ImageCopier>>) {
	commands.insert_resource(ImageCopiers(
		image_copiers.iter().cloned().collect::<Vec<ImageCopier>>(),
	));
}

/// Render graph label for image copy node
#[derive(Debug, PartialEq, Eq, Clone, Hash, RenderLabel)]
struct ImageCopyLabel;

/// Render graph node that copies texture to buffer
#[derive(Default)]
struct ImageCopyDriver;

impl render_graph::Node for ImageCopyDriver {
	fn run(
		&self,
		_graph: &mut RenderGraphContext,
		render_context: &mut RenderContext,
		world: &World,
	) -> Result<(), NodeRunError> {
		let image_copiers = world.get_resource::<ImageCopiers>().unwrap();
		let gpu_images = world
			.get_resource::<RenderAssets<bevy::render::texture::GpuImage>>()
			.unwrap();

		for image_copier in image_copiers.iter() {
			if !image_copier.enabled() {
				continue;
			}

			let Some(src_image) = gpu_images.get(&image_copier.src_image) else {
				continue;
			};

			let mut encoder = render_context
				.render_device()
				.create_command_encoder(&CommandEncoderDescriptor::default());

			let block_dimensions = src_image.texture_format.block_dimensions();
			let block_size = src_image.texture_format.block_copy_size(None).unwrap();

			let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
				(src_image.size.width as usize / block_dimensions.0 as usize) * block_size as usize,
			);

			encoder.copy_texture_to_buffer(
				src_image.texture.as_image_copy(),
				TexelCopyBufferInfo {
					buffer: &image_copier.buffer,
					layout: TexelCopyBufferLayout {
						offset: 0,
						bytes_per_row: Some(
							std::num::NonZero::<u32>::new(padded_bytes_per_row as u32)
								.unwrap()
								.into(),
						),
						rows_per_image: None,
					},
				},
				src_image.size,
			);

			let render_queue = world.get_resource::<RenderQueue>().unwrap();
			render_queue.submit(std::iter::once(encoder.finish()));
		}

		Ok(())
	}
}

/// Receives image data from GPU buffer and sends to main world
fn receive_image_from_buffer(
	image_copiers: Res<ImageCopiers>,
	render_device: Res<RenderDevice>,
	sender: Res<RenderWorldSender>,
) {
	for image_copier in image_copiers.0.iter() {
		if !image_copier.enabled() {
			continue;
		}

		let buffer_slice = image_copier.buffer.slice(..);

		let (s, r) = crossbeam_channel::bounded(1);

		buffer_slice.map_async(MapMode::Read, move |result| match result {
			Ok(()) => s.send(()).expect("Failed to send map update"),
			Err(err) => panic!("Failed to map buffer: {err}"),
		});

		render_device
			.poll(PollType::wait_indefinitely())
			.expect("Failed to poll device");

		r.recv().expect("Failed to receive map_async message");

		let _ = sender.send(buffer_slice.get_mapped_range().to_vec());

		image_copier.buffer.unmap();
	}
}
