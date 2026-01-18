//! Single Gerstner wave example.
//!
//! Demonstrates a single animated Gerstner wave with vertex displacement
//! using the custom OceanMaterial shader.

use bevy::camera::CameraProjection;
use bevy::prelude::*;
use bevy_screenshot_harness::{
    CameraPreset, HarnessCameraReady, ScreenshotConfig, ScreenshotHarnessPlugin,
    default_example_plugins, headless_runner,
};
use ocean_core::{
    GerstnerWave, OceanMaterial, OceanMesh, OceanMeshConfig, OceanPlugin, ProjectedGridConfig,
    ocean::create_projected_grid_mesh,
};

/// Ocean-specific camera presets for wave visualization
const WAVE_PRESETS: &[CameraPreset] = &[
    CameraPreset {
        name: "side_view",
        radius: 0.0,
        height: 8.0,
        angle: 0.0,
        look_offset: Vec3::new(30.0, 0.0, 0.0),
    },
    CameraPreset {
        name: "elevated",
        radius: 0.0,
        height: 25.0,
        angle: 0.0,
        look_offset: Vec3::new(20.0, 0.0, 20.0),
    },
    CameraPreset {
        name: "low_angle",
        radius: 0.0,
        height: 4.0,
        angle: 0.0,
        look_offset: Vec3::new(0.0, 0.0, -40.0),
    },
];

/// Marker component for the ocean entity
#[derive(Component)]
struct AnimatedOcean;

fn main() {
    let config =
        ScreenshotConfig::from_cli("02_gerstner_single").with_presets(WAVE_PRESETS.to_vec());

    App::new()
        .add_plugins(default_example_plugins(None))
        .add_plugins(headless_runner())
        .add_plugins(OceanPlugin)
        .add_plugins(ScreenshotHarnessPlugin::with_config(config))
        .add_systems(
            Startup,
            setup_ocean.run_if(resource_exists::<HarnessCameraReady>),
        )
        .add_systems(Update, animate_ocean)
        .run();
}

fn setup_ocean(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<OceanMaterial>>,
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

    let wave = GerstnerWave::new(
        Vec2::new(1.0, 0.3).normalize(), // Slightly angled direction
        0.5,                             // steepness
        60.0,                            // wavelength
        2.0,                             // amplitude
        1.0,                             // speed
    );

    let ocean_material = OceanMaterial::new(
        &[wave],
        Color::srgb(0.0, 0.1, 0.3), // deep color
        Color::srgb(0.0, 0.4, 0.5), // shallow color
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
