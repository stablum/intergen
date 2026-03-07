use bevy::prelude::*;

use crate::camera::SceneCamera;
use crate::config::EffectsConfig;
use crate::effects::{CameraEffectsSettings, camera_effects_from_config};

const OVERLAY_HOLD_SECS: f32 = 2.5;
const HOLD_DELAY_SECS: f32 = 0.32;
const REPEAT_INTERVAL_SECS: f32 = 0.08;

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
    selected_index: usize,
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
            selected_index: 0,
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

    pub(crate) fn is_visible(&self, now_secs: f32) -> bool {
        self.pinned || now_secs <= self.visible_until_secs
    }

    pub(crate) fn overlay_text(&self) -> String {
        let parameter = self.selected_parameter();
        format!(
            concat!(
                "FX Tuner{}\n",
                "{}: {}\n",
                "default {}  ctrl+arrows adjust/select  shift coarse  alt fine  enter reset"
            ),
            if self.pinned { " [Pinned]" } else { "" },
            parameter.label(),
            parameter.display_value(&self.current),
            parameter.display_value(&self.defaults),
        )
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

    fn adjust_selected(
        &mut self,
        direction: f32,
        shift_pressed: bool,
        alt_pressed: bool,
        now_secs: f32,
    ) {
        let parameter = self.selected_parameter();
        let current_value = parameter.value(&self.current);
        let next_value =
            current_value + direction * parameter.adjustment_step(shift_pressed, alt_pressed);
        parameter.set_value(&mut self.current, next_value);
        self.note_interaction(now_secs);
    }

    fn reset_selected(&mut self, now_secs: f32) {
        let parameter = self.selected_parameter();
        parameter.set_value(&mut self.current, parameter.value(&self.defaults));
        self.note_interaction(now_secs);
    }

    fn reset_all(&mut self, now_secs: f32) {
        self.current = self.defaults.clone();
        self.note_interaction(now_secs);
    }
}

pub(crate) fn effect_tuner_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut effect_tuner: ResMut<EffectTunerState>,
) {
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
            "{} = {}",
            effect_tuner.selected_parameter().label(),
            effect_tuner
                .selected_parameter()
                .display_value(&effect_tuner.current)
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
            "{} = {}",
            effect_tuner.selected_parameter().label(),
            effect_tuner
                .selected_parameter()
                .display_value(&effect_tuner.current)
        );
    }

    if keys.just_pressed(KeyCode::Enter) {
        if shift_pressed {
            effect_tuner.reset_all(now_secs);
            println!("Reset all FX parameters to config defaults.");
        } else {
            let selected_parameter = effect_tuner.selected_parameter();
            effect_tuner.reset_selected(now_secs);
            println!(
                "Reset {} to {}.",
                selected_parameter.label(),
                selected_parameter.display_value(&effect_tuner.current)
            );
        }
    }
}

pub(crate) fn apply_effect_tuner_system(
    effect_tuner: Res<EffectTunerState>,
    mut camera_effects: Query<&mut CameraEffectsSettings, With<SceneCamera>>,
) {
    if !effect_tuner.is_changed() {
        return;
    }

    let Ok(mut camera_effects) = camera_effects.single_mut() else {
        return;
    };

    *camera_effects = camera_effects_from_config(&effect_tuner.current);
}

fn modifier_pressed(keys: &ButtonInput<KeyCode>, key_codes: &[KeyCode]) -> bool {
    key_codes
        .iter()
        .copied()
        .any(|key_code| keys.pressed(key_code))
}

#[cfg(test)]
mod tests {
    use super::{EffectNumericParameter, EffectTunerState};
    use crate::config::EffectsConfig;

    #[test]
    fn reset_selected_restores_config_default() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
        effect_tuner.current.bloom.intensity = 2.5;
        effect_tuner.selected_index = 16;

        effect_tuner.reset_selected(1.0);

        assert_eq!(
            effect_tuner.current.bloom.intensity,
            effect_tuner.defaults.bloom.intensity
        );
    }

    #[test]
    fn integer_parameters_round_when_set() {
        let mut effects = EffectsConfig::default();

        EffectNumericParameter::BloomRadius.set_value(&mut effects, 7.6);

        assert_eq!(effects.bloom.radius_pixels, 8);
    }
}
