//! GPU to CPU image copy via render graph.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{
	self, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel,
};
use bevy::render::render_resource::{
	Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, MapMode, PollType,
	TexelCopyBufferInfo, TexelCopyBufferLayout,
};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::{Extract, Render, RenderApp, RenderSystems};
use crossbeam_channel::{Receiver, Sender};

/// Channel receiver for image data from render world
#[derive(Resource, Deref)]
pub struct MainWorldReceiver(pub Receiver<Vec<u8>>);

/// Channel sender for image data to main world
#[derive(Resource, Deref)]
pub struct RenderWorldSender(Sender<Vec<u8>>);

/// Plugin for copying rendered images from GPU to CPU via render graph.
pub struct ImageCopyPlugin;

impl Plugin for ImageCopyPlugin {
	fn build(&self, app: &mut App) {
		let (sender, receiver) = crossbeam_channel::unbounded();

		let render_app = app
			.insert_resource(MainWorldReceiver(receiver))
			.sub_app_mut(RenderApp);

		let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
		graph.add_node(ImageCopyLabel, ImageCopyDriver);
		graph.add_node_edge(bevy::render::graph::CameraDriverLabel, ImageCopyLabel);

		render_app
			.insert_resource(RenderWorldSender(sender))
			.add_systems(ExtractSchedule, image_copy_extract)
			.add_systems(
				Render,
				receive_image_from_buffer.after(RenderSystems::Render),
			);
	}
}

/// Aggregator for image copiers in render world
#[derive(Clone, Default, Resource, Deref, DerefMut)]
pub struct ImageCopiers(pub Vec<ImageCopier>);

/// Component for copying from render target to buffer
#[derive(Clone, Component)]
pub struct ImageCopier {
	buffer: Buffer,
	enabled: Arc<AtomicBool>,
	src_image: Handle<Image>,
}

impl ImageCopier {
	pub fn new(src_image: Handle<Image>, size: Extent3d, render_device: &RenderDevice) -> Self {
		let padded_bytes_per_row =
			RenderDevice::align_copy_bytes_per_row((size.width) as usize) * 4;

		let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
			label: Some("screenshot_buffer"),
			size: padded_bytes_per_row as u64 * size.height as u64,
			usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		Self {
			buffer: cpu_buffer,
			src_image,
			enabled: Arc::new(AtomicBool::new(true)),
		}
	}

	pub fn enabled(&self) -> bool {
		self.enabled.load(Ordering::Relaxed)
	}
}

/// CPU-side image handle for saving
#[derive(Component, Deref, DerefMut)]
pub struct ImageToSave(pub Handle<Image>);

fn image_copy_extract(mut commands: Commands, image_copiers: Extract<Query<&ImageCopier>>) {
	commands.insert_resource(ImageCopiers(image_copiers.iter().cloned().collect()));
}

/// Render graph label for image copy node
#[derive(Debug, PartialEq, Eq, Clone, Hash, RenderLabel)]
struct ImageCopyLabel;

/// Render graph node that copies texture to buffer
#[derive(Default)]
struct ImageCopyDriver;

impl render_graph::Node for ImageCopyDriver {
	fn run(
		&self,
		_graph: &mut RenderGraphContext,
		render_context: &mut RenderContext,
		world: &World,
	) -> Result<(), NodeRunError> {
		let image_copiers = world.get_resource::<ImageCopiers>().unwrap();
		let gpu_images = world
			.get_resource::<RenderAssets<bevy::render::texture::GpuImage>>()
			.unwrap();

		for image_copier in image_copiers.iter() {
			if !image_copier.enabled() {
				continue;
			}

			let Some(src_image) = gpu_images.get(&image_copier.src_image) else {
				continue;
			};

			let mut encoder = render_context
				.render_device()
				.create_command_encoder(&CommandEncoderDescriptor::default());

			let block_dimensions = src_image.texture_format.block_dimensions();
			let block_size = src_image.texture_format.block_copy_size(None).unwrap();

			let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
				(src_image.size.width as usize / block_dimensions.0 as usize) * block_size as usize,
			);

			encoder.copy_texture_to_buffer(
				src_image.texture.as_image_copy(),
				TexelCopyBufferInfo {
					buffer: &image_copier.buffer,
					layout: TexelCopyBufferLayout {
						offset: 0,
						bytes_per_row: Some(
							std::num::NonZero::<u32>::new(padded_bytes_per_row as u32)
								.unwrap()
								.into(),
						),
						rows_per_image: None,
					},
				},
				src_image.size,
			);

			let render_queue = world.get_resource::<RenderQueue>().unwrap();
			render_queue.submit(std::iter::once(encoder.finish()));
		}

		Ok(())
	}
}

/// Receives image data from GPU buffer and sends to main world
fn receive_image_from_buffer(
	image_copiers: Res<ImageCopiers>,
	render_device: Res<RenderDevice>,
	sender: Res<RenderWorldSender>,
) {
	for image_copier in image_copiers.0.iter() {
		if !image_copier.enabled() {
			continue;
		}

		let buffer_slice = image_copier.buffer.slice(..);

		let (s, r) = crossbeam_channel::bounded(1);

		buffer_slice.map_async(MapMode::Read, move |result| match result {
			Ok(()) => s.send(()).expect("Failed to send map update"),
			Err(err) => panic!("Failed to map buffer: {err}"),
		});

		render_device
			.poll(PollType::wait_indefinitely())
			.expect("Failed to poll device");

		r.recv().expect("Failed to receive map_async message");

		let _ = sender.send(buffer_slice.get_mapped_range().to_vec());

		image_copier.buffer.unmap();
	}
}
