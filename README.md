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
- compact runtime FX tuner for all numeric effect parameters
- built-in screenshot capture for manual and scripted verification
- containment rejection so obviously hidden fully-inside spawns are skipped
- camera-output shader stack with hard-wrap wavefolder, lens distortion, gaussian blur, bloom, and edge detection

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

Live child-offset controls use these `generation` settings:
- `default_vertex_offset_ratio`: startup default for the center offset from a parent vertex, measured in child-radius units
- `vertex_offset_adjust_step`: per-keypress offset change
- `vertex_offset_hold_delay_secs`: how long to hold before offset repeat starts
- `vertex_offset_repeat_interval_secs`: time between repeated offset updates while held
- `min_vertex_offset_ratio` / `max_vertex_offset_ratio`: live clamp range, with `0.0` as the minimum allowed floor

Camera-output effects run in this order:
- `effects.lens_distortion`: warp the camera image with radial, tangential, and chromatic lens terms
- `effects.color_wavefolder`: hard-wrap the distorted camera color by amplification plus remainder
- `effects.gaussian_blur`: blur the distorted and wavefolded image
- `effects.bloom`: add a bright-pass glow over the processed image
- `effects.edge_detection`: detect edges from the distorted and wavefolded image and mix a configurable edge color over it

Camera-output color wavefolder uses these `effects.color_wavefolder` settings:
- `enabled`: turns the hard-wrap post-process on or off
- `gain`: amplifies the color before wrapping
- `modulus`: the divisor whose remainder is kept after amplification

Camera-output lens distortion uses these `effects.lens_distortion` settings:
- `enabled`: turns lens warping on or off
- `strength`: primary radial barrel/pincushion term (`k1`)
- `radial_k2` / `radial_k3`: higher-order radial shaping terms for the shoulder of the warp
- `center`: distortion center in normalized screen coordinates
- `scale`: per-axis distortion scale for anamorphic or elliptical warping
- `tangential`: tangential skew terms that decenter the lens model
- `zoom`: scales the distorted image to keep more or less of the warped frame in view
- `chromatic_aberration`: shifts color channels apart along the distortion field

Camera-output gaussian blur uses these `effects.gaussian_blur` settings:
- `enabled`: turns blur on or off
- `sigma`: controls the gaussian falloff
- `radius_pixels`: blur radius in pixels, clamped to `16` in the current single-pass shader

Camera-output bloom uses these `effects.bloom` settings:
- `enabled`: turns bright-pass bloom on or off
- `threshold`: minimum brightness that contributes to the glow
- `intensity`: bloom contribution added back onto the processed image
- `radius_pixels`: bloom blur radius in pixels, clamped to `16` in the current single-pass shader

Camera-output edge detection uses these `effects.edge_detection` settings:
- `enabled`: turns the edge pass on or off
- `strength`: scales edge magnitude before thresholding
- `threshold`: subtracts a floor from the detected edge magnitude
- `mix`: blends the edge color over the processed image
- `color`: RGB edge overlay color

The in-app FX tuner starts from the values loaded from `config.toml` at launch.
- Live edits affect the running app only.
- `Enter` resets the selected effect parameter to its startup config value.
- `Shift + Enter` resets all numeric effect parameters to their startup config values.
- The tuner does not write changes back to `config.toml` automatically.

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
- `Backspace`: stop camera rotation momentum
- `F1` or `H`: toggle the keybinding overlay
- `F2`: pin or unpin the FX tuner overlay
- `Ctrl + Up` / `Ctrl + Down`: select the active FX parameter
- `Ctrl + Left` / `Ctrl + Right`: decrease or increase the active FX parameter, with hold-to-repeat
- `Shift`: coarse FX adjustment modifier
- `Alt`: fine FX adjustment modifier
- `Enter`: reset the active FX parameter to the startup config value
- `Shift + Enter`: reset all FX numeric parameters to their startup config values
- `F12`: save a screenshot to `screenshots/`
- `R`: reset the scene with the currently selected polyhedron as the new root
- `Space`: spawn child polyhedra, or hold to keep spawning
- `1`: select cube
- `2`: select tetrahedron
- `3`: select octahedron
- `4`: select dodecahedron
- `-`: decrease child scale ratio
- `+`: increase child scale ratio
- `[` or `,`: decrease child twist angle, or hold to keep decreasing
- `]` or `.`: increase child twist angle, or hold to keep increasing
- `Z`: decrease the child vertex offset, or hold to keep decreasing
- `X`: increase the child vertex offset, or hold to keep increasing
- `C`: reset the child vertex offset to the configured default
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
- camera-output hard-wrap wavefolder, lens distortion, gaussian blur, bloom, and edge-detection post process
- unit tests for geometry counts and spawn ordering

Not implemented yet:
- mouse controls
- hardware ray tracing
- more advanced visibility heuristics than simple containment rejection
- export / save workflows
