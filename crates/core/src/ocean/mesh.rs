//! Projected grid mesh generation for ocean rendering.
//!
//! A projected grid creates vertices in screen-space, then projects them onto the ocean plane (y=0).
//! This gives uniform screen-space vertex density regardless of view distance, creating the
//! illusion of an infinite ocean.

use bevy::camera::CameraProjection;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// Configuration for the projected grid ocean mesh.
#[derive(Resource)]
pub struct ProjectedGridConfig {
	/// Number of vertices along each axis of the grid.
	pub resolution: u32,
	/// Maximum distance from camera to clamp projected vertices.
	pub max_distance: f32,
	/// Height of the ocean plane (y coordinate).
	pub ocean_height: f32,
	/// Threshold for camera movement before regenerating mesh.
	pub update_threshold: f32,
}

impl Default for ProjectedGridConfig {
	fn default() -> Self {
		Self {
			resolution: 128,
			max_distance: 2000.0,
			ocean_height: 0.0,
			update_threshold: 0.5,
		}
	}
}

/// Configuration for creating an ocean mesh.
pub struct OceanMeshConfig {
	/// Number of vertices along each axis.
	pub resolution: u32,
	/// Maximum distance for projected vertices.
	pub max_distance: f32,
	/// Camera transform for projection.
	pub camera_transform: Transform,
	/// Camera projection matrix.
	pub projection: Mat4,
	/// Ocean plane height.
	pub ocean_height: f32,
}

/// Marker component for ocean mesh entities.
#[derive(Component)]
pub struct OceanMesh {
	/// Last camera position used for mesh generation.
	pub last_camera_position: Vec3,
	/// Last camera rotation used for mesh generation.
	pub last_camera_rotation: Quat,
}

/// Creates a projected grid mesh for ocean rendering.
///
/// The mesh is created by:
/// 1. Generating a grid of vertices in clip-space (-1 to 1)
/// 2. Projecting each vertex ray from camera onto y=0 plane
/// 3. Handling horizon edge cases (rays parallel to plane)
pub fn create_projected_grid_mesh(config: &OceanMeshConfig) -> Mesh {
	let resolution = config.resolution;
	let max_distance = config.max_distance;
	let ocean_height = config.ocean_height;

	let view_matrix = config.camera_transform.to_matrix();
	let inv_view_proj = (config.projection * view_matrix.inverse()).inverse();
	let camera_pos = config.camera_transform.translation;

	let num_vertices = (resolution * resolution) as usize;
	let mut positions = Vec::with_capacity(num_vertices);
	let mut normals = Vec::with_capacity(num_vertices);
	let mut uvs = Vec::with_capacity(num_vertices);

	// Generate grid vertices
	for j in 0..resolution {
		for i in 0..resolution {
			// Map to normalized coordinates [0, 1]
			let u = i as f32 / (resolution - 1) as f32;
			let v = j as f32 / (resolution - 1) as f32;

			// Map to clip space [-1, 1]
			let clip_x = u * 2.0 - 1.0;
			let clip_y = v * 2.0 - 1.0;

			// Use a near plane position (z close to 0 in NDC, which maps to near plane)
			// and far plane position to create a ray
			let near_clip = Vec4::new(clip_x, clip_y, 0.0, 1.0);
			let far_clip = Vec4::new(clip_x, clip_y, 1.0, 1.0);

			let near_world = inv_view_proj * near_clip;
			let far_world = inv_view_proj * far_clip;

			let near_world = near_world.xyz() / near_world.w;
			let far_world = far_world.xyz() / far_world.w;

			// Ray from near to far (camera looks down -Z in view space)
			let ray_dir = (far_world - near_world).normalize();
			let ray_origin = near_world;

			// Intersect ray with ocean plane (y = ocean_height)
			let world_pos = intersect_ray_plane(ray_origin, ray_dir, ocean_height, camera_pos, max_distance);

			positions.push([world_pos.x, world_pos.y, world_pos.z]);
			normals.push([0.0, 1.0, 0.0]); // Ocean normal points up
			uvs.push([u, v]);
		}
	}

	// Generate indices for triangle strip
	let num_indices = ((resolution - 1) * (resolution - 1) * 6) as usize;
	let mut indices = Vec::with_capacity(num_indices);

	for j in 0..(resolution - 1) {
		for i in 0..(resolution - 1) {
			let top_left = j * resolution + i;
			let top_right = top_left + 1;
			let bottom_left = (j + 1) * resolution + i;
			let bottom_right = bottom_left + 1;

			// First triangle (counter-clockwise for front-facing)
			indices.push(top_left);
			indices.push(bottom_left);
			indices.push(top_right);

			// Second triangle
			indices.push(top_right);
			indices.push(bottom_left);
			indices.push(bottom_right);
		}
	}

	let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::default());
	mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
	mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
	mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
	mesh.insert_indices(Indices::U32(indices));

	mesh
}

/// Intersects a ray with the ocean plane (y = ocean_height).
/// Returns clamped position if ray is parallel or points away from plane.
fn intersect_ray_plane(
	ray_origin: Vec3,
	ray_dir: Vec3,
	ocean_height: f32,
	camera_pos: Vec3,
	max_distance: f32,
) -> Vec3 {
	// Plane normal is (0, 1, 0), plane equation: y = ocean_height
	let denom = ray_dir.y;

	// Check if ray is roughly parallel to plane (looking at horizon)
	if denom.abs() < 0.0001 {
		// Project to maximum distance along XZ plane
		let horizontal_dir = Vec3::new(ray_dir.x, 0.0, ray_dir.z).normalize_or_zero();
		if horizontal_dir == Vec3::ZERO {
			return Vec3::new(camera_pos.x, ocean_height, camera_pos.z);
		}
		return Vec3::new(
			camera_pos.x + horizontal_dir.x * max_distance,
			ocean_height,
			camera_pos.z + horizontal_dir.z * max_distance,
		);
	}

	let t = (ocean_height - ray_origin.y) / denom;

	// If t is negative, ray points away from plane
	if t < 0.0 {
		// Use the direction on the XZ plane, clamped to max distance
		let horizontal_dir = Vec3::new(ray_dir.x, 0.0, ray_dir.z).normalize_or_zero();
		if horizontal_dir == Vec3::ZERO {
			return Vec3::new(camera_pos.x, ocean_height, camera_pos.z);
		}
		return Vec3::new(
			camera_pos.x + horizontal_dir.x * max_distance,
			ocean_height,
			camera_pos.z + horizontal_dir.z * max_distance,
		);
	}

	let intersection = ray_origin + ray_dir * t;

	// Clamp to maximum distance from camera
	let to_intersection = Vec3::new(
		intersection.x - camera_pos.x,
		0.0,
		intersection.z - camera_pos.z,
	);
	let distance = to_intersection.length();

	if distance > max_distance {
		let clamped_dir = to_intersection.normalize();
		Vec3::new(
			camera_pos.x + clamped_dir.x * max_distance,
			ocean_height,
			camera_pos.z + clamped_dir.z * max_distance,
		)
	} else {
		intersection
	}
}

/// System that updates the projected grid mesh when the camera moves significantly.
pub fn update_projected_grid(
	config: Res<ProjectedGridConfig>,
	mut meshes: ResMut<Assets<Mesh>>,
	camera_query: Query<(&Transform, &Projection), With<Camera3d>>,
	mut ocean_query: Query<(&mut OceanMesh, &Mesh3d)>,
) {
	let Ok((camera_transform, projection)) = camera_query.single() else {
		return;
	};

	let proj_matrix = match projection {
		Projection::Perspective(p) => p.get_clip_from_view(),
		Projection::Orthographic(o) => o.get_clip_from_view(),
		Projection::Custom(c) => c.get_clip_from_view(),
	};

	for (mut ocean_mesh, mesh_handle) in ocean_query.iter_mut() {
		// Check if camera moved enough to warrant update
		let position_delta = (camera_transform.translation - ocean_mesh.last_camera_position).length();
		let rotation_delta = camera_transform.rotation.angle_between(ocean_mesh.last_camera_rotation);

		if position_delta < config.update_threshold && rotation_delta < 0.01 {
			continue;
		}

		// Update the mesh
		let mesh_config = OceanMeshConfig {
			resolution: config.resolution,
			max_distance: config.max_distance,
			camera_transform: *camera_transform,
			projection: proj_matrix,
			ocean_height: config.ocean_height,
		};

		let new_mesh = create_projected_grid_mesh(&mesh_config);

		if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
			*mesh = new_mesh;
		}

		ocean_mesh.last_camera_position = camera_transform.translation;
		ocean_mesh.last_camera_rotation = camera_transform.rotation;
	}
}
