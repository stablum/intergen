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

    const LFO_CAPABLE: [Self; 19] = [
        Self::ChildTwistPerVertexRadians,
        Self::ChildOutwardOffsetRatio,
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
        Self::MaterialAccentEveryNLevels,
        Self::MaterialLevelLightnessShift,
        Self::MaterialLevelSaturationShift,
        Self::MaterialLevelMetallicShift,
        Self::MaterialLevelRoughnessShift,
        Self::MaterialLevelReflectanceShift,
    ];

    const MATERIAL_LFO_CAPABLE: [Self; 17] = [
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

    fn lfo_capable() -> &'static [Self] {
        &Self::LFO_CAPABLE
    }

    fn material_lfo_capable() -> &'static [Self] {
        &Self::MATERIAL_LFO_CAPABLE
    }

    fn supports_lfo(self) -> bool {
        Self::lfo_capable().contains(&self)
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

    fn lfo_scene_index(self) -> Option<usize> {
        Self::lfo_capable()
            .iter()
            .position(|candidate| *candidate == self)
    }

    fn material_lfo_base_index(self) -> Option<usize> {
        Self::material_lfo_capable()
            .iter()
            .position(|candidate| *candidate == self)
    }

    fn runtime_default_lfo_amplitude(self, context: &EffectTunerViewContext<'_>) -> f32 {
        self.adjustment_step(context, false, false) * 5.0
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
            None => {
                if let Some(applied) = self.apply_material_numeric_value(
                    context.material_config,
                    context.material_state,
                    value,
                ) {
                    applied
                } else {
                    match self {
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
                        _ => unreachable!(),
                    }
                }
            }
        }
    }

    fn apply_material_numeric_value(
        self,
        material_config: &MaterialConfig,
        material_state: &mut MaterialState,
        value: f32,
    ) -> Option<f32> {
        Some(match self {
            Self::GlobalOpacity => {
                let (min_opacity, max_opacity) = material_config.opacity_bounds();
                material_state.opacity = value.clamp(min_opacity, max_opacity);
                material_state.opacity
            }
            Self::MaterialHueStepPerLevel => {
                material_state.hue_step_per_level = value;
                material_state.hue_step_per_level
            }
            Self::MaterialSaturation => {
                material_state.saturation = value.clamp(0.0, 1.0);
                material_state.saturation
            }
            Self::MaterialLightness => {
                material_state.lightness = value.clamp(0.0, 1.0);
                material_state.lightness
            }
            Self::MaterialMetallic => {
                material_state.metallic = value.clamp(0.0, 1.0);
                material_state.metallic
            }
            Self::MaterialPerceptualRoughness => {
                material_state.perceptual_roughness = value.clamp(0.0, 1.0);
                material_state.perceptual_roughness
            }
            Self::MaterialReflectance => {
                material_state.reflectance = value.clamp(0.0, 1.0);
                material_state.reflectance
            }
            Self::MaterialCubeHueBias => {
                material_state.cube_hue_bias = value;
                material_state.cube_hue_bias
            }
            Self::MaterialTetrahedronHueBias => {
                material_state.tetrahedron_hue_bias = value;
                material_state.tetrahedron_hue_bias
            }
            Self::MaterialOctahedronHueBias => {
                material_state.octahedron_hue_bias = value;
                material_state.octahedron_hue_bias
            }
            Self::MaterialDodecahedronHueBias => {
                material_state.dodecahedron_hue_bias = value;
                material_state.dodecahedron_hue_bias
            }
            Self::MaterialAccentEveryNLevels => {
                material_state.accent_every_n_levels = value.round().max(0.0) as usize;
                material_state.accent_every_n_levels as f32
            }
            Self::MaterialLevelLightnessShift => {
                material_state.level_lightness_shift = value;
                material_state.level_lightness_shift
            }
            Self::MaterialLevelSaturationShift => {
                material_state.level_saturation_shift = value;
                material_state.level_saturation_shift
            }
            Self::MaterialLevelMetallicShift => {
                material_state.level_metallic_shift = value;
                material_state.level_metallic_shift
            }
            Self::MaterialLevelRoughnessShift => {
                material_state.level_roughness_shift = value;
                material_state.level_roughness_shift
            }
            Self::MaterialLevelReflectanceShift => {
                material_state.level_reflectance_shift = value;
                material_state.level_reflectance_shift
            }
            _ => return None,
        })
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

    fn display_numeric_value(self, value: f32) -> Option<String> {
        if self == Self::MaterialAccentEveryNLevels {
            return Some(value.round().max(0.0).to_string());
        }

        self.is_numeric().then(|| format!("{value:.3}"))
    }

    fn live_value(self, context: &EffectTunerViewContext<'_>) -> Option<f32> {
        match self {
            Self::ChildTwistPerVertexRadians => Some(
                context
                    .generation_state
                    .twist_per_vertex_radians(context.generation_config),
            ),
            Self::ChildOutwardOffsetRatio => Some(
                context
                    .generation_state
                    .vertex_offset_ratio(context.generation_config),
            ),
            _ if self.is_numeric() => Some(self.value(context)),
            _ => None,
        }
    }

    fn material_numeric_value(self, material_state: &MaterialState) -> Option<f32> {
        Some(match self {
            Self::GlobalOpacity => material_state.opacity,
            Self::MaterialHueStepPerLevel => material_state.hue_step_per_level,
            Self::MaterialSaturation => material_state.saturation,
            Self::MaterialLightness => material_state.lightness,
            Self::MaterialMetallic => material_state.metallic,
            Self::MaterialPerceptualRoughness => material_state.perceptual_roughness,
            Self::MaterialReflectance => material_state.reflectance,
            Self::MaterialCubeHueBias => material_state.cube_hue_bias,
            Self::MaterialTetrahedronHueBias => material_state.tetrahedron_hue_bias,
            Self::MaterialOctahedronHueBias => material_state.octahedron_hue_bias,
            Self::MaterialDodecahedronHueBias => material_state.dodecahedron_hue_bias,
            Self::MaterialAccentEveryNLevels => material_state.accent_every_n_levels as f32,
            Self::MaterialLevelLightnessShift => material_state.level_lightness_shift,
            Self::MaterialLevelSaturationShift => material_state.level_saturation_shift,
            Self::MaterialLevelMetallicShift => material_state.level_metallic_shift,
            Self::MaterialLevelRoughnessShift => material_state.level_roughness_shift,
            Self::MaterialLevelReflectanceShift => material_state.level_reflectance_shift,
            _ => return None,
        })
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
}
