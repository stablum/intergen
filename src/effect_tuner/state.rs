use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{
    EffectGroup, EffectNumericParameter, EffectsConfig, GenerationConfig, MaterialConfig,
    MaterialSurfaceFamily, MaterialSurfaceMode, StageConfig,
};
use crate::generation::{
    selected_child_shape_status_message, spawn_add_mode_status_message,
    spawn_placement_mode_status_message,
};
use crate::parameters::{GenerationParameter, HoldInput, HoldRepeatState};
use crate::scene::{opacity_status_message, GenerationState, MaterialState, StageState};
use crate::shapes::{ShapeKind, SpawnAddMode, SpawnPlacementMode};

use super::lfo::{LfoShape, ParameterLfo, DEFAULT_LFO_FREQUENCY_HZ, LFO_FREQUENCY_STEP_HZ};
use super::metadata::{EffectEditMode, EffectOverlayField};

const OVERLAY_HOLD_SECS: f32 = 2.5;
const HOLD_DELAY_SECS: f32 = 0.32;
const REPEAT_INTERVAL_SECS: f32 = 0.08;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerOverlaySnapshot {
    pub(crate) pinned: bool,
    pub(crate) effect_label: &'static str,
    pub(crate) effect_state_text: &'static str,
    pub(crate) effect_state_emphasized: bool,
    pub(crate) parameter_label: &'static str,
    pub(crate) value_text: String,
    pub(crate) live_value_text: String,
    pub(crate) lfo_state_text: &'static str,
    pub(crate) lfo_state_emphasized: bool,
    pub(crate) amplitude_text: String,
    pub(crate) frequency_text: String,
    pub(crate) shape_text: &'static str,
    pub(crate) active_field: EffectOverlayField,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectTunerPageMode {
    Compact,
    List,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerListRowSnapshot {
    pub(crate) effect_label: &'static str,
    pub(crate) effect_state_text: &'static str,
    pub(crate) effect_state_emphasized: bool,
    pub(crate) parameter_label: &'static str,
    pub(crate) value_text: String,
    pub(crate) live_value_text: String,
    pub(crate) lfo_state_text: &'static str,
    pub(crate) lfo_state_emphasized: bool,
    pub(crate) selected: bool,
    pub(crate) active_field: Option<EffectOverlayField>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerListOverlaySnapshot {
    pub(crate) pinned: bool,
    pub(crate) total_parameters: usize,
    pub(crate) window_start: usize,
    pub(crate) rows: Vec<EffectTunerListRowSnapshot>,
    pub(crate) detail: EffectTunerOverlaySnapshot,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct EffectRuntimeSnapshot {
    pub(crate) current: EffectsConfig,
    pub(crate) lfos: Vec<ParameterLfo>,
}

#[derive(Clone, Copy)]
pub(crate) struct AdjustmentModifiers {
    pub(crate) shift_pressed: bool,
    pub(crate) alt_pressed: bool,
}

pub(crate) struct EffectTunerViewContext<'a> {
    pub(crate) generation_config: &'a GenerationConfig,
    pub(crate) generation_state: &'a GenerationState,
    pub(crate) material_config: &'a MaterialConfig,
    pub(crate) material_state: &'a MaterialState,
    pub(crate) stage_state: &'a StageState,
}

pub(crate) struct EffectTunerEditContext<'a> {
    pub(crate) generation_config: &'a GenerationConfig,
    pub(crate) generation_state: &'a mut GenerationState,
    pub(crate) material_config: &'a MaterialConfig,
    pub(crate) material_state: &'a mut MaterialState,
    pub(crate) stage_config: &'a StageConfig,
    pub(crate) stage_state: &'a mut StageState,
}

impl EffectTunerEditContext<'_> {
    fn view(&self) -> EffectTunerViewContext<'_> {
        EffectTunerViewContext {
            generation_config: self.generation_config,
            generation_state: &*self.generation_state,
            material_config: self.material_config,
            material_state: &*self.material_state,
            stage_state: &*self.stage_state,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectTunerSceneParameter {
    ChildKind,
    SpawnPlacementMode,
    SpawnAddMode,
    ChildScaleRatio,
    ChildTwistPerVertexRadians,
    ChildOutwardOffsetRatio,
    ChildSpawnExclusionProbability,
    StageEnabled,
    StageFloorEnabled,
    StageBackdropEnabled,
    GlobalOpacity,
    MaterialHueStepPerLevel,
    MaterialSaturation,
    MaterialLightness,
    MaterialMetallic,
    MaterialPerceptualRoughness,
    MaterialReflectance,
    MaterialCubeHueBias,
    MaterialTetrahedronHueBias,
    MaterialOctahedronHueBias,
    MaterialDodecahedronHueBias,
    MaterialSurfaceMode,
    MaterialBaseSurface,
    MaterialRootSurface,
    MaterialAccentSurface,
    MaterialAccentEveryNLevels,
    MaterialLevelLightnessShift,
    MaterialLevelSaturationShift,
    MaterialLevelMetallicShift,
    MaterialLevelRoughnessShift,
    MaterialLevelReflectanceShift,
}

impl EffectTunerSceneParameter {
    const ALL: [Self; 31] = [
        Self::ChildKind,
        Self::SpawnPlacementMode,
        Self::SpawnAddMode,
        Self::ChildScaleRatio,
        Self::ChildTwistPerVertexRadians,
        Self::ChildOutwardOffsetRatio,
        Self::ChildSpawnExclusionProbability,
        Self::StageEnabled,
        Self::StageFloorEnabled,
        Self::StageBackdropEnabled,
        Self::GlobalOpacity,
        Self::MaterialHueStepPerLevel,
        Self::MaterialSaturation,
        Self::MaterialLightness,
        Self::MaterialMetallic,
        Self::MaterialPerceptualRoughness,
        Self::MaterialReflectance,
        Self::MaterialCubeHueBias,
        Self::MaterialTetrahedronHueBias,
        Self::MaterialOctahedronHueBias,
        Self::MaterialDodecahedronHueBias,
        Self::MaterialSurfaceMode,
        Self::MaterialBaseSurface,
        Self::MaterialRootSurface,
        Self::MaterialAccentSurface,
        Self::MaterialAccentEveryNLevels,
        Self::MaterialLevelLightnessShift,
        Self::MaterialLevelSaturationShift,
        Self::MaterialLevelMetallicShift,
        Self::MaterialLevelRoughnessShift,
        Self::MaterialLevelReflectanceShift,
    ];

    pub(crate) fn all() -> &'static [Self] {
        &Self::ALL
    }

    fn label(self) -> &'static str {
        match self {
            Self::ChildKind => "generation.child_kind",
            Self::SpawnPlacementMode => "generation.spawn_placement_mode",
            Self::SpawnAddMode => "generation.spawn_add_mode",
            Self::ChildScaleRatio => "generation.child_scale_ratio",
            Self::ChildTwistPerVertexRadians => "generation.child_twist_per_vertex_radians",
            Self::ChildOutwardOffsetRatio => "generation.child_outward_offset_ratio",
            Self::ChildSpawnExclusionProbability => "generation.child_spawn_exclusion_probability",
            Self::StageEnabled => "stage.enabled",
            Self::StageFloorEnabled => "stage.floor.enabled",
            Self::StageBackdropEnabled => "stage.backdrop.enabled",
            Self::GlobalOpacity => "materials.opacity",
            Self::MaterialHueStepPerLevel => "materials.hue_step_per_level",
            Self::MaterialSaturation => "materials.saturation",
            Self::MaterialLightness => "materials.lightness",
            Self::MaterialMetallic => "materials.metallic",
            Self::MaterialPerceptualRoughness => "materials.perceptual_roughness",
            Self::MaterialReflectance => "materials.reflectance",
            Self::MaterialCubeHueBias => "materials.cube_hue_bias",
            Self::MaterialTetrahedronHueBias => "materials.tetrahedron_hue_bias",
            Self::MaterialOctahedronHueBias => "materials.octahedron_hue_bias",
            Self::MaterialDodecahedronHueBias => "materials.dodecahedron_hue_bias",
            Self::MaterialSurfaceMode => "materials.surface_mode",
            Self::MaterialBaseSurface => "materials.base_surface",
            Self::MaterialRootSurface => "materials.root_surface",
            Self::MaterialAccentSurface => "materials.accent_surface",
            Self::MaterialAccentEveryNLevels => "materials.accent_every_n_levels",
            Self::MaterialLevelLightnessShift => "materials.level_lightness_shift",
            Self::MaterialLevelSaturationShift => "materials.level_saturation_shift",
            Self::MaterialLevelMetallicShift => "materials.level_metallic_shift",
            Self::MaterialLevelRoughnessShift => "materials.level_roughness_shift",
            Self::MaterialLevelReflectanceShift => "materials.level_reflectance_shift",
        }
    }

    fn short_label(self) -> &'static str {
        match self {
            Self::ChildKind => "shape",
            Self::SpawnPlacementMode => "placement",
            Self::SpawnAddMode => "add mode",
            Self::ChildScaleRatio => "scale",
            Self::ChildTwistPerVertexRadians => "twist",
            Self::ChildOutwardOffsetRatio => "offset",
            Self::ChildSpawnExclusionProbability => "spawn%",
            Self::StageEnabled => "enabled",
            Self::StageFloorEnabled => "floor",
            Self::StageBackdropEnabled => "backdrop",
            Self::GlobalOpacity => "opacity",
            Self::MaterialHueStepPerLevel => "hue step",
            Self::MaterialSaturation => "sat",
            Self::MaterialLightness => "light",
            Self::MaterialMetallic => "metallic",
            Self::MaterialPerceptualRoughness => "roughness",
            Self::MaterialReflectance => "reflect",
            Self::MaterialCubeHueBias => "cube hue",
            Self::MaterialTetrahedronHueBias => "tetra hue",
            Self::MaterialOctahedronHueBias => "octa hue",
            Self::MaterialDodecahedronHueBias => "dodec hue",
            Self::MaterialSurfaceMode => "surface",
            Self::MaterialBaseSurface => "base surf",
            Self::MaterialRootSurface => "root surf",
            Self::MaterialAccentSurface => "accent srf",
            Self::MaterialAccentEveryNLevels => "accent nth",
            Self::MaterialLevelLightnessShift => "lvl light",
            Self::MaterialLevelSaturationShift => "lvl sat",
            Self::MaterialLevelMetallicShift => "lvl metal",
            Self::MaterialLevelRoughnessShift => "lvl rough",
            Self::MaterialLevelReflectanceShift => "lvl refl",
        }
    }

    pub(crate) fn group_label(self) -> &'static str {
        match self {
            Self::StageEnabled | Self::StageFloorEnabled | Self::StageBackdropEnabled => "stage",
            Self::GlobalOpacity
            | Self::MaterialHueStepPerLevel
            | Self::MaterialSaturation
            | Self::MaterialLightness
            | Self::MaterialMetallic
            | Self::MaterialPerceptualRoughness
            | Self::MaterialReflectance
            | Self::MaterialCubeHueBias
            | Self::MaterialTetrahedronHueBias
            | Self::MaterialOctahedronHueBias
            | Self::MaterialDodecahedronHueBias
            | Self::MaterialSurfaceMode
            | Self::MaterialBaseSurface
            | Self::MaterialRootSurface
            | Self::MaterialAccentSurface
            | Self::MaterialAccentEveryNLevels
            | Self::MaterialLevelLightnessShift
            | Self::MaterialLevelSaturationShift
            | Self::MaterialLevelMetallicShift
            | Self::MaterialLevelRoughnessShift
            | Self::MaterialLevelReflectanceShift => "mat",
            Self::ChildKind
            | Self::SpawnPlacementMode
            | Self::SpawnAddMode
            | Self::ChildScaleRatio
            | Self::ChildTwistPerVertexRadians
            | Self::ChildOutwardOffsetRatio
            | Self::ChildSpawnExclusionProbability => "scene",
        }
    }

    fn is_numeric(self) -> bool {
        matches!(
            self,
            Self::ChildScaleRatio
                | Self::ChildTwistPerVertexRadians
                | Self::ChildOutwardOffsetRatio
                | Self::ChildSpawnExclusionProbability
                | Self::GlobalOpacity
                | Self::MaterialHueStepPerLevel
                | Self::MaterialSaturation
                | Self::MaterialLightness
                | Self::MaterialMetallic
                | Self::MaterialPerceptualRoughness
                | Self::MaterialReflectance
                | Self::MaterialCubeHueBias
                | Self::MaterialTetrahedronHueBias
                | Self::MaterialOctahedronHueBias
                | Self::MaterialDodecahedronHueBias
                | Self::MaterialAccentEveryNLevels
                | Self::MaterialLevelLightnessShift
                | Self::MaterialLevelSaturationShift
                | Self::MaterialLevelMetallicShift
                | Self::MaterialLevelRoughnessShift
                | Self::MaterialLevelReflectanceShift
        )
    }

    fn generation_parameter(self) -> Option<GenerationParameter> {
        match self {
            Self::ChildScaleRatio => Some(GenerationParameter::ChildScaleRatio),
            Self::ChildTwistPerVertexRadians => {
                Some(GenerationParameter::ChildTwistPerVertexRadians)
            }
            Self::ChildOutwardOffsetRatio => Some(GenerationParameter::ChildOutwardOffsetRatio),
            Self::ChildSpawnExclusionProbability => {
                Some(GenerationParameter::ChildSpawnExclusionProbability)
            }
            Self::ChildKind
            | Self::SpawnPlacementMode
            | Self::SpawnAddMode
            | Self::StageEnabled
            | Self::StageFloorEnabled
            | Self::StageBackdropEnabled
            | Self::GlobalOpacity
            | Self::MaterialHueStepPerLevel
            | Self::MaterialSaturation
            | Self::MaterialLightness
            | Self::MaterialMetallic
            | Self::MaterialPerceptualRoughness
            | Self::MaterialReflectance
            | Self::MaterialCubeHueBias
            | Self::MaterialTetrahedronHueBias
            | Self::MaterialOctahedronHueBias
            | Self::MaterialDodecahedronHueBias
            | Self::MaterialSurfaceMode
            | Self::MaterialBaseSurface
            | Self::MaterialRootSurface
            | Self::MaterialAccentSurface
            | Self::MaterialAccentEveryNLevels
            | Self::MaterialLevelLightnessShift
            | Self::MaterialLevelSaturationShift
            | Self::MaterialLevelMetallicShift
            | Self::MaterialLevelRoughnessShift
            | Self::MaterialLevelReflectanceShift => None,
        }
    }

    fn base_step(self, context: &EffectTunerViewContext<'_>) -> f32 {
        match self.generation_parameter() {
            Some(parameter) => context.generation_config.parameter_spec(parameter).step(),
            None => match self {
                Self::GlobalOpacity => context.material_config.opacity_adjust_step.abs(),
                Self::MaterialHueStepPerLevel
                | Self::MaterialCubeHueBias
                | Self::MaterialTetrahedronHueBias
                | Self::MaterialOctahedronHueBias
                | Self::MaterialDodecahedronHueBias => 5.0,
                Self::MaterialAccentEveryNLevels => 1.0,
                Self::MaterialSaturation
                | Self::MaterialLightness
                | Self::MaterialMetallic
                | Self::MaterialPerceptualRoughness
                | Self::MaterialReflectance
                | Self::MaterialLevelLightnessShift
                | Self::MaterialLevelSaturationShift
                | Self::MaterialLevelMetallicShift
                | Self::MaterialLevelRoughnessShift
                | Self::MaterialLevelReflectanceShift => 0.05,
                Self::ChildKind
                | Self::SpawnPlacementMode
                | Self::SpawnAddMode
                | Self::StageEnabled
                | Self::StageFloorEnabled
                | Self::StageBackdropEnabled
                | Self::MaterialSurfaceMode
                | Self::MaterialBaseSurface
                | Self::MaterialRootSurface
                | Self::MaterialAccentSurface
                | Self::ChildScaleRatio
                | Self::ChildTwistPerVertexRadians
                | Self::ChildOutwardOffsetRatio
                | Self::ChildSpawnExclusionProbability => 1.0,
            },
        }
    }

    fn adjustment_step(
        self,
        context: &EffectTunerViewContext<'_>,
        shift_pressed: bool,
        alt_pressed: bool,
    ) -> f32 {
        let mut step = self.base_step(context);
        if shift_pressed {
            step *= 10.0;
        }
        if alt_pressed {
            step *= 0.1;
        }
        step
    }

    fn value(self, context: &EffectTunerViewContext<'_>) -> f32 {
        match self {
            Self::ChildKind
            | Self::SpawnPlacementMode
            | Self::SpawnAddMode
            | Self::MaterialSurfaceMode
            | Self::MaterialBaseSurface
            | Self::MaterialRootSurface
            | Self::MaterialAccentSurface => 0.0,
            Self::ChildScaleRatio => context.generation_state.scale_ratio_base(),
            Self::ChildTwistPerVertexRadians => {
                context.generation_state.twist_per_vertex_radians_base()
            }
            Self::ChildOutwardOffsetRatio => context.generation_state.vertex_offset_ratio_base(),
            Self::ChildSpawnExclusionProbability => context
                .generation_state
                .vertex_spawn_exclusion_probability_base(),
            Self::StageEnabled => {
                if context.stage_state.enabled {
                    1.0
                } else {
                    0.0
                }
            }
            Self::StageFloorEnabled => {
                if context.stage_state.floor_enabled {
                    1.0
                } else {
                    0.0
                }
            }
            Self::StageBackdropEnabled => {
                if context.stage_state.backdrop_enabled {
                    1.0
                } else {
                    0.0
                }
            }
            Self::GlobalOpacity => context.material_state.opacity,
            Self::MaterialHueStepPerLevel => context.material_state.hue_step_per_level,
            Self::MaterialSaturation => context.material_state.saturation,
            Self::MaterialLightness => context.material_state.lightness,
            Self::MaterialMetallic => context.material_state.metallic,
            Self::MaterialPerceptualRoughness => context.material_state.perceptual_roughness,
            Self::MaterialReflectance => context.material_state.reflectance,
            Self::MaterialCubeHueBias => context.material_state.cube_hue_bias,
            Self::MaterialTetrahedronHueBias => context.material_state.tetrahedron_hue_bias,
            Self::MaterialOctahedronHueBias => context.material_state.octahedron_hue_bias,
            Self::MaterialDodecahedronHueBias => context.material_state.dodecahedron_hue_bias,
            Self::MaterialAccentEveryNLevels => context.material_state.accent_every_n_levels as f32,
            Self::MaterialLevelLightnessShift => context.material_state.level_lightness_shift,
            Self::MaterialLevelSaturationShift => context.material_state.level_saturation_shift,
            Self::MaterialLevelMetallicShift => context.material_state.level_metallic_shift,
            Self::MaterialLevelRoughnessShift => context.material_state.level_roughness_shift,
            Self::MaterialLevelReflectanceShift => context.material_state.level_reflectance_shift,
        }
    }

    fn set_value(self, context: &mut EffectTunerEditContext<'_>, value: f32) -> f32 {
        match self.generation_parameter() {
            Some(parameter) => {
                let spec = context.generation_config.parameter_spec(parameter);
                let parameter_state = context.generation_state.parameter_mut(parameter);
                let current = parameter_state.base_value();
                parameter_state.adjust_clamped_base_value(value - current, spec)
            }
            None => match self {
                Self::GlobalOpacity => {
                    let (min_opacity, max_opacity) = context.material_config.opacity_bounds();
                    context.material_state.opacity = value.clamp(min_opacity, max_opacity);
                    context.material_state.opacity
                }
                Self::MaterialHueStepPerLevel => {
                    context.material_state.hue_step_per_level = value;
                    context.material_state.hue_step_per_level
                }
                Self::MaterialSaturation => {
                    context.material_state.saturation = value.clamp(0.0, 1.0);
                    context.material_state.saturation
                }
                Self::MaterialLightness => {
                    context.material_state.lightness = value.clamp(0.0, 1.0);
                    context.material_state.lightness
                }
                Self::MaterialMetallic => {
                    context.material_state.metallic = value.clamp(0.0, 1.0);
                    context.material_state.metallic
                }
                Self::MaterialPerceptualRoughness => {
                    context.material_state.perceptual_roughness = value.clamp(0.0, 1.0);
                    context.material_state.perceptual_roughness
                }
                Self::MaterialReflectance => {
                    context.material_state.reflectance = value.clamp(0.0, 1.0);
                    context.material_state.reflectance
                }
                Self::MaterialCubeHueBias => {
                    context.material_state.cube_hue_bias = value;
                    context.material_state.cube_hue_bias
                }
                Self::MaterialTetrahedronHueBias => {
                    context.material_state.tetrahedron_hue_bias = value;
                    context.material_state.tetrahedron_hue_bias
                }
                Self::MaterialOctahedronHueBias => {
                    context.material_state.octahedron_hue_bias = value;
                    context.material_state.octahedron_hue_bias
                }
                Self::MaterialDodecahedronHueBias => {
                    context.material_state.dodecahedron_hue_bias = value;
                    context.material_state.dodecahedron_hue_bias
                }
                Self::MaterialAccentEveryNLevels => {
                    context.material_state.accent_every_n_levels = value.round().max(0.0) as usize;
                    context.material_state.accent_every_n_levels as f32
                }
                Self::MaterialLevelLightnessShift => {
                    context.material_state.level_lightness_shift = value;
                    context.material_state.level_lightness_shift
                }
                Self::MaterialLevelSaturationShift => {
                    context.material_state.level_saturation_shift = value;
                    context.material_state.level_saturation_shift
                }
                Self::MaterialLevelMetallicShift => {
                    context.material_state.level_metallic_shift = value;
                    context.material_state.level_metallic_shift
                }
                Self::MaterialLevelRoughnessShift => {
                    context.material_state.level_roughness_shift = value;
                    context.material_state.level_roughness_shift
                }
                Self::MaterialLevelReflectanceShift => {
                    context.material_state.level_reflectance_shift = value;
                    context.material_state.level_reflectance_shift
                }
                Self::ChildKind
                | Self::SpawnPlacementMode
                | Self::SpawnAddMode
                | Self::StageEnabled
                | Self::StageFloorEnabled
                | Self::StageBackdropEnabled
                | Self::MaterialSurfaceMode
                | Self::MaterialBaseSurface
                | Self::MaterialRootSurface
                | Self::MaterialAccentSurface => 0.0,
                Self::ChildScaleRatio
                | Self::ChildTwistPerVertexRadians
                | Self::ChildOutwardOffsetRatio
                | Self::ChildSpawnExclusionProbability => unreachable!(),
            },
        }
    }

    fn default_value(self, context: &EffectTunerViewContext<'_>) -> f32 {
        match self.generation_parameter() {
            Some(parameter) => context
                .generation_config
                .parameter_spec(parameter)
                .default_value(),
            None => match self {
                Self::GlobalOpacity => context.material_config.default_opacity_clamped(),
                Self::MaterialHueStepPerLevel => context.material_config.hue_step_per_level,
                Self::MaterialSaturation => context.material_config.saturation,
                Self::MaterialLightness => context.material_config.lightness,
                Self::MaterialMetallic => context.material_config.metallic,
                Self::MaterialPerceptualRoughness => context.material_config.perceptual_roughness,
                Self::MaterialReflectance => context.material_config.reflectance,
                Self::MaterialCubeHueBias => context.material_config.cube_hue_bias,
                Self::MaterialTetrahedronHueBias => context.material_config.tetrahedron_hue_bias,
                Self::MaterialOctahedronHueBias => context.material_config.octahedron_hue_bias,
                Self::MaterialDodecahedronHueBias => context.material_config.dodecahedron_hue_bias,
                Self::MaterialAccentEveryNLevels => {
                    context.material_config.accent_every_n_levels as f32
                }
                Self::MaterialLevelLightnessShift => context.material_config.level_lightness_shift,
                Self::MaterialLevelSaturationShift => {
                    context.material_config.level_saturation_shift
                }
                Self::MaterialLevelMetallicShift => context.material_config.level_metallic_shift,
                Self::MaterialLevelRoughnessShift => context.material_config.level_roughness_shift,
                Self::MaterialLevelReflectanceShift => {
                    context.material_config.level_reflectance_shift
                }
                Self::ChildKind
                | Self::SpawnPlacementMode
                | Self::SpawnAddMode
                | Self::StageEnabled
                | Self::StageFloorEnabled
                | Self::StageBackdropEnabled
                | Self::MaterialSurfaceMode
                | Self::MaterialBaseSurface
                | Self::MaterialRootSurface
                | Self::MaterialAccentSurface => 0.0,
                Self::ChildScaleRatio
                | Self::ChildTwistPerVertexRadians
                | Self::ChildOutwardOffsetRatio
                | Self::ChildSpawnExclusionProbability => unreachable!(),
            },
        }
    }

    fn display_value(self, context: &EffectTunerViewContext<'_>) -> String {
        match self {
            Self::ChildKind => {
                shape_kind_value_text(context.generation_state.selected_shape_kind).to_string()
            }
            Self::SpawnPlacementMode => context
                .generation_state
                .spawn_placement_mode
                .plural_label()
                .to_string(),
            Self::SpawnAddMode => context.generation_state.spawn_add_mode.label().to_string(),
            Self::StageEnabled => boolean_value_text(context.stage_state.enabled).to_string(),
            Self::StageFloorEnabled => {
                boolean_value_text(context.stage_state.floor_enabled).to_string()
            }
            Self::StageBackdropEnabled => {
                boolean_value_text(context.stage_state.backdrop_enabled).to_string()
            }
            Self::MaterialSurfaceMode => {
                material_surface_mode_value_text(context.material_state.surface_mode).to_string()
            }
            Self::MaterialBaseSurface => {
                material_surface_family_value_text(context.material_state.base_surface).to_string()
            }
            Self::MaterialRootSurface => {
                material_surface_family_value_text(context.material_state.root_surface).to_string()
            }
            Self::MaterialAccentSurface => {
                material_surface_family_value_text(context.material_state.accent_surface)
                    .to_string()
            }
            Self::MaterialAccentEveryNLevels => {
                context.material_state.accent_every_n_levels.to_string()
            }
            Self::ChildScaleRatio
            | Self::ChildTwistPerVertexRadians
            | Self::ChildOutwardOffsetRatio
            | Self::ChildSpawnExclusionProbability
            | Self::GlobalOpacity
            | Self::MaterialHueStepPerLevel
            | Self::MaterialSaturation
            | Self::MaterialLightness
            | Self::MaterialMetallic
            | Self::MaterialPerceptualRoughness
            | Self::MaterialReflectance
            | Self::MaterialCubeHueBias
            | Self::MaterialTetrahedronHueBias
            | Self::MaterialOctahedronHueBias
            | Self::MaterialDodecahedronHueBias
            | Self::MaterialLevelLightnessShift
            | Self::MaterialLevelSaturationShift
            | Self::MaterialLevelMetallicShift
            | Self::MaterialLevelRoughnessShift
            | Self::MaterialLevelReflectanceShift => format!("{:.3}", self.value(context)),
        }
    }

    fn apply_numeric_input(&self, context: &mut EffectTunerEditContext<'_>, value: f32) -> bool {
        if !self.is_numeric() {
            return false;
        }

        let _ = self.set_value(context, value);
        true
    }

    fn adjust_value(
        self,
        context: &mut EffectTunerEditContext<'_>,
        direction: f32,
        shift_pressed: bool,
        alt_pressed: bool,
    ) {
        match self {
            Self::ChildKind => {
                context.generation_state.selected_shape_kind = cycle_shape_kind(
                    context.generation_state.selected_shape_kind,
                    direction as isize,
                );
            }
            Self::SpawnPlacementMode => {
                context.generation_state.spawn_placement_mode = cycle_spawn_placement_mode(
                    context.generation_state.spawn_placement_mode,
                    direction as isize,
                );
            }
            Self::SpawnAddMode => {
                context.generation_state.spawn_add_mode = cycle_spawn_add_mode(
                    context.generation_state.spawn_add_mode,
                    direction as isize,
                );
            }
            Self::StageEnabled => {
                context.stage_state.enabled = cycle_bool(context.stage_state.enabled, direction);
            }
            Self::StageFloorEnabled => {
                context.stage_state.floor_enabled =
                    cycle_bool(context.stage_state.floor_enabled, direction);
            }
            Self::StageBackdropEnabled => {
                context.stage_state.backdrop_enabled =
                    cycle_bool(context.stage_state.backdrop_enabled, direction);
            }
            Self::MaterialSurfaceMode => {
                context.material_state.surface_mode = cycle_material_surface_mode(
                    context.material_state.surface_mode,
                    direction as isize,
                );
            }
            Self::MaterialBaseSurface => {
                context.material_state.base_surface = cycle_material_surface_family(
                    context.material_state.base_surface,
                    direction as isize,
                );
            }
            Self::MaterialRootSurface => {
                context.material_state.root_surface = cycle_material_surface_family(
                    context.material_state.root_surface,
                    direction as isize,
                );
            }
            Self::MaterialAccentSurface => {
                context.material_state.accent_surface = cycle_material_surface_family(
                    context.material_state.accent_surface,
                    direction as isize,
                );
            }
            Self::ChildScaleRatio
            | Self::ChildTwistPerVertexRadians
            | Self::ChildOutwardOffsetRatio
            | Self::ChildSpawnExclusionProbability
            | Self::GlobalOpacity
            | Self::MaterialHueStepPerLevel
            | Self::MaterialSaturation
            | Self::MaterialLightness
            | Self::MaterialMetallic
            | Self::MaterialPerceptualRoughness
            | Self::MaterialReflectance
            | Self::MaterialCubeHueBias
            | Self::MaterialTetrahedronHueBias
            | Self::MaterialOctahedronHueBias
            | Self::MaterialDodecahedronHueBias
            | Self::MaterialAccentEveryNLevels
            | Self::MaterialLevelLightnessShift
            | Self::MaterialLevelSaturationShift
            | Self::MaterialLevelMetallicShift
            | Self::MaterialLevelRoughnessShift
            | Self::MaterialLevelReflectanceShift => {
                let current_value = self.value(&context.view());
                let next_value = current_value
                    + direction * self.adjustment_step(&context.view(), shift_pressed, alt_pressed);
                let _ = self.set_value(context, next_value);
            }
        }
    }

    fn reset_value(self, context: &mut EffectTunerEditContext<'_>) {
        match self {
            Self::ChildKind => {
                context.generation_state.selected_shape_kind =
                    context.generation_config.default_child_shape_kind;
            }
            Self::SpawnPlacementMode => {
                context.generation_state.spawn_placement_mode =
                    context.generation_config.default_spawn_placement_mode;
            }
            Self::SpawnAddMode => {
                context.generation_state.spawn_add_mode = SpawnAddMode::default();
            }
            Self::StageEnabled => {
                context.stage_state.enabled = context.stage_config.enabled;
            }
            Self::StageFloorEnabled => {
                context.stage_state.floor_enabled = context.stage_config.floor.enabled;
            }
            Self::StageBackdropEnabled => {
                context.stage_state.backdrop_enabled = context.stage_config.backdrop.enabled;
            }
            Self::MaterialSurfaceMode => {
                context.material_state.surface_mode = context.material_config.surface_mode;
            }
            Self::MaterialBaseSurface => {
                context.material_state.base_surface = context.material_config.base_surface;
            }
            Self::MaterialRootSurface => {
                context.material_state.root_surface = context.material_config.root_surface;
            }
            Self::MaterialAccentSurface => {
                context.material_state.accent_surface = context.material_config.accent_surface;
            }
            Self::ChildScaleRatio
            | Self::ChildTwistPerVertexRadians
            | Self::ChildOutwardOffsetRatio
            | Self::ChildSpawnExclusionProbability
            | Self::GlobalOpacity
            | Self::MaterialHueStepPerLevel
            | Self::MaterialSaturation
            | Self::MaterialLightness
            | Self::MaterialMetallic
            | Self::MaterialPerceptualRoughness
            | Self::MaterialReflectance
            | Self::MaterialCubeHueBias
            | Self::MaterialTetrahedronHueBias
            | Self::MaterialOctahedronHueBias
            | Self::MaterialDodecahedronHueBias
            | Self::MaterialAccentEveryNLevels
            | Self::MaterialLevelLightnessShift
            | Self::MaterialLevelSaturationShift
            | Self::MaterialLevelMetallicShift
            | Self::MaterialLevelRoughnessShift
            | Self::MaterialLevelReflectanceShift => {
                let default_value = self.default_value(&context.view());
                let _ = self.set_value(context, default_value);
            }
        }
    }

    fn status_message(self, context: &EffectTunerViewContext<'_>) -> String {
        match self {
            Self::ChildKind => {
                selected_child_shape_status_message(context.generation_state.selected_shape_kind)
            }
            Self::SpawnPlacementMode => {
                spawn_placement_mode_status_message(context.generation_state.spawn_placement_mode)
            }
            Self::SpawnAddMode => {
                spawn_add_mode_status_message(context.generation_state.spawn_add_mode)
            }
            Self::StageEnabled => {
                format!(
                    "Stage visibility: {}",
                    boolean_value_text(context.stage_state.enabled)
                )
            }
            Self::StageFloorEnabled => format!(
                "Stage floor: {}",
                boolean_value_text(context.stage_state.floor_enabled)
            ),
            Self::StageBackdropEnabled => format!(
                "Stage backdrop: {}",
                boolean_value_text(context.stage_state.backdrop_enabled)
            ),
            Self::ChildScaleRatio => {
                format!("Child scale ratio: {:.2}", self.value(context))
            }
            Self::ChildTwistPerVertexRadians => {
                let radians = self.value(context);
                format!(
                    "Child twist angle: {:.3} rad ({:.1} deg)",
                    radians,
                    radians * 180.0 / std::f32::consts::PI
                )
            }
            Self::ChildOutwardOffsetRatio => {
                format!(
                    "Child outward offset: {:.2}x child radius",
                    self.value(context).max(0.0)
                )
            }
            Self::ChildSpawnExclusionProbability => {
                format!(
                    "Global spawn exclusion probability: {:.0}%",
                    self.value(context).clamp(0.0, 1.0) * 100.0
                )
            }
            Self::GlobalOpacity => opacity_status_message(self.value(context)),
            Self::MaterialHueStepPerLevel => {
                format!(
                    "Material hue step per level: {:.1} deg",
                    self.value(context)
                )
            }
            Self::MaterialSaturation => format!("Material saturation: {:.3}", self.value(context)),
            Self::MaterialLightness => format!("Material lightness: {:.3}", self.value(context)),
            Self::MaterialMetallic => format!("Material metallic: {:.3}", self.value(context)),
            Self::MaterialPerceptualRoughness => {
                format!("Material roughness: {:.3}", self.value(context))
            }
            Self::MaterialReflectance => {
                format!("Material reflectance: {:.3}", self.value(context))
            }
            Self::MaterialCubeHueBias => {
                format!("Cube hue bias: {:.1} deg", self.value(context))
            }
            Self::MaterialTetrahedronHueBias => {
                format!("Tetrahedron hue bias: {:.1} deg", self.value(context))
            }
            Self::MaterialOctahedronHueBias => {
                format!("Octahedron hue bias: {:.1} deg", self.value(context))
            }
            Self::MaterialDodecahedronHueBias => {
                format!("Dodecahedron hue bias: {:.1} deg", self.value(context))
            }
            Self::MaterialSurfaceMode => format!(
                "Material surface mode: {}",
                material_surface_mode_value_text(context.material_state.surface_mode)
            ),
            Self::MaterialBaseSurface => format!(
                "Base surface family: {}",
                material_surface_family_value_text(context.material_state.base_surface)
            ),
            Self::MaterialRootSurface => format!(
                "Root surface family: {}",
                material_surface_family_value_text(context.material_state.root_surface)
            ),
            Self::MaterialAccentSurface => format!(
                "Accent surface family: {}",
                material_surface_family_value_text(context.material_state.accent_surface)
            ),
            Self::MaterialAccentEveryNLevels => {
                let cadence = context.material_state.accent_every_n_levels;
                if cadence == 0 {
                    "Accent surface cadence: disabled".to_string()
                } else {
                    format!("Accent surface every {cadence} levels")
                }
            }
            Self::MaterialLevelLightnessShift => {
                format!("Level lightness shift: {:.3}", self.value(context))
            }
            Self::MaterialLevelSaturationShift => {
                format!("Level saturation shift: {:.3}", self.value(context))
            }
            Self::MaterialLevelMetallicShift => {
                format!("Level metallic shift: {:.3}", self.value(context))
            }
            Self::MaterialLevelRoughnessShift => {
                format!("Level roughness shift: {:.3}", self.value(context))
            }
            Self::MaterialLevelReflectanceShift => {
                format!("Level reflectance shift: {:.3}", self.value(context))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectTunerParameter {
    Effect(EffectNumericParameter),
    Scene(EffectTunerSceneParameter),
}

impl EffectTunerParameter {
    const ALL: [Self; 55] = [
        Self::Effect(EffectNumericParameter::WavefolderGain),
        Self::Effect(EffectNumericParameter::WavefolderModulus),
        Self::Effect(EffectNumericParameter::LensStrength),
        Self::Effect(EffectNumericParameter::LensRadialK2),
        Self::Effect(EffectNumericParameter::LensRadialK3),
        Self::Effect(EffectNumericParameter::LensCenterX),
        Self::Effect(EffectNumericParameter::LensCenterY),
        Self::Effect(EffectNumericParameter::LensScaleX),
        Self::Effect(EffectNumericParameter::LensScaleY),
        Self::Effect(EffectNumericParameter::LensTangentialX),
        Self::Effect(EffectNumericParameter::LensTangentialY),
        Self::Effect(EffectNumericParameter::LensZoom),
        Self::Effect(EffectNumericParameter::LensChromaticAberration),
        Self::Effect(EffectNumericParameter::GaussianBlurSigma),
        Self::Effect(EffectNumericParameter::GaussianBlurRadius),
        Self::Effect(EffectNumericParameter::BloomThreshold),
        Self::Effect(EffectNumericParameter::BloomIntensity),
        Self::Effect(EffectNumericParameter::BloomRadius),
        Self::Effect(EffectNumericParameter::EdgeStrength),
        Self::Effect(EffectNumericParameter::EdgeThreshold),
        Self::Effect(EffectNumericParameter::EdgeMix),
        Self::Effect(EffectNumericParameter::EdgeColorR),
        Self::Effect(EffectNumericParameter::EdgeColorG),
        Self::Effect(EffectNumericParameter::EdgeColorB),
        Self::Scene(EffectTunerSceneParameter::ChildKind),
        Self::Scene(EffectTunerSceneParameter::SpawnPlacementMode),
        Self::Scene(EffectTunerSceneParameter::SpawnAddMode),
        Self::Scene(EffectTunerSceneParameter::ChildScaleRatio),
        Self::Scene(EffectTunerSceneParameter::ChildTwistPerVertexRadians),
        Self::Scene(EffectTunerSceneParameter::ChildOutwardOffsetRatio),
        Self::Scene(EffectTunerSceneParameter::ChildSpawnExclusionProbability),
        Self::Scene(EffectTunerSceneParameter::StageEnabled),
        Self::Scene(EffectTunerSceneParameter::StageFloorEnabled),
        Self::Scene(EffectTunerSceneParameter::StageBackdropEnabled),
        Self::Scene(EffectTunerSceneParameter::GlobalOpacity),
        Self::Scene(EffectTunerSceneParameter::MaterialHueStepPerLevel),
        Self::Scene(EffectTunerSceneParameter::MaterialSaturation),
        Self::Scene(EffectTunerSceneParameter::MaterialLightness),
        Self::Scene(EffectTunerSceneParameter::MaterialMetallic),
        Self::Scene(EffectTunerSceneParameter::MaterialPerceptualRoughness),
        Self::Scene(EffectTunerSceneParameter::MaterialReflectance),
        Self::Scene(EffectTunerSceneParameter::MaterialCubeHueBias),
        Self::Scene(EffectTunerSceneParameter::MaterialTetrahedronHueBias),
        Self::Scene(EffectTunerSceneParameter::MaterialOctahedronHueBias),
        Self::Scene(EffectTunerSceneParameter::MaterialDodecahedronHueBias),
        Self::Scene(EffectTunerSceneParameter::MaterialSurfaceMode),
        Self::Scene(EffectTunerSceneParameter::MaterialBaseSurface),
        Self::Scene(EffectTunerSceneParameter::MaterialRootSurface),
        Self::Scene(EffectTunerSceneParameter::MaterialAccentSurface),
        Self::Scene(EffectTunerSceneParameter::MaterialAccentEveryNLevels),
        Self::Scene(EffectTunerSceneParameter::MaterialLevelLightnessShift),
        Self::Scene(EffectTunerSceneParameter::MaterialLevelSaturationShift),
        Self::Scene(EffectTunerSceneParameter::MaterialLevelMetallicShift),
        Self::Scene(EffectTunerSceneParameter::MaterialLevelRoughnessShift),
        Self::Scene(EffectTunerSceneParameter::MaterialLevelReflectanceShift),
    ];

    pub(crate) fn all() -> &'static [Self] {
        &Self::ALL
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Effect(parameter) => parameter.label(),
            Self::Scene(parameter) => parameter.label(),
        }
    }

    pub(crate) fn short_label(self) -> &'static str {
        match self {
            Self::Effect(parameter) => parameter.short_label(),
            Self::Scene(parameter) => parameter.short_label(),
        }
    }

    pub(crate) fn group_label(self) -> &'static str {
        match self {
            Self::Effect(parameter) => parameter.effect_group().compact_label(),
            Self::Scene(parameter) => parameter.group_label(),
        }
    }

    fn effect_group(self) -> Option<EffectGroup> {
        match self {
            Self::Effect(parameter) => Some(parameter.effect_group()),
            Self::Scene(_) => None,
        }
    }

    fn supports_lfo(self) -> bool {
        matches!(self, Self::Effect(_))
    }

    fn value_accepts_numeric_input(self) -> bool {
        match self {
            Self::Effect(_) => true,
            Self::Scene(parameter) => parameter.is_numeric(),
        }
    }

    fn adjustment_step(
        self,
        context: &EffectTunerViewContext<'_>,
        shift_pressed: bool,
        alt_pressed: bool,
    ) -> f32 {
        match self {
            Self::Effect(parameter) => parameter.adjustment_step(shift_pressed, alt_pressed),
            Self::Scene(parameter) => {
                parameter.adjustment_step(context, shift_pressed, alt_pressed)
            }
        }
    }

    fn default_lfo_amplitude(self) -> f32 {
        match self {
            Self::Effect(parameter) => parameter.default_lfo_amplitude(),
            Self::Scene(_) => 0.0,
        }
    }

    fn display_value(
        self,
        effects: &EffectsConfig,
        context: &EffectTunerViewContext<'_>,
    ) -> String {
        match self {
            Self::Effect(parameter) => parameter.display_value(effects),
            Self::Scene(parameter) => parameter.display_value(context),
        }
    }

    fn apply_numeric_value_input(
        self,
        effects: &mut EffectsConfig,
        context: &mut EffectTunerEditContext<'_>,
        value: f32,
    ) -> bool {
        match self {
            Self::Effect(parameter) => {
                parameter.set_value(effects, value);
                true
            }
            Self::Scene(parameter) => parameter.apply_numeric_input(context, value),
        }
    }

    fn adjust_value(
        self,
        effects: &mut EffectsConfig,
        context: &mut EffectTunerEditContext<'_>,
        direction: f32,
        modifiers: AdjustmentModifiers,
    ) {
        match self {
            Self::Effect(parameter) => {
                let current_value = parameter.value(effects);
                let next_value = current_value
                    + direction
                        * parameter.adjustment_step(modifiers.shift_pressed, modifiers.alt_pressed);
                parameter.set_value(effects, next_value);
            }
            Self::Scene(parameter) => parameter.adjust_value(
                context,
                direction,
                modifiers.shift_pressed,
                modifiers.alt_pressed,
            ),
        }
    }

    fn reset_value(
        self,
        defaults: &EffectsConfig,
        effects: &mut EffectsConfig,
        context: &mut EffectTunerEditContext<'_>,
    ) {
        match self {
            Self::Effect(parameter) => parameter.set_value(effects, parameter.value(defaults)),
            Self::Scene(parameter) => parameter.reset_value(context),
        }
    }

    fn status_message(
        self,
        effects: &EffectsConfig,
        context: &EffectTunerViewContext<'_>,
    ) -> String {
        match self {
            Self::Effect(parameter) => format!(
                "{} = {}",
                parameter.label(),
                parameter.display_value(effects)
            ),
            Self::Scene(parameter) => parameter.status_message(context),
        }
    }
}

#[derive(Default, Clone)]
struct NumericEntryBuffer {
    buffer: String,
}

impl NumericEntryBuffer {
    fn displayed_text(&self) -> Option<&str> {
        (!self.buffer.is_empty()).then_some(self.buffer.as_str())
    }

    fn push(&mut self, character: char) -> bool {
        match character {
            '0'..='9' => {
                self.buffer.push(character);
                true
            }
            '.' => {
                if self.buffer.contains('.') {
                    return false;
                }
                if self.buffer.is_empty() {
                    self.buffer.push('0');
                } else if matches!(self.buffer.as_str(), "-" | "+") {
                    self.buffer.push('0');
                }
                self.buffer.push('.');
                true
            }
            '-' | '+' => {
                if self.buffer.is_empty() {
                    self.buffer.push(character);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn backspace(&mut self) -> bool {
        self.buffer.pop().is_some()
    }

    fn parsed_value(&self) -> Option<f32> {
        match self.buffer.as_str() {
            "" | "-" | "+" | "." | "-." | "+." => None,
            value => value.parse::<f32>().ok(),
        }
    }

    fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[derive(Resource, Clone)]
pub(crate) struct EffectTunerState {
    defaults: EffectsConfig,
    current: EffectsConfig,
    lfos: Vec<ParameterLfo>,
    selected_index: usize,
    page_mode: EffectTunerPageMode,
    edit_mode: EffectEditMode,
    numeric_entry: NumericEntryBuffer,
    pinned: bool,
    visible_until_secs: f32,
    select_previous_hold: HoldRepeatState,
    select_next_hold: HoldRepeatState,
    decrease_hold: HoldRepeatState,
    increase_hold: HoldRepeatState,
}

impl EffectTunerState {
    pub(crate) fn from_config(effects_config: &EffectsConfig) -> Self {
        Self {
            defaults: effects_config.clone(),
            current: effects_config.clone(),
            lfos: default_lfos(),
            selected_index: 0,
            page_mode: EffectTunerPageMode::Compact,
            edit_mode: EffectEditMode::Value,
            numeric_entry: NumericEntryBuffer::default(),
            pinned: false,
            visible_until_secs: 0.0,
            select_previous_hold: HoldRepeatState::default(),
            select_next_hold: HoldRepeatState::default(),
            decrease_hold: HoldRepeatState::default(),
            increase_hold: HoldRepeatState::default(),
        }
    }

    pub(crate) fn selected_parameter(&self) -> EffectTunerParameter {
        EffectTunerParameter::all()[self.selected_index]
    }

    pub(crate) fn page_mode(&self) -> EffectTunerPageMode {
        self.page_mode
    }

    pub(crate) fn selected_effect_group(&self) -> Option<EffectGroup> {
        self.selected_parameter().effect_group()
    }

    pub(crate) fn active_field(&self) -> EffectOverlayField {
        self.displayed_edit_mode().overlay_field()
    }

    pub(crate) fn is_visible(&self, now_secs: f32) -> bool {
        self.pinned || now_secs <= self.visible_until_secs
    }

    pub(crate) fn has_active_lfos(&self) -> bool {
        self.lfos.iter().copied().any(ParameterLfo::is_active)
    }

    pub(crate) fn runtime_snapshot(&self) -> EffectRuntimeSnapshot {
        EffectRuntimeSnapshot {
            current: self.current.clone(),
            lfos: self.lfos.clone(),
        }
    }

    pub(crate) fn apply_runtime_snapshot(&mut self, snapshot: &EffectRuntimeSnapshot) {
        self.current = snapshot.current.clone();
        self.lfos = default_lfos();
        for (target, source) in self.lfos.iter_mut().zip(snapshot.lfos.iter().copied()) {
            *target = source;
        }
        self.selected_index = 0;
        self.page_mode = EffectTunerPageMode::Compact;
        self.edit_mode = EffectEditMode::Value;
        self.clear_numeric_entry();
        self.reset_hold_states();
    }

    pub(crate) fn evaluated_effects(&self, now_secs: f32) -> EffectsConfig {
        let mut effects = self.current.clone();

        for (index, parameter) in EffectNumericParameter::all().iter().copied().enumerate() {
            let lfo = self.lfos[index];
            if !lfo.is_active() {
                continue;
            }

            let base_value = parameter.value(&self.current);
            let lfo_offset = lfo.amplitude
                * lfo
                    .shape
                    .sample(now_secs * lfo.frequency_hz, index as u32 + 1);
            parameter.set_value(&mut effects, base_value + lfo_offset);
        }

        effects
    }

    pub(crate) fn overlay_snapshot(
        &self,
        context: &EffectTunerViewContext<'_>,
        now_secs: f32,
    ) -> EffectTunerOverlaySnapshot {
        let parameter = self.selected_parameter();
        let live_effects = self.evaluated_effects(now_secs);
        let (effect_state_text, effect_state_emphasized) = match parameter.effect_group() {
            Some(effect) => {
                let enabled = effect.is_enabled(&self.current);
                (if enabled { "ON" } else { "OFF" }, enabled)
            }
            None => ("VAL", false),
        };
        let (lfo_state_text, lfo_state_emphasized, amplitude_text, frequency_text, shape_text) =
            if parameter.supports_lfo() {
                let lfo = self.selected_lfo();
                (
                    if lfo.enabled { "ON" } else { "OFF" },
                    lfo.enabled,
                    self.overlay_numeric_text(
                        EffectOverlayField::LfoAmplitude,
                        format!("{:.3}", lfo.amplitude),
                    ),
                    self.overlay_numeric_text(
                        EffectOverlayField::LfoFrequency,
                        format!("{:.3}", lfo.frequency_hz),
                    ),
                    lfo.shape.label(),
                )
            } else {
                ("--", false, "--".to_string(), "--".to_string(), "--")
            };

        EffectTunerOverlaySnapshot {
            pinned: self.pinned,
            effect_label: parameter.group_label(),
            effect_state_text,
            effect_state_emphasized,
            parameter_label: parameter.short_label(),
            value_text: self.overlay_numeric_text(
                EffectOverlayField::Value,
                parameter.display_value(&self.current, context),
            ),
            live_value_text: parameter.display_value(&live_effects, context),
            lfo_state_text,
            lfo_state_emphasized,
            amplitude_text,
            frequency_text,
            shape_text,
            active_field: self.active_field(),
        }
    }

    pub(crate) fn list_overlay_snapshot(
        &self,
        context: &EffectTunerViewContext<'_>,
        now_secs: f32,
        visible_rows: usize,
    ) -> EffectTunerListOverlaySnapshot {
        let live_effects = self.evaluated_effects(now_secs);
        let (window_start, window_end) = self.selection_window_bounds(visible_rows.max(1));
        let rows = (window_start..window_end)
            .map(|index| {
                self.list_row_snapshot(EffectTunerParameter::all()[index], context, &live_effects)
            })
            .collect();

        EffectTunerListOverlaySnapshot {
            pinned: self.pinned,
            total_parameters: EffectTunerParameter::all().len(),
            window_start,
            rows,
            detail: self.overlay_snapshot(context, now_secs),
        }
    }

    pub(crate) fn edit_mode_label(&self) -> &'static str {
        self.displayed_edit_mode().label()
    }

    pub(crate) fn open_page(&mut self, now_secs: f32) {
        self.page_mode = EffectTunerPageMode::Compact;
        self.pinned = true;
        self.note_interaction(now_secs);
    }

    pub(crate) fn show_list_page(&mut self, now_secs: f32) {
        self.page_mode = EffectTunerPageMode::List;
        self.note_interaction(now_secs);
    }

    pub(crate) fn close_page(&mut self) {
        self.page_mode = EffectTunerPageMode::Compact;
        self.pinned = false;
        self.visible_until_secs = 0.0;
        self.clear_numeric_entry();
        self.reset_hold_states();
    }

    pub(crate) fn toggle_selected_effect(&mut self, now_secs: f32) -> Option<bool> {
        self.clear_numeric_entry();
        let effect = self.selected_effect_group()?;
        let next_enabled = !effect.is_enabled(&self.current);
        effect.set_enabled(&mut self.current, next_enabled);
        self.note_interaction(now_secs);
        Some(next_enabled)
    }

    pub(crate) fn toggle_selected_lfo(&mut self, now_secs: f32) -> Option<bool> {
        if !self.selected_parameter().supports_lfo() {
            return None;
        }

        self.clear_numeric_entry();
        let lfo = self.selected_lfo_mut();
        lfo.enabled = !lfo.enabled;
        let enabled = lfo.enabled;
        self.note_interaction(now_secs);
        Some(enabled)
    }

    pub(crate) fn step_edit_mode(&mut self, direction: isize, now_secs: f32) -> bool {
        self.clear_numeric_entry();
        let previous_mode = self.edit_mode;
        let mut next_mode = self.edit_mode;
        for _ in 0..4 {
            next_mode = next_mode.step(direction);
            if self.mode_supported_for_parameter(next_mode, self.selected_parameter()) {
                break;
            }
        }
        self.edit_mode = next_mode;
        self.note_interaction(now_secs);
        self.edit_mode != previous_mode
    }

    pub(crate) fn step_selection(
        &mut self,
        direction: isize,
        input: HoldInput,
        now_secs: f32,
    ) -> bool {
        let hold = if direction < 0 {
            &mut self.select_previous_hold
        } else {
            &mut self.select_next_hold
        };

        if hold.update_with_input(input, HOLD_DELAY_SECS, REPEAT_INTERVAL_SECS) {
            self.cycle_selection(direction, now_secs);
            true
        } else {
            false
        }
    }

    pub(crate) fn step_adjustment(
        &mut self,
        direction: f32,
        input: HoldInput,
        modifiers: AdjustmentModifiers,
        context: &mut EffectTunerEditContext<'_>,
        now_secs: f32,
    ) -> bool {
        let hold = if direction < 0.0 {
            &mut self.decrease_hold
        } else {
            &mut self.increase_hold
        };

        if hold.update_with_input(input, HOLD_DELAY_SECS, REPEAT_INTERVAL_SECS) {
            self.adjust_selected(direction, modifiers, context, now_secs);
            true
        } else {
            false
        }
    }

    pub(crate) fn append_numeric_input(
        &mut self,
        character: char,
        context: &mut EffectTunerEditContext<'_>,
        now_secs: f32,
    ) -> bool {
        if !self.active_field_accepts_numeric_entry() {
            return false;
        }

        if !self.numeric_entry.push(character) {
            return false;
        }

        self.apply_numeric_entry_to_selected(context);
        self.note_interaction(now_secs);
        true
    }

    pub(crate) fn backspace_numeric_input(
        &mut self,
        context: &mut EffectTunerEditContext<'_>,
        now_secs: f32,
    ) -> bool {
        if !self.numeric_entry.backspace() {
            return false;
        }

        self.apply_numeric_entry_to_selected(context);
        self.note_interaction(now_secs);
        true
    }

    pub(crate) fn reset_selected(
        &mut self,
        context: &mut EffectTunerEditContext<'_>,
        now_secs: f32,
    ) {
        self.clear_numeric_entry();
        let parameter = self.selected_parameter();
        match self.displayed_edit_mode() {
            EffectEditMode::Value => {
                parameter.reset_value(&self.defaults, &mut self.current, context);
            }
            EffectEditMode::LfoAmplitude => {
                self.selected_lfo_mut().amplitude = parameter.default_lfo_amplitude();
            }
            EffectEditMode::LfoFrequency => {
                self.selected_lfo_mut().frequency_hz = DEFAULT_LFO_FREQUENCY_HZ;
            }
            EffectEditMode::LfoShape => {
                self.selected_lfo_mut().shape = LfoShape::Sine;
            }
        }
        self.note_interaction(now_secs);
    }

    pub(crate) fn reset_all(&mut self, context: &mut EffectTunerEditContext<'_>, now_secs: f32) {
        self.current = self.defaults.clone();
        self.lfos = default_lfos();
        self.edit_mode = EffectEditMode::Value;
        self.clear_numeric_entry();
        for parameter in EffectTunerSceneParameter::all() {
            parameter.reset_value(context);
        }
        self.note_interaction(now_secs);
    }

    pub(crate) fn selected_status_message(
        &self,
        context: &EffectTunerViewContext<'_>,
        now_secs: f32,
    ) -> String {
        let parameter = self.selected_parameter();
        let live_effects = self.evaluated_effects(now_secs);
        match self.displayed_edit_mode() {
            EffectEditMode::Value => match parameter {
                EffectTunerParameter::Effect(effect_parameter) => format!(
                    "{} = {} (live {})",
                    effect_parameter.label(),
                    effect_parameter.display_value(&self.current),
                    effect_parameter.display_value(&live_effects)
                ),
                EffectTunerParameter::Scene(_) => parameter.status_message(&self.current, context),
            },
            EffectEditMode::LfoAmplitude => {
                let lfo = self.selected_lfo();
                format!("{} lfo amplitude = {:.3}", parameter.label(), lfo.amplitude)
            }
            EffectEditMode::LfoFrequency => {
                let lfo = self.selected_lfo();
                format!(
                    "{} lfo frequency = {:.3}Hz",
                    parameter.label(),
                    lfo.frequency_hz
                )
            }
            EffectEditMode::LfoShape => {
                let lfo = self.selected_lfo();
                format!("{} lfo shape = {}", parameter.label(), lfo.shape.label())
            }
        }
    }

    fn note_interaction(&mut self, now_secs: f32) {
        self.visible_until_secs = now_secs + OVERLAY_HOLD_SECS;
    }

    fn cycle_selection(&mut self, direction: isize, now_secs: f32) {
        self.clear_numeric_entry();
        let parameter_count = EffectTunerParameter::all().len() as isize;
        let next_index =
            (self.selected_index as isize + direction).rem_euclid(parameter_count) as usize;
        self.selected_index = next_index;
        self.coerce_edit_mode_for_selected();
        self.note_interaction(now_secs);
    }

    fn adjust_selected(
        &mut self,
        direction: f32,
        modifiers: AdjustmentModifiers,
        context: &mut EffectTunerEditContext<'_>,
        now_secs: f32,
    ) {
        self.clear_numeric_entry();
        let parameter = self.selected_parameter();
        match self.displayed_edit_mode() {
            EffectEditMode::Value => {
                parameter.adjust_value(&mut self.current, context, direction, modifiers);
            }
            EffectEditMode::LfoAmplitude => {
                let step = parameter.adjustment_step(
                    &context.view(),
                    modifiers.shift_pressed,
                    modifiers.alt_pressed,
                );
                let lfo = self.selected_lfo_mut();
                lfo.amplitude = (lfo.amplitude + direction * step).max(0.0);
            }
            EffectEditMode::LfoFrequency => {
                let mut step = LFO_FREQUENCY_STEP_HZ;
                if modifiers.shift_pressed {
                    step *= 10.0;
                }
                if modifiers.alt_pressed {
                    step *= 0.1;
                }
                let lfo = self.selected_lfo_mut();
                lfo.frequency_hz = (lfo.frequency_hz + direction * step).max(0.0);
            }
            EffectEditMode::LfoShape => {
                let lfo = self.selected_lfo_mut();
                lfo.shape = lfo.shape.cycle(direction);
            }
        }
        self.note_interaction(now_secs);
    }

    fn selected_lfo(&self) -> ParameterLfo {
        let index = self
            .selected_effect_parameter()
            .and_then(effect_parameter_index)
            .expect("selected parameter should support LFOs");
        self.lfos[index]
    }

    fn selected_lfo_mut(&mut self) -> &mut ParameterLfo {
        let index = self
            .selected_effect_parameter()
            .and_then(effect_parameter_index)
            .expect("selected parameter should support LFOs");
        &mut self.lfos[index]
    }

    fn overlay_numeric_text(&self, field: EffectOverlayField, fallback: String) -> String {
        if self.active_field() == field {
            if let Some(buffer) = self.numeric_entry.displayed_text() {
                return buffer.to_string();
            }
        }

        fallback
    }

    fn apply_numeric_entry_to_selected(
        &mut self,
        context: &mut EffectTunerEditContext<'_>,
    ) -> bool {
        let Some(value) = self.numeric_entry.parsed_value() else {
            return false;
        };

        let parameter = self.selected_parameter();
        match self.displayed_edit_mode() {
            EffectEditMode::Value => {
                return parameter.apply_numeric_value_input(&mut self.current, context, value);
            }
            EffectEditMode::LfoAmplitude => self.selected_lfo_mut().amplitude = value.max(0.0),
            EffectEditMode::LfoFrequency => self.selected_lfo_mut().frequency_hz = value.max(0.0),
            EffectEditMode::LfoShape => return false,
        }

        true
    }

    fn selected_effect_parameter(&self) -> Option<EffectNumericParameter> {
        match self.selected_parameter() {
            EffectTunerParameter::Effect(parameter) => Some(parameter),
            EffectTunerParameter::Scene(_) => None,
        }
    }

    fn lfo_for_parameter(&self, parameter: EffectTunerParameter) -> Option<ParameterLfo> {
        let EffectTunerParameter::Effect(parameter) = parameter else {
            return None;
        };
        let index = effect_parameter_index(parameter)?;
        self.lfos.get(index).copied()
    }

    fn mode_supported_for_parameter(
        &self,
        edit_mode: EffectEditMode,
        parameter: EffectTunerParameter,
    ) -> bool {
        matches!(edit_mode, EffectEditMode::Value) || parameter.supports_lfo()
    }

    fn displayed_edit_mode(&self) -> EffectEditMode {
        if self.mode_supported_for_parameter(self.edit_mode, self.selected_parameter()) {
            self.edit_mode
        } else {
            EffectEditMode::Value
        }
    }

    fn active_field_accepts_numeric_entry(&self) -> bool {
        match self.displayed_edit_mode() {
            EffectEditMode::Value => self.selected_parameter().value_accepts_numeric_input(),
            EffectEditMode::LfoAmplitude | EffectEditMode::LfoFrequency => true,
            EffectEditMode::LfoShape => false,
        }
    }

    fn coerce_edit_mode_for_selected(&mut self) {
        if !self.mode_supported_for_parameter(self.edit_mode, self.selected_parameter()) {
            self.edit_mode = EffectEditMode::Value;
        }
    }

    fn clear_numeric_entry(&mut self) {
        self.numeric_entry.clear();
    }

    fn reset_hold_states(&mut self) {
        self.select_previous_hold.reset();
        self.select_next_hold.reset();
        self.decrease_hold.reset();
        self.increase_hold.reset();
    }

    fn selection_window_bounds(&self, visible_rows: usize) -> (usize, usize) {
        let total = EffectTunerParameter::all().len();
        if total <= visible_rows {
            return (0, total);
        }

        let half_window = visible_rows / 2;
        let max_start = total - visible_rows;
        let window_start = self
            .selected_index
            .saturating_sub(half_window)
            .min(max_start);
        let window_end = (window_start + visible_rows).min(total);
        (window_start, window_end)
    }

    fn list_row_snapshot(
        &self,
        parameter: EffectTunerParameter,
        context: &EffectTunerViewContext<'_>,
        live_effects: &EffectsConfig,
    ) -> EffectTunerListRowSnapshot {
        let selected = parameter == self.selected_parameter();
        let (effect_state_text, effect_state_emphasized) = match parameter.effect_group() {
            Some(effect) => {
                let enabled = effect.is_enabled(&self.current);
                (if enabled { "ON" } else { "OFF" }, enabled)
            }
            None => ("VAL", false),
        };
        let (lfo_state_text, lfo_state_emphasized) = match self.lfo_for_parameter(parameter) {
            Some(lfo) => (if lfo.enabled { "ON" } else { "OFF" }, lfo.enabled),
            None => ("--", false),
        };

        EffectTunerListRowSnapshot {
            effect_label: parameter.group_label(),
            effect_state_text,
            effect_state_emphasized,
            parameter_label: parameter.short_label(),
            value_text: if selected {
                self.overlay_numeric_text(
                    EffectOverlayField::Value,
                    parameter.display_value(&self.current, context),
                )
            } else {
                parameter.display_value(&self.current, context)
            },
            live_value_text: parameter.display_value(live_effects, context),
            lfo_state_text,
            lfo_state_emphasized,
            selected,
            active_field: selected.then_some(self.active_field()),
        }
    }
}

fn cycle_from_all<T>(all: &[T], current: T, direction: isize) -> T
where
    T: Copy + Eq,
{
    let current_index = all
        .iter()
        .position(|candidate| *candidate == current)
        .unwrap_or(0) as isize;
    let next_index = (current_index + direction).rem_euclid(all.len() as isize) as usize;
    all[next_index]
}

fn cycle_shape_kind(current: ShapeKind, direction: isize) -> ShapeKind {
    const ALL: [ShapeKind; 4] = [
        ShapeKind::Cube,
        ShapeKind::Tetrahedron,
        ShapeKind::Octahedron,
        ShapeKind::Dodecahedron,
    ];
    cycle_from_all(&ALL, current, direction)
}

fn cycle_spawn_placement_mode(current: SpawnPlacementMode, direction: isize) -> SpawnPlacementMode {
    const ALL: [SpawnPlacementMode; 3] = [
        SpawnPlacementMode::Vertex,
        SpawnPlacementMode::Edge,
        SpawnPlacementMode::Face,
    ];
    cycle_from_all(&ALL, current, direction)
}

fn cycle_spawn_add_mode(current: SpawnAddMode, direction: isize) -> SpawnAddMode {
    const ALL: [SpawnAddMode; 2] = [SpawnAddMode::Single, SpawnAddMode::FillLevel];
    cycle_from_all(&ALL, current, direction)
}

fn cycle_bool(current: bool, direction: f32) -> bool {
    cycle_from_all(&[false, true], current, direction as isize)
}

fn cycle_material_surface_mode(
    current: MaterialSurfaceMode,
    direction: isize,
) -> MaterialSurfaceMode {
    const ALL: [MaterialSurfaceMode; 2] =
        [MaterialSurfaceMode::Legacy, MaterialSurfaceMode::Procedural];
    cycle_from_all(&ALL, current, direction)
}

fn cycle_material_surface_family(
    current: MaterialSurfaceFamily,
    direction: isize,
) -> MaterialSurfaceFamily {
    const ALL: [MaterialSurfaceFamily; 6] = [
        MaterialSurfaceFamily::Legacy,
        MaterialSurfaceFamily::Matte,
        MaterialSurfaceFamily::Satin,
        MaterialSurfaceFamily::Glossy,
        MaterialSurfaceFamily::Metal,
        MaterialSurfaceFamily::Frosted,
    ];
    cycle_from_all(&ALL, current, direction)
}

fn shape_kind_value_text(kind: ShapeKind) -> &'static str {
    match kind {
        ShapeKind::Cube => "cube",
        ShapeKind::Tetrahedron => "tetrahedron",
        ShapeKind::Octahedron => "octahedron",
        ShapeKind::Dodecahedron => "dodecahedron",
    }
}

fn boolean_value_text(enabled: bool) -> &'static str {
    if enabled {
        "on"
    } else {
        "off"
    }
}

fn material_surface_mode_value_text(mode: MaterialSurfaceMode) -> &'static str {
    match mode {
        MaterialSurfaceMode::Legacy => "legacy",
        MaterialSurfaceMode::Procedural => "procedural",
    }
}

fn material_surface_family_value_text(family: MaterialSurfaceFamily) -> &'static str {
    match family {
        MaterialSurfaceFamily::Legacy => "legacy",
        MaterialSurfaceFamily::Matte => "matte",
        MaterialSurfaceFamily::Satin => "satin",
        MaterialSurfaceFamily::Glossy => "glossy",
        MaterialSurfaceFamily::Metal => "metal",
        MaterialSurfaceFamily::Frosted => "frosted",
    }
}

fn default_lfos() -> Vec<ParameterLfo> {
    EffectNumericParameter::all()
        .iter()
        .copied()
        .map(ParameterLfo::default_for)
        .collect()
}

fn effect_parameter_index(parameter: EffectNumericParameter) -> Option<usize> {
    EffectNumericParameter::all()
        .iter()
        .position(|candidate| *candidate == parameter)
}

#[cfg(test)]
mod tests {
    use crate::config::{
        EffectGroup, EffectsConfig, GenerationConfig, MaterialConfig, MaterialSurfaceMode,
        StageConfig,
    };
    use crate::effect_tuner::lfo::{LfoShape, DEFAULT_LFO_FREQUENCY_HZ};
    use crate::effect_tuner::metadata::{EffectEditMode, EffectOverlayField};
    use crate::scene::{GenerationState, MaterialState, StageState};

    use super::{
        EffectTunerEditContext, EffectTunerPageMode, EffectTunerParameter,
        EffectTunerSceneParameter, EffectTunerState, EffectTunerViewContext, HoldInput,
    };

    fn default_scene_state() -> (
        GenerationConfig,
        GenerationState,
        MaterialConfig,
        MaterialState,
        StageConfig,
        StageState,
    ) {
        let generation_config = GenerationConfig::default();
        let generation_state = GenerationState::from_config(&generation_config);
        let material_config = MaterialConfig::default();
        let material_state = MaterialState::from_config(&material_config);
        let stage_config = StageConfig::default();
        let stage_state = StageState::from_config(&stage_config);
        (
            generation_config,
            generation_state,
            material_config,
            material_state,
            stage_config,
            stage_state,
        )
    }

    fn view_context<'a>(
        generation_config: &'a GenerationConfig,
        generation_state: &'a GenerationState,
        material_config: &'a MaterialConfig,
        material_state: &'a MaterialState,
        _stage_config: &'a StageConfig,
        stage_state: &'a StageState,
    ) -> EffectTunerViewContext<'a> {
        EffectTunerViewContext {
            generation_config,
            generation_state,
            material_config,
            material_state,
            stage_state,
        }
    }

    fn edit_context<'a>(
        generation_config: &'a GenerationConfig,
        generation_state: &'a mut GenerationState,
        material_config: &'a MaterialConfig,
        material_state: &'a mut MaterialState,
        stage_config: &'a StageConfig,
        stage_state: &'a mut StageState,
    ) -> EffectTunerEditContext<'a> {
        EffectTunerEditContext {
            generation_config,
            generation_state,
            material_config,
            material_state,
            stage_config,
            stage_state,
        }
    }

    fn select_parameter(effect_tuner: &mut EffectTunerState, parameter: EffectTunerParameter) {
        effect_tuner.selected_index = EffectTunerParameter::all()
            .iter()
            .position(|candidate| *candidate == parameter)
            .expect("parameter should exist in the F2 parameter list");
    }

    #[test]
    fn selected_effect_matches_parameter_group() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        effect_tuner.selected_index = 17;

        assert_eq!(
            effect_tuner.selected_effect_group(),
            Some(EffectGroup::Bloom)
        );
    }

    #[test]
    fn toggle_selected_lfo_updates_enabled_state() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        effect_tuner.selected_index = 13;

        let enabled = effect_tuner.toggle_selected_lfo(1.0);

        assert_eq!(enabled, Some(true));
        assert!(effect_tuner.selected_lfo().enabled);
    }

    #[test]
    fn evaluated_effects_apply_lfo_offset() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        effect_tuner.selected_index = 0;
        effect_tuner.current.color_wavefolder.gain = 2.0;
        let lfo = effect_tuner.selected_lfo_mut();
        lfo.enabled = true;
        lfo.shape = LfoShape::Sine;
        lfo.amplitude = 0.5;
        lfo.frequency_hz = 1.0;

        let evaluated = effect_tuner.evaluated_effects(0.25);

        assert!((evaluated.color_wavefolder.gain - 2.5).abs() < 1e-6);
    }

    #[test]
    fn overlay_snapshot_uses_compact_labels_and_active_field() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            generation_state,
            material_config,
            material_state,
            stage_config,
            stage_state,
        ) = default_scene_state();
        effect_tuner.selected_index = 3;
        effect_tuner.edit_mode = EffectEditMode::LfoShape;
        effect_tuner.pinned = true;

        let snapshot = effect_tuner.overlay_snapshot(
            &view_context(
                &generation_config,
                &generation_state,
                &material_config,
                &material_state,
                &stage_config,
                &stage_state,
            ),
            0.0,
        );

        assert_eq!(snapshot.effect_label, "lens");
        assert_eq!(snapshot.parameter_label, "k2");
        assert_eq!(snapshot.active_field, EffectOverlayField::LfoShape);
        assert!(snapshot.pinned);
    }

    #[test]
    fn open_and_close_page_manage_page_modes() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());

        effect_tuner.open_page(1.0);
        assert_eq!(effect_tuner.page_mode(), EffectTunerPageMode::Compact);

        effect_tuner.show_list_page(1.1);
        assert_eq!(effect_tuner.page_mode(), EffectTunerPageMode::List);

        effect_tuner.close_page();
        assert_eq!(effect_tuner.page_mode(), EffectTunerPageMode::Compact);
    }

    #[test]
    fn list_overlay_snapshot_scrolls_to_keep_selection_visible() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            generation_state,
            material_config,
            material_state,
            stage_config,
            stage_state,
        ) = default_scene_state();
        effect_tuner.selected_index = EffectTunerParameter::all().len() - 1;
        effect_tuner.show_list_page(1.0);

        let snapshot = effect_tuner.list_overlay_snapshot(
            &view_context(
                &generation_config,
                &generation_state,
                &material_config,
                &material_state,
                &stage_config,
                &stage_state,
            ),
            1.0,
            7,
        );

        assert_eq!(snapshot.total_parameters, EffectTunerParameter::all().len());
        assert_eq!(snapshot.rows.len(), 7);
        assert_eq!(
            snapshot.window_start + snapshot.rows.len(),
            snapshot.total_parameters
        );
        assert!(snapshot.rows.last().is_some_and(|row| row.selected));
        assert_eq!(snapshot.detail.parameter_label, "lvl refl");
    }

    #[test]
    fn reset_selected_restores_lfo_frequency_default() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            mut generation_state,
            material_config,
            mut material_state,
            stage_config,
            mut stage_state,
        ) = default_scene_state();
        effect_tuner.selected_index = 16;
        effect_tuner.edit_mode = EffectEditMode::LfoFrequency;
        effect_tuner.selected_lfo_mut().frequency_hz = 3.0;

        effect_tuner.reset_selected(
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        );

        assert_eq!(
            effect_tuner.selected_lfo().frequency_hz,
            DEFAULT_LFO_FREQUENCY_HZ
        );
    }

    #[test]
    fn reset_all_restores_effect_enable_defaults_and_disables_lfos() {
        let mut defaults = EffectsConfig::default();
        let (
            generation_config,
            mut generation_state,
            material_config,
            mut material_state,
            stage_config,
            mut stage_state,
        ) = default_scene_state();
        defaults.edge_detection.enabled = true;
        let mut effect_tuner = EffectTunerState::from_config(&defaults);
        effect_tuner.current.edge_detection.enabled = false;
        effect_tuner.selected_index = 22;
        effect_tuner.selected_lfo_mut().enabled = true;
        effect_tuner.selected_lfo_mut().shape = LfoShape::Square;

        effect_tuner.reset_all(
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        );

        assert!(effect_tuner.current.edge_detection.enabled);
        assert!(!effect_tuner.selected_lfo().enabled);
        assert_eq!(effect_tuner.selected_lfo().shape, LfoShape::Sine);
    }

    #[test]
    fn runtime_snapshot_round_trips_lfo_state() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        effect_tuner.selected_lfo_mut().enabled = true;
        effect_tuner.selected_lfo_mut().shape = LfoShape::Triangle;
        effect_tuner.selected_lfo_mut().amplitude = 1.25;
        effect_tuner.selected_lfo_mut().frequency_hz = 0.75;

        let snapshot = effect_tuner.runtime_snapshot();
        let mut restored = EffectTunerState::from_config(&EffectsConfig::default());
        restored.apply_runtime_snapshot(&snapshot);

        let restored_snapshot = restored.runtime_snapshot();
        assert_eq!(restored_snapshot.lfos.len(), snapshot.lfos.len());
        assert_eq!(
            restored_snapshot.current.color_wavefolder.gain,
            snapshot.current.color_wavefolder.gain
        );
        assert!(restored_snapshot.lfos[0].enabled);
        assert_eq!(restored_snapshot.lfos[0].shape, LfoShape::Triangle);
    }

    #[test]
    fn numeric_entry_updates_selected_value() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            mut generation_state,
            material_config,
            mut material_state,
            stage_config,
            mut stage_state,
        ) = default_scene_state();

        for character in ['0', '.', '1', '5', '7'] {
            assert!(effect_tuner.append_numeric_input(
                character,
                &mut edit_context(
                    &generation_config,
                    &mut generation_state,
                    &material_config,
                    &mut material_state,
                    &stage_config,
                    &mut stage_state,
                ),
                1.0,
            ));
        }

        let snapshot = effect_tuner.overlay_snapshot(
            &view_context(
                &generation_config,
                &generation_state,
                &material_config,
                &material_state,
                &stage_config,
                &stage_state,
            ),
            1.0,
        );
        assert_eq!(snapshot.value_text, "0.157");
        assert!((effect_tuner.current.color_wavefolder.gain - 0.157).abs() < 1.0e-6);
    }

    #[test]
    fn numeric_entry_updates_lfo_frequency_and_backspace_reparses() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            mut generation_state,
            material_config,
            mut material_state,
            stage_config,
            mut stage_state,
        ) = default_scene_state();
        effect_tuner.edit_mode = EffectEditMode::LfoFrequency;

        for character in ['0', '.', '1', '5', '7'] {
            assert!(effect_tuner.append_numeric_input(
                character,
                &mut edit_context(
                    &generation_config,
                    &mut generation_state,
                    &material_config,
                    &mut material_state,
                    &stage_config,
                    &mut stage_state,
                ),
                1.0,
            ));
        }
        assert!(effect_tuner.backspace_numeric_input(
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.2,
        ));

        let snapshot = effect_tuner.overlay_snapshot(
            &view_context(
                &generation_config,
                &generation_state,
                &material_config,
                &material_state,
                &stage_config,
                &stage_state,
            ),
            1.2,
        );
        assert_eq!(snapshot.frequency_text, "0.15");
        assert!((effect_tuner.selected_lfo().frequency_hz - 0.15).abs() < 1.0e-6);
    }

    #[test]
    fn switching_field_clears_numeric_entry_highlight_text() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            mut generation_state,
            material_config,
            mut material_state,
            stage_config,
            mut stage_state,
        ) = default_scene_state();
        assert!(effect_tuner.append_numeric_input(
            '0',
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        ));

        assert!(effect_tuner.step_edit_mode(1, 1.1));

        let view = view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        );
        let snapshot = effect_tuner.overlay_snapshot(&view, 1.1);
        assert_eq!(snapshot.active_field, EffectOverlayField::LfoAmplitude);
        assert_eq!(
            snapshot.value_text,
            effect_tuner
                .selected_parameter()
                .display_value(&effect_tuner.current, &view)
        );
    }

    #[test]
    fn selecting_another_parameter_clears_numeric_entry() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            mut generation_state,
            material_config,
            mut material_state,
            stage_config,
            mut stage_state,
        ) = default_scene_state();
        assert!(effect_tuner.append_numeric_input(
            '0',
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        ));

        assert!(effect_tuner.step_selection(
            1,
            HoldInput {
                just_pressed: true,
                pressed: true,
                just_released: false,
                delta_secs: 0.0,
            },
            1.1,
        ));

        let view = view_context(
            &generation_config,
            &generation_state,
            &material_config,
            &material_state,
            &stage_config,
            &stage_state,
        );
        let snapshot = effect_tuner.overlay_snapshot(&view, 1.1);
        assert_eq!(snapshot.parameter_label, "mod");
        assert_eq!(
            snapshot.value_text,
            effect_tuner
                .selected_parameter()
                .display_value(&effect_tuner.current, &view)
        );
    }

    #[test]
    fn shape_field_ignores_numeric_entry() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            mut generation_state,
            material_config,
            mut material_state,
            stage_config,
            mut stage_state,
        ) = default_scene_state();
        effect_tuner.edit_mode = EffectEditMode::LfoShape;

        assert!(!effect_tuner.append_numeric_input(
            '1',
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        ));
        assert_eq!(effect_tuner.selected_lfo().shape, LfoShape::Sine);
    }

    #[test]
    fn scene_parameters_only_expose_the_value_field() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            generation_state,
            material_config,
            material_state,
            stage_config,
            stage_state,
        ) = default_scene_state();
        select_parameter(
            &mut effect_tuner,
            EffectTunerParameter::Scene(EffectTunerSceneParameter::ChildKind),
        );
        effect_tuner.edit_mode = EffectEditMode::LfoShape;

        let snapshot = effect_tuner.overlay_snapshot(
            &view_context(
                &generation_config,
                &generation_state,
                &material_config,
                &material_state,
                &stage_config,
                &stage_state,
            ),
            0.0,
        );

        assert_eq!(snapshot.effect_label, "scene");
        assert_eq!(snapshot.effect_state_text, "VAL");
        assert_eq!(snapshot.lfo_state_text, "--");
        assert_eq!(snapshot.amplitude_text, "--");
        assert_eq!(snapshot.active_field, EffectOverlayField::Value);
    }

    #[test]
    fn reset_all_restores_scene_defaults() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            mut generation_state,
            material_config,
            mut material_state,
            stage_config,
            mut stage_state,
        ) = default_scene_state();
        generation_state.selected_shape_kind = crate::shapes::ShapeKind::Cube;
        generation_state.spawn_placement_mode = crate::shapes::SpawnPlacementMode::Face;
        generation_state.spawn_add_mode = crate::shapes::SpawnAddMode::FillLevel;
        generation_state
            .parameter_mut(crate::parameters::GenerationParameter::ChildOutwardOffsetRatio)
            .adjust_clamped_base_value(
                1.5,
                generation_config.parameter_spec(
                    crate::parameters::GenerationParameter::ChildOutwardOffsetRatio,
                ),
            );
        material_state.opacity = 0.25;
        material_state.saturation = 0.12;
        material_state.surface_mode = MaterialSurfaceMode::Procedural;
        stage_state.enabled = true;
        stage_state.floor_enabled = true;

        effect_tuner.reset_all(
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        );

        assert_eq!(
            generation_state.vertex_offset_ratio_base(),
            EffectTunerSceneParameter::ChildOutwardOffsetRatio.default_value(&view_context(
                &generation_config,
                &generation_state,
                &material_config,
                &material_state,
                &stage_config,
                &stage_state,
            ))
        );
        assert_eq!(
            generation_state.selected_shape_kind,
            generation_config.default_child_shape_kind
        );
        assert_eq!(
            generation_state.spawn_placement_mode,
            generation_config.default_spawn_placement_mode
        );
        assert_eq!(
            generation_state.spawn_add_mode,
            crate::shapes::SpawnAddMode::default()
        );
        assert_eq!(
            material_state.opacity,
            material_config.default_opacity_clamped()
        );
        assert_eq!(material_state.saturation, material_config.saturation);
        assert_eq!(material_state.surface_mode, material_config.surface_mode);
        assert_eq!(stage_state.enabled, stage_config.enabled);
        assert_eq!(stage_state.floor_enabled, stage_config.floor.enabled);
    }

    #[test]
    fn enum_scene_parameter_cycles_with_adjustment() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            mut generation_state,
            material_config,
            mut material_state,
            stage_config,
            mut stage_state,
        ) = default_scene_state();
        select_parameter(
            &mut effect_tuner,
            EffectTunerParameter::Scene(EffectTunerSceneParameter::SpawnAddMode),
        );

        assert!(effect_tuner.step_adjustment(
            1.0,
            HoldInput {
                just_pressed: true,
                pressed: true,
                just_released: false,
                delta_secs: 0.0,
            },
            crate::effect_tuner::state::AdjustmentModifiers {
                shift_pressed: false,
                alt_pressed: false,
            },
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        ));

        assert_eq!(
            generation_state.spawn_add_mode,
            crate::shapes::SpawnAddMode::FillLevel
        );
    }

    #[test]
    fn enum_scene_parameter_ignores_numeric_entry() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            mut generation_state,
            material_config,
            mut material_state,
            stage_config,
            mut stage_state,
        ) = default_scene_state();
        select_parameter(
            &mut effect_tuner,
            EffectTunerParameter::Scene(EffectTunerSceneParameter::ChildKind),
        );
        let before = generation_state.selected_shape_kind;

        assert!(!effect_tuner.append_numeric_input(
            '1',
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        ));
        assert_eq!(generation_state.selected_shape_kind, before);
    }

    #[test]
    fn material_numeric_parameter_updates_runtime_state() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            mut generation_state,
            material_config,
            mut material_state,
            stage_config,
            mut stage_state,
        ) = default_scene_state();
        select_parameter(
            &mut effect_tuner,
            EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialMetallic),
        );

        assert!(effect_tuner.append_numeric_input(
            '1',
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        ));

        assert_eq!(material_state.metallic, 1.0);
    }

    #[test]
    fn material_enum_parameter_cycles_with_adjustment() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            mut generation_state,
            material_config,
            mut material_state,
            stage_config,
            mut stage_state,
        ) = default_scene_state();
        select_parameter(
            &mut effect_tuner,
            EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialSurfaceMode),
        );

        assert!(effect_tuner.step_adjustment(
            1.0,
            HoldInput {
                just_pressed: true,
                pressed: true,
                just_released: false,
                delta_secs: 0.0,
            },
            crate::effect_tuner::state::AdjustmentModifiers {
                shift_pressed: false,
                alt_pressed: false,
            },
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        ));

        assert_eq!(material_state.surface_mode, MaterialSurfaceMode::Procedural);
    }

    #[test]
    fn stage_enum_parameter_cycles_with_adjustment() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        let (
            generation_config,
            mut generation_state,
            material_config,
            mut material_state,
            stage_config,
            mut stage_state,
        ) = default_scene_state();
        select_parameter(
            &mut effect_tuner,
            EffectTunerParameter::Scene(EffectTunerSceneParameter::StageEnabled),
        );

        assert!(effect_tuner.step_adjustment(
            1.0,
            HoldInput {
                just_pressed: true,
                pressed: true,
                just_released: false,
                delta_secs: 0.0,
            },
            crate::effect_tuner::state::AdjustmentModifiers {
                shift_pressed: false,
                alt_pressed: false,
            },
            &mut edit_context(
                &generation_config,
                &mut generation_state,
                &material_config,
                &mut material_state,
                &stage_config,
                &mut stage_state,
            ),
            1.0,
        ));

        assert!(stage_state.enabled);
    }
}
