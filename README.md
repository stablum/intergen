# intergen

Interactive 3D polyhedron generation tool built with Rust and Bevy.

The current prototype focuses on a fast local development loop and a usable vertical slice:
- inertial camera rotation on all 3 axes
- keyboard zoom
- recursive polyhedron spawning from parent vertices
- selectable child shape type
- adjustable child scale ratio
- toggleable in-app keybinding overlay
- built-in screenshot capture for manual and scripted verification
- containment rejection so obviously hidden fully-inside spawns are skipped

## Requirements

- Windows
- Rust stable MSVC toolchain
- Visual Studio 2022 Build Tools with the C++ workload

## Font

UI text prefers `Carbon Plus` if you place a licensed font file in `assets/fonts/`.

Supported filenames:
- `assets/fonts/CarbonPlus-Regular.ttf`
- `assets/fonts/CarbonPlus-Regular.otf`
- `assets/fonts/Carbon Plus Regular.ttf`
- `assets/fonts/Carbon Plus Regular.otf`
- `assets/fonts/CarbonPlus.ttf`
- `assets/fonts/Carbon Plus.ttf`

If none of those files are present, the app falls back to Bevy's default font.

## Run

```powershell
cargo run
```

The default development build enables Bevy dynamic linking for faster rebuilds.

If you want to run without that mode:

```powershell
cargo run-plain
```

To save a verification screenshot and exit automatically:

```powershell
cargo run -- --capture screenshots\check.png --capture-delay-frames 120
```

During a normal interactive run, press `F12` to save a screenshot under `screenshots/`.

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
- `F1` or `H`: toggle the keybinding overlay
- `F12`: save a screenshot to `screenshots/`
- `R`: reset the scene to the root polyhedron
- `Space`: spawn child polyhedra, or hold to keep spawning
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

