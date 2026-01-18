//! Multiple overlapping Gerstner waves example.
//!
//! Demonstrates 3 overlapping Gerstner waves for a realistic chaotic ocean surface.
//! Uses the OceanConfig resource for wave parameters and calculated normals for proper lighting.
//!
//! Run with `--interactive` for windowed mode, or without for headless screenshots.

use bevy::camera::CameraProjection;
use bevy::prelude::*;
use bevy_screenshot_harness::{
	CameraPreset, HarnessCameraReady, ScreenshotConfig, ScreenshotHarnessPlugin, headless_plugins,
	headless_runner, interactive_plugins, is_interactive,
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
	let mut app = App::new();

	if is_interactive() {
		app.add_plugins(interactive_plugins(None))
			.add_plugins(OceanPlugin)
			.add_systems(Startup, (setup_camera, setup_ocean.after(setup_camera)))
			.add_systems(Update, animate_ocean);
	} else {
		let config = ScreenshotConfig::from_cli("03_gerstner_multi")
			.with_presets(MULTI_WAVE_PRESETS.to_vec());
		app.add_plugins(headless_plugins(None))
			.add_plugins(headless_runner())
			.add_plugins(OceanPlugin)
			.add_plugins(ScreenshotHarnessPlugin::with_config(config))
			.add_systems(
				Startup,
				setup_ocean.run_if(resource_exists::<HarnessCameraReady>),
			)
			.add_systems(Update, animate_ocean);
	}

	app.run();
}

fn setup_camera(mut commands: Commands) {
	commands.spawn((
		Camera3d::default(),
		Transform::from_xyz(0.0, 30.0, 0.0).looking_at(Vec3::new(20.0, 0.0, 20.0), Vec3::Y),
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

	let ocean_material = OceanMaterial::new(
		ocean_config.active_waves(),
		ocean_config.deep_color,
		ocean_config.shallow_color,
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
