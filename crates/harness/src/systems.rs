//! Camera setup and screenshot sequence systems.

use bevy::image::TextureFormatPixelInfo;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureFormat, TextureUsages};
use bevy::render::renderer::RenderDevice;

use crate::cleanup::cleanup_old_sessions;
use crate::config::ScreenshotConfig;
use crate::image_copy::{ImageCopier, ImageToSave, MainWorldReceiver};
use crate::plugin::HarnessCameraReady;
use crate::presets::CameraPreset;
use crate::state::{SETTLE_FRAMES, ScreenshotPhase, ScreenshotState};

/// Marker component for the main camera
#[derive(Component)]
pub struct MainCamera;

/// Sets up the camera and render target.
pub fn setup_camera(
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

	commands.spawn((
		Camera3d::default(),
		Camera {
			clear_color: ClearColorConfig::Custom(Color::BLACK),
			..default()
		},
		bevy::camera::RenderTarget::Image(render_target_handle.into()),
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

	commands.insert_resource(HarnessCameraReady);
}

pub(crate) fn prepare_screenshot_dir(config: Res<ScreenshotConfig>, state: Res<ScreenshotState>) {
	let _ = std::fs::create_dir_all(config.screenshot_dir().join(&state.session_dir));
	cleanup_old_sessions(&config.screenshot_dir(), config.retain_sessions);
}

pub(crate) fn screenshot_sequence(
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
				let path = config
					.screenshot_dir()
					.join(&state.session_dir)
					.join(format!("{}.png", preset.name));

				if let Some(parent) = path.parent() {
					let _ = std::fs::create_dir_all(parent);
				}

				match img.save(&path) {
					Ok(()) => state.captured_paths.push(path.display().to_string()),
					Err(e) => error!(%e, ?path, "Failed to save screenshot"),
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
				for path in &state.captured_paths {
					info!(path, "saved");
				}
				info!("Agents: Read screenshots ONE AT A TIME. Analyze before proceeding.");
				app_exit.write(AppExit::Success);
			}
		}
	}
}
