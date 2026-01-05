//! Environment map reflections example.
//!
//! Demonstrates realistic ocean rendering with HDR environment cubemap reflections.
//! This is the culmination of the ocean rendering pipeline: projected grid mesh,
//! multiple Gerstner waves, Fresnel shading, and environment map reflections.

mod common;

use bevy::camera::CameraProjection;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy::scene::SceneRoot;
use common::{
	CameraPreset, ExampleHarnessPlugin, ScreenshotConfig, default_example_plugins, headless_runner,
};
use ocean_core::{
	OceanConfig, OceanMaterial, OceanMesh, OceanMeshConfig, OceanPlugin, ProjectedGridConfig,
	ocean::create_projected_grid_mesh,
};

/// Camera presets for dramatic ocean views with environment reflections.
const SKYBOX_PRESETS: &[CameraPreset] = &[
	// Wide shot showing ocean and ship with horizon reflections
	CameraPreset {
		name: "wide_horizon",
		radius: 80.0,
		height: 25.0,
		angle: 0.3,
		look_offset: Vec3::new(0.0, 5.0, 0.0),
	},
	// Close-up on ship with ocean reflections
	CameraPreset {
		name: "ship_closeup",
		radius: 25.0,
		height: 12.0,
		angle: 1.2,
		look_offset: Vec3::new(0.0, 8.0, 0.0),
	},
	// Low angle emphasizing sky reflections
	CameraPreset {
		name: "low_dramatic",
		radius: 50.0,
		height: 4.0,
		angle: 2.5,
		look_offset: Vec3::new(0.0, 0.0, -50.0),
	},
	// Top-down showing wave patterns
	CameraPreset {
		name: "top_waves",
		radius: 5.0,
		height: 45.0,
		angle: 0.0,
		look_offset: Vec3::new(0.0, 0.0, 0.0),
	},
];

/// Marker component for the ocean entity
#[derive(Component)]
struct AnimatedOcean;

/// Marker component for the ship entity
#[derive(Component)]
struct Ship;

fn main() {
	let config = ScreenshotConfig::from_env("05_skybox_reflect").with_presets(SKYBOX_PRESETS.to_vec());

	App::new()
		.add_plugins(default_example_plugins(None))
		.add_plugins(headless_runner())
		.add_plugins(OceanPlugin)
		.add_plugins(ExampleHarnessPlugin::with_config(config))
		.add_systems(Startup, (setup_ocean, setup_ship, setup_environment))
		.add_systems(Update, animate_ocean)
		.run();
}

fn setup_ocean(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<OceanMaterial>>,
	asset_server: Res<AssetServer>,
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

	let env_map: Handle<Image> =
		asset_server.load("environment_maps/table_mountain_2_puresky_4k_cubemap.ktx2");

	let deep_color = Color::srgb(0.0, 0.08, 0.2);
	let shallow_color = Color::srgb(0.0, 0.35, 0.45);

	let ocean_material = OceanMaterial::with_environment_map(
		ocean_config.active_waves(),
		deep_color,
		shallow_color,
		0.02,
		5.0,
		0.0,
		env_map,
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

fn setup_ship(mut commands: Commands, asset_server: Res<AssetServer>) {
	let ship_scene =
		asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/dutch_ship_medium_1k/dutch_ship_medium_1k.gltf"));

	commands.spawn((
		SceneRoot(ship_scene),
		Transform::from_xyz(0.0, 2.0, 0.0)
			.with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4))
			.with_scale(Vec3::splat(3.0)),
		Ship,
	));
}

fn setup_environment(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	commands.spawn((
		EnvironmentMapLight {
			diffuse_map: asset_server.load("environment_maps/table_mountain_2_puresky_4k_diffuse.ktx2"),
			specular_map: asset_server.load("environment_maps/table_mountain_2_puresky_4k_specular.ktx2"),
			intensity: 900.0,
			..default()
		},
	));

	commands.spawn((
		DirectionalLight {
			illuminance: 12000.0,
			shadows_enabled: true,
			..default()
		},
		Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.6, 0.5, 0.0)),
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
