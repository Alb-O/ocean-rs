//! Screenshot configuration with CLI argument support.

use std::path::PathBuf;

use bevy::prelude::*;
use clap::Parser;

use crate::presets::{CameraPreset, DETAIL_PRESETS, SIMPLE_PRESETS, STANDARD_PRESETS};

/// Default screenshot output directory
pub const DEFAULT_OUTPUT_DIR: &str = "output";

/// Screenshot image width
pub const SCREENSHOT_WIDTH: u32 = 1920;

/// Screenshot image height
pub const SCREENSHOT_HEIGHT: u32 = 1080;

/// CLI arguments for screenshot configuration
#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Screenshot harness for Bevy examples")]
pub struct CliArgs {
	/// Run with interactive window instead of headless screenshot mode
	#[arg(long, short = 'i')]
	pub interactive: bool,

	/// Output directory for screenshots
	#[arg(long, short = 'o', default_value = DEFAULT_OUTPUT_DIR)]
	pub output_dir: PathBuf,

	/// Screenshot width
	#[arg(long, short = 'W', default_value_t = SCREENSHOT_WIDTH)]
	pub width: u32,

	/// Screenshot height
	#[arg(long, short = 'H', default_value_t = SCREENSHOT_HEIGHT)]
	pub height: u32,

	/// Exit application after capturing screenshots
	#[arg(long, default_value_t = true)]
	pub exit_after: bool,

	/// Capture multiple presets (vs just one)
	#[arg(long, default_value_t = true)]
	pub multi_shot: bool,

	/// Number of recent sessions to retain
	#[arg(long, default_value_t = 5)]
	pub retain_sessions: usize,
}

impl CliArgs {
	/// Parse CLI arguments (cached after first call)
	pub fn get() -> &'static Self {
		use std::sync::OnceLock;
		static ARGS: OnceLock<CliArgs> = OnceLock::new();
		ARGS.get_or_init(Self::parse)
	}
}

impl Default for CliArgs {
	fn default() -> Self {
		Self {
			interactive: false,
			output_dir: PathBuf::from(DEFAULT_OUTPUT_DIR),
			width: SCREENSHOT_WIDTH,
			height: SCREENSHOT_HEIGHT,
			exit_after: true,
			multi_shot: true,
			retain_sessions: 5,
		}
	}
}

/// Configuration for the screenshot system
#[derive(Resource, Clone)]
pub struct ScreenshotConfig {
	/// Exit application after capturing all screenshots
	pub exit_after: bool,
	/// Capture multiple presets (vs just one)
	pub multi_shot: bool,
	/// Camera presets to use
	pub presets: Vec<CameraPreset>,
	/// Example name (determines screenshot location)
	pub example_name: String,
	/// Screenshot width
	pub width: u32,
	/// Screenshot height
	pub height: u32,
	/// Output directory
	pub output_dir: PathBuf,
	/// Number of sessions to retain
	pub retain_sessions: usize,
}

impl ScreenshotConfig {
	/// Returns the base directory for screenshots: {output_dir}/{example_name}/screenshots
	pub fn screenshot_dir(&self) -> PathBuf {
		self.output_dir.join(&self.example_name).join("screenshots")
	}
}

impl Default for ScreenshotConfig {
	fn default() -> Self {
		Self {
			exit_after: true,
			multi_shot: true,
			presets: STANDARD_PRESETS.to_vec(),
			example_name: "default".to_string(),
			width: SCREENSHOT_WIDTH,
			height: SCREENSHOT_HEIGHT,
			output_dir: PathBuf::from(DEFAULT_OUTPUT_DIR),
			retain_sessions: 5,
		}
	}
}

impl ScreenshotConfig {
	/// Create config from CLI arguments
	pub fn from_cli(example_name: &str) -> Self {
		let args = CliArgs::parse();

		Self {
			exit_after: args.exit_after,
			multi_shot: args.multi_shot,
			presets: STANDARD_PRESETS.to_vec(),
			example_name: example_name.to_string(),
			width: args.width,
			height: args.height,
			output_dir: args.output_dir,
			retain_sessions: args.retain_sessions,
		}
	}

	/// Create config from environment variables (legacy compatibility)
	pub fn from_env(example_name: &str) -> Self {
		let exit_after = std::env::var("SCREENSHOT_EXIT")
			.map(|v| v != "0")
			.unwrap_or(true);
		let multi_shot = std::env::var("SCREENSHOT_MULTI")
			.map(|v| v != "0")
			.unwrap_or(true);

		Self {
			exit_after,
			multi_shot,
			presets: STANDARD_PRESETS.to_vec(),
			example_name: example_name.to_string(),
			width: SCREENSHOT_WIDTH,
			height: SCREENSHOT_HEIGHT,
			output_dir: PathBuf::from(DEFAULT_OUTPUT_DIR),
			retain_sessions: 5,
		}
	}

	/// Use detail presets instead of standard ones
	pub fn with_detail_presets(mut self) -> Self {
		self.presets = DETAIL_PRESETS.to_vec();
		self
	}

	/// Use simple presets for basic examples
	pub fn with_simple_presets(mut self) -> Self {
		self.presets = SIMPLE_PRESETS.to_vec();
		self
	}

	/// Use custom presets
	pub fn with_presets(mut self, presets: Vec<CameraPreset>) -> Self {
		self.presets = presets;
		self
	}

	/// Don't exit after screenshots
	pub fn no_exit(mut self) -> Self {
		self.exit_after = false;
		self
	}

	/// Set custom resolution
	pub fn with_resolution(mut self, width: u32, height: u32) -> Self {
		self.width = width;
		self.height = height;
		self
	}

	/// Set custom output directory
	pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
		self.output_dir = dir.into();
		self
	}
}
