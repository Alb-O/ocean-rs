//! Flat ocean example demonstrating projected grid mesh.
//!
//! This example spawns a flat blue ocean plane extending to the horizon,
//! demonstrating the projected grid technique for infinite ocean rendering.
//!
//! Run with `--interactive` for windowed mode, or without for headless screenshots.

use bevy::camera::CameraProjection;
use bevy::prelude::*;
use bevy_screenshot_harness::{
	CameraPreset, HarnessCameraReady, ScreenshotConfig, ScreenshotHarnessPlugin, headless_plugins,
	headless_runner, interactive_plugins, is_interactive,
};
use ocean_core::{
	OceanMesh, OceanMeshConfig, OceanPlugin, ProjectedGridConfig, ocean::create_projected_grid_mesh,
};

/// Ocean-specific camera presets
const OCEAN_PRESETS: &[CameraPreset] = &[
	CameraPreset {
		name: "horizon",
		radius: 0.0,
		height: 15.0,
		angle: 0.0,
		look_offset: Vec3::new(0.0, 0.0, -100.0),
	},
	CameraPreset {
		name: "elevated",
		radius: 0.0,
		height: 50.0,
		angle: 0.0,
		look_offset: Vec3::new(50.0, 0.0, 50.0),
	},
	CameraPreset {
		name: "low_angle",
		radius: 0.0,
		height: 5.0,
		angle: 0.0,
		look_offset: Vec3::new(0.0, 0.0, -50.0),
	},
];

fn main() {
	let mut app = App::new();

	if is_interactive() {
		app.add_plugins(interactive_plugins(None))
			.add_plugins(OceanPlugin)
			.add_systems(Startup, (setup_camera, setup_ocean.after(setup_camera)));
	} else {
		let config =
			ScreenshotConfig::from_cli("01_flat_ocean").with_presets(OCEAN_PRESETS.to_vec());
		app.add_plugins(headless_plugins(None))
			.add_plugins(headless_runner())
			.add_plugins(OceanPlugin)
			.add_plugins(ScreenshotHarnessPlugin::with_config(config))
			.add_systems(
				Startup,
				setup_ocean.run_if(resource_exists::<HarnessCameraReady>),
			);
	}

	app.run();
}

fn setup_camera(mut commands: Commands) {
	commands.spawn((
		Camera3d::default(),
		Transform::from_xyz(0.0, 15.0, 0.0).looking_at(Vec3::new(0.0, 0.0, -100.0), Vec3::Y),
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

fn setup_ocean(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	config: Res<ProjectedGridConfig>,
	camera_query: Query<(&Transform, &Projection), With<Camera3d>>,
) {
	let Ok((camera_transform, projection)) = camera_query.single() else {
		return;
	};

	let proj_matrix = match projection {
		Projection::Perspective(p) => p.get_clip_from_view(),
		Projection::Orthographic(o) => o.get_clip_from_view(),
		Projection::Custom(c) => c.get_clip_from_view(),
	};

	let mesh_config = OceanMeshConfig {
		resolution: config.resolution,
		max_distance: config.max_distance,
		camera_transform: *camera_transform,
		projection: proj_matrix,
		ocean_height: config.ocean_height,
	};

	let ocean_mesh = create_projected_grid_mesh(&mesh_config);

	commands.spawn((
		Mesh3d(meshes.add(ocean_mesh)),
		MeshMaterial3d(materials.add(StandardMaterial {
			base_color: Color::srgb(0.0, 0.3, 0.5),
			metallic: 0.1,
			perceptual_roughness: 0.4,
			cull_mode: None,
			..default()
		})),
		OceanMesh {
			last_camera_position: camera_transform.translation,
			last_camera_rotation: camera_transform.rotation,
		},
	));
}
