# intergen Data Model

This document describes the current scene data model in the codebase: what is stored per shape, what is shared across the whole scene, and how that data is serialized into presets and exports.

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

## Aggregation Levels

### 1. App-wide startup defaults

`AppConfig` is the root configuration object loaded from `config.toml`.

Top-level sections:

- `window`
- `rendering`
- `camera`
- `generation`
- `lighting`
- `effects`
- `materials`
- `capture`
- `ui`

This layer is mostly defaults and parameter specs, not the current live scene. For example:

- `GenerationConfig` stores default values and bounds for scale, twist, outward offset, and spawn exclusion.
- `MaterialConfig` stores the default palette/PBR values and procedural surface-family rules.
- `RenderingConfig` stores clear color, ambient light, and stage defaults.
- `LightingConfig` stores directional, point, and accent light defaults.

### 2. Scene-wide runtime state

The current scene is spread across shared resources:

- `GenerationState`
- `MaterialState`
- `StageState`
- `RenderingState`
- `LightingState`
- `CameraRig`
- `EffectTunerState`

These are scene-wide, not per object.

#### `GenerationState`

`GenerationState` owns:

- `nodes: Vec<ShapeNode>`: the actual recursive shape tree
- `selected_shape_kind`: which shape kind the next spawn should use
- `spawn_placement_mode`: whether new children attach to vertices, edges, or faces
- `spawn_add_mode`: single spawn vs fill-current-level behavior
- `parameters`: shared generation parameter states
- `spawn_hold`: key-repeat state for spawning input

`GenerationParameters` currently contains four shared scalar parameters:

- `scale_ratio`
- `child_twist`
- `child_offset`
- `child_spawn_exclusion_probability`

Important distinction:

- `selected_shape_kind`, `spawn_placement_mode`, and `spawn_add_mode` are editor/spawn-mode state for future spawning.
- `scale_ratio` is also a future-spawn input; existing nodes keep their own already-materialized `scale`.
- `child_twist` and `child_offset` are shared scene parameters that can recompute existing child transforms from their stored parent/attachment relationship.
- `child_spawn_exclusion_probability` affects future spawn candidate selection only; it does not rewrite existing nodes.

#### `MaterialState`

`MaterialState` is shared by the whole generated shape set. It stores:

- global opacity
- hue progression and base HSL/PBR values
- per-shape-kind hue biases
- surface-mode and surface-family selectors
- accent cadence by level
- per-level shifts for lightness, saturation, metallic, roughness, and reflectance

There is no per-node material state struct for generated shapes.

#### `StageState`

`StageState` is scene-wide and contains:

- `enabled`
- `floor_enabled`
- `backdrop_enabled`
- `floor: StageSurfaceState`
- `backdrop: StageSurfaceState`

Each `StageSurfaceState` contains:

- `color`
- `translation`
- `rotation_degrees`
- `size`
- `thickness`
- `metallic`
- `perceptual_roughness`
- `reflectance`

#### `RenderingState`

`RenderingState` is shared and contains:

- `clear_color`
- `ambient_light_color`
- `ambient_light_brightness`

#### `LightingState`

`LightingState` is shared and contains:

- `directional: DirectionalLightState`
- `point: PointLightState`
- `accent: PointLightState`

Those light structs store shared light color/intensity/range/position data for each light type.

#### `EffectTunerState`

`EffectTunerState` is also scene-wide. It stores:

- current effect values (`EffectsConfig`)
- one `ParameterLfo` slot per LFO-capable effect or scene parameter
- internal base values for scene-parameter LFO application
- control-page selection/edit UI state

This is where runtime modulation lives. The effect tuner does not create per-node effect state.

### 3. Per-shape node level

Each generated object in the recursive tree is a `ShapeNode`.

`ShapeNode` stores:

- `kind`
- `level`
- `center`
- `rotation`
- `scale`
- `radius`
- `occupied_attachments`
- `origin`

What those mean:

- `kind`: cube, tetrahedron, octahedron, or dodecahedron
- `level`: tree depth, with the root at level `0`
- `center`, `rotation`, `scale`: the node's current transform data
- `radius`: cached scaled bounding radius for spawn checks and placement math
- `occupied_attachments`: which of this node's vertices/edges/faces are already used for children
- `origin`: how this node was created

`NodeOrigin` is:

- `Root`
- `Child { parent_index, attachment }`

`attachment` is a `SpawnAttachment`:

- `mode`: vertex, edge, or face
- `index`: which specific vertex/edge/face on the parent

This parent-plus-attachment link is what allows the app to recompute child positions and rotations when shared twist or offset settings change.

### 4. Per-attachment occupancy level

`AttachmentOccupancy` lives inside each `ShapeNode` and contains three boolean arrays:

- `vertices`
- `edges`
- `faces`

These arrays track which parent attachment points are already occupied by spawned children.

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
- transform (`center`, `rotation`, `scale`)
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
- saturation/lightness/metallic/roughness/reflectance can also shift by level
- opacity is one shared global object opacity value

So if you are asking "does this value belong to one shape or to the whole scene?", a good test is:

- If every shape would update together when it changes, it is probably shared state.
- If only one node's transform/topology/history changes, it is probably stored on `ShapeNode`.

## Runtime Entity Layer

The Bevy entity layer is intentionally thin.

`ShapeEntity` only stores:

- `node_index`

That means the ECS entity is just a render-side handle pointing back into `GenerationState.nodes`.

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

Notable detail:

- most material settings serialize through `materials: MaterialConfig`
- `material_state` currently only stores runtime `opacity`

### `GenerationSnapshot`

`GenerationSnapshot` stores:

- `selected_shape_kind`
- `spawn_placement_mode`
- `spawn_add_mode`
- `scale_ratio`
- `twist_per_vertex_radians`
- `vertex_offset_ratio`
- `vertex_spawn_exclusion_probability`
- `nodes`

Each serialized node stores:

- `shape_kind`
- `level`
- `center`
- `rotation`
- `scale`
- `radius`
- `occupied_vertices`
- `occupied_edges`
- `occupied_faces`
- `origin`

### Preset wrapper

Scene preset files add an outer wrapper with metadata:

- `format_version`
- `id`
- `saved_at_unix_ms`
- `summary`
- `assignment`
- `scene`

The actual scene data still lives inside `scene: SceneStateSnapshot`.

## Base Values vs Live Modulation

One of the most important implementation details is the split between base values and live LFO-modulated values.

- `ScalarParameterState` stores a `base_value` plus runtime modulation.
- `EffectTunerState` keeps the current effect config and LFO definitions separately.
- Scene preset capture uses base scene/material/camera/light/stage values plus the effect/LFO runtime snapshot.

That means presets do not freeze one arbitrary sampled frame when LFOs are active. Instead they preserve:

- the base scene values
- the active effect values
- the LFO configuration

For generation specifically:

- `scale_ratio` is stored as a base value for future spawning
- `twist_per_vertex_radians` and `vertex_offset_ratio` are stored as base values and can be reapplied to recompute the existing tree
- existing nodes still serialize their concrete transform values and origins

## Practical Rules Of Thumb

- Put transform/history/topology data on `ShapeNode`.
- Put scene-wide controls and palettes in shared runtime resources.
- Treat `GenerationState.selected_shape_kind` and the spawn modes as "what the next spawn should do", not as properties of existing nodes.
- Treat material settings as shared rules that derive per-node appearance from `kind` and `level`.
- Treat preset files as serialized scene-wide state plus the concrete shape tree, not as a second independent data model.
