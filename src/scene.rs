use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::math::primitives::Cuboid;
use bevy::prelude::*;

use crate::camera::{CameraRig, SceneCamera};
use crate::config::{
    AppConfig, GenerationConfig, LightingConfig, MaterialConfig, MaterialSurfaceFamily,
    MaterialSurfaceMode, RenderingConfig, StageConfig, StageSurfaceConfig,
};
use crate::effects::{camera_effects_from_config, effects_status_messages};
use crate::generation::{
    spawn_add_mode_status_message, spawn_placement_mode_status_message, twist_status_message,
    vertex_exclusion_status_message, vertex_offset_status_message,
};
use crate::help_text::{startup_controls_message, startup_fx_message};
use crate::parameters::{GenerationParameter, HoldRepeatState, ScalarParameterState};
use crate::shapes::{
    ShapeCatalog, ShapeGeometry, ShapeKind, ShapeNode, SpawnAddMode, SpawnPlacementMode,
    SpawnTuning, build_mesh, root_node_with_axis_scale,
};
use crate::ui::{UiFontSource, load_ui_theme, spawn_help_ui};

include!("scene/state.rs");
include!("scene/setup.rs");
include!("scene/materials.rs");

#[cfg(test)]
mod tests {
    use super::*;

    include!("scene/tests.rs");
}
