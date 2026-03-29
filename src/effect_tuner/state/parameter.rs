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
