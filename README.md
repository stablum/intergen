# intergen

Interactive 3D polyhedron generation tool built with Rust and Bevy.

Runtime tuning lives in `config.toml` at the repository root.

The current prototype focuses on a fast local development loop and a usable vertical slice:
- inertial camera rotation on all 3 axes with preserved angular momentum
- keyboard zoom
- recursive polyhedron spawning from parent vertices
- selectable child shape type
- adjustable child scale ratio
- toggleable in-app keybinding overlay
- built-in screenshot capture for manual and scripted verification
- containment rejection so obviously hidden fully-inside spawns are skipped
- camera-output shader stack with hard-wrap wavefolder, gaussian blur, and edge detection

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

## Configuration

The app loads `config.toml` from the repository root on startup.

Current configuration sections:
- `window`: title, resolution, and present mode
- `rendering`: clear color and ambient light
- `camera`: initial orbit, motion tuning, and angular-momentum preservation
- `generation`: root shape, default child shape, scale limits, twist defaults and bounds, spawn cadence, and spawn heuristics
- `lighting`: directional and point light colors, positions, and intensities
- `effects`: camera-output shader effects
- `materials`: color progression, PBR tuning, and live opacity defaults
- `capture`: screenshot output directory and default capture delay
- `ui`: font candidates plus overlay sizing and colors

If `config.toml` is missing, the app falls back to the same built-in defaults.

Live twist controls use these `generation` settings:
- `twist_per_vertex_radians`: startup default for the child twist angle
- `twist_adjust_step`: per-keypress twist change
- `twist_hold_delay_secs`: how long to hold before twist repeat starts
- `twist_repeat_interval_secs`: time between repeated twist updates while held
- `min_twist_per_vertex_radians` / `max_twist_per_vertex_radians`: live clamp range, with `0.0` as the minimum allowed floor

Camera-output effects run in this order:
- `effects.color_wavefolder`: hard-wrap the camera color by amplification plus remainder
- `effects.gaussian_blur`: blur the already-wavefolded image
- `effects.edge_detection`: detect edges from the processed image and mix a configurable edge color over it

Camera-output color wavefolder uses these `effects.color_wavefolder` settings:
- `enabled`: turns the hard-wrap post-process on or off
- `gain`: amplifies the color before wrapping
- `modulus`: the divisor whose remainder is kept after amplification

Camera-output gaussian blur uses these `effects.gaussian_blur` settings:
- `enabled`: turns blur on or off
- `sigma`: controls the gaussian falloff
- `radius_pixels`: blur radius in pixels, clamped to `16` in the current single-pass shader

Camera-output edge detection uses these `effects.edge_detection` settings:
- `enabled`: turns the edge pass on or off
- `strength`: scales edge magnitude before thresholding
- `threshold`: subtracts a floor from the detected edge magnitude
- `mix`: blends the edge color over the processed image
- `color`: RGB edge overlay color

Live opacity controls use these `materials` settings:
- `default_opacity`: startup default for all object materials
- `opacity_adjust_step`: per-keypress opacity change
- `min_opacity` / `max_opacity`: live clamp range

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
- `[` or `,`: decrease child twist angle, or hold to keep decreasing
- `]` or `.`: increase child twist angle, or hold to keep increasing
- `O`: decrease global object opacity
- `P`: increase global object opacity
- `I`: reset global object opacity to the configured default
- `T`: reset the child twist angle to the configured default

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
- camera-output hard-wrap wavefolder, gaussian blur, and edge-detection post process
- unit tests for geometry counts and spawn ordering

Not implemented yet:
- mouse controls
- hardware ray tracing
- more advanced visibility heuristics than simple containment rejection
- export / save workflows