# intergen Data Model

This document describes the current scene data model in the codebase: what is stored per shape, what is shared across the whole scene, how values are derived, and what domain each named field lives in.

## Overview

The scene state is split across a few layers:

1. `config.toml` and [`src/config.rs`](src/config.rs): startup defaults, clamp ranges, and UI/control tuning.
2. Runtime scene resources in [`src/scene/state.rs`](src/scene/state.rs): the live scene-wide state used by the app.
3. Per-shape nodes in [`src/polyhedra/spawn.rs`](src/polyhedra/spawn.rs): the actual recursive shape tree.
4. Snapshot and preset structs in [`src/scene_snapshot.rs`](src/scene_snapshot.rs) and [`src/presets/storage.rs`](src/presets/storage.rs): the serialized form used by scene presets and Blender export metadata.
5. Effect/LFO runtime state in [`src/effect_tuner/state`](src/effect_tuner/state): live post-process values plus scene-parameter modulation.

The main rule is:

- If something belongs to the whole scene or to the current editing/spawn mode, it lives in a shared runtime resource.
- If something belongs to one generated object in the tree, it lives on `ShapeNode`.
- If something can be recomputed from shared state and node metadata, it is usually derived at render time instead of stored per node.

## Domain Conventions

This document uses "domain" in the mathematical sense: the set of values a field is allowed or expected to hold.

- "Stored domain" means the type and value shape the struct field carries.
- "Effective domain" means the value range that actually reaches rendering/spawn logic after the code clamps or normalizes it.
- `finite f32` means a normal floating-point value, not `NaN` or infinity.
- `Vec<T>` means variable length, with extra rules called out in the notes column when they matter.
- Rust range notation like `0..8` means `0` through `7`.

For several fields, the code stores a broad `f32` but clamps later when it is consumed. In those cases, both the stored domain and the effective domain are listed.

## AppConfig

`AppConfig` is the root configuration object loaded from `config.toml`.

| Field | Domain | Notes |
| --- | --- | --- |
| `window` | `WindowConfig` | Window title, size, and present mode defaults. |
| `rendering` | `RenderingConfig` | Clear color, ambient light, and stage defaults. |
| `camera` | `CameraConfig` | Camera startup pose and motion tuning. |
| `generation` | `GenerationConfig` | Root shape, spawn defaults, and generation parameter specs. |
| `lighting` | `LightingConfig` | Directional, point, and accent light defaults. |
| `effects` | `EffectsConfig` | Post-process effect defaults. |
| `materials` | `MaterialConfig` | Shared material palette and procedural surface defaults. |
| `capture` | `CaptureConfig` | Screenshot/output settings. |
| `ui` | `UiConfig` | Overlay layout, colors, and font candidates. |

The top-level config fields above are not the live scene themselves. They are the defaults and specs from which runtime state is built.

## Scene-Wide Runtime State

The current scene is spread across shared resources:

- `GenerationState`
- `MaterialState`
- `StageState`
- `RenderingState`
- `LightingState`
- `CameraRig`
- `EffectTunerState`

These are scene-wide, not per object.

### `GenerationState`

| Field | Stored Domain | Effective Domain / Notes |
| --- | --- | --- |
| `nodes` | `Vec<ShapeNode>` | Ordered node tree. In valid runtime scenes and prepared snapshots, length is at least `1` and node `0` is the root. Child `parent_index` values refer into this vector. |
| `selected_shape_kind` | `ShapeKind` | One of `Cube`, `Tetrahedron`, `Octahedron`, `Dodecahedron`. This is the shape kind for future spawns, not a property of all existing nodes. |
| `spawn_placement_mode` | `SpawnPlacementMode` | One of `Vertex`, `Edge`, `Face`. Controls where future children attach. |
| `spawn_add_mode` | `SpawnAddMode` | One of `Single`, `FillLevel`. Controls how many objects one spawn action adds. |
| `parameters` | `GenerationParameters` | Shared scalar generation state. See the next table. |
| `spawn_hold` | `HoldRepeatState` | Internal input state. `elapsed_secs` is expected to stay `>= 0.0`; `repeating` is boolean. |

### `GenerationParameters`

Each generation parameter is stored as a `ScalarParameterState`, which means:

- stored base domain: one finite `f32` base value plus one finite `f32` additive offset
- effective domain: the base value plus offset, clamped by the matching `GenerationConfig` spec

| Parameter | Stored Domain | Effective Domain / Notes |
| --- | --- | --- |
| `scale_ratio` | `ScalarParameterState` over finite `f32` | Effective value is clamped to `GenerationConfig.min_scale_ratio..max_scale_ratio`; default config is `0.15..1.0`. Affects future spawns only. Existing per-node `ShapeNode.scale` and `ShapeNode.axis_scale` values are not rewritten. |
| `child_axis_scale_x` | `ScalarParameterState` over finite `f32` | Effective value is clamped to the positive `GenerationConfig.min_child_axis_scale..max_child_axis_scale` range; default config is `0.01..100.0`. Affects future spawns only, plus the root node created by scene reset. Existing per-node `ShapeNode.axis_scale` values are not rewritten. |
| `child_axis_scale_y` | `ScalarParameterState` over finite `f32` | Same semantics as `child_axis_scale_x`, applied to the Y axis. |
| `child_axis_scale_z` | `ScalarParameterState` over finite `f32` | Same semantics as `child_axis_scale_x`, applied to the Z axis. |
| `child_twist` | `ScalarParameterState` over finite `f32` | Effective value is clamped to the nonnegative twist bounds in `GenerationConfig`; default config is `0.0..PI`. Affects child orientation and can recompute existing child node rotations. |
| `child_offset` | `ScalarParameterState` over finite `f32` | Effective value is clamped to the nonnegative offset bounds in `GenerationConfig`; default config is `0.0..6.0`. Measured in child-radius units. Can recompute existing child node centers. |
| `child_position_offset_x` | `ScalarParameterState` over finite `f32` | Effective value is clamped to `[-1.0, 1.0]`. Affects future spawns only. The evaluated value is copied into each spawned child node's stored local position offset X component. |
| `child_position_offset_y` | `ScalarParameterState` over finite `f32` | Same semantics as `child_position_offset_x`, applied to the local Y component. |
| `child_position_offset_z` | `ScalarParameterState` over finite `f32` | Same semantics as `child_position_offset_x`, applied to the local Z component. |
| `child_spawn_exclusion_probability` | `ScalarParameterState` over finite `f32` | Effective value is clamped to `[0.0, 1.0]`. Affects future spawn candidate filtering only. No persistent per-attachment exclusion flag is stored. |

Important distinction:

- `selected_shape_kind`, `spawn_placement_mode`, and `spawn_add_mode` are editor/spawn-mode state for future spawning.
- `scale_ratio`, `child_axis_scale_x/y/z`, `child_position_offset_x/y/z`, and `child_spawn_exclusion_probability` are latent spawn-time parameters: they affect future spawn decisions but do not reflow the existing tree.
- `child_twist` and `child_offset` are shared scene parameters that can recompute existing child transforms from their stored parent/attachment relationship.

### `MaterialState`

`MaterialState` is shared by the whole generated shape set. There is no per-node material state struct for generated shapes.

| Field | Stored Domain | Effective Domain / Notes |
| --- | --- | --- |
| `opacity` | finite `f32` | Effective value is clamped to `[0.0, 1.0]` when applied to materials. Reset/input paths usually respect `MaterialConfig.min_opacity..max_opacity`. |
| `hue_step_per_level` | finite `f32` in degrees per level | No hard storage clamp. Effective hue is wrapped with `rem_euclid(360.0)` when computing color. |
| `saturation` | finite `f32` | Intended as an HSL saturation base. Effective per-node saturation is clamped to `[0.0, 1.0]` after surface bias and level shifts. |
| `lightness` | finite `f32` | Intended as an HSL lightness base. Effective per-node lightness is clamped to `[0.0, 1.0]` after surface bias and level shifts. |
| `metallic` | finite `f32` | Intended base metallic value. Effective per-node value is clamped to `[0.0, 1.0]`. |
| `perceptual_roughness` | finite `f32` | Intended base roughness value. Effective per-node value is clamped to `[0.02, 1.0]` for generated shape materials. |
| `reflectance` | finite `f32` | Effective per-node value is clamped to `[0.0, 1.0]`. |
| `cube_hue_bias` | finite `f32` in degrees | Added to hue for cube nodes, then wrapped mod `360`. |
| `tetrahedron_hue_bias` | finite `f32` in degrees | Added to hue for tetrahedron nodes, then wrapped mod `360`. |
| `octahedron_hue_bias` | finite `f32` in degrees | Added to hue for octahedron nodes, then wrapped mod `360`. |
| `dodecahedron_hue_bias` | finite `f32` in degrees | Added to hue for dodecahedron nodes, then wrapped mod `360`. |
| `surface_mode` | `MaterialSurfaceMode` | One of `Legacy`, `Procedural`. |
| `base_surface` | `MaterialSurfaceFamily` | One of `Legacy`, `Matte`, `Satin`, `Glossy`, `Metal`, `Frosted`. Used for non-root, non-accent levels in procedural mode. |
| `root_surface` | `MaterialSurfaceFamily` | Same enum domain as above. Used for level `0` in procedural mode when not `Legacy`. |
| `accent_surface` | `MaterialSurfaceFamily` | Same enum domain as above. Used for accent levels in procedural mode when not `Legacy`. |
| `accent_every_n_levels` | `usize` | `0` disables accent cadence. Positive values mean "every N levels." |
| `level_lightness_shift` | finite `f32` per level | Added before final `[0.0, 1.0]` clamping. |
| `level_saturation_shift` | finite `f32` per level | Added before final `[0.0, 1.0]` clamping. |
| `level_metallic_shift` | finite `f32` per level | Added before final `[0.0, 1.0]` clamping. |
| `level_roughness_shift` | finite `f32` per level | Added before final `[0.02, 1.0]` clamping. |
| `level_reflectance_shift` | finite `f32` per level | Added before final `[0.0, 1.0]` clamping. |

### `StageState`

| Field | Stored Domain | Effective Domain / Notes |
| --- | --- | --- |
| `enabled` | `bool` | `true` or `false`. Master stage toggle. |
| `floor_enabled` | `bool` | `true` or `false`. Floor surface toggle. |
| `backdrop_enabled` | `bool` | `true` or `false`. Backdrop surface toggle. |
| `floor` | `StageSurfaceState` | Floor surface settings. |
| `backdrop` | `StageSurfaceState` | Backdrop surface settings. |

### `StageSurfaceState`

| Field | Stored Domain | Effective Domain / Notes |
| --- | --- | --- |
| `color` | `[f32; 3]` | RGB triple. No hard clamp in state conversion; intended as display color components. |
| `translation` | `[f32; 3]` | Position vector in scene units. |
| `rotation_degrees` | `[f32; 3]` | Euler rotation in degrees. |
| `size` | `[f32; 2]` | Width and height. Runtime export/config conversion clamps each component to `>= 0.01`. |
| `thickness` | finite `f32` | Runtime export/config conversion clamps to `>= 0.01`. |
| `metallic` | finite `f32` | Runtime export/config conversion clamps to `[0.0, 1.0]`. |
| `perceptual_roughness` | finite `f32` | Runtime export/config conversion clamps to `[0.0, 1.0]`. |
| `reflectance` | finite `f32` | Runtime export/config conversion clamps to `[0.0, 1.0]`. |

### `RenderingState`

| Field | Stored Domain | Effective Domain / Notes |
| --- | --- | --- |
| `clear_color` | `[f32; 3]` | RGB triple. No hard clamp in `RenderingState`; passed through to Bevy as sRGB color. |
| `ambient_light_color` | `[f32; 3]` | RGB triple. No hard clamp in `RenderingState`; passed through to Bevy as sRGB color. |
| `ambient_light_brightness` | finite `f32` | Runtime export/config conversion clamps to `>= 0.0`. |

### `LightingState`

| Field | Stored Domain | Effective Domain / Notes |
| --- | --- | --- |
| `directional` | `DirectionalLightState` | Shared directional light state. |
| `point` | `PointLightState` | Shared point light state. |
| `accent` | `PointLightState` | Shared accent light state. |

### `DirectionalLightState`

| Field | Stored Domain | Effective Domain / Notes |
| --- | --- | --- |
| `color` | `[f32; 3]` | RGB triple. No hard clamp in state conversion. |
| `illuminance` | finite `f32` | Runtime export/config conversion clamps to `>= 0.0`. |
| `translation` | `[f32; 3]` | Position vector in scene units. |
| `look_at` | `[f32; 3]` | Look target in scene units. |

### `PointLightState`

The same struct type is used for both the point light and the accent light.

| Field | Stored Domain | Effective Domain / Notes |
| --- | --- | --- |
| `color` | `[f32; 3]` | RGB triple. No hard clamp in state conversion. |
| `intensity` | finite `f32` | Runtime export/config conversion clamps to `>= 0.0`. |
| `range` | finite `f32` | Runtime export/config conversion clamps to `>= 0.0`. |
| `translation` | `[f32; 3]` | Position vector in scene units. |

### `CameraRig`

| Field | Stored Domain | Effective Domain / Notes |
| --- | --- | --- |
| `orientation` | `Quat` | Expected to be a normalized quaternion. Camera motion renormalizes after integration; snapshot load normalizes too. |
| `angular_velocity` | `Vec3` | Angular velocity vector in radians per second. Components are finite `f32`. No hard clamp in runtime state. |
| `distance` | finite `f32` | Camera distance from origin. Camera motion clamps it to `CameraConfig.min_distance..max_distance` on each update. |
| `zoom_velocity` | finite `f32` | Zoom speed scalar. No hard storage clamp; damped each frame. |

### `EffectTunerState`

`EffectTunerState` is scene-wide runtime modulation state.

| Field / Concept | Domain | Notes |
| --- | --- | --- |
| `current` | `EffectsConfig` | Current effect values. Same schema as `config.effects`, but live and mutable. |
| `lfos` | `Vec<ParameterLfo>` | Ordered LFO slot array. Length equals the current number of LFO-capable effect and scene parameters in this build. Serialized snapshots may be shorter for backward compatibility. |
| scene-parameter base values | `Vec<f32>` | One base numeric value per LFO-capable numeric scene parameter. Used to preserve base values while LFOs are active. |
| selection/edit UI state | indices, enums, booleans, hold states | Internal overlay state only. Domains include bounded selection indices, page/edit enums, booleans, and hold-repeat timers. |

## Per-Shape Node Level

Each generated object in the recursive tree is a `ShapeNode`.

### `ShapeNode`

| Field | Stored Domain | Effective Domain / Notes |
| --- | --- | --- |
| `kind` | `ShapeKind` | One of `Cube`, `Tetrahedron`, `Octahedron`, `Dodecahedron`. |
| `level` | `usize` | Tree depth. Root is level `0`. |
| `center` | `Vec3` | Position in scene space. Valid generated scenes use finite components. |
| `rotation` | `Quat` | Expected to be a normalized quaternion. This is stored per node, but for non-root nodes it is usually derived from the parent transform, the attachment, and the shared twist parameter. |
| `scale` | finite `f32` | Valid generated scenes use `scale > 0.0`. It is the uniform scalar materialized at spawn time from the parent's scale and the shared scale ratio. Later scale-ratio changes do not retroactively rewrite existing node scales. |
| `axis_scale` | `Vec3` of finite `f32` | Per-axis scale multiplier stored per node. `Vec3::ONE` means no non-uniform scaling. There is no hard runtime clamp on the components; negative values are mathematically possible and would mirror the mesh on that axis. |
| `local_position_offset` | `Vec3` of finite `f32` | Child-only copied spawn-time offset stored in the child's local spawn frame. Roots use `Vec3::ZERO`. Recompute paths preserve the stored value and reuse it before deriving the shared outward offset direction. |
| `radius` | finite `f32` | Valid generated scenes use `radius > 0.0`. Cached scaled bounding radius for containment and placement math, derived from the largest absolute component of `scale * axis_scale`. |
| `occupied_attachments` | `AttachmentOccupancy` | Per-node occupancy flags for vertices, edges, and faces. |
| `origin` | `NodeOrigin` | Either `Root` or `Child { parent_index, attachment }`. |

### Per-node scale model

The current per-node scale model has two stored parameters and two important derived quantities:

| Quantity | Domain | Notes |
| --- | --- | --- |
| `ShapeNode.scale` | finite `f32`, normally `> 0.0` in valid generated scenes | Uniform scalar portion of the node scale. Spawned children compute this from `parent.scale * scale_ratio`. |
| `ShapeNode.axis_scale` | `Vec3` of finite `f32` | Per-axis multiplier. New roots and spawned children copy the current shared generation axis-scale values at creation time. After that, the value is stored per node and preserved across recomputes and snapshots. |
| `ShapeNode.local_position_offset` | `Vec3` of finite `f32` | Spawned children copy the current shared child-position offset. After that, the value is stored per node and preserved across recomputes and snapshots. |
| `ShapeNode.combined_scale()` | `Vec3` of finite `f32` | Defined as `axis_scale * scale` component-wise. This is the scale actually sent into Bevy transforms and mesh export. |
| `ShapeNode.bounding_radius(geometry)` | finite `f32`, normally `> 0.0` | Defined from `geometry.radius * max(abs(combined_scale.x), abs(combined_scale.y), abs(combined_scale.z))`. This is what the node caches in `radius`. |

Current runtime behavior:

- root nodes start with the clamped shared generation axis scale
- newly spawned child nodes copy the current clamped shared generation axis scale
- newly spawned child nodes copy the current clamped shared generation local position offset
- spawn-time shared `scale_ratio` changes the uniform `scale` term for future children
- recomputing the tree preserves each node's stored `axis_scale` and `local_position_offset`, and recomputes `radius` from `axis_scale`

### `ShapeKind`

| Value | Meaning |
| --- | --- |
| `Cube` | 8 vertices, 12 edges, 6 faces |
| `Tetrahedron` | 4 vertices, 6 edges, 4 faces |
| `Octahedron` | 6 vertices, 12 edges, 8 faces |
| `Dodecahedron` | 20 vertices, 30 edges, 12 faces |

### `NodeOrigin`

| Variant | Domain | Notes |
| --- | --- | --- |
| `Root` | no payload | Used only for the root node. |
| `Child { parent_index, attachment }` | `parent_index: usize`, `attachment: SpawnAttachment` | In valid trees, `parent_index` points to an earlier node in `GenerationState.nodes`. |

### `SpawnAttachment`

| Field | Domain | Notes |
| --- | --- | --- |
| `mode` | `SpawnPlacementMode` | One of `Vertex`, `Edge`, `Face`. |
| `index` | `usize` | Must be in `0..attachment_count(parent.kind, mode)`. See the next table for concrete counts. |

### `SpawnAttachment.index` by parent shape

| Parent `ShapeKind` | Vertex indices | Edge indices | Face indices |
| --- | --- | --- | --- |
| `Cube` | `0..8` | `0..12` | `0..6` |
| `Tetrahedron` | `0..4` | `0..6` | `0..4` |
| `Octahedron` | `0..6` | `0..12` | `0..8` |
| `Dodecahedron` | `0..20` | `0..30` | `0..12` |

### `AttachmentOccupancy`

`AttachmentOccupancy` lives inside each `ShapeNode`.

| Field | Domain | Notes |
| --- | --- | --- |
| `vertices` | `Vec<bool>` | Length equals the current geometry vertex count for the node's `kind`. |
| `edges` | `Vec<bool>` | Length equals the current geometry edge count for the node's `kind`. |
| `faces` | `Vec<bool>` | Length equals the current geometry face count for the node's `kind`. |

Important distinction:

- Occupancy is stored.
- Spawn exclusion is not stored.

The spawn-exclusion probability is a shared generation parameter. When the app checks whether a candidate attachment is excluded, it derives that result deterministically from:

- `parent_index`
- `attachment.mode`
- `attachment.index`
- the current global exclusion probability

So there is no persistent per-attachment "excluded" flag in the data model.

## Shared vs Per-Shape

### Shared scene-level parameters

These belong to the scene as a whole:

- current spawn settings: child shape kind, placement mode, add mode
- generation scalars: scale ratio, twist, outward offset, spawn exclusion probability
- all material palette and surface settings
- all stage settings
- all rendering settings
- all light settings
- camera rig state
- shader-effect values and all LFO state

### Per-shape parameters

These belong to individual generated objects:

- shape kind
- level
- transform (`center`, `rotation`, `scale`, `axis_scale`)
- derived transform scale (`combined_scale()`)
- cached radius
- parent/origin relationship
- attachment occupancy for that node

### Derived per-shape appearance

Generated shapes do not store their own material/color block. Appearance is derived from:

- node `kind`
- node `level`
- shared `MaterialState`
- shared opacity

In [`src/scene/materials.rs`](src/scene/materials.rs), the current material appearance is computed roughly like this:

- hue comes from `level * hue_step_per_level + hue_bias(kind)`
- surface family comes from the shared surface-mode rules and the node level
- saturation, lightness, metallic, roughness, and reflectance can all shift by level
- opacity is one shared global object opacity value

The derived-value domains are:

| Derived Value | Domain | Notes |
| --- | --- | --- |
| hue | finite `f32`, wrapped to `[0.0, 360.0)` | `rem_euclid(360.0)` is applied. |
| saturation | `[0.0, 1.0]` | After surface bias and level shift. |
| lightness | `[0.0, 1.0]` | After surface bias and level shift. |
| metallic | `[0.0, 1.0]` | After level shift. |
| perceptual roughness | `[0.02, 1.0]` | After level shift. |
| reflectance | `[0.0, 1.0]` | After level shift. |
| opacity | `[0.0, 1.0]` | Global object opacity after clamping. |

So if you are asking "does this value belong to one shape or to the whole scene?", a good test is:

- If every shape would update together when it changes, it is probably shared state.
- If only one node's transform/topology/history changes, it is probably stored on `ShapeNode`.

## Runtime Entity Layer

The Bevy entity layer is intentionally thin.

| Field | Domain | Notes |
| --- | --- | --- |
| `ShapeEntity.node_index` | `usize` | Index into `GenerationState.nodes`. The ECS entity is just a render-side handle pointing back into the authoritative node vector. |

The authoritative recursive scene model is the node vector, not the spawned Bevy entities.

## Serialization and Presets

### `SceneStateSnapshot`

The main serialized snapshot contains:

- `rendering`
- `lighting`
- `materials`
- `camera`
- `generation`
- `material_state`
- `effects`

| Field | Domain | Notes |
| --- | --- | --- |
| `rendering` | `RenderingConfig` | Runtime-rendered rendering config, including stage data. |
| `lighting` | `LightingConfig` | Runtime-rendered lighting config. |
| `materials` | `MaterialConfig` | Runtime material config for generated objects. Most shared material settings serialize here. |
| `camera` | `CameraRigSnapshot` | Serializable camera state. |
| `generation` | `GenerationSnapshot` | Serializable generation tree and generation parameters. |
| `material_state` | `MaterialRuntimeSnapshot` | Currently just runtime opacity. |
| `effects` | `EffectRuntimeSnapshot` | Current effect values plus serialized LFO slots. |

### `CameraRigSnapshot`

| Field | Domain | Notes |
| --- | --- | --- |
| `orientation` | `[f32; 4]` | Quaternion components `[x, y, z, w]`. Normalized on load. |
| `angular_velocity` | `[f32; 3]` | Angular velocity vector components. |
| `distance` | finite `f32` | Stored as-is in the snapshot; runtime camera motion later respects distance bounds. |
| `zoom_velocity` | finite `f32` | Stored zoom velocity scalar. |

### `GenerationSnapshot`

| Field | Domain | Notes |
| --- | --- | --- |
| `selected_shape_kind` | `ShapeKind` | Future-spawn selected child kind. |
| `spawn_placement_mode` | `SpawnPlacementMode` | Future-spawn placement mode. |
| `spawn_add_mode` | `SpawnAddMode` | Future-spawn add mode. |
| `scale_ratio` | finite `f32` | Serialized base scale ratio. In runtime evaluation it is clamped by `GenerationConfig`. |
| `child_axis_scale` | `[f32; 3]` of finite values | Serialized base shared child/root axis scale. Missing serialized values default to `[1.0, 1.0, 1.0]` for backward compatibility. Runtime evaluation clamps each component to the positive axis-scale bounds. |
| `twist_per_vertex_radians` | finite `f32` | Serialized base twist. In runtime evaluation it is clamped by `GenerationConfig`. |
| `vertex_offset_ratio` | finite `f32` | Serialized base outward offset. In runtime evaluation it is clamped by `GenerationConfig`. |
| `vertex_spawn_exclusion_probability` | finite `f32` | Serialized base spawn exclusion probability. Runtime evaluation clamps it to `[0.0, 1.0]`. |
| `nodes` | `Vec<ShapeNodeSnapshot>` | Prepared runtime rejects empty node arrays, so valid prepared snapshots have `len >= 1`. |

### `MaterialRuntimeSnapshot`

| Field | Domain | Notes |
| --- | --- | --- |
| `opacity` | finite `f32` | Clamped to `[0.0, 1.0]` during `prepare_runtime`. |

### `ShapeNodeSnapshot`

| Field | Domain | Notes |
| --- | --- | --- |
| `shape_kind` | `ShapeKind` | Serialized node shape kind. |
| `level` | `usize` | Serialized node depth. |
| `center` | `[f32; 3]` | Serialized world-space center. |
| `rotation` | `[f32; 4]` | Serialized quaternion components. Normalized on load. |
| `scale` | finite `f32` | Valid generated scenes use positive values. This is the stored uniform scalar. |
| `axis_scale` | `[f32; 3]` of finite values | Per-axis scale multiplier. Missing serialized values default to `[1.0, 1.0, 1.0]` for backward compatibility. |
| `radius` | finite `f32` | Valid generated scenes use positive values. Runtime loading recomputes it from geometry and `combined_scale = scale * axis_scale` so cached values stay consistent. |
| `occupied_vertices` | `Vec<bool>` | Resized to current geometry length on load. |
| `occupied_edges` | `Vec<bool>` | Resized to current geometry length on load. |
| `occupied_faces` | `Vec<bool>` | Resized to current geometry length on load. |
| `origin` | `NodeOriginSnapshot` | Serialized root/child origin data. |

### `NodeOriginSnapshot`

| Variant | Domain | Notes |
| --- | --- | --- |
| `Root` | no payload | Root node marker. |
| `Child { parent_index, attachment_mode, attachment_index }` | `parent_index: usize`, `attachment_mode: SpawnPlacementMode`, `attachment_index: usize` | Same attachment index domain rules as runtime `SpawnAttachment`. |

### Preset wrapper

Scene preset files add an outer wrapper with metadata:

- `format_version`
- `id`
- `saved_at_unix_ms`
- `summary`
- `assignment`
- `scene`

| Field | Domain | Notes |
| --- | --- | --- |
| `format_version` | `u32` | Currently written as `1`. Loader currently expects the current preset format version. |
| `id` | `String` | Current writer uses the pattern `preset-<saved_at_unix_ms>`. |
| `saved_at_unix_ms` | `u64` | Unix timestamp in milliseconds. |
| `summary` | `String` | Human-readable summary like `"Cube root, 12637 nodes"`. |
| `assignment` | `Option<PresetIndex>` | `Some` means a bank/slot assignment exists; `None` means the file is unassigned. |
| `scene` | `SceneStateSnapshot` | The actual serialized scene. |

### `PresetIndex`

| Field | Domain | Notes |
| --- | --- | --- |
| `bank` | `u8` | Current UI convention uses digit banks `0..=9`, meaning values `0` through `9`. |
| `slot` | `u8` | Current UI convention uses digit slots `0..=9`, meaning values `0` through `9`. |

## Base Values vs Live Modulation

One of the most important implementation details is the split between base values and live LFO-modulated values.

### `ScalarParameterState`

| Field / Concept | Domain | Notes |
| --- | --- | --- |
| base value | finite `f32` | Stored independently from modulation. Presets save base generation values rather than one arbitrary sampled frame. |
| additive offset | finite `f32` | Runtime-only modulation offset. |
| effective value | finite `f32` | Computed as `base_value + additive_offset`, then clamped by the matching parameter spec. |
| input state | `ScalarParameterInputState` | Internal hold-repeat state for keyboard editing. |

### `ParameterLfo`

| Field | Domain | Notes |
| --- | --- | --- |
| `enabled` | `bool` | `true` or `false`. |
| `shape` | `LfoShape` | One of `Sine`, `Triangle`, `Saw`, `Square`, `SteppedRandom`, `BrownianMotion`. |
| `amplitude` | finite `f32` | UI/edit paths keep it `>= 0.0`. An LFO is only active when amplitude is `> 0.0`. |
| `frequency_hz` | finite `f32` | UI/edit paths keep it `>= 0.0`. An LFO is only active when frequency is `> 0.0`. |

### `LfoShape.sample(...)`

| Shape | Output Domain | Notes |
| --- | --- | --- |
| `Sine` | `[-1.0, 1.0]` | Continuous sinusoid. |
| `Triangle` | `[-1.0, 1.0]` | Continuous triangle wave. |
| `Saw` | `[-1.0, 1.0)` | Sawtooth wave over one cycle. |
| `Square` | `{-1.0, 1.0}` | Binary square wave. |
| `SteppedRandom` | `[-1.0, 1.0]` | Piecewise-constant random values. |
| `BrownianMotion` | `[-1.0, 1.0]` | Smooth bounded random walk. |

That means presets do not freeze one arbitrary sampled frame when LFOs are active. Instead they preserve:

- the base scene values
- the active effect values
- the LFO configuration

For generation specifically:

- `scale_ratio` and `vertex_spawn_exclusion_probability` can carry live LFO offsets as latent spawn-time parameters for future spawn decisions
- `twist_per_vertex_radians` and `vertex_offset_ratio` can carry live LFO offsets and can be reapplied to recompute the existing tree
- existing nodes still serialize their concrete transform values and origins

## Practical Rules Of Thumb

- Put transform, history, and topology data on `ShapeNode`.
- Put scene-wide controls and palettes in shared runtime resources.
- Treat `GenerationState.selected_shape_kind` and the spawn modes as "what the next spawn should do," not as properties of existing nodes.
- Treat `ShapeNode.rotation` as per-node stored state, but remember that child rotations are normally derived from per-node origin data plus shared twist.
- Treat material settings as shared rules that derive per-node appearance from `kind` and `level`.
- Treat preset files as serialized scene-wide state plus the concrete shape tree, not as a second independent data model.
