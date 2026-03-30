use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::camera::CameraRig;
use crate::config::{
    CameraConfig, EffectGroup, EffectNumericParameter, EffectsConfig, GenerationConfig,
    LightingConfig, MaterialConfig, MaterialSurfaceFamily, MaterialSurfaceMode, RenderingConfig,
};
use crate::parameters::{GenerationParameter, HoldInput, HoldRepeatState};
use crate::scene::{GenerationState, LightingState, MaterialState, RenderingState, StageState};
use crate::shapes::{ShapeKind, SpawnAddMode, SpawnPlacementMode};

use super::lfo::{DEFAULT_LFO_FREQUENCY_HZ, LFO_FREQUENCY_STEP_HZ, LfoShape, ParameterLfo};
use super::metadata::{EffectEditMode, EffectOverlayField};

const OVERLAY_HOLD_SECS: f32 = 2.5;
const HOLD_DELAY_SECS: f32 = 0.32;
const REPEAT_INTERVAL_SECS: f32 = 0.08;
const NUMERIC_ENTRY_RESTART_SECS: f32 = 1.0;

include!("state/core.rs");
include!("state/scene_parameter.rs");
include!("state/parameter.rs");
include!("state/numeric_entry.rs");
include!("state/helpers.rs");

#[cfg(test)]
mod tests {
    include!("state/tests.rs");
}
