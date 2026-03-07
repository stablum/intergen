use std::f32::consts::TAU;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::camera::SceneCamera;
use crate::config::EffectsConfig;
use crate::effects::{CameraEffectsSettings, camera_effects_from_config};
use crate::presets::PresetBrowserState;

const OVERLAY_HOLD_SECS: f32 = 2.5;
const HOLD_DELAY_SECS: f32 = 0.32;
const REPEAT_INTERVAL_SECS: f32 = 0.08;
const DEFAULT_LFO_FREQUENCY_HZ: f32 = 0.25;
const LFO_FREQUENCY_STEP_HZ: f32 = 0.05;
const BROWNIAN_OCTAVES: usize = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EffectGroup {
    ColorWavefolder,
    LensDistortion,
    GaussianBlur,
    Bloom,
    EdgeDetection,
}

impl EffectGroup {
    fn label(self) -> &'static str {
        match self {
            Self::ColorWavefolder => "color_wavefolder",
            Self::LensDistortion => "lens_distortion",
            Self::GaussianBlur => "gaussian_blur",
            Self::Bloom => "bloom",
            Self::EdgeDetection => "edge_detection",
        }
    }

    fn compact_label(self) -> &'static str {
        match self {
            Self::ColorWavefolder => "wavefolder",
            Self::LensDistortion => "lens",
            Self::GaussianBlur => "blur",
            Self::Bloom => "bloom",
            Self::EdgeDetection => "edge",
        }
    }

    fn is_enabled(self, effects: &EffectsConfig) -> bool {
        match self {
            Self::ColorWavefolder => effects.color_wavefolder.enabled,
            Self::LensDistortion => effects.lens_distortion.enabled,
            Self::GaussianBlur => effects.gaussian_blur.enabled,
            Self::Bloom => effects.bloom.enabled,
            Self::EdgeDetection => effects.edge_detection.enabled,
        }
    }

    fn set_enabled(self, effects: &mut EffectsConfig, enabled: bool) {
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

    fn effect_group(self) -> EffectGroup {
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

    fn default_lfo_amplitude(self) -> f32 {
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

    fn value(self, effects: &EffectsConfig) -> f32 {
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

    fn set_value(self, effects: &mut EffectsConfig, value: f32) {
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

    fn adjustment_step(self, shift_pressed: bool, alt_pressed: bool) -> f32 {
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum LfoShape {
    Sine,
    Triangle,
    Saw,
    Square,
    SteppedRandom,
    BrownianMotion,
}

impl LfoShape {
    fn label(self) -> &'static str {
        match self {
            Self::Sine => "sine",
            Self::Triangle => "triangle",
            Self::Saw => "saw",
            Self::Square => "square",
            Self::SteppedRandom => "stepped random",
            Self::BrownianMotion => "brownian motion",
        }
    }

    fn sample(self, phase_cycles: f32, seed: u32) -> f32 {
        let phase = phase_cycles.rem_euclid(1.0);
        match self {
            Self::Sine => (phase * TAU).sin(),
            Self::Triangle => {
                if phase < 0.25 {
                    phase * 4.0
                } else if phase < 0.75 {
                    2.0 - phase * 4.0
                } else {
                    phase * 4.0 - 4.0
                }
            }
            Self::Saw => phase * 2.0 - 1.0,
            Self::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Self::SteppedRandom => sample_stepped_random(phase_cycles, seed),
            Self::BrownianMotion => sample_brownian_motion(phase_cycles, seed),
        }
    }

    fn cycle(self, direction: f32) -> Self {
        let all = [
            Self::Sine,
            Self::Triangle,
            Self::Saw,
            Self::Square,
            Self::SteppedRandom,
            Self::BrownianMotion,
        ];
        let index = all.iter().position(|shape| *shape == self).unwrap_or(0) as isize;
        let delta = if direction < 0.0 { -1 } else { 1 };
        let next_index = (index + delta).rem_euclid(all.len() as isize) as usize;
        all[next_index]
    }
}

fn sample_stepped_random(phase_cycles: f32, seed: u32) -> f32 {
    signed_hash(phase_cycles.floor() as i32, seed)
}

fn sample_brownian_motion(phase_cycles: f32, seed: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut amplitude_sum = 0.0;

    for octave in 0..BROWNIAN_OCTAVES {
        let octave_seed = seed.wrapping_add((octave as u32).wrapping_mul(0x9E37_79B9));
        value += amplitude * smooth_value_noise_1d(phase_cycles * frequency, octave_seed);
        amplitude_sum += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    if amplitude_sum <= f32::EPSILON {
        0.0
    } else {
        (value / amplitude_sum).clamp(-1.0, 1.0)
    }
}

fn smooth_value_noise_1d(position: f32, seed: u32) -> f32 {
    let left_index = position.floor() as i32;
    let local_t = position - left_index as f32;
    let blend = local_t * local_t * (3.0 - 2.0 * local_t);
    let left_value = signed_hash(left_index, seed);
    let right_value = signed_hash(left_index + 1, seed);
    left_value + (right_value - left_value) * blend
}

fn signed_hash(index: i32, seed: u32) -> f32 {
    let mut state = (index as u32).wrapping_add(seed.wrapping_mul(0x85EB_CA6B));
    state ^= state >> 16;
    state = state.wrapping_mul(0x7FEB_352D);
    state ^= state >> 15;
    state = state.wrapping_mul(0x846C_A68B);
    state ^= state >> 16;
    (state as f32 / u32::MAX as f32) * 2.0 - 1.0
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EffectEditMode {
    Value,
    LfoAmplitude,
    LfoFrequency,
    LfoShape,
}

impl EffectEditMode {
    fn label(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::LfoAmplitude => "lfo amp",
            Self::LfoFrequency => "lfo freq",
            Self::LfoShape => "lfo shape",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Value => Self::LfoAmplitude,
            Self::LfoAmplitude => Self::LfoFrequency,
            Self::LfoFrequency => Self::LfoShape,
            Self::LfoShape => Self::Value,
        }
    }

    fn overlay_field(self) -> EffectOverlayField {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EffectTunerOverlaySnapshot {
    pub(crate) pinned: bool,
    pub(crate) effect_label: &'static str,
    pub(crate) effect_enabled: bool,
    pub(crate) parameter_label: &'static str,
    pub(crate) value_text: String,
    pub(crate) live_value_text: String,
    pub(crate) lfo_enabled: bool,
    pub(crate) amplitude_text: String,
    pub(crate) frequency_text: String,
    pub(crate) shape_text: &'static str,
    pub(crate) active_field: EffectOverlayField,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct EffectRuntimeSnapshot {
    pub(crate) current: EffectsConfig,
    pub(crate) lfos: Vec<ParameterLfo>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub(crate) struct ParameterLfo {
    enabled: bool,
    shape: LfoShape,
    amplitude: f32,
    frequency_hz: f32,
}

impl ParameterLfo {
    fn default_for(parameter: EffectNumericParameter) -> Self {
        Self {
            enabled: false,
            shape: LfoShape::Sine,
            amplitude: parameter.default_lfo_amplitude(),
            frequency_hz: DEFAULT_LFO_FREQUENCY_HZ,
        }
    }

    fn is_active(self) -> bool {
        self.enabled && self.amplitude > 0.0 && self.frequency_hz > 0.0
    }
}

#[derive(Default, Clone)]
struct RepeatHoldState {
    elapsed_secs: f32,
    repeating: bool,
}

impl RepeatHoldState {
    fn update(
        &mut self,
        just_pressed: bool,
        pressed: bool,
        just_released: bool,
        delta_secs: f32,
    ) -> bool {
        if just_released || !pressed {
            self.reset();
            return false;
        }

        if just_pressed {
            self.reset();
            return true;
        }

        self.elapsed_secs += delta_secs;
        let threshold = if self.repeating {
            REPEAT_INTERVAL_SECS
        } else {
            HOLD_DELAY_SECS
        };

        if self.elapsed_secs < threshold {
            return false;
        }

        self.elapsed_secs = 0.0;
        self.repeating = true;
        true
    }

    fn reset(&mut self) {
        self.elapsed_secs = 0.0;
        self.repeating = false;
    }
}

#[derive(Resource, Clone)]
pub(crate) struct EffectTunerState {
    pub(crate) defaults: EffectsConfig,
    pub(crate) current: EffectsConfig,
    lfos: Vec<ParameterLfo>,
    selected_index: usize,
    edit_mode: EffectEditMode,
    pinned: bool,
    visible_until_secs: f32,
    select_previous_hold: RepeatHoldState,
    select_next_hold: RepeatHoldState,
    decrease_hold: RepeatHoldState,
    increase_hold: RepeatHoldState,
}

impl EffectTunerState {
    pub(crate) fn from_config(effects_config: &EffectsConfig) -> Self {
        Self {
            defaults: effects_config.clone(),
            current: effects_config.clone(),
            lfos: default_lfos(),
            selected_index: 0,
            edit_mode: EffectEditMode::Value,
            pinned: false,
            visible_until_secs: 0.0,
            select_previous_hold: RepeatHoldState::default(),
            select_next_hold: RepeatHoldState::default(),
            decrease_hold: RepeatHoldState::default(),
            increase_hold: RepeatHoldState::default(),
        }
    }

    pub(crate) fn selected_parameter(&self) -> EffectNumericParameter {
        EffectNumericParameter::all()[self.selected_index]
    }

    pub(crate) fn selected_effect(&self) -> EffectGroup {
        self.selected_parameter().effect_group()
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
        self.edit_mode = EffectEditMode::Value;
        self.select_previous_hold.reset();
        self.select_next_hold.reset();
        self.decrease_hold.reset();
        self.increase_hold.reset();
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

    pub(crate) fn overlay_snapshot(&self, now_secs: f32) -> EffectTunerOverlaySnapshot {
        let parameter = self.selected_parameter();
        let effect = self.selected_effect();
        let lfo = self.selected_lfo();
        let live_effects = self.evaluated_effects(now_secs);

        EffectTunerOverlaySnapshot {
            pinned: self.pinned,
            effect_label: effect.compact_label(),
            effect_enabled: effect.is_enabled(&self.current),
            parameter_label: parameter.short_label(),
            value_text: parameter.display_value(&self.current),
            live_value_text: parameter.display_value(&live_effects),
            lfo_enabled: lfo.enabled,
            amplitude_text: format!("{:.3}", lfo.amplitude),
            frequency_text: format!("{:.3}Hz", lfo.frequency_hz),
            shape_text: lfo.shape.label(),
            active_field: self.edit_mode.overlay_field(),
        }
    }

    fn note_interaction(&mut self, now_secs: f32) {
        self.visible_until_secs = now_secs + OVERLAY_HOLD_SECS;
    }

    fn toggle_pinned(&mut self, now_secs: f32) {
        self.pinned = !self.pinned;
        self.note_interaction(now_secs);
    }

    fn cycle_selection(&mut self, direction: isize, now_secs: f32) {
        let parameter_count = EffectNumericParameter::all().len() as isize;
        let next_index =
            (self.selected_index as isize + direction).rem_euclid(parameter_count) as usize;
        self.selected_index = next_index;
        self.note_interaction(now_secs);
    }

    fn cycle_edit_mode(&mut self, now_secs: f32) {
        self.edit_mode = self.edit_mode.next();
        self.note_interaction(now_secs);
    }

    fn adjust_selected(
        &mut self,
        direction: f32,
        shift_pressed: bool,
        alt_pressed: bool,
        now_secs: f32,
    ) {
        let parameter = self.selected_parameter();
        match self.edit_mode {
            EffectEditMode::Value => {
                let current_value = parameter.value(&self.current);
                let next_value = current_value
                    + direction * parameter.adjustment_step(shift_pressed, alt_pressed);
                parameter.set_value(&mut self.current, next_value);
            }
            EffectEditMode::LfoAmplitude => {
                let step = parameter.adjustment_step(shift_pressed, alt_pressed);
                let lfo = self.selected_lfo_mut();
                lfo.amplitude = (lfo.amplitude + direction * step).max(0.0);
            }
            EffectEditMode::LfoFrequency => {
                let mut step = LFO_FREQUENCY_STEP_HZ;
                if shift_pressed {
                    step *= 10.0;
                }
                if alt_pressed {
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

    fn toggle_selected_effect(&mut self, now_secs: f32) -> bool {
        let effect = self.selected_effect();
        let next_enabled = !effect.is_enabled(&self.current);
        effect.set_enabled(&mut self.current, next_enabled);
        self.note_interaction(now_secs);
        next_enabled
    }

    fn toggle_selected_lfo(&mut self, now_secs: f32) -> bool {
        let lfo = self.selected_lfo_mut();
        lfo.enabled = !lfo.enabled;
        let enabled = lfo.enabled;
        self.note_interaction(now_secs);
        enabled
    }

    fn reset_selected(&mut self, now_secs: f32) {
        let parameter = self.selected_parameter();
        match self.edit_mode {
            EffectEditMode::Value => {
                parameter.set_value(&mut self.current, parameter.value(&self.defaults));
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

    fn reset_all(&mut self, now_secs: f32) {
        self.current = self.defaults.clone();
        self.lfos = default_lfos();
        self.edit_mode = EffectEditMode::Value;
        self.note_interaction(now_secs);
    }

    fn selected_lfo(&self) -> ParameterLfo {
        self.lfos[self.selected_index]
    }

    fn selected_lfo_mut(&mut self) -> &mut ParameterLfo {
        &mut self.lfos[self.selected_index]
    }
}

pub(crate) fn effect_tuner_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    preset_browser: Res<PresetBrowserState>,
    mut effect_tuner: ResMut<EffectTunerState>,
) {
    if preset_browser.blocks_input() {
        return;
    }

    let now_secs = time.elapsed_secs();
    if keys.just_pressed(KeyCode::F2) {
        effect_tuner.toggle_pinned(now_secs);
        println!(
            "FX tuner {}.",
            if effect_tuner.pinned {
                "pinned open"
            } else {
                "unpinned"
            }
        );
    }

    if keys.just_pressed(KeyCode::Tab) {
        let selected_effect = effect_tuner.selected_effect();
        let enabled = effect_tuner.toggle_selected_effect(now_secs);
        println!(
            "{} {}.",
            selected_effect.label(),
            if enabled { "enabled" } else { "disabled" }
        );
    }

    if keys.just_pressed(KeyCode::KeyL) {
        let selected_parameter = effect_tuner.selected_parameter();
        let enabled = effect_tuner.toggle_selected_lfo(now_secs);
        println!(
            "LFO for {} {}.",
            selected_parameter.label(),
            if enabled { "enabled" } else { "disabled" }
        );
    }

    if keys.just_pressed(KeyCode::KeyM) {
        effect_tuner.cycle_edit_mode(now_secs);
        println!("FX tuner edit mode: {}.", effect_tuner.edit_mode.label());
    }

    let ctrl_pressed = modifier_pressed(&keys, &[KeyCode::ControlLeft, KeyCode::ControlRight]);
    let shift_pressed = modifier_pressed(&keys, &[KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let alt_pressed = modifier_pressed(&keys, &[KeyCode::AltLeft, KeyCode::AltRight]);
    let delta_secs = time.delta_secs();

    if effect_tuner.select_previous_hold.update(
        ctrl_pressed && keys.just_pressed(KeyCode::ArrowUp),
        ctrl_pressed && keys.pressed(KeyCode::ArrowUp),
        !ctrl_pressed || keys.just_released(KeyCode::ArrowUp),
        delta_secs,
    ) {
        effect_tuner.cycle_selection(-1, now_secs);
        println!(
            "Selected FX parameter: {}",
            effect_tuner.selected_parameter().label()
        );
    }

    if effect_tuner.select_next_hold.update(
        ctrl_pressed && keys.just_pressed(KeyCode::ArrowDown),
        ctrl_pressed && keys.pressed(KeyCode::ArrowDown),
        !ctrl_pressed || keys.just_released(KeyCode::ArrowDown),
        delta_secs,
    ) {
        effect_tuner.cycle_selection(1, now_secs);
        println!(
            "Selected FX parameter: {}",
            effect_tuner.selected_parameter().label()
        );
    }

    if effect_tuner.decrease_hold.update(
        ctrl_pressed && keys.just_pressed(KeyCode::ArrowLeft),
        ctrl_pressed && keys.pressed(KeyCode::ArrowLeft),
        !ctrl_pressed || keys.just_released(KeyCode::ArrowLeft),
        delta_secs,
    ) {
        effect_tuner.adjust_selected(-1.0, shift_pressed, alt_pressed, now_secs);
        println!(
            "{}",
            selected_status_message(&effect_tuner, time.elapsed_secs())
        );
    }

    if effect_tuner.increase_hold.update(
        ctrl_pressed && keys.just_pressed(KeyCode::ArrowRight),
        ctrl_pressed && keys.pressed(KeyCode::ArrowRight),
        !ctrl_pressed || keys.just_released(KeyCode::ArrowRight),
        delta_secs,
    ) {
        effect_tuner.adjust_selected(1.0, shift_pressed, alt_pressed, now_secs);
        println!(
            "{}",
            selected_status_message(&effect_tuner, time.elapsed_secs())
        );
    }

    if keys.just_pressed(KeyCode::Enter) {
        if shift_pressed {
            effect_tuner.reset_all(now_secs);
            println!("Reset all FX settings and LFOs to defaults.");
        } else {
            effect_tuner.reset_selected(now_secs);
            println!(
                "Reset {}.",
                selected_status_message(&effect_tuner, time.elapsed_secs())
            );
        }
    }
}

pub(crate) fn apply_effect_tuner_system(
    time: Res<Time>,
    effect_tuner: Res<EffectTunerState>,
    mut camera_effects: Query<&mut CameraEffectsSettings, With<SceneCamera>>,
) {
    if !effect_tuner.is_changed() && !effect_tuner.has_active_lfos() {
        return;
    }

    let Ok(mut camera_effects) = camera_effects.single_mut() else {
        return;
    };

    *camera_effects =
        camera_effects_from_config(&effect_tuner.evaluated_effects(time.elapsed_secs()));
}

fn default_lfos() -> Vec<ParameterLfo> {
    EffectNumericParameter::all()
        .iter()
        .copied()
        .map(ParameterLfo::default_for)
        .collect()
}

fn selected_status_message(effect_tuner: &EffectTunerState, now_secs: f32) -> String {
    let parameter = effect_tuner.selected_parameter();
    let lfo = effect_tuner.selected_lfo();
    let live_effects = effect_tuner.evaluated_effects(now_secs);
    match effect_tuner.edit_mode {
        EffectEditMode::Value => format!(
            "{} = {} (live {})",
            parameter.label(),
            parameter.display_value(&effect_tuner.current),
            parameter.display_value(&live_effects)
        ),
        EffectEditMode::LfoAmplitude => {
            format!("{} lfo amplitude = {:.3}", parameter.label(), lfo.amplitude)
        }
        EffectEditMode::LfoFrequency => {
            format!(
                "{} lfo frequency = {:.3}Hz",
                parameter.label(),
                lfo.frequency_hz
            )
        }
        EffectEditMode::LfoShape => {
            format!("{} lfo shape = {}", parameter.label(), lfo.shape.label())
        }
    }
}

fn modifier_pressed(keys: &ButtonInput<KeyCode>, key_codes: &[KeyCode]) -> bool {
    key_codes
        .iter()
        .copied()
        .any(|key_code| keys.pressed(key_code))
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_LFO_FREQUENCY_HZ, EffectEditMode, EffectGroup, EffectNumericParameter,
        EffectOverlayField, EffectTunerState, LfoShape,
    };
    use crate::config::EffectsConfig;

    #[test]
    fn selected_effect_matches_parameter_group() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        effect_tuner.selected_index = 17;

        assert_eq!(effect_tuner.selected_effect(), EffectGroup::Bloom);
    }

    #[test]
    fn toggle_selected_lfo_updates_enabled_state() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        effect_tuner.selected_index = 13;

        let enabled = effect_tuner.toggle_selected_lfo(1.0);

        assert!(enabled);
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
        effect_tuner.selected_index = 3;
        effect_tuner.edit_mode = EffectEditMode::LfoShape;
        effect_tuner.pinned = true;

        let snapshot = effect_tuner.overlay_snapshot(0.0);

        assert_eq!(snapshot.effect_label, "lens");
        assert_eq!(snapshot.parameter_label, "k2");
        assert_eq!(snapshot.active_field, EffectOverlayField::LfoShape);
        assert!(snapshot.pinned);
    }

    #[test]
    fn stepped_random_holds_its_value_within_a_cycle() {
        let first = LfoShape::SteppedRandom.sample(0.10, 17);
        let second = LfoShape::SteppedRandom.sample(0.90, 17);

        assert_eq!(first, second);
    }

    #[test]
    fn brownian_motion_is_bounded_and_smooth() {
        let first = LfoShape::BrownianMotion.sample(0.50, 23);
        let second = LfoShape::BrownianMotion.sample(0.52, 23);

        assert!(first.abs() <= 1.0);
        assert!(second.abs() <= 1.0);
        assert!((second - first).abs() < 0.2);
    }

    #[test]
    fn lfo_shape_cycle_includes_random_shapes() {
        assert_eq!(LfoShape::Square.cycle(1.0), LfoShape::SteppedRandom);
        assert_eq!(LfoShape::SteppedRandom.cycle(1.0), LfoShape::BrownianMotion);
        assert_eq!(LfoShape::BrownianMotion.cycle(1.0), LfoShape::Sine);
    }

    #[test]
    fn reset_selected_restores_lfo_frequency_default() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        effect_tuner.selected_index = 16;
        effect_tuner.edit_mode = EffectEditMode::LfoFrequency;
        effect_tuner.selected_lfo_mut().frequency_hz = 3.0;

        effect_tuner.reset_selected(1.0);

        assert_eq!(
            effect_tuner.selected_lfo().frequency_hz,
            DEFAULT_LFO_FREQUENCY_HZ
        );
    }

    #[test]
    fn reset_all_restores_effect_enable_defaults_and_disables_lfos() {
        let mut defaults = EffectsConfig::default();
        defaults.edge_detection.enabled = true;
        let mut effect_tuner = EffectTunerState::from_config(&defaults);
        effect_tuner.current.edge_detection.enabled = false;
        effect_tuner.selected_index = 22;
        effect_tuner.selected_lfo_mut().enabled = true;
        effect_tuner.selected_lfo_mut().shape = LfoShape::Square;

        effect_tuner.reset_all(1.0);

        assert!(effect_tuner.current.edge_detection.enabled);
        assert!(!effect_tuner.selected_lfo().enabled);
        assert_eq!(effect_tuner.selected_lfo().shape, LfoShape::Sine);
    }

    #[test]
    fn integer_parameters_round_when_set() {
        let mut effects = EffectsConfig::default();

        EffectNumericParameter::BloomRadius.set_value(&mut effects, 7.6);

        assert_eq!(effects.bloom.radius_pixels, 8);
    }
}
