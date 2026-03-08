use crate::config::EffectsConfig;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectGroup {
    ColorWavefolder,
    LensDistortion,
    GaussianBlur,
    Bloom,
    EdgeDetection,
}

impl EffectGroup {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::ColorWavefolder => "color_wavefolder",
            Self::LensDistortion => "lens_distortion",
            Self::GaussianBlur => "gaussian_blur",
            Self::Bloom => "bloom",
            Self::EdgeDetection => "edge_detection",
        }
    }

    pub(crate) fn compact_label(self) -> &'static str {
        match self {
            Self::ColorWavefolder => "wavefolder",
            Self::LensDistortion => "lens",
            Self::GaussianBlur => "blur",
            Self::Bloom => "bloom",
            Self::EdgeDetection => "edge",
        }
    }

    pub(crate) fn is_enabled(self, effects: &EffectsConfig) -> bool {
        match self {
            Self::ColorWavefolder => effects.color_wavefolder.enabled,
            Self::LensDistortion => effects.lens_distortion.enabled,
            Self::GaussianBlur => effects.gaussian_blur.enabled,
            Self::Bloom => effects.bloom.enabled,
            Self::EdgeDetection => effects.edge_detection.enabled,
        }
    }

    pub(crate) fn set_enabled(self, effects: &mut EffectsConfig, enabled: bool) {
        match self {
            Self::ColorWavefolder => effects.color_wavefolder.enabled = enabled,
            Self::LensDistortion => effects.lens_distortion.enabled = enabled,
            Self::GaussianBlur => effects.gaussian_blur.enabled = enabled,
            Self::Bloom => effects.bloom.enabled = enabled,
            Self::EdgeDetection => effects.edge_detection.enabled = enabled,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectNumericParameter {
    WavefolderGain,
    WavefolderModulus,
    LensStrength,
    LensRadialK2,
    LensRadialK3,
    LensCenterX,
    LensCenterY,
    LensScaleX,
    LensScaleY,
    LensTangentialX,
    LensTangentialY,
    LensZoom,
    LensChromaticAberration,
    GaussianBlurSigma,
    GaussianBlurRadius,
    BloomThreshold,
    BloomIntensity,
    BloomRadius,
    EdgeStrength,
    EdgeThreshold,
    EdgeMix,
    EdgeColorR,
    EdgeColorG,
    EdgeColorB,
}

impl EffectNumericParameter {
    const ALL: [Self; 24] = [
        Self::WavefolderGain,
        Self::WavefolderModulus,
        Self::LensStrength,
        Self::LensRadialK2,
        Self::LensRadialK3,
        Self::LensCenterX,
        Self::LensCenterY,
        Self::LensScaleX,
        Self::LensScaleY,
        Self::LensTangentialX,
        Self::LensTangentialY,
        Self::LensZoom,
        Self::LensChromaticAberration,
        Self::GaussianBlurSigma,
        Self::GaussianBlurRadius,
        Self::BloomThreshold,
        Self::BloomIntensity,
        Self::BloomRadius,
        Self::EdgeStrength,
        Self::EdgeThreshold,
        Self::EdgeMix,
        Self::EdgeColorR,
        Self::EdgeColorG,
        Self::EdgeColorB,
    ];

    pub(crate) fn all() -> &'static [Self] {
        &Self::ALL
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::WavefolderGain => "color_wavefolder.gain",
            Self::WavefolderModulus => "color_wavefolder.modulus",
            Self::LensStrength => "lens_distortion.strength",
            Self::LensRadialK2 => "lens_distortion.radial_k2",
            Self::LensRadialK3 => "lens_distortion.radial_k3",
            Self::LensCenterX => "lens_distortion.center.x",
            Self::LensCenterY => "lens_distortion.center.y",
            Self::LensScaleX => "lens_distortion.scale.x",
            Self::LensScaleY => "lens_distortion.scale.y",
            Self::LensTangentialX => "lens_distortion.tangential.x",
            Self::LensTangentialY => "lens_distortion.tangential.y",
            Self::LensZoom => "lens_distortion.zoom",
            Self::LensChromaticAberration => "lens_distortion.chromatic_aberration",
            Self::GaussianBlurSigma => "gaussian_blur.sigma",
            Self::GaussianBlurRadius => "gaussian_blur.radius_pixels",
            Self::BloomThreshold => "bloom.threshold",
            Self::BloomIntensity => "bloom.intensity",
            Self::BloomRadius => "bloom.radius_pixels",
            Self::EdgeStrength => "edge_detection.strength",
            Self::EdgeThreshold => "edge_detection.threshold",
            Self::EdgeMix => "edge_detection.mix",
            Self::EdgeColorR => "edge_detection.color.r",
            Self::EdgeColorG => "edge_detection.color.g",
            Self::EdgeColorB => "edge_detection.color.b",
        }
    }

    pub(crate) fn short_label(self) -> &'static str {
        match self {
            Self::WavefolderGain => "gain",
            Self::WavefolderModulus => "mod",
            Self::LensStrength => "strength",
            Self::LensRadialK2 => "k2",
            Self::LensRadialK3 => "k3",
            Self::LensCenterX => "center.x",
            Self::LensCenterY => "center.y",
            Self::LensScaleX => "scale.x",
            Self::LensScaleY => "scale.y",
            Self::LensTangentialX => "tan.x",
            Self::LensTangentialY => "tan.y",
            Self::LensZoom => "zoom",
            Self::LensChromaticAberration => "ca",
            Self::GaussianBlurSigma => "sigma",
            Self::GaussianBlurRadius => "radius",
            Self::BloomThreshold => "threshold",
            Self::BloomIntensity => "intensity",
            Self::BloomRadius => "radius",
            Self::EdgeStrength => "strength",
            Self::EdgeThreshold => "threshold",
            Self::EdgeMix => "mix",
            Self::EdgeColorR => "color.r",
            Self::EdgeColorG => "color.g",
            Self::EdgeColorB => "color.b",
        }
    }

    pub(crate) fn effect_group(self) -> EffectGroup {
        match self {
            Self::WavefolderGain | Self::WavefolderModulus => EffectGroup::ColorWavefolder,
            Self::LensStrength
            | Self::LensRadialK2
            | Self::LensRadialK3
            | Self::LensCenterX
            | Self::LensCenterY
            | Self::LensScaleX
            | Self::LensScaleY
            | Self::LensTangentialX
            | Self::LensTangentialY
            | Self::LensZoom
            | Self::LensChromaticAberration => EffectGroup::LensDistortion,
            Self::GaussianBlurSigma | Self::GaussianBlurRadius => EffectGroup::GaussianBlur,
            Self::BloomThreshold | Self::BloomIntensity | Self::BloomRadius => EffectGroup::Bloom,
            Self::EdgeStrength
            | Self::EdgeThreshold
            | Self::EdgeMix
            | Self::EdgeColorR
            | Self::EdgeColorG
            | Self::EdgeColorB => EffectGroup::EdgeDetection,
        }
    }

    fn base_step(self) -> f32 {
        match self {
            Self::WavefolderGain => 0.1,
            Self::WavefolderModulus => 0.05,
            Self::LensStrength | Self::LensRadialK2 | Self::LensRadialK3 => 0.05,
            Self::LensCenterX | Self::LensCenterY => 0.01,
            Self::LensScaleX | Self::LensScaleY => 0.05,
            Self::LensTangentialX | Self::LensTangentialY => 0.01,
            Self::LensZoom => 0.02,
            Self::LensChromaticAberration => 0.005,
            Self::GaussianBlurSigma => 0.05,
            Self::GaussianBlurRadius => 1.0,
            Self::BloomThreshold | Self::BloomIntensity => 0.05,
            Self::BloomRadius => 1.0,
            Self::EdgeStrength | Self::EdgeThreshold => 0.05,
            Self::EdgeMix => 0.02,
            Self::EdgeColorR | Self::EdgeColorG | Self::EdgeColorB => 0.02,
        }
    }

    pub(crate) fn default_lfo_amplitude(self) -> f32 {
        self.base_step() * 5.0
    }

    fn is_integer(self) -> bool {
        matches!(self, Self::GaussianBlurRadius | Self::BloomRadius)
    }

    pub(crate) fn display_value(self, effects: &EffectsConfig) -> String {
        let value = self.value(effects);
        if self.is_integer() {
            format!("{:.0}", value.round())
        } else {
            format!("{value:.3}")
        }
    }

    pub(crate) fn value(self, effects: &EffectsConfig) -> f32 {
        match self {
            Self::WavefolderGain => effects.color_wavefolder.gain,
            Self::WavefolderModulus => effects.color_wavefolder.modulus,
            Self::LensStrength => effects.lens_distortion.strength,
            Self::LensRadialK2 => effects.lens_distortion.radial_k2,
            Self::LensRadialK3 => effects.lens_distortion.radial_k3,
            Self::LensCenterX => effects.lens_distortion.center[0],
            Self::LensCenterY => effects.lens_distortion.center[1],
            Self::LensScaleX => effects.lens_distortion.scale[0],
            Self::LensScaleY => effects.lens_distortion.scale[1],
            Self::LensTangentialX => effects.lens_distortion.tangential[0],
            Self::LensTangentialY => effects.lens_distortion.tangential[1],
            Self::LensZoom => effects.lens_distortion.zoom,
            Self::LensChromaticAberration => effects.lens_distortion.chromatic_aberration,
            Self::GaussianBlurSigma => effects.gaussian_blur.sigma,
            Self::GaussianBlurRadius => effects.gaussian_blur.radius_pixels as f32,
            Self::BloomThreshold => effects.bloom.threshold,
            Self::BloomIntensity => effects.bloom.intensity,
            Self::BloomRadius => effects.bloom.radius_pixels as f32,
            Self::EdgeStrength => effects.edge_detection.strength,
            Self::EdgeThreshold => effects.edge_detection.threshold,
            Self::EdgeMix => effects.edge_detection.mix,
            Self::EdgeColorR => effects.edge_detection.color[0],
            Self::EdgeColorG => effects.edge_detection.color[1],
            Self::EdgeColorB => effects.edge_detection.color[2],
        }
    }

    pub(crate) fn set_value(self, effects: &mut EffectsConfig, value: f32) {
        match self {
            Self::WavefolderGain => effects.color_wavefolder.gain = value.max(0.0),
            Self::WavefolderModulus => effects.color_wavefolder.modulus = value.max(0.0001),
            Self::LensStrength => effects.lens_distortion.strength = value.clamp(-4.0, 4.0),
            Self::LensRadialK2 => effects.lens_distortion.radial_k2 = value.clamp(-4.0, 4.0),
            Self::LensRadialK3 => effects.lens_distortion.radial_k3 = value.clamp(-4.0, 4.0),
            Self::LensCenterX => effects.lens_distortion.center[0] = value.clamp(0.0, 1.0),
            Self::LensCenterY => effects.lens_distortion.center[1] = value.clamp(0.0, 1.0),
            Self::LensScaleX => effects.lens_distortion.scale[0] = value.clamp(0.1, 4.0),
            Self::LensScaleY => effects.lens_distortion.scale[1] = value.clamp(0.1, 4.0),
            Self::LensTangentialX => effects.lens_distortion.tangential[0] = value.clamp(-2.0, 2.0),
            Self::LensTangentialY => effects.lens_distortion.tangential[1] = value.clamp(-2.0, 2.0),
            Self::LensZoom => effects.lens_distortion.zoom = value.max(0.1),
            Self::LensChromaticAberration => {
                effects.lens_distortion.chromatic_aberration = value.clamp(0.0, 0.5)
            }
            Self::GaussianBlurSigma => effects.gaussian_blur.sigma = value.max(0.0001),
            Self::GaussianBlurRadius => {
                effects.gaussian_blur.radius_pixels = value.round().clamp(0.0, 16.0) as u32
            }
            Self::BloomThreshold => effects.bloom.threshold = value.max(0.0),
            Self::BloomIntensity => effects.bloom.intensity = value.max(0.0),
            Self::BloomRadius => {
                effects.bloom.radius_pixels = value.round().clamp(0.0, 16.0) as u32
            }
            Self::EdgeStrength => effects.edge_detection.strength = value.max(0.0),
            Self::EdgeThreshold => effects.edge_detection.threshold = value.max(0.0),
            Self::EdgeMix => effects.edge_detection.mix = value.clamp(0.0, 1.0),
            Self::EdgeColorR => effects.edge_detection.color[0] = value.clamp(0.0, 1.0),
            Self::EdgeColorG => effects.edge_detection.color[1] = value.clamp(0.0, 1.0),
            Self::EdgeColorB => effects.edge_detection.color[2] = value.clamp(0.0, 1.0),
        }
    }

    pub(crate) fn adjustment_step(self, shift_pressed: bool, alt_pressed: bool) -> f32 {
        let mut step = self.base_step();
        if shift_pressed {
            step *= 10.0;
        }
        if alt_pressed {
            step *= 0.1;
        }
        step
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectEditMode {
    Value,
    LfoAmplitude,
    LfoFrequency,
    LfoShape,
}

impl EffectEditMode {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::LfoAmplitude => "lfo amp",
            Self::LfoFrequency => "lfo freq",
            Self::LfoShape => "lfo shape",
        }
    }

    pub(crate) fn next(self) -> Self {
        match self {
            Self::Value => Self::LfoAmplitude,
            Self::LfoAmplitude => Self::LfoFrequency,
            Self::LfoFrequency => Self::LfoShape,
            Self::LfoShape => Self::Value,
        }
    }

    pub(crate) fn overlay_field(self) -> EffectOverlayField {
        match self {
            Self::Value => EffectOverlayField::Value,
            Self::LfoAmplitude => EffectOverlayField::LfoAmplitude,
            Self::LfoFrequency => EffectOverlayField::LfoFrequency,
            Self::LfoShape => EffectOverlayField::LfoShape,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectOverlayField {
    Value,
    LfoAmplitude,
    LfoFrequency,
    LfoShape,
}

#[cfg(test)]
mod tests {
    use super::EffectNumericParameter;
    use crate::config::EffectsConfig;

    #[test]
    fn integer_parameters_round_when_set() {
        let mut effects = EffectsConfig::default();

        EffectNumericParameter::BloomRadius.set_value(&mut effects, 7.6);

        assert_eq!(effects.bloom.radius_pixels, 8);
    }
}
