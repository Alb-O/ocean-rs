//! Gerstner wave mathematics for ocean surface displacement.
//!
//! Gerstner (trochoidal) waves create realistic circular orbital motion,
//! displacing vertices both vertically (Y) and horizontally (XZ) to produce
//! characteristic peaked crests and rounded troughs.

use bevy::prelude::*;
use std::f32::consts::PI;

/// Gravity constant for deep water wave dispersion (m/s²).
pub const GRAVITY: f32 = 9.81;

/// Parameters for a single Gerstner wave.
#[derive(Debug, Clone, Copy, Reflect)]
pub struct GerstnerWave {
	/// Normalized wave propagation direction in XZ plane.
	pub direction: Vec2,
	/// Steepness parameter Q (0-1), affects sharpness of wave crests.
	/// Higher values create sharper peaks but may cause looping at Q > 1/(k*A*n).
	pub steepness: f32,
	/// Distance between wave crests in world units.
	pub wavelength: f32,
	/// Wave height / 2 (vertical displacement amplitude).
	pub amplitude: f32,
	/// Phase speed multiplier (1.0 = physically accurate).
	pub speed: f32,
}

impl Default for GerstnerWave {
	fn default() -> Self {
		Self {
			direction: Vec2::new(1.0, 0.0),
			steepness: 0.5,
			wavelength: 60.0,
			amplitude: 2.0,
			speed: 1.0,
		}
	}
}

impl GerstnerWave {
	/// Creates a new Gerstner wave with the given parameters.
	///
	/// The direction is automatically normalized.
	#[must_use]
	pub fn new(direction: Vec2, steepness: f32, wavelength: f32, amplitude: f32, speed: f32) -> Self {
		Self {
			direction: direction.normalize_or_zero(),
			steepness,
			wavelength,
			amplitude,
			speed,
		}
	}

	/// Returns the wave number k = 2π / wavelength.
	#[inline]
	#[must_use]
	pub fn wave_number(&self) -> f32 {
		2.0 * PI / self.wavelength
	}

	/// Returns the angular frequency ω = sqrt(g * k) for deep water dispersion.
	#[inline]
	#[must_use]
	pub fn angular_frequency(&self) -> f32 {
		(GRAVITY * self.wave_number()).sqrt() * self.speed
	}

	/// Evaluates the wave at a world XZ position and time.
	///
	/// Returns `(position_offset, normal)` where:
	/// - `position_offset` is the displacement to add to the base position
	/// - `normal` is the surface normal at this point
	#[must_use]
	pub fn evaluate(&self, world_xz: Vec2, time: f32) -> (Vec3, Vec3) {
		let k = self.wave_number();
		let omega = self.angular_frequency();
		let d = self.direction;
		let a = self.amplitude;
		let q = self.steepness;

		// Phase at this point and time
		let phase = k * d.dot(world_xz) - omega * time;
		let cos_phase = phase.cos();
		let sin_phase = phase.sin();

		// Position displacement (Gerstner formula)
		// Q controls horizontal displacement relative to vertical
		let x_offset = q * a * d.x * cos_phase;
		let z_offset = q * a * d.y * cos_phase;
		let y_offset = a * sin_phase;

		let position_offset = Vec3::new(x_offset, y_offset, z_offset);

		// Analytical normal calculation
		// Partial derivatives of the displaced surface
		let wa = k * a;
		let s = sin_phase;
		let c = cos_phase;

		// Binormal (partial derivative in X direction) and Tangent (partial in Z)
		// These form the tangent space of the displaced surface
		let b = Vec3::new(1.0 - q * d.x * d.x * wa * s, d.x * wa * c, -q * d.x * d.y * wa * s);

		let t = Vec3::new(-q * d.x * d.y * wa * s, d.y * wa * c, 1.0 - q * d.y * d.y * wa * s);

		let normal = b.cross(t).normalize();

		(position_offset, normal)
	}

	/// Evaluates only the position displacement (faster when normal not needed).
	#[must_use]
	pub fn evaluate_position(&self, world_xz: Vec2, time: f32) -> Vec3 {
		let k = self.wave_number();
		let omega = self.angular_frequency();
		let d = self.direction;
		let a = self.amplitude;
		let q = self.steepness;

		let phase = k * d.dot(world_xz) - omega * time;
		let cos_phase = phase.cos();
		let sin_phase = phase.sin();

		Vec3::new(q * a * d.x * cos_phase, a * sin_phase, q * a * d.y * cos_phase)
	}
}

/// Evaluates multiple Gerstner waves and sums their contributions.
///
/// Returns `(total_offset, combined_normal)`.
#[must_use]
pub fn evaluate_waves(waves: &[GerstnerWave], world_xz: Vec2, time: f32) -> (Vec3, Vec3) {
	if waves.is_empty() {
		return (Vec3::ZERO, Vec3::Y);
	}

	let mut total_offset = Vec3::ZERO;

	// For combined normal, we sum the partial derivatives then compute the final normal
	let mut binormal = Vec3::X;
	let mut tangent = Vec3::Z;

	for wave in waves {
		let k = wave.wave_number();
		let omega = wave.angular_frequency();
		let d = wave.direction;
		let a = wave.amplitude;
		let q = wave.steepness;

		let phase = k * d.dot(world_xz) - omega * time;
		let cos_phase = phase.cos();
		let sin_phase = phase.sin();

		// Accumulate position offset
		total_offset.x += q * a * d.x * cos_phase;
		total_offset.y += a * sin_phase;
		total_offset.z += q * a * d.y * cos_phase;

		// Accumulate tangent frame modifications
		let wa = k * a;
		let s = sin_phase;
		let c = cos_phase;

		binormal.x -= q * d.x * d.x * wa * s;
		binormal.y += d.x * wa * c;
		binormal.z -= q * d.x * d.y * wa * s;

		tangent.x -= q * d.x * d.y * wa * s;
		tangent.y += d.y * wa * c;
		tangent.z -= q * d.y * d.y * wa * s;
	}

	let normal = binormal.cross(tangent).normalize();

	(total_offset, normal)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_default_wave() {
		let wave = GerstnerWave::default();
		assert!(wave.direction.length() > 0.99 && wave.direction.length() < 1.01);
		assert!(wave.wavelength > 0.0);
		assert!(wave.amplitude > 0.0);
	}

	#[test]
	fn test_wave_number() {
		let wave = GerstnerWave::new(Vec2::X, 0.5, 60.0, 2.0, 1.0);
		let k = wave.wave_number();
		assert!((k - 2.0 * PI / 60.0).abs() < 0.001);
	}

	#[test]
	fn test_evaluate_returns_valid_normal() {
		let wave = GerstnerWave::default();
		let (_, normal) = wave.evaluate(Vec2::ZERO, 0.0);
		assert!((normal.length() - 1.0).abs() < 0.001, "Normal should be normalized");
	}

	#[test]
	fn test_multi_wave_combines() {
		let waves = vec![
			GerstnerWave::new(Vec2::X, 0.5, 60.0, 2.0, 1.0),
			GerstnerWave::new(Vec2::new(0.7, 0.7), 0.5, 30.0, 1.0, 1.0),
		];
		let (offset, normal) = evaluate_waves(&waves, Vec2::ZERO, 0.0);
		assert!((normal.length() - 1.0).abs() < 0.001, "Combined normal should be normalized");
		// With steepness 0.5, we should have some horizontal offset
		assert!(offset.x.abs() > 0.0 || offset.z.abs() > 0.0);
	}
}
