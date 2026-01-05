//! Custom ocean material with Gerstner wave displacement.
//!
//! This material implements GPU-side Gerstner wave calculations for realistic
//! ocean surface animation. Wave parameters are passed as uniforms and processed
//! in the vertex shader.

use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use super::waves::GerstnerWave;

/// Maximum number of concurrent Gerstner waves supported by the shader.
pub const MAX_WAVES: usize = 4;

/// GPU-compatible representation of a Gerstner wave.
///
/// Layout matches the WGSL struct for uniform buffer transfer.
#[derive(Debug, Clone, Copy, ShaderType)]
pub struct GpuGerstnerWave {
	/// Wave direction (normalized) in xy, zw unused for padding.
	pub direction: Vec4,
	/// x: steepness, y: wavelength, z: amplitude, w: speed.
	pub params: Vec4,
}

impl Default for GpuGerstnerWave {
	fn default() -> Self {
		Self {
			direction: Vec4::new(1.0, 0.0, 0.0, 0.0),
			params: Vec4::new(0.5, 60.0, 2.0, 1.0),
		}
	}
}

impl From<&GerstnerWave> for GpuGerstnerWave {
	fn from(wave: &GerstnerWave) -> Self {
		Self {
			direction: Vec4::new(wave.direction.x, wave.direction.y, 0.0, 0.0),
			params: Vec4::new(wave.steepness, wave.wavelength, wave.amplitude, wave.speed),
		}
	}
}

/// GPU uniform structure for ocean rendering.
///
/// Contains all wave data and material parameters needed by the shader.
#[derive(Debug, Clone, Copy, ShaderType)]
pub struct OceanUniforms {
	/// Array of up to 4 Gerstner waves.
	pub waves: [GpuGerstnerWave; MAX_WAVES],
	/// x: time, y: active_wave_count, z: use_env_map (1.0 = true), w: unused.
	pub time_and_config: Vec4,
	/// Deep water color (viewed from above).
	pub deep_color: LinearRgba,
	/// Shallow water color (viewed at angle).
	pub shallow_color: LinearRgba,
	/// Fresnel parameters: x: F0 (base reflectance), y: power, z: bias, w: unused.
	pub fresnel_params: Vec4,
	/// Sky/reflection color placeholder (used when no environment map is set).
	pub sky_color: LinearRgba,
}

impl Default for OceanUniforms {
	fn default() -> Self {
		Self {
			waves: [GpuGerstnerWave::default(); MAX_WAVES],
			time_and_config: Vec4::new(0.0, 1.0, 0.0, 0.0),
			deep_color: LinearRgba::new(0.0, 0.1, 0.3, 1.0),
			shallow_color: LinearRgba::new(0.0, 0.4, 0.5, 1.0),
			// F0 for water ≈ 0.02, power = 5.0 (standard Schlick), bias = 0.0
			fresnel_params: Vec4::new(0.02, 5.0, 0.0, 0.0),
			sky_color: LinearRgba::new(0.5, 0.7, 0.9, 1.0),
		}
	}
}

/// Custom ocean material with Gerstner wave support.
///
/// This material handles GPU-side wave displacement and basic water shading.
/// Wave parameters are configurable via the [`OceanConfig`] resource.
#[derive(Asset, AsBindGroup, Debug, Clone, Default, TypePath)]
pub struct OceanMaterial {
	/// Ocean rendering uniforms including wave data.
	#[uniform(0)]
	pub uniforms: OceanUniforms,

	/// Environment cubemap for sky reflections.
	#[texture(1, dimension = "cube")]
	#[sampler(2)]
	pub environment_map: Option<Handle<Image>>,
}

impl OceanMaterial {
	/// Creates a new ocean material with the specified waves.
	#[must_use]
	pub fn new(waves: &[GerstnerWave], deep_color: Color, shallow_color: Color) -> Self {
		let mut gpu_waves = [GpuGerstnerWave::default(); MAX_WAVES];
		let wave_count = waves.len().min(MAX_WAVES);

		for (i, wave) in waves.iter().take(wave_count).enumerate() {
			gpu_waves[i] = GpuGerstnerWave::from(wave);
		}

		// Zero out unused waves
		for wave in gpu_waves.iter_mut().skip(wave_count) {
			wave.params.z = 0.0; // Zero amplitude disables wave
		}

		Self {
			uniforms: OceanUniforms {
				waves: gpu_waves,
				time_and_config: Vec4::new(0.0, wave_count as f32, 0.0, 0.0),
				deep_color: deep_color.to_linear(),
				shallow_color: shallow_color.to_linear(),
				..Default::default()
			},
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
		material.uniforms.fresnel_params = Vec4::new(fresnel_f0, fresnel_power, fresnel_bias, 0.0);
		material.uniforms.sky_color = sky_color.to_linear();
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
		material.uniforms.fresnel_params = Vec4::new(fresnel_f0, fresnel_power, fresnel_bias, 0.0);
		// Set use_env_map flag (z component of time_and_config)
		material.uniforms.time_and_config.z = 1.0;
		material.environment_map = Some(environment_map);
		material
	}

	/// Updates the time uniform for wave animation.
	pub fn set_time(&mut self, time: f32) {
		self.uniforms.time_and_config.x = time;
	}

	/// Sets the Fresnel parameters.
	pub fn set_fresnel(&mut self, f0: f32, power: f32, bias: f32) {
		self.uniforms.fresnel_params = Vec4::new(f0, power, bias, 0.0);
	}

	/// Sets the sky/reflection color placeholder.
	pub fn set_sky_color(&mut self, color: Color) {
		self.uniforms.sky_color = color.to_linear();
	}

	/// Sets the environment map for reflections.
	pub fn set_environment_map(&mut self, environment_map: Handle<Image>) {
		self.environment_map = Some(environment_map);
		self.uniforms.time_and_config.z = 1.0;
	}
}

impl Material for OceanMaterial {
	fn vertex_shader() -> ShaderRef {
		"shaders/ocean.wgsl".into()
	}

	fn fragment_shader() -> ShaderRef {
		"shaders/ocean.wgsl".into()
	}
}

/// Plugin that registers the ocean material and its shader.
pub struct OceanMaterialPlugin;

impl Plugin for OceanMaterialPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(MaterialPlugin::<OceanMaterial>::default());
	}
}
