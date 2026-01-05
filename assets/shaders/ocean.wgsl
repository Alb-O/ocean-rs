// Ocean surface shader with Gerstner wave displacement.
//
// This shader displaces vertices using Gerstner waves in the vertex stage
// and applies basic shading in the fragment stage.

#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
}

// Gerstner wave parameters - up to 4 waves
struct GerstnerWave {
    // xy: direction (normalized), zw: unused padding
    direction: vec4<f32>,
    // x: steepness, y: wavelength, z: amplitude, w: speed
    params: vec4<f32>,
}

struct OceanUniforms {
    waves: array<GerstnerWave, 4>,
    // x: time, y: active_wave_count, z: unused, w: unused
    time_and_config: vec4<f32>,
    deep_color: vec4<f32>,
    shallow_color: vec4<f32>,
}

@group(2) @binding(0) var<uniform> ocean: OceanUniforms;

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

// Calculate wave number k = 2π / wavelength
fn wave_number(wavelength: f32) -> f32 {
    return 2.0 * PI / wavelength;
}

// Calculate angular frequency ω = sqrt(g * k) * speed
fn angular_frequency(k: f32, speed: f32) -> f32 {
    return sqrt(GRAVITY * k) * speed;
}

// Evaluate a single Gerstner wave
// Returns: xyz = position offset
fn evaluate_gerstner_position(
    wave: GerstnerWave,
    world_xz: vec2<f32>,
    time: f32
) -> vec3<f32> {
    let d = wave.direction.xy;
    let steepness = wave.params.x;
    let wavelength = wave.params.y;
    let amplitude = wave.params.z;
    let speed = wave.params.w;
    
    let k = wave_number(wavelength);
    let omega = angular_frequency(k, speed);
    
    let phase = k * dot(d, world_xz) - omega * time;
    let cos_phase = cos(phase);
    let sin_phase = sin(phase);
    
    let q = steepness;
    let a = amplitude;
    
    return vec3<f32>(
        q * a * d.x * cos_phase,
        a * sin_phase,
        q * a * d.y * cos_phase
    );
}

// Evaluate Gerstner wave normal contribution
// Returns binormal and tangent modifications for this wave
fn evaluate_gerstner_tangent_frame(
    wave: GerstnerWave,
    world_xz: vec2<f32>,
    time: f32
) -> mat2x3<f32> {
    let d = wave.direction.xy;
    let steepness = wave.params.x;
    let wavelength = wave.params.y;
    let amplitude = wave.params.z;
    let speed = wave.params.w;
    
    let k = wave_number(wavelength);
    let omega = angular_frequency(k, speed);
    
    let phase = k * dot(d, world_xz) - omega * time;
    let cos_phase = cos(phase);
    let sin_phase = sin(phase);
    
    let q = steepness;
    let wa = k * amplitude;
    let s = sin_phase;
    let c = cos_phase;
    
    // Binormal modification
    let b = vec3<f32>(
        -q * d.x * d.x * wa * s,
        d.x * wa * c,
        -q * d.x * d.y * wa * s
    );
    
    // Tangent modification
    let t = vec3<f32>(
        -q * d.x * d.y * wa * s,
        d.y * wa * c,
        -q * d.y * d.y * wa * s
    );
    
    return mat2x3<f32>(b, t);
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    let time = ocean.time_and_config.x;
    let wave_count = u32(ocean.time_and_config.y);
    
    // Start with base position
    var world_pos = vertex.position;
    let base_xz = vec2<f32>(world_pos.x, world_pos.z);
    
    // Accumulate wave displacements
    var total_offset = vec3<f32>(0.0, 0.0, 0.0);
    var binormal = vec3<f32>(1.0, 0.0, 0.0);
    var tangent = vec3<f32>(0.0, 0.0, 1.0);
    
    for (var i = 0u; i < wave_count; i = i + 1u) {
        let wave = ocean.waves[i];
        
        // Skip waves with zero amplitude
        if wave.params.z > 0.001 {
            total_offset = total_offset + evaluate_gerstner_position(wave, base_xz, time);
            
            let frame_mod = evaluate_gerstner_tangent_frame(wave, base_xz, time);
            binormal = binormal + frame_mod[0];
            tangent = tangent + frame_mod[1];
        }
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

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple shading based on normal - will be enhanced in later phases
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let n = normalize(in.world_normal);
    
    // Basic diffuse lighting
    let ndotl = max(dot(n, light_dir), 0.0);
    
    // Blend between deep and shallow based on surface angle
    // (simple approximation until Fresnel is implemented)
    let up_factor = max(dot(n, vec3<f32>(0.0, 1.0, 0.0)), 0.0);
    let base_color = mix(ocean.deep_color.rgb, ocean.shallow_color.rgb, up_factor * 0.5);
    
    // Apply lighting
    let ambient = 0.3;
    let lit_color = base_color * (ambient + (1.0 - ambient) * ndotl);
    
    return vec4<f32>(lit_color, 1.0);
}
