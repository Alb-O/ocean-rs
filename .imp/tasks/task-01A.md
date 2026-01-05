# Ocean-RS Task 01A: Core Ocean Rendering Foundation

## Model Directive

This specification defines the first milestone for ocean-rs: a realtime realistic ocean surface in Bevy. The goal is to establish the foundational rendering pipeline with projected-grid geometry, Gerstner wave displacement, proper normal calculation, Fresnel shading, and environment map reflections.

This task produces 5 progressive examples, each verified via headless screenshot capture using the existing `examples/common` harness.

---

## Implementation Expectations

<mandatory_execution_requirements>

This is not a review task. When given implementation requests:

1. Edit files using tools to modify actual source files
2. Debug and fix by running builds, reading errors, iterating until it compiles
3. Run each example to generate screenshots: `cargo run --example <name>`
4. Complete the full implementation; do not stop at partial solutions
5. After completing each example, run it and verify screenshots are generated

Verification command (run after each example is complete):
```bash
cargo clippy --all-targets -- -D warnings && cargo run --example <example_name>
```

Unacceptable responses:
- Providing code blocks without writing them to files
- Stopping after the first compilation error
- Skipping screenshot verification

</mandatory_execution_requirements>

---

## Behavioral Constraints

<verbosity_and_scope_constraints>

- Build incrementally: each example extends the previous
- Use existing `examples/common/mod.rs` harness for screenshots
- Follow Rust 2024 edition idioms and Bevy main branch APIs
- Shader code in WGSL, embedded via `include_str!` or asset loading
- Target desktop (Vulkan/Metal/DX12) only; no WebGL constraints
- Do NOT implement tessellation, foam, SSS, or object interaction in this task

</verbosity_and_scope_constraints>

<design_freedom>

- Ocean material can be a custom `Material` impl or extended `StandardMaterial`
- Projected grid algorithm choice is flexible (screen-space projection, radial rings, etc.)
- Wave parameters should be configurable via `Resource` for runtime tweaking
- New modules/files are fine when they improve organization

</design_freedom>

---

## Architecture

```
main/
├── crates/
│   ├── core/src/
│   │   ├── lib.rs
│   │   ├── ocean/
│   │   │   ├── mod.rs           # Ocean plugin, resources
│   │   │   ├── mesh.rs          # Projected grid mesh generation
│   │   │   ├── waves.rs         # Gerstner wave math
│   │   │   └── material.rs      # Custom ocean material
│   │   └── shaders/
│   │       ├── ocean.wgsl       # Vertex + fragment shader
│   │       └── common.wgsl      # Shared math (gerstner, fresnel)
│   └── visualizations/
│       └── examples/
│           ├── common/mod.rs    # Existing harness (do not modify core logic)
│           ├── 01_flat_ocean.rs
│           ├── 02_gerstner_single.rs
│           ├── 03_gerstner_multi.rs
│           ├── 04_fresnel.rs
│           └── 05_skybox_reflect.rs
└── assets/
    ├── environment_maps/        # HDR cubemaps (already present)
    └── textures/wave_normals.png
```

---

## Implementation Roadmap

### Phase 1: Projected Grid Mesh

**Objective**: Create an "infinite" ocean plane that follows the camera using a projected grid technique.

**Background**: A projected grid creates vertices in screen-space, then projects them onto the ocean plane (y=0). This gives uniform screen-space vertex density regardless of view distance, creating the illusion of an infinite ocean.

**Tasks**:

- [x] 1.1 Create `crates/core/src/ocean/mod.rs` - Ocean plugin that registers systems and resources
- [x] 1.2 Create `crates/core/src/ocean/mesh.rs` - Projected grid mesh generator
  - Generate a grid of vertices in clip-space (-1 to 1)
  - Project each vertex ray from camera onto y=0 plane
  - Handle horizon edge cases (rays parallel to plane)
  - Mesh should be ~128x128 or configurable resolution
  - Regenerate mesh each frame based on camera transform
- [x] 1.3 Create `examples/01_flat_ocean.rs`
  - Spawn ocean with basic blue `StandardMaterial`
  - Camera at height 10-20 looking at horizon
  - Verify: flat blue plane extending to horizon

**Code sketch for projected grid**:
```rust
// For each grid vertex (u, v) in [0,1]x[0,1]:
// 1. Map to clip space: clip_pos = (u * 2 - 1, v * 2 - 1, 0.99, 1)
// 2. Unproject to world: world_pos = inv_view_proj * clip_pos
// 3. Ray from camera origin through world_pos
// 4. Intersect ray with y=0 plane
// 5. Clamp to max distance for horizon
```

**Acceptance**: Screenshot shows blue ocean plane extending to horizon from elevated camera.

---

### Phase 2: Single Gerstner Wave

**Objective**: Implement vertex displacement using a single Gerstner (trochoidal) wave.

**Background**: Gerstner waves create realistic circular orbital motion. The formula displaces vertices both vertically (Y) and horizontally (XZ), creating the characteristic peaked crests and rounded troughs.

**Tasks**:

- [x] 2.1 Create `crates/core/src/ocean/waves.rs` - Gerstner wave math (CPU reference)
  ```rust
  pub struct GerstnerWave {
      pub direction: Vec2,    // Normalized wave direction
      pub steepness: f32,     // Q parameter (0-1), affects sharpness
      pub wavelength: f32,    // Distance between crests
      pub amplitude: f32,     // Wave height / 2
      pub speed: f32,         // Phase speed
  }
  
  impl GerstnerWave {
      /// Returns (position_offset, normal) for a point at world_xz and time
      pub fn evaluate(&self, world_xz: Vec2, time: f32) -> (Vec3, Vec3);
  }
  ```
- [x] 2.2 Create `crates/core/src/shaders/ocean.wgsl` with Gerstner in vertex shader
- [x] 2.3 Create `crates/core/src/ocean/material.rs` - Custom `OceanMaterial` impl
  - Bind wave parameters as uniforms
  - Bind time uniform
  - Pass world position to vertex shader for displacement
- [x] 2.4 Create `examples/02_gerstner_single.rs`
  - Single wave: wavelength=60, amplitude=2, steepness=0.5
  - Animate time via `Res<Time>`
  - Basic shading to see wave shape

**Gerstner formula reference**:
```
k = 2π / wavelength
ω = sqrt(g * k)  // deep water dispersion
phase = k * dot(D, P.xz) - ω * t

// Displacement:
x_offset = Q * A * D.x * cos(phase)
z_offset = Q * A * D.y * cos(phase)
y_offset = A * sin(phase)

// Where Q = steepness / (k * A * num_waves)
```

**Acceptance**: Screenshot shows single wave with visible crests and troughs, animated motion.

---

### Phase 3: Multiple Overlapping Gerstner Waves

**Objective**: Layer 3+ Gerstner waves for realistic chaotic ocean surface.

**Background**: Real ocean surfaces result from many superimposed waves. Using 3 waves with different directions, wavelengths, and amplitudes creates convincing complexity without excessive GPU cost.

**Tasks**:

- [x] 3.1 Create `OceanConfig` resource with wave array
  ```rust
  #[derive(Resource)]
  pub struct OceanConfig {
      pub waves: [GerstnerWave; 4], // Support up to 4 waves
      pub active_wave_count: u32,
  }
  
  impl Default for OceanConfig {
      fn default() -> Self {
          Self {
              waves: [
                  GerstnerWave { direction: Vec2::new(1.0, 0.0), wavelength: 60.0, amplitude: 2.0, steepness: 0.5, speed: 1.0 },
                  GerstnerWave { direction: Vec2::new(0.7, 0.7), wavelength: 31.0, amplitude: 1.0, steepness: 0.6, speed: 1.2 },
                  GerstnerWave { direction: Vec2::new(-0.3, 0.9), wavelength: 18.0, amplitude: 0.5, steepness: 0.4, speed: 0.9 },
                  GerstnerWave { direction: Vec2::ZERO, wavelength: 1.0, amplitude: 0.0, steepness: 0.0, speed: 0.0 }, // unused
                  ],
              active_wave_count: 3,
          }
      }
  }
  ```
- [x] 3.2 Update shader to sum multiple waves in vertex shader
- [x] 3.3 Calculate analytical normals from wave derivatives
  ```
  // Binormal and tangent from wave derivatives:
  B = (1 - Σ(Q*k*Dx*Dx*sin), Σ(Q*k*Dx*cos), -Σ(Q*k*Dx*Dz*sin))
  T = (-Σ(Q*k*Dx*Dz*sin), Σ(Q*k*Dz*cos), 1 - Σ(Q*k*Dz*Dz*sin))
  N = normalize(cross(B, T))
  ```
- [x] 3.4 Create `examples/03_gerstner_multi.rs`
  - 3 overlapping waves
  - Better shading using calculated normals
  - Multiple camera angles to show wave complexity

**Acceptance**: Screenshots show complex, realistic wave patterns with proper lighting.

---

### Phase 4: Fresnel Reflection/Transmission

**Objective**: Implement Fresnel effect for view-angle-dependent reflection vs water color.

**Background**: Water reflects more at grazing angles (near horizon) and shows more of its own color when viewed from above. The Fresnel term blends between reflection and base water color.

**Tasks**:

- [x] 4.1 Add Fresnel calculation to fragment shader
  ```wgsl
  fn fresnel_schlick(cos_theta: f32, F0: f32) -> f32 {
      return F0 + (1.0 - F0) * pow(1.0 - cos_theta, 5.0);
  }
  // F0 for water ≈ 0.02
  ```
- [x] 4.2 Add water color parameters to material
  ```rust
  pub struct OceanMaterial {
      pub deep_color: Color,      // Dark blue-green for steep viewing
      pub shallow_color: Color,   // Lighter teal for shallow viewing
      pub fresnel_power: f32,     // Controls blend sharpness
      pub fresnel_bias: f32,      // Minimum reflection
  }
  ```
- [x] 4.3 Blend between water color and sky reflection placeholder (solid color for now)
- [x] 4.4 Create `examples/04_fresnel.rs`
  - Show Fresnel effect with contrasting deep/shallow colors
  - Camera angles that demonstrate the effect (high and low views)

**Acceptance**: Screenshots show darker water color when looking down, lighter/reflective near horizon.

---

### Phase 5: Environment Map Reflections

**Objective**: Add HDR environment map for realistic sky reflections on the ocean surface.

**Background**: The environment maps in `assets/environment_maps/` are KTX2 cubemaps. Bevy's `EnvironmentMapLight` provides IBL, but we need to sample the cubemap directly in our shader for accurate water reflections.

**Tasks**:

- [ ] 5.1 Load environment cubemap in ocean material
  - Use `table_mountain_2_puresky_4k_cubemap.ktx2` or `kloppenheim_01_puresky_4k_cubemap.ktx2`
- [ ] 5.2 Sample cubemap in fragment shader using reflected view vector
  ```wgsl
  let view_dir = normalize(camera_pos - world_pos);
  let reflect_dir = reflect(-view_dir, normal);
  let env_color = textureSample(env_cubemap, env_sampler, reflect_dir).rgb;
  ```
- [ ] 5.3 Blend reflection with water color using Fresnel term
- [ ] 5.4 Add `EnvironmentMapLight` for consistent scene IBL
- [ ] 5.5 Create `examples/05_skybox_reflect.rs`
  - Full integration: projected grid + 3 waves + Fresnel + env reflections
  - Spawn the dutch ship model floating on water (static, no physics)
  - Multiple dramatic camera angles

**Acceptance**: Screenshots show realistic ocean with sky reflections, visible ship for scale.

---

## Example Registration

Update `crates/visualizations/Cargo.toml`:
```toml
[[example]]
name = "01_flat_ocean"

[[example]]
name = "02_gerstner_single"

[[example]]
name = "03_gerstner_multi"

[[example]]
name = "04_fresnel"

[[example]]
name = "05_skybox_reflect"
```

Add dependency on `ocean-core` crate.

---

## Anti-Patterns

1. **Hardcoded magic numbers**: All wave parameters, colors, and distances should be in `OceanConfig` or material -> Use configurable resources
2. **Mesh recreation every frame**: Only regenerate projected grid when camera moves significantly -> Cache mesh, update on threshold
3. **CPU wave calculation for rendering**: Waves must be calculated in shader -> CPU code is for reference/physics queries only
4. **Ignoring horizon edge cases**: Projected grid rays may miss y=0 plane -> Clamp to max distance, handle parallel rays

---

## Verification Checklist

After completing all phases, run:
```bash
cargo clippy --all-targets -- -D warnings
for ex in 01_flat_ocean 02_gerstner_single 03_gerstner_multi 04_fresnel 05_skybox_reflect; do
  cargo run --example $ex
done
```

Each example should:
1. Compile without warnings
2. Generate screenshots in `examples/<name>/screenshots/<timestamp>/`
3. Exit cleanly after screenshot capture


