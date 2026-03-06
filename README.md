# intergen

Interactive 3D polyhedron generation tool built with Rust and Bevy.

The current prototype focuses on a fast local development loop and a usable vertical slice:
- inertial camera rotation on all 3 axes
- keyboard zoom
- recursive polyhedron spawning from parent vertices
- selectable child shape type
- adjustable child scale ratio
- containment rejection so obviously hidden fully-inside spawns are skipped

## Requirements

- Windows
- Rust stable MSVC toolchain
- Visual Studio 2022 Build Tools with the C++ workload

## Run

```powershell
cargo run
```

The default development build enables Bevy dynamic linking for faster rebuilds.

If you want to run without that mode:

```powershell
cargo run-plain
```

## Test

```powershell
cargo test
```

Without dynamic linking:

```powershell
cargo test-plain
```

## Controls

- `Arrow Up` / `Arrow Down`: pitch camera
- `Arrow Left` / `Arrow Right`: yaw camera
- `Q` / `E`: roll camera
- `W` / `S`: zoom in / out
- `Space`: spawn the next child polyhedron
- `1`: select cube
- `2`: select tetrahedron
- `3`: select octahedron
- `4`: select dodecahedron
- `-`: decrease child scale ratio
- `+`: increase child scale ratio

## Build Notes

- `cargo run` does not always recompile. If nothing relevant changed, Cargo reuses the existing build output.
- Editing Rust source files should usually rebuild only `intergen`.
- Changing `Cargo.toml`, enabled features, toolchain, or profile settings can trigger a larger one-time rebuild.
- `cargo check` is the fastest command for type-checking without launching the app.

## Current Scope

Implemented now:
- custom meshes for cube, tetrahedron, octahedron, and dodecahedron
- recursive level-by-level spawning
- metallic lit PBR scene
- unit tests for geometry counts and spawn ordering

Not implemented yet:
- mouse controls
- hardware ray tracing
- more advanced visibility heuristics than simple containment rejection
- export / save workflows
