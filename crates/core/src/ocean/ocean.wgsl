// Ocean surface shader with Gerstner wave displacement and Fresnel shading.
//
// This shader displaces vertices using Gerstner waves in the vertex stage
// and applies Fresnel-based reflection/refraction blending in the fragment stage.
// Supports optional environment cubemap for realistic sky reflections.

#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
}
#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;

// Individual uniform bindings for each field
@group(2) @binding(0) var<uniform> wave0_direction: vec4<f32>;
@group(2) @binding(1) var<uniform> wave0_params: vec4<f32>;
@group(2) @binding(2) var<uniform> wave1_direction: vec4<f32>;
@group(2) @binding(3) var<uniform> wave1_params: vec4<f32>;
@group(2) @binding(4) var<uniform> wave2_direction: vec4<f32>;
@group(2) @binding(5) var<uniform> wave2_params: vec4<f32>;
@group(2) @binding(6) var<uniform> wave3_direction: vec4<f32>;
@group(2) @binding(7) var<uniform> wave3_params: vec4<f32>;
@group(2) @binding(8) var<uniform> time_and_config: vec4<f32>;
@group(2) @binding(9) var<uniform> deep_color: vec4<f32>;
@group(2) @binding(10) var<uniform> shallow_color: vec4<f32>;
@group(2) @binding(11) var<uniform> fresnel_params: vec4<f32>;
@group(2) @binding(12) var<uniform> sky_color: vec4<f32>;
@group(2) @binding(13) var env_cubemap: texture_cube<f32>;
@group(2) @binding(14) var env_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

const PI: f32 = 3.14159265359;
const GRAVITY: f32 = 9.81;

// Calculate wave number k = 2pi / wavelength
fn wave_number(wavelength: f32) -> f32 {
    return 2.0 * PI / wavelength;
}

// Calculate angular frequency omega = sqrt(g * k) * speed
fn angular_frequency(k: f32, speed: f32) -> f32 {
    return sqrt(GRAVITY * k) * speed;
}

// Evaluate a single Gerstner wave position offset
fn evaluate_gerstner_position(
    direction: vec2<f32>,
    steepness: f32,
    wavelength: f32,
    amplitude: f32,
    speed: f32,
    world_xz: vec2<f32>,
    time: f32
) -> vec3<f32> {
    let k = wave_number(wavelength);
    let omega = angular_frequency(k, speed);
    
    let phase = k * dot(direction, world_xz) - omega * time;
    let cos_phase = cos(phase);
    let sin_phase = sin(phase);
    
    return vec3<f32>(
        steepness * amplitude * direction.x * cos_phase,
        amplitude * sin_phase,
        steepness * amplitude * direction.y * cos_phase
    );
}

// Evaluate Gerstner wave tangent frame contribution
// Returns binormal and tangent modifications
fn evaluate_gerstner_tangent_frame(
    direction: vec2<f32>,
    steepness: f32,
    wavelength: f32,
    amplitude: f32,
    speed: f32,
    world_xz: vec2<f32>,
    time: f32
) -> mat2x3<f32> {
    let k = wave_number(wavelength);
    let omega = angular_frequency(k, speed);
    
    let phase = k * dot(direction, world_xz) - omega * time;
    let cos_phase = cos(phase);
    let sin_phase = sin(phase);
    
    let wa = k * amplitude;
    
    // Binormal modification
    let b = vec3<f32>(
        -steepness * direction.x * direction.x * wa * sin_phase,
        direction.x * wa * cos_phase,
        -steepness * direction.x * direction.y * wa * sin_phase
    );
    
    // Tangent modification
    let t = vec3<f32>(
        -steepness * direction.x * direction.y * wa * sin_phase,
        direction.y * wa * cos_phase,
        -steepness * direction.y * direction.y * wa * sin_phase
    );
    
    return mat2x3<f32>(b, t);
}

// Helper to process one wave
fn process_wave(
    direction: vec4<f32>,
    params: vec4<f32>,
    base_xz: vec2<f32>,
    time: f32,
    offset: ptr<function, vec3<f32>>,
    binormal: ptr<function, vec3<f32>>,
    tangent: ptr<function, vec3<f32>>
) {
    let d = direction.xy;
    let steepness = params.x;
    let wavelength = params.y;
    let amplitude = params.z;
    let speed = params.w;
    
    // Skip waves with zero amplitude
    if amplitude > 0.001 {
        *offset = *offset + evaluate_gerstner_position(d, steepness, wavelength, amplitude, speed, base_xz, time);
        let frame_mod = evaluate_gerstner_tangent_frame(d, steepness, wavelength, amplitude, speed, base_xz, time);
        *binormal = *binormal + frame_mod[0];
        *tangent = *tangent + frame_mod[1];
    }
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    let time = time_and_config.x;
    let wave_count = u32(time_and_config.y);
    
    // Start with base position
    var world_pos = vertex.position;
    let base_xz = vec2<f32>(world_pos.x, world_pos.z);
    
    // Accumulate wave displacements
    var total_offset = vec3<f32>(0.0, 0.0, 0.0);
    var binormal = vec3<f32>(1.0, 0.0, 0.0);
    var tangent = vec3<f32>(0.0, 0.0, 1.0);
    
    // Process each wave (unrolled since we can't use arrays)
    if wave_count >= 1u {
        process_wave(wave0_direction, wave0_params, base_xz, time, &total_offset, &binormal, &tangent);
    }
    if wave_count >= 2u {
        process_wave(wave1_direction, wave1_params, base_xz, time, &total_offset, &binormal, &tangent);
    }
    if wave_count >= 3u {
        process_wave(wave2_direction, wave2_params, base_xz, time, &total_offset, &binormal, &tangent);
    }
    if wave_count >= 4u {
        process_wave(wave3_direction, wave3_params, base_xz, time, &total_offset, &binormal, &tangent);
    }
    
    // Apply displacement
    world_pos = world_pos + total_offset;
    
    // Compute normal from tangent frame
    let normal = normalize(cross(binormal, tangent));
    
    out.clip_position = position_world_to_clip(world_pos);
    out.world_position = world_pos;
    out.world_normal = normal;
    out.uv = vertex.uv;
    
    return out;
}

// Schlick's Fresnel approximation
// F = F0 + (1 - F0) * (1 - cos_theta)^power
fn fresnel_schlick(cos_theta: f32, f0: f32, power: f32, bias: f32) -> f32 {
    let base = 1.0 - saturate(cos_theta);
    return clamp(f0 + (1.0 - f0) * pow(base, power) + bias, 0.0, 1.0);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let n = normalize(in.world_normal);
    
    // Calculate view direction from camera position
    let camera_pos = view.world_position;
    let view_dir = normalize(camera_pos - in.world_position);
    
    // Fresnel calculation - how much reflection vs water color
    let n_dot_v = max(dot(n, view_dir), 0.0);
    let f0 = fresnel_params.x;
    let power = fresnel_params.y;
    let bias = fresnel_params.z;
    let fresnel = fresnel_schlick(n_dot_v, f0, power, bias);
    
    // Basic diffuse lighting
    let n_dot_l = max(dot(n, light_dir), 0.0);
    
    // Blend between deep and shallow based on view angle
    // Water appears darker (deep) when viewed from above, lighter (shallow) at grazing angles
    let water_color = mix(deep_color.rgb, shallow_color.rgb, 1.0 - n_dot_v);
    
    // Apply lighting to water color
    let ambient = 0.3;
    let lit_water = water_color * (ambient + (1.0 - ambient) * n_dot_l);
    
    // Reflection direction for environment sampling
    let reflect_dir = reflect(-view_dir, n);
    
    // Check if we should use environment map
    let use_env_map = time_and_config.z > 0.5;
    
    var sky_reflection: vec3<f32>;
    if use_env_map {
        // Sample environment cubemap using reflection direction
        let env_color = textureSample(env_cubemap, env_sampler, reflect_dir).rgb;
        // Add subtle specular highlight on top
        let spec = pow(max(dot(reflect_dir, light_dir), 0.0), 64.0);
        sky_reflection = env_color + spec * 0.3;
    } else {
        // Fallback to solid sky color with specular
        let spec = pow(max(dot(reflect_dir, light_dir), 0.0), 64.0);
        sky_reflection = sky_color.rgb + spec * 0.5;
    }
    
    // Blend water color and reflection based on Fresnel term
    // Higher Fresnel (grazing angles) = more reflection
    // Lower Fresnel (looking down) = more water color
    let final_color = mix(lit_water, sky_reflection, fresnel);
    
    return vec4<f32>(final_color, 1.0);
}
