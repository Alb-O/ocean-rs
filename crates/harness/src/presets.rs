//! Camera presets for screenshot capture.

use bevy::prelude::*;

/// Minimum camera height
pub const MIN_CAMERA_HEIGHT: f32 = 2.0;

/// A camera position preset for screenshots
#[derive(Clone, Copy, Debug)]
pub struct CameraPreset {
	pub name: &'static str,
	pub radius: f32,
	pub height: f32,
	pub angle: f32,
	pub look_offset: Vec3,
}

impl CameraPreset {
	pub fn to_position(self) -> Vec3 {
		let x = self.radius * self.angle.sin();
		let z = self.radius * self.angle.cos();
		let y = self.height.max(MIN_CAMERA_HEIGHT);
		Vec3::new(x, y, z)
	}
}

/// Standard camera presets for visualization
pub const STANDARD_PRESETS: &[CameraPreset] = &[
	CameraPreset {
		name: "wide",
		radius: 120.0,
		height: 35.0,
		angle: 0.0,
		look_offset: Vec3::ZERO,
	},
	CameraPreset {
		name: "close",
		radius: 40.0,
		height: 15.0,
		angle: 2.5,
		look_offset: Vec3::ZERO,
	},
	CameraPreset {
		name: "dramatic",
		radius: 70.0,
		height: 20.0,
		angle: 5.5,
		look_offset: Vec3::new(10.0, 0.0, 10.0),
	},
];

/// Close-up presets for detail inspection
pub const DETAIL_PRESETS: &[CameraPreset] = &[
	CameraPreset {
		name: "detail_top",
		radius: 20.0,
		height: 30.0,
		angle: 0.0,
		look_offset: Vec3::ZERO,
	},
	CameraPreset {
		name: "detail_angle",
		radius: 15.0,
		height: 8.0,
		angle: 0.8,
		look_offset: Vec3::ZERO,
	},
	CameraPreset {
		name: "detail_low",
		radius: 12.0,
		height: 3.0,
		angle: 1.2,
		look_offset: Vec3::ZERO,
	},
];

/// Simple presets for basic examples (cube, etc.)
pub const SIMPLE_PRESETS: &[CameraPreset] = &[
	CameraPreset {
		name: "front",
		radius: 5.0,
		height: 3.0,
		angle: 0.5,
		look_offset: Vec3::ZERO,
	},
	CameraPreset {
		name: "angle",
		radius: 6.0,
		height: 4.0,
		angle: 2.0,
		look_offset: Vec3::ZERO,
	},
	CameraPreset {
		name: "top",
		radius: 4.0,
		height: 6.0,
		angle: 0.0,
		look_offset: Vec3::ZERO,
	},
];
