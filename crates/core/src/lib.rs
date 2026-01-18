//! Core library crate for ocean rendering.

pub mod ocean;

pub use ocean::{
	GRAVITY, GerstnerWave, MAX_WAVES, OceanConfig, OceanMaterial, OceanMesh, OceanMeshConfig,
	OceanPlugin, ProjectedGridConfig, evaluate_waves,
};
