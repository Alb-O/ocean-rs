# Agent Notes

## Asset Loading

Assets are loaded from the workspace root `assets/` directory. Examples must call `set_workspace_asset_root()` before creating the App to set `BEVY_ASSET_ROOT`.

```rust
use bevy_screenshot_harness::set_workspace_asset_root;

fn main() {
    set_workspace_asset_root();
    let mut app = App::new();
    // ...
}
```

## GLTF Models

Load GLTF scenes using `GltfAssetLabel`:

```rust
use bevy::gltf::GltfAssetLabel;
use bevy::scene::SceneRoot;

let scene = asset_server.load(
    GltfAssetLabel::Scene(0).from_asset("models/example/model.gltf")
);
commands.spawn(SceneRoot(scene));
```

## Embedded Shaders

Shaders can be embedded at compile time via `embedded_asset!`.
