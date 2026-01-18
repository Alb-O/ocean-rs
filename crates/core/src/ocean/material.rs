//! Custom ocean material with Gerstner wave displacement.
//!
//! This material implements GPU-side Gerstner wave calculations for realistic
//! ocean surface animation. Wave parameters are passed as uniforms and processed
//! in the vertex shader.

use std::path::PathBuf;

use bevy::asset::{AssetPath, embedded_asset, embedded_path};
use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

use super::waves::GerstnerWave;

/// Maximum number of concurrent Gerstner waves supported by the shader.
pub const MAX_WAVES: usize = 4;

/// Custom ocean material with Gerstner wave support.
///
/// This material handles GPU-side wave displacement and basic water shading.
/// Each field is bound as a separate uniform to avoid storage buffer issues.
#[derive(Asset, AsBindGroup, Debug, Clone, TypePath)]
pub struct OceanMaterial {
	/// Wave 0 direction (xy: normalized direction, zw: unused).
	#[uniform(0)]
	pub wave0_direction: Vec4,
	/// Wave 0 params (x: steepness, y: wavelength, z: amplitude, w: speed).
	#[uniform(1)]
	pub wave0_params: Vec4,
	/// Wave 1 direction.
	#[uniform(2)]
	pub wave1_direction: Vec4,
	/// Wave 1 params.
	#[uniform(3)]
	pub wave1_params: Vec4,
	/// Wave 2 direction.
	#[uniform(4)]
	pub wave2_direction: Vec4,
	/// Wave 2 params.
	#[uniform(5)]
	pub wave2_params: Vec4,
	/// Wave 3 direction.
	#[uniform(6)]
	pub wave3_direction: Vec4,
	/// Wave 3 params.
	#[uniform(7)]
	pub wave3_params: Vec4,
	/// Time and config (x: time, y: wave_count, z: use_env_map, w: unused).
	#[uniform(8)]
	pub time_and_config: Vec4,
	/// Deep water color (viewed from above).
	#[uniform(9)]
	pub deep_color: Vec4,
	/// Shallow water color (viewed at angle).
	#[uniform(10)]
	pub shallow_color: Vec4,
	/// Fresnel params (x: F0, y: power, z: bias, w: unused).
	#[uniform(11)]
	pub fresnel_params: Vec4,
	/// Sky color for non-envmap reflections.
	#[uniform(12)]
	pub sky_color: Vec4,

	/// Environment cubemap for sky reflections.
	#[texture(13, dimension = "cube")]
	#[sampler(14)]
	pub environment_map: Option<Handle<Image>>,
}

impl Default for OceanMaterial {
	fn default() -> Self {
		Self {
			wave0_direction: Vec4::new(1.0, 0.0, 0.0, 0.0),
			wave0_params: Vec4::new(0.5, 60.0, 2.0, 1.0),
			wave1_direction: Vec4::new(1.0, 0.0, 0.0, 0.0),
			wave1_params: Vec4::new(0.5, 60.0, 0.0, 1.0), // amplitude 0 = disabled
			wave2_direction: Vec4::new(1.0, 0.0, 0.0, 0.0),
			wave2_params: Vec4::new(0.5, 60.0, 0.0, 1.0),
			wave3_direction: Vec4::new(1.0, 0.0, 0.0, 0.0),
			wave3_params: Vec4::new(0.5, 60.0, 0.0, 1.0),
			time_and_config: Vec4::new(0.0, 1.0, 0.0, 0.0),
			deep_color: Vec4::new(0.0, 0.1, 0.3, 1.0),
			shallow_color: Vec4::new(0.0, 0.4, 0.5, 1.0),
			fresnel_params: Vec4::new(0.02, 5.0, 0.0, 0.0),
			sky_color: Vec4::new(0.5, 0.7, 0.9, 1.0),
			environment_map: None,
		}
	}
}

impl OceanMaterial {
	/// Creates a new ocean material with the specified waves.
	#[must_use]
	pub fn new(waves: &[GerstnerWave], deep_color: Color, shallow_color: Color) -> Self {
		let wave_count = waves.len().min(MAX_WAVES);

		let wave_dir = |i: usize| -> Vec4 {
			waves
				.get(i)
				.map(|w| Vec4::new(w.direction.x, w.direction.y, 0.0, 0.0))
				.unwrap_or(Vec4::new(1.0, 0.0, 0.0, 0.0))
		};

		let wave_params = |i: usize| -> Vec4 {
			waves
				.get(i)
				.map(|w| Vec4::new(w.steepness, w.wavelength, w.amplitude, w.speed))
				.unwrap_or(Vec4::new(0.5, 60.0, 0.0, 1.0)) // amplitude 0 = disabled
		};

		Self {
			wave0_direction: wave_dir(0),
			wave0_params: wave_params(0),
			wave1_direction: wave_dir(1),
			wave1_params: wave_params(1),
			wave2_direction: wave_dir(2),
			wave2_params: wave_params(2),
			wave3_direction: wave_dir(3),
			wave3_params: wave_params(3),
			time_and_config: Vec4::new(0.0, wave_count as f32, 0.0, 0.0),
			deep_color: deep_color.to_linear().to_vec4(),
			shallow_color: shallow_color.to_linear().to_vec4(),
			fresnel_params: Vec4::new(0.02, 5.0, 0.0, 0.0),
			sky_color: Vec4::new(0.5, 0.7, 0.9, 1.0),
			environment_map: None,
		}
	}

	/// Creates a new ocean material with Fresnel parameters.
	#[must_use]
	pub fn with_fresnel(
		waves: &[GerstnerWave],
		deep_color: Color,
		shallow_color: Color,
		sky_color: Color,
		fresnel_f0: f32,
		fresnel_power: f32,
		fresnel_bias: f32,
	) -> Self {
		let mut material = Self::new(waves, deep_color, shallow_color);
		material.fresnel_params = Vec4::new(fresnel_f0, fresnel_power, fresnel_bias, 0.0);
		material.sky_color = sky_color.to_linear().to_vec4();
		material
	}

	/// Creates a new ocean material with environment map reflections.
	#[must_use]
	pub fn with_environment_map(
		waves: &[GerstnerWave],
		deep_color: Color,
		shallow_color: Color,
		fresnel_f0: f32,
		fresnel_power: f32,
		fresnel_bias: f32,
		environment_map: Handle<Image>,
	) -> Self {
		let mut material = Self::new(waves, deep_color, shallow_color);
		material.fresnel_params = Vec4::new(fresnel_f0, fresnel_power, fresnel_bias, 0.0);
		material.time_and_config.z = 1.0;
		material.environment_map = Some(environment_map);
		material
	}

	/// Updates the time uniform for wave animation.
	pub fn set_time(&mut self, time: f32) {
		self.time_and_config.x = time;
	}

	/// Sets the Fresnel parameters.
	pub fn set_fresnel(&mut self, f0: f32, power: f32, bias: f32) {
		self.fresnel_params = Vec4::new(f0, power, bias, 0.0);
	}

	/// Sets the sky/reflection color placeholder.
	pub fn set_sky_color(&mut self, color: Color) {
		self.sky_color = color.to_linear().to_vec4();
	}

	/// Sets the environment map for reflections.
	pub fn set_environment_map(&mut self, environment_map: Handle<Image>) {
		self.environment_map = Some(environment_map);
		self.time_and_config.z = 1.0;
	}
}

fn shader_ref(path: PathBuf) -> ShaderRef {
	ShaderRef::Path(AssetPath::from_path_buf(path).with_source("embedded"))
}

impl Material for OceanMaterial {
	fn vertex_shader() -> ShaderRef {
		shader_ref(embedded_path!("ocean.wgsl"))
	}

	fn fragment_shader() -> ShaderRef {
		shader_ref(embedded_path!("ocean.wgsl"))
	}
}

/// Plugin that registers the ocean material and its shader.
pub struct OceanMaterialPlugin;

impl Plugin for OceanMaterialPlugin {
	fn build(&self, app: &mut App) {
		embedded_asset!(app, "ocean.wgsl");
		app.add_plugins(MaterialPlugin::<OceanMaterial>::default());
	}
}
