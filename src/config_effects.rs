use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct EffectsConfig {
    pub(crate) color_wavefolder: ColorWavefolderConfig,
    pub(crate) lens_distortion: LensDistortionConfig,
    pub(crate) gaussian_blur: GaussianBlurConfig,
    pub(crate) bloom: BloomConfig,
    pub(crate) edge_detection: EdgeDetectionConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct ColorWavefolderConfig {
    pub(crate) enabled: bool,
    pub(crate) gain: f32,
    pub(crate) modulus: f32,
}

impl ColorWavefolderConfig {
    pub(crate) fn gain_clamped(&self) -> f32 {
        self.gain.max(0.0)
    }

    pub(crate) fn modulus_clamped(&self) -> f32 {
        self.modulus.max(0.0001)
    }
}

impl Default for ColorWavefolderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            gain: 2.4,
            modulus: 1.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct LensDistortionConfig {
    pub(crate) enabled: bool,
    pub(crate) strength: f32,
    pub(crate) radial_k2: f32,
    pub(crate) radial_k3: f32,
    pub(crate) zoom: f32,
    pub(crate) center: [f32; 2],
    pub(crate) scale: [f32; 2],
    pub(crate) tangential: [f32; 2],
    pub(crate) chromatic_aberration: f32,
}

impl LensDistortionConfig {
    pub(crate) fn strength_clamped(&self) -> f32 {
        self.strength.clamp(-4.0, 4.0)
    }

    pub(crate) fn radial_k2_clamped(&self) -> f32 {
        self.radial_k2.clamp(-4.0, 4.0)
    }

    pub(crate) fn radial_k3_clamped(&self) -> f32 {
        self.radial_k3.clamp(-4.0, 4.0)
    }

    pub(crate) fn zoom_clamped(&self) -> f32 {
        self.zoom.max(0.1)
    }

    pub(crate) fn center_clamped(&self) -> [f32; 2] {
        [
            self.center[0].clamp(0.0, 1.0),
            self.center[1].clamp(0.0, 1.0),
        ]
    }

    pub(crate) fn scale_clamped(&self) -> [f32; 2] {
        [self.scale[0].clamp(0.1, 4.0), self.scale[1].clamp(0.1, 4.0)]
    }

    pub(crate) fn tangential_clamped(&self) -> [f32; 2] {
        [
            self.tangential[0].clamp(-2.0, 2.0),
            self.tangential[1].clamp(-2.0, 2.0),
        ]
    }

    pub(crate) fn chromatic_aberration_clamped(&self) -> f32 {
        self.chromatic_aberration.clamp(0.0, 0.5)
    }
}

impl Default for LensDistortionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strength: 0.18,
            radial_k2: 0.0,
            radial_k3: 0.0,
            zoom: 1.0,
            center: [0.5, 0.5],
            scale: [1.0, 1.0],
            tangential: [0.0, 0.0],
            chromatic_aberration: 0.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct GaussianBlurConfig {
    pub(crate) enabled: bool,
    pub(crate) sigma: f32,
    pub(crate) radius_pixels: u32,
}

impl GaussianBlurConfig {
    pub(crate) fn sigma_clamped(&self) -> f32 {
        self.sigma.max(0.0001)
    }

    pub(crate) fn radius_pixels_clamped(&self) -> u32 {
        self.radius_pixels.min(16)
    }
}

impl Default for GaussianBlurConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sigma: 1.2,
            radius_pixels: 2,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct BloomConfig {
    pub(crate) enabled: bool,
    pub(crate) threshold: f32,
    pub(crate) intensity: f32,
    pub(crate) radius_pixels: u32,
}

impl BloomConfig {
    pub(crate) fn threshold_clamped(&self) -> f32 {
        self.threshold.max(0.0)
    }

    pub(crate) fn intensity_clamped(&self) -> f32 {
        self.intensity.max(0.0)
    }

    pub(crate) fn radius_pixels_clamped(&self) -> u32 {
        self.radius_pixels.min(16)
    }
}

impl Default for BloomConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: 0.8,
            intensity: 0.65,
            radius_pixels: 8,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct EdgeDetectionConfig {
    pub(crate) enabled: bool,
    pub(crate) strength: f32,
    pub(crate) threshold: f32,
    pub(crate) mix: f32,
    pub(crate) color: [f32; 3],
}

impl EdgeDetectionConfig {
    pub(crate) fn strength_clamped(&self) -> f32 {
        self.strength.max(0.0)
    }

    pub(crate) fn threshold_clamped(&self) -> f32 {
        self.threshold.max(0.0)
    }

    pub(crate) fn mix_clamped(&self) -> f32 {
        self.mix.clamp(0.0, 1.0)
    }
}

impl Default for EdgeDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strength: 3.0,
            threshold: 0.2,
            mix: 0.85,
            color: [1.0, 1.0, 1.0],
        }
    }
}

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

    pub(crate) fn stable_id(self) -> &'static str {
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

    pub(crate) fn from_stable_id(stable_id: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|parameter| parameter.stable_id() == stable_id)
    }

    pub(crate) fn label(self) -> &'static str {
        self.stable_id()
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
            Self::LensStrength | Self::LensRadialK2 | Self::LensRadialK3 => 0.005,
            Self::LensCenterX | Self::LensCenterY => 0.001,
            Self::LensScaleX | Self::LensScaleY => 0.005,
            Self::LensTangentialX | Self::LensTangentialY => 0.001,
            Self::LensZoom => 0.002,
            Self::LensChromaticAberration => 0.0005,
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

#[cfg(test)]
mod tests {
    use super::{EffectNumericParameter, EffectsConfig};

    #[test]
    fn integer_parameters_round_when_set() {
        let mut effects = EffectsConfig::default();

        EffectNumericParameter::BloomRadius.set_value(&mut effects, 7.6);

        assert_eq!(effects.bloom.radius_pixels, 8);
    }

    #[test]
    fn lens_parameters_use_fine_adjustment_steps() {
        let expected_steps = [
            (EffectNumericParameter::LensStrength, 0.005),
            (EffectNumericParameter::LensRadialK2, 0.005),
            (EffectNumericParameter::LensRadialK3, 0.005),
            (EffectNumericParameter::LensCenterX, 0.001),
            (EffectNumericParameter::LensCenterY, 0.001),
            (EffectNumericParameter::LensScaleX, 0.005),
            (EffectNumericParameter::LensScaleY, 0.005),
            (EffectNumericParameter::LensTangentialX, 0.001),
            (EffectNumericParameter::LensTangentialY, 0.001),
            (EffectNumericParameter::LensZoom, 0.002),
            (EffectNumericParameter::LensChromaticAberration, 0.0005),
        ];

        for (parameter, expected_step) in expected_steps {
            assert_eq!(parameter.adjustment_step(false, false), expected_step);
        }
    }
}
