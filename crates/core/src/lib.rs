//! Core library crate for ocean rendering.

pub mod ocean;

pub use ocean::{
	GerstnerWave, GpuGerstnerWave, OceanConfig, OceanMaterial, OceanMesh, OceanMeshConfig,
	OceanPlugin, OceanUniforms, ProjectedGridConfig, evaluate_waves, GRAVITY, MAX_WAVES,
};
