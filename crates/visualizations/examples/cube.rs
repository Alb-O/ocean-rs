//! Simple cube example demonstrating headless screenshot capture.
//!
//! This example spawns a colored cube with basic lighting and captures
//! screenshots from multiple camera angles.

use bevy::prelude::*;
use bevy_screenshot_harness::{
    ScreenshotConfig, ScreenshotHarnessPlugin, default_example_plugins, headless_runner,
};

fn main() {
    let config = ScreenshotConfig::from_cli("cube").with_simple_presets();

    App::new()
        .add_plugins(default_example_plugins(None))
        .add_plugins(headless_runner())
        .add_plugins(ScreenshotHarnessPlugin::with_config(config))
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a simple colored cube
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

    // Ground plane for visual reference
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.4, 0.45),
            metallic: 0.0,
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
