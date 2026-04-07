#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectTunerParameter {
    Effect(EffectNumericParameter),
    Scene(EffectTunerSceneParameter),
}

impl EffectTunerParameter {
    const ALL: [Self; 126] = [
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
        Self::Scene(EffectTunerSceneParameter::ChildAxisScaleX),
        Self::Scene(EffectTunerSceneParameter::ChildAxisScaleY),
        Self::Scene(EffectTunerSceneParameter::ChildAxisScaleZ),
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
        Self::Scene(EffectTunerSceneParameter::CameraDistance),
        Self::Scene(EffectTunerSceneParameter::CameraAngularVelocityX),
        Self::Scene(EffectTunerSceneParameter::CameraAngularVelocityY),
        Self::Scene(EffectTunerSceneParameter::CameraAngularVelocityZ),
        Self::Scene(EffectTunerSceneParameter::CameraZoomVelocity),
        Self::Scene(EffectTunerSceneParameter::RenderingClearColorR),
        Self::Scene(EffectTunerSceneParameter::RenderingClearColorG),
        Self::Scene(EffectTunerSceneParameter::RenderingClearColorB),
        Self::Scene(EffectTunerSceneParameter::RenderingAmbientColorR),
        Self::Scene(EffectTunerSceneParameter::RenderingAmbientColorG),
        Self::Scene(EffectTunerSceneParameter::RenderingAmbientColorB),
        Self::Scene(EffectTunerSceneParameter::RenderingAmbientBrightness),
        Self::Scene(EffectTunerSceneParameter::StageFloorColorR),
        Self::Scene(EffectTunerSceneParameter::StageFloorColorG),
        Self::Scene(EffectTunerSceneParameter::StageFloorColorB),
        Self::Scene(EffectTunerSceneParameter::StageFloorTranslationX),
        Self::Scene(EffectTunerSceneParameter::StageFloorTranslationY),
        Self::Scene(EffectTunerSceneParameter::StageFloorTranslationZ),
        Self::Scene(EffectTunerSceneParameter::StageFloorRotationX),
        Self::Scene(EffectTunerSceneParameter::StageFloorRotationY),
        Self::Scene(EffectTunerSceneParameter::StageFloorRotationZ),
        Self::Scene(EffectTunerSceneParameter::StageFloorSizeX),
        Self::Scene(EffectTunerSceneParameter::StageFloorSizeY),
        Self::Scene(EffectTunerSceneParameter::StageFloorThickness),
        Self::Scene(EffectTunerSceneParameter::StageFloorMetallic),
        Self::Scene(EffectTunerSceneParameter::StageFloorPerceptualRoughness),
        Self::Scene(EffectTunerSceneParameter::StageFloorReflectance),
        Self::Scene(EffectTunerSceneParameter::StageBackdropColorR),
        Self::Scene(EffectTunerSceneParameter::StageBackdropColorG),
        Self::Scene(EffectTunerSceneParameter::StageBackdropColorB),
        Self::Scene(EffectTunerSceneParameter::StageBackdropTranslationX),
        Self::Scene(EffectTunerSceneParameter::StageBackdropTranslationY),
        Self::Scene(EffectTunerSceneParameter::StageBackdropTranslationZ),
        Self::Scene(EffectTunerSceneParameter::StageBackdropRotationX),
        Self::Scene(EffectTunerSceneParameter::StageBackdropRotationY),
        Self::Scene(EffectTunerSceneParameter::StageBackdropRotationZ),
        Self::Scene(EffectTunerSceneParameter::StageBackdropSizeX),
        Self::Scene(EffectTunerSceneParameter::StageBackdropSizeY),
        Self::Scene(EffectTunerSceneParameter::StageBackdropThickness),
        Self::Scene(EffectTunerSceneParameter::StageBackdropMetallic),
        Self::Scene(EffectTunerSceneParameter::StageBackdropPerceptualRoughness),
        Self::Scene(EffectTunerSceneParameter::StageBackdropReflectance),
        Self::Scene(EffectTunerSceneParameter::LightingDirectionalColorR),
        Self::Scene(EffectTunerSceneParameter::LightingDirectionalColorG),
        Self::Scene(EffectTunerSceneParameter::LightingDirectionalColorB),
        Self::Scene(EffectTunerSceneParameter::LightingDirectionalIlluminance),
        Self::Scene(EffectTunerSceneParameter::LightingDirectionalTranslationX),
        Self::Scene(EffectTunerSceneParameter::LightingDirectionalTranslationY),
        Self::Scene(EffectTunerSceneParameter::LightingDirectionalTranslationZ),
        Self::Scene(EffectTunerSceneParameter::LightingDirectionalLookAtX),
        Self::Scene(EffectTunerSceneParameter::LightingDirectionalLookAtY),
        Self::Scene(EffectTunerSceneParameter::LightingDirectionalLookAtZ),
        Self::Scene(EffectTunerSceneParameter::LightingPointColorR),
        Self::Scene(EffectTunerSceneParameter::LightingPointColorG),
        Self::Scene(EffectTunerSceneParameter::LightingPointColorB),
        Self::Scene(EffectTunerSceneParameter::LightingPointIntensity),
        Self::Scene(EffectTunerSceneParameter::LightingPointRange),
        Self::Scene(EffectTunerSceneParameter::LightingPointTranslationX),
        Self::Scene(EffectTunerSceneParameter::LightingPointTranslationY),
        Self::Scene(EffectTunerSceneParameter::LightingPointTranslationZ),
        Self::Scene(EffectTunerSceneParameter::LightingAccentColorR),
        Self::Scene(EffectTunerSceneParameter::LightingAccentColorG),
        Self::Scene(EffectTunerSceneParameter::LightingAccentColorB),
        Self::Scene(EffectTunerSceneParameter::LightingAccentIntensity),
        Self::Scene(EffectTunerSceneParameter::LightingAccentRange),
        Self::Scene(EffectTunerSceneParameter::LightingAccentTranslationX),
        Self::Scene(EffectTunerSceneParameter::LightingAccentTranslationY),
        Self::Scene(EffectTunerSceneParameter::LightingAccentTranslationZ),
    ];

    pub(crate) fn all() -> &'static [Self] {
        &Self::ALL
    }

    pub(crate) fn stable_id(self) -> &'static str {
        match self {
            Self::Effect(parameter) => parameter.stable_id(),
            Self::Scene(parameter) => parameter.stable_id(),
        }
    }

    pub(crate) fn from_stable_id(stable_id: &str) -> Option<Self> {
        EffectNumericParameter::from_stable_id(stable_id)
            .map(Self::Effect)
            .or_else(|| EffectTunerSceneParameter::from_stable_id(stable_id).map(Self::Scene))
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
        match self {
            Self::Effect(_) => true,
            Self::Scene(parameter) => parameter.supports_lfo(),
        }
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

    fn default_lfo_amplitude(self, context: &EffectTunerViewContext<'_>) -> f32 {
        match self {
            Self::Effect(parameter) => parameter.default_lfo_amplitude(),
            Self::Scene(parameter) => parameter.runtime_default_lfo_amplitude(context),
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
}
