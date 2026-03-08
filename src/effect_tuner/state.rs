use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::EffectsConfig;

use super::lfo::{DEFAULT_LFO_FREQUENCY_HZ, LFO_FREQUENCY_STEP_HZ, LfoShape, ParameterLfo};
use super::metadata::{EffectEditMode, EffectGroup, EffectNumericParameter, EffectOverlayField};

const OVERLAY_HOLD_SECS: f32 = 2.5;
const HOLD_DELAY_SECS: f32 = 0.32;
const REPEAT_INTERVAL_SECS: f32 = 0.08;

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

#[derive(Clone, Copy)]
pub(crate) struct HoldInput {
    pub(crate) just_pressed: bool,
    pub(crate) pressed: bool,
    pub(crate) just_released: bool,
    pub(crate) delta_secs: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct AdjustmentModifiers {
    pub(crate) shift_pressed: bool,
    pub(crate) alt_pressed: bool,
}

#[derive(Default, Clone)]
struct RepeatHoldState {
    elapsed_secs: f32,
    repeating: bool,
}

impl RepeatHoldState {
    fn update(&mut self, input: HoldInput) -> bool {
        if input.just_released || !input.pressed {
            self.reset();
            return false;
        }

        if input.just_pressed {
            self.reset();
            return true;
        }

        self.elapsed_secs += input.delta_secs;
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
    defaults: EffectsConfig,
    current: EffectsConfig,
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

    pub(crate) fn is_pinned(&self) -> bool {
        self.pinned
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

    pub(crate) fn edit_mode_label(&self) -> &'static str {
        self.edit_mode.label()
    }

    pub(crate) fn toggle_pinned(&mut self, now_secs: f32) {
        self.pinned = !self.pinned;
        self.note_interaction(now_secs);
    }

    pub(crate) fn toggle_selected_effect(&mut self, now_secs: f32) -> bool {
        let effect = self.selected_effect();
        let next_enabled = !effect.is_enabled(&self.current);
        effect.set_enabled(&mut self.current, next_enabled);
        self.note_interaction(now_secs);
        next_enabled
    }

    pub(crate) fn toggle_selected_lfo(&mut self, now_secs: f32) -> bool {
        let lfo = self.selected_lfo_mut();
        lfo.enabled = !lfo.enabled;
        let enabled = lfo.enabled;
        self.note_interaction(now_secs);
        enabled
    }

    pub(crate) fn cycle_edit_mode(&mut self, now_secs: f32) {
        self.edit_mode = self.edit_mode.next();
        self.note_interaction(now_secs);
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

        if hold.update(input) {
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
        now_secs: f32,
    ) -> bool {
        let hold = if direction < 0.0 {
            &mut self.decrease_hold
        } else {
            &mut self.increase_hold
        };

        if hold.update(input) {
            self.adjust_selected(direction, modifiers, now_secs);
            true
        } else {
            false
        }
    }

    pub(crate) fn reset_selected(&mut self, now_secs: f32) {
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

    pub(crate) fn reset_all(&mut self, now_secs: f32) {
        self.current = self.defaults.clone();
        self.lfos = default_lfos();
        self.edit_mode = EffectEditMode::Value;
        self.note_interaction(now_secs);
    }

    pub(crate) fn selected_status_message(&self, now_secs: f32) -> String {
        let parameter = self.selected_parameter();
        let lfo = self.selected_lfo();
        let live_effects = self.evaluated_effects(now_secs);
        match self.edit_mode {
            EffectEditMode::Value => format!(
                "{} = {} (live {})",
                parameter.label(),
                parameter.display_value(&self.current),
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

    fn note_interaction(&mut self, now_secs: f32) {
        self.visible_until_secs = now_secs + OVERLAY_HOLD_SECS;
    }

    fn cycle_selection(&mut self, direction: isize, now_secs: f32) {
        let parameter_count = EffectNumericParameter::all().len() as isize;
        let next_index =
            (self.selected_index as isize + direction).rem_euclid(parameter_count) as usize;
        self.selected_index = next_index;
        self.note_interaction(now_secs);
    }

    fn adjust_selected(&mut self, direction: f32, modifiers: AdjustmentModifiers, now_secs: f32) {
        let parameter = self.selected_parameter();
        match self.edit_mode {
            EffectEditMode::Value => {
                let current_value = parameter.value(&self.current);
                let next_value = current_value
                    + direction
                        * parameter.adjustment_step(modifiers.shift_pressed, modifiers.alt_pressed);
                parameter.set_value(&mut self.current, next_value);
            }
            EffectEditMode::LfoAmplitude => {
                let step =
                    parameter.adjustment_step(modifiers.shift_pressed, modifiers.alt_pressed);
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
        self.lfos[self.selected_index]
    }

    fn selected_lfo_mut(&mut self) -> &mut ParameterLfo {
        &mut self.lfos[self.selected_index]
    }
}

fn default_lfos() -> Vec<ParameterLfo> {
    EffectNumericParameter::all()
        .iter()
        .copied()
        .map(ParameterLfo::default_for)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::config::EffectsConfig;

    use super::EffectTunerState;
    use crate::effect_tuner::lfo::{DEFAULT_LFO_FREQUENCY_HZ, LfoShape};
    use crate::effect_tuner::metadata::{EffectEditMode, EffectGroup, EffectOverlayField};

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
}
