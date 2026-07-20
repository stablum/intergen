# Intergen user guide

This guide covers installing, running, and using Intergen. Keep the
[keybinding reference](KEYBINDINGS.md) nearby for the complete control list;
the [spawn-geometry guide](SPAWN_GEOMETRY_GUIDE.md) provides an illustrated
deep dive into how generated shapes are placed and transformed.

## Requirements

- Windows
- the stable Rust MSVC toolchain
- Visual Studio 2022 Build Tools with the C++ workload
- Blender 5.x in `PATH` only if Blender export is needed

## Run Intergen

From the repository root:

```powershell
cargo run
```

The default development build enables Bevy dynamic linking for faster rebuilds.
To run without dynamic linking:

```powershell
cargo run-plain
```

Intergen loads `config.toml` from the repository root. If the file is missing,
the application uses equivalent built-in defaults.

## First session

1. Press `F1` for the in-app quick reference; press it again for a hoverable
   keyboard map.
2. Use the arrow keys and `Q` / `E` to rotate the camera, and `W` / `S` to zoom.
3. Select a child shape with `1`–`4`, then press or hold `Space` to grow the
   structure.
4. Press `G` to try vertex, edge, and face placement. Use `Ctrl + Space` to
   switch between adding one object and filling the current level. Press `D`
   to show or hide the current spawn parent and latest child as unoccluded red
   wireframes.
5. Press `F2` to browse and tune scene, geometry, stage, material, lighting,
   camera, and shader parameters.
6. Press `F3` to save or recall a scene, `F4` to export it to Blender, and `F12`
   to capture a screenshot.

See [all F-modes together](KEYBINDINGS.md#f-modes-at-a-glance) and the
[complete neutral-mode controls](KEYBINDINGS.md#neutral-mode-controls) for
details.

## Live controls and LFOs

The `F2` pages start from values loaded from `config.toml`. Live edits affect
only the running application; they do not write back to the configuration file.

The controls include shader effects and numeric or enum-like scene parameters,
such as child shape, placement and add modes, stage toggles, procedural surface
families, lighting, camera, and materials. Supported numeric parameters expose
LFO amplitude, frequency, and shape fields. Available shapes are sine,
triangle, saw, square, stepped random, and brownian motion.

Generation LFOs are sampled when new geometry is created. In
fill-current-level mode, every successfully spawned child advances the sample
time by `generation.fill_mode_lfo_virtual_time_step_secs`, allowing one batch
to contain varied children without waiting for real time to pass.

The [F2 reference](KEYBINDINGS.md#f2-live-controls) lists navigation, exact
entry, modifiers, LFO controls, and reset behavior in one table. Press `F5` to
monitor explicit interactive edits without listing continuous LFO samples.

## Scene presets

Press `F3` to open the preset page. Its 10 banks and 10 slots provide addresses
from `00` through `99`. The [F3 reference](KEYBINDINGS.md#f3-scene-presets)
collects every preset-page binding.

Preset files are TOML documents stored under `scene-presets/`. Their filenames
are unique and independent of bank and slot, so saving does not overwrite an
older file by filename. The slot assignment lives in the preset metadata. If
multiple files claim one slot, Intergen asks which file should keep it.

Typing a slot directly loads the whole preset. Prefix the two-digit slot with
`O` to load only its 3D object tree, `E` to load only its 2D post-scene effects,
or `P` to load its non-effect scene parameters while keeping both the current
object tree and post effects. The active operation and typed slot are highlighted
in red in the F3 panel.

Partial saves use the same prefixes after `S`: type `S O`, `S E`, or `S P`, then
the two-digit slot. This replaces only that component in the assigned preset and
preserves the others, including the opposite LFO family. A partial save to an
empty slot creates a complete backing preset from the current scene. If several
files claim the slot, choose the file to update in the collision chooser.

A preset contains:

- render background, ambient light, directional light, and point light
- stage visibility and floor/backdrop toggles
- material palette, PBR, surface-family, and base opacity settings
- camera position, distance, and momentum
- generation controls and the complete shape tree
- effect controls and all per-parameter LFO settings

Scene and material LFOs are stored as their base values plus LFO configuration,
not as a single sampled frame.

To load a preset on startup:

```powershell
cargo run -- --load-scene-preset scene-presets\example.toml
```

## Screenshots

Press `F12` during an interactive run to write a timestamped image under
`screenshots/`.

For an automated capture-and-exit run:

```powershell
cargo run -- --capture screenshots\check.png --capture-delay-frames 120
```

## Blender export

Press `F4` during an interactive run to write a timestamped `.blend` under
`blend-exports/`. The export contains:

- the complete shape scene as Blender mesh objects
- the camera, directional and point lights, and world background
- per-object transparency, metallic, roughness, and reflectance-derived specular
- compositor approximations of lens distortion, hard-wrap wavefolder, Gaussian
  blur, bloom, and edge detection
- embedded text datablocks with the Intergen snapshot, evaluated effects, and
  effect/LFO runtime settings

LFOs are preserved as metadata but are not converted into native Blender
animation drivers. Blender's compositor also cannot express every Intergen
lens-distortion term, so the node setup is a best-effort reconstruction while
the full original parameters remain in the embedded metadata.

For an automated export-and-exit run:

```powershell
cargo run -- --export-blend blend-exports\check.blend --export-blend-delay-frames 120
```

## Configuration

`config.toml` is divided into these sections:

| Section | Purpose |
| --- | --- |
| `window` | title, resolution, and present mode |
| `rendering` | clear color, ambient light, floor, and backdrop |
| `camera` | initial orbit, motion tuning, and angular momentum |
| `generation` | shapes, placement, scale, twist, offsets, spawn cadence, and heuristics |
| `lighting` | directional, point, and accent lights |
| `effects` | camera-output shader effects |
| `materials` | palette, PBR tuning, procedural surfaces, and opacity |
| `capture` | screenshot output directory and capture delay |
| `ui` | font candidates, overlay sizing, and colors |

### Generation controls

The main groups of live generation settings are:

- twist: `twist_per_vertex_radians`, its adjustment/hold/repeat values, and its
  minimum/maximum bounds
- outward offset: `default_vertex_offset_ratio`, its adjustment/hold/repeat
  values, and its minimum/maximum bounds
- local position: `default_child_position_offset` and
  `child_position_offset_adjust_step`; components are clamped to `[-1, 1]`
- exclusion: `default_vertex_spawn_exclusion_probability`, its
  adjustment/hold/repeat values, and bounds within `[0, 1]`
- single-attachment capacity: `default_single_attachment_repeat_count`; `0`
  keeps the current attachment indefinitely, `1` allows one child per
  attachment, and larger values allow that many total children per attachment.
  Raising the capacity reactivates earlier attachments; press `H` to rewind the
  single-spawn frontier so root attachments are considered first

The runtime-only `D` debug overlay shows the current spawning focus without
changing the generated tree. It highlights the active parent and latest child;
the child highlight follows each new sibling, and both highlights move when the
spawning parent changes.

For all configuration fields and their relationship to runtime and preset
state, see the [data-model reference](DATA_MODEL.md). For geometry-specific
defaults, bounds, and examples, see
[Configuration reference](SPAWN_GEOMETRY_GUIDE.md#configuration-reference).

### Camera-output effects

Effects run in this order:

1. lens distortion, including radial, tangential, chromatic, center, scale, and
   zoom controls
2. hard-wrap color wavefolder using gain and modulus
3. Gaussian blur using sigma and a radius currently clamped to 16 pixels
4. bloom using threshold, intensity, and a radius currently clamped to 16 pixels
5. edge detection using strength, threshold, mix, and overlay color

### Fonts

UI text prefers Carbon Plus when a licensed font is placed in `assets/fonts/`.
See [`assets/fonts/README.md`](../assets/fonts/README.md) for supported filenames
and fallback behavior. Additional candidates can be set with
`ui.font_candidates` in `config.toml`.

## Development and verification

Run the complete test suite with the repository wrapper:

```powershell
.\scripts\verify.ps1 -Mode full
```

Other supported verification levels are:

```powershell
# Formatting only
.\scripts\verify.ps1 -Mode format

# One focused test or test-name substring
.\scripts\verify.ps1 -Mode targeted -TestFilter reset_confirmation

# Automated render-and-exit smoke run
.\scripts\verify.ps1 -Mode smoke

# Full tests followed by smoke verification
.\scripts\verify.ps1 -Mode all
```

The wrapper reuses `.target-verification-runs/cargo-cache` for Cargo artifacts.
Use targeted tests while iterating and run the full suite once after a final
change when its risk warrants it. A test or run already compiles affected code,
so a preceding `cargo check` is unnecessary.

The dynamic-linking-free aliases `cargo test-plain` and `cargo run-plain` are
available when that configuration specifically needs testing.
