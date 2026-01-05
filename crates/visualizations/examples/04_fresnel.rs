//! Fresnel reflection/transmission example.
//!
//! Demonstrates the Fresnel effect: water reflects more at grazing angles (near horizon)
//! and shows more of its own color when viewed from above.

mod common;

use bevy::camera::CameraProjection;
use bevy::prelude::*;
use common::{
	CameraPreset, ExampleHarnessPlugin, ScreenshotConfig, default_example_plugins, headless_runner,
};
use ocean_core::{
	OceanConfig, OceanMaterial, OceanMesh, OceanMeshConfig, OceanPlugin, ProjectedGridConfig,
	ocean::create_projected_grid_mesh,
};

/// Camera presets designed to showcase the Fresnel effect.
/// High angles show more water color, low angles show more reflection.
const FRESNEL_PRESETS: &[CameraPreset] = &[
	// Looking down - should show deep water color with minimal reflection
	CameraPreset {
		name: "top_down",
		radius: 0.0,
		height: 40.0,
		angle: 0.0,
		look_offset: Vec3::new(0.0, 0.0, 0.0),
	},
	// Medium angle - blend of water color and reflection
	CameraPreset {
		name: "medium_angle",
		radius: 0.0,
		height: 15.0,
		angle: 0.0,
		look_offset: Vec3::new(30.0, 0.0, 0.0),
	},
	// Low angle near horizon - should show strong sky reflection
	CameraPreset {
		name: "grazing_angle",
		radius: 0.0,
		height: 3.0,
		angle: 0.0,
		look_offset: Vec3::new(0.0, 0.0, -100.0),
	},
	// Side view to see full range of Fresnel effect
	CameraPreset {
		name: "side_panorama",
		radius: 0.0,
		height: 8.0,
		angle: 0.0,
		look_offset: Vec3::new(50.0, 0.0, 50.0),
	},
];

/// Marker component for the ocean entity
#[derive(Component)]
struct AnimatedOcean;

fn main() {
	let config = ScreenshotConfig::from_env("04_fresnel").with_presets(FRESNEL_PRESETS.to_vec());

	App::new()
		.add_plugins(default_example_plugins(None))
		.add_plugins(headless_runner())
		.add_plugins(OceanPlugin)
		.add_plugins(ExampleHarnessPlugin::with_config(config))
		.add_systems(Startup, setup_ocean)
		.add_systems(Update, animate_ocean)
		.run();
}

fn setup_ocean(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<OceanMaterial>>,
	grid_config: Res<ProjectedGridConfig>,
	ocean_config: Res<OceanConfig>,
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
		resolution: grid_config.resolution,
		max_distance: grid_config.max_distance,
		camera_transform: *camera_transform,
		projection: proj_matrix,
		ocean_height: grid_config.ocean_height,
	};

	let ocean_mesh = create_projected_grid_mesh(&mesh_config);

	let deep_color = Color::srgb(0.0, 0.05, 0.15);
	let shallow_color = Color::srgb(0.0, 0.3, 0.4);
	let sky_color = Color::srgb(0.6, 0.8, 1.0);

	let ocean_material = OceanMaterial::with_fresnel(
		ocean_config.active_waves(),
		deep_color,
		shallow_color,
		sky_color,
		0.02,
		5.0,
		0.0,
	);

	commands.spawn((
		Mesh3d(meshes.add(ocean_mesh)),
		MeshMaterial3d(materials.add(ocean_material)),
		OceanMesh {
			last_camera_position: camera_transform.translation,
			last_camera_rotation: camera_transform.rotation,
		},
		AnimatedOcean,
	));
}

/// Updates the ocean material time uniform each frame for animation.
fn animate_ocean(
	time: Res<Time>,
	ocean_query: Query<&MeshMaterial3d<OceanMaterial>, With<AnimatedOcean>>,
	mut materials: ResMut<Assets<OceanMaterial>>,
) {
	for material_handle in ocean_query.iter() {
		if let Some(material) = materials.get_mut(&material_handle.0) {
			material.set_time(time.elapsed_secs());
		}
	}
}
