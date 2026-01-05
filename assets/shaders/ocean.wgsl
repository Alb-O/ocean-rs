// Ocean surface shader with Gerstner wave displacement and Fresnel shading.
//
// This shader displaces vertices using Gerstner waves in the vertex stage
// and applies Fresnel-based reflection/refraction blending in the fragment stage.

#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
}
#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;

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
    // x: F0 (base reflectance), y: power, z: bias, w: unused
    fresnel_params: vec4<f32>,
    sky_color: vec4<f32>,
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
    let f0 = ocean.fresnel_params.x;
    let power = ocean.fresnel_params.y;
    let bias = ocean.fresnel_params.z;
    let fresnel = fresnel_schlick(n_dot_v, f0, power, bias);
    
    // Basic diffuse lighting
    let n_dot_l = max(dot(n, light_dir), 0.0);
    
    // Blend between deep and shallow based on view angle
    // Water appears darker (deep) when viewed from above, lighter (shallow) at grazing angles
    let water_color = mix(ocean.deep_color.rgb, ocean.shallow_color.rgb, 1.0 - n_dot_v);
    
    // Apply lighting to water color
    let ambient = 0.3;
    let lit_water = water_color * (ambient + (1.0 - ambient) * n_dot_l);
    
    // Reflection direction for specular highlight
    let reflect_dir = reflect(-view_dir, n);
    let spec = pow(max(dot(reflect_dir, light_dir), 0.0), 64.0);
    
    // Sky reflection (placeholder solid color until environment maps)
    let sky_reflection = ocean.sky_color.rgb + spec * 0.5;
    
    // Blend water color and reflection based on Fresnel term
    // Higher Fresnel (grazing angles) = more reflection
    // Lower Fresnel (looking down) = more water color
    let final_color = mix(lit_water, sky_reflection, fresnel);
    
    return vec4<f32>(final_color, 1.0);
}
