//! Ocean rendering plugin and resources.
//!
//! Provides a projected-grid ocean surface that follows the camera,
//! creating the illusion of an infinite ocean plane.

mod material;
mod mesh;
pub mod waves;

pub use material::{MAX_WAVES, OceanMaterial, OceanMaterialPlugin};
pub use mesh::{
	OceanMesh, OceanMeshConfig, ProjectedGridConfig, create_projected_grid_mesh,
	update_projected_grid,
};
pub use waves::{GRAVITY, GerstnerWave, evaluate_waves};

use bevy::prelude::*;

/// Configuration resource for ocean wave parameters.
///
/// Provides default multi-wave presets for realistic ocean rendering.
/// Waves can be modified at runtime for dynamic ocean conditions.
#[derive(Resource, Debug, Clone, Reflect)]
pub struct OceanConfig {
	/// Array of up to 4 Gerstner waves.
	pub waves: [GerstnerWave; 4],
	/// Number of waves to render (1-4).
	pub active_wave_count: u32,
	/// Deep water color (viewed from above).
	pub deep_color: Color,
	/// Shallow water color (viewed at angle).
	pub shallow_color: Color,
}

impl Default for OceanConfig {
	fn default() -> Self {
		Self {
			waves: [
				GerstnerWave::new(Vec2::new(1.0, 0.0), 0.5, 60.0, 2.0, 1.0),
				GerstnerWave::new(Vec2::new(0.7, 0.7).normalize(), 0.6, 31.0, 1.0, 1.2),
				GerstnerWave::new(Vec2::new(-0.3, 0.9).normalize(), 0.4, 18.0, 0.5, 0.9),
				GerstnerWave::new(Vec2::ZERO, 0.0, 1.0, 0.0, 0.0), // unused
			],
			active_wave_count: 3,
			deep_color: Color::srgb(0.0, 0.1, 0.3),
			shallow_color: Color::srgb(0.0, 0.4, 0.5),
		}
	}
}

impl OceanConfig {
	/// Returns a slice of the active waves.
	#[must_use]
	pub fn active_waves(&self) -> &[GerstnerWave] {
		&self.waves[..self.active_wave_count as usize]
	}
}

/// Plugin that sets up ocean rendering systems and resources.
pub struct OceanPlugin;

impl Plugin for OceanPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<ProjectedGridConfig>()
			.init_resource::<OceanConfig>()
			.register_type::<OceanConfig>()
			.add_plugins(OceanMaterialPlugin)
			.add_systems(PostUpdate, update_projected_grid);
	}
}
