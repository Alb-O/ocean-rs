//! Multiple overlapping Gerstner waves example.
//!
//! Demonstrates 3 overlapping Gerstner waves for a realistic chaotic ocean surface.
//! Uses the OceanConfig resource for wave parameters and calculated normals for proper lighting.

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

/// Ocean-specific camera presets for multi-wave visualization
const MULTI_WAVE_PRESETS: &[CameraPreset] = &[
	CameraPreset {
		name: "overview",
		radius: 0.0,
		height: 30.0,
		angle: 0.0,
		look_offset: Vec3::new(20.0, 0.0, 20.0),
	},
	CameraPreset {
		name: "wave_detail",
		radius: 0.0,
		height: 12.0,
		angle: 0.0,
		look_offset: Vec3::new(25.0, 0.0, 0.0),
	},
	CameraPreset {
		name: "surface_level",
		radius: 0.0,
		height: 5.0,
		angle: 0.0,
		look_offset: Vec3::new(0.0, 0.0, -50.0),
	},
	CameraPreset {
		name: "dramatic_angle",
		radius: 0.0,
		height: 18.0,
		angle: 0.0,
		look_offset: Vec3::new(-30.0, 0.0, 30.0),
	},
];

/// Marker component for the ocean entity
#[derive(Component)]
struct AnimatedOcean;

fn main() {
	let config =
		ScreenshotConfig::from_env("03_gerstner_multi").with_presets(MULTI_WAVE_PRESETS.to_vec());

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
	// Get camera info for initial mesh generation
	let Ok((camera_transform, projection)) = camera_query.single() else {
		return;
	};

	let proj_matrix = match projection {
		Projection::Perspective(p) => p.get_clip_from_view(),
		Projection::Orthographic(o) => o.get_clip_from_view(),
		Projection::Custom(c) => c.get_clip_from_view(),
	};

	// Create initial ocean mesh
	let mesh_config = OceanMeshConfig {
		resolution: grid_config.resolution,
		max_distance: grid_config.max_distance,
		camera_transform: *camera_transform,
		projection: proj_matrix,
		ocean_height: grid_config.ocean_height,
	};

	let ocean_mesh = create_projected_grid_mesh(&mesh_config);

	// Create ocean material using the 3 active waves from OceanConfig
	let ocean_material = OceanMaterial::new(
		ocean_config.active_waves(),
		ocean_config.deep_color,
		ocean_config.shallow_color,
	);

	// Spawn ocean entity with custom material
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
