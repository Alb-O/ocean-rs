//! Simple cube example.
//!
//! Run with `--interactive` for windowed mode, or without for headless screenshots.

use bevy::prelude::*;
use bevy_screenshot_harness::{
	ScreenshotConfig, ScreenshotHarnessPlugin, headless_plugins, headless_runner,
	interactive_plugins, is_interactive,
};

fn main() {
	let mut app = App::new();

	if is_interactive() {
		app.add_plugins(interactive_plugins(None))
			.add_systems(Startup, setup_camera);
	} else {
		let config = ScreenshotConfig::from_cli("cube").with_simple_presets();
		app.add_plugins(headless_plugins(None))
			.add_plugins(headless_runner())
			.add_plugins(ScreenshotHarnessPlugin::with_config(config));
	}

	app.add_systems(Startup, setup_scene).run();
}

fn setup_camera(mut commands: Commands) {
	commands.spawn((
		Camera3d::default(),
		Transform::from_xyz(5.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
	));

	commands.insert_resource(GlobalAmbientLight {
		brightness: 500.0,
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

fn setup_scene(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	commands.spawn((
		Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
		MeshMaterial3d(materials.add(StandardMaterial {
			base_color: Color::srgb(0.8, 0.3, 0.2),
			metallic: 0.3,
			perceptual_roughness: 0.5,
			..default()
		})),
		Transform::from_xyz(0.0, 0.5, 0.0),
	));

	commands.spawn((
		Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
		MeshMaterial3d(materials.add(StandardMaterial {
			base_color: Color::srgb(0.4, 0.4, 0.45),
			metallic: 0.0,
			perceptual_roughness: 0.9,
			..default()
		})),
	));
}
