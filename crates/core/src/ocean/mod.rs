//! Ocean rendering plugin and resources.
//!
//! Provides a projected-grid ocean surface that follows the camera,
//! creating the illusion of an infinite ocean plane.

mod mesh;

pub use mesh::{
	OceanMesh, OceanMeshConfig, ProjectedGridConfig, create_projected_grid_mesh,
	update_projected_grid,
};

use bevy::prelude::*;

/// Plugin that sets up ocean rendering systems and resources.
pub struct OceanPlugin;

impl Plugin for OceanPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<ProjectedGridConfig>()
			.add_systems(PostUpdate, update_projected_grid);
	}
}
