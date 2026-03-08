use bevy::prelude::*;

use crate::camera::SceneCamera;
use crate::effects::{CameraEffectsSettings, camera_effects_from_config};
use crate::presets::PresetBrowserState;

use super::state::{AdjustmentModifiers, EffectTunerState, HoldInput};

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
            if effect_tuner.is_pinned() {
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
        println!("FX tuner edit mode: {}.", effect_tuner.edit_mode_label());
    }

    let ctrl_pressed = modifier_pressed(&keys, &[KeyCode::ControlLeft, KeyCode::ControlRight]);
    let modifiers = AdjustmentModifiers {
        shift_pressed: modifier_pressed(&keys, &[KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        alt_pressed: modifier_pressed(&keys, &[KeyCode::AltLeft, KeyCode::AltRight]),
    };
    let delta_secs = time.delta_secs();

    if effect_tuner.step_selection(
        -1,
        HoldInput {
            just_pressed: ctrl_pressed && keys.just_pressed(KeyCode::ArrowUp),
            pressed: ctrl_pressed && keys.pressed(KeyCode::ArrowUp),
            just_released: !ctrl_pressed || keys.just_released(KeyCode::ArrowUp),
            delta_secs,
        },
        now_secs,
    ) {
        println!(
            "Selected FX parameter: {}",
            effect_tuner.selected_parameter().label()
        );
    }

    if effect_tuner.step_selection(
        1,
        HoldInput {
            just_pressed: ctrl_pressed && keys.just_pressed(KeyCode::ArrowDown),
            pressed: ctrl_pressed && keys.pressed(KeyCode::ArrowDown),
            just_released: !ctrl_pressed || keys.just_released(KeyCode::ArrowDown),
            delta_secs,
        },
        now_secs,
    ) {
        println!(
            "Selected FX parameter: {}",
            effect_tuner.selected_parameter().label()
        );
    }

    if effect_tuner.step_adjustment(
        -1.0,
        HoldInput {
            just_pressed: ctrl_pressed && keys.just_pressed(KeyCode::ArrowLeft),
            pressed: ctrl_pressed && keys.pressed(KeyCode::ArrowLeft),
            just_released: !ctrl_pressed || keys.just_released(KeyCode::ArrowLeft),
            delta_secs,
        },
        modifiers,
        now_secs,
    ) {
        println!("{}", effect_tuner.selected_status_message(now_secs));
    }

    if effect_tuner.step_adjustment(
        1.0,
        HoldInput {
            just_pressed: ctrl_pressed && keys.just_pressed(KeyCode::ArrowRight),
            pressed: ctrl_pressed && keys.pressed(KeyCode::ArrowRight),
            just_released: !ctrl_pressed || keys.just_released(KeyCode::ArrowRight),
            delta_secs,
        },
        modifiers,
        now_secs,
    ) {
        println!("{}", effect_tuner.selected_status_message(now_secs));
    }

    if keys.just_pressed(KeyCode::Enter) {
        if modifiers.shift_pressed {
            effect_tuner.reset_all(now_secs);
            println!("Reset all FX settings and LFOs to defaults.");
        } else {
            effect_tuner.reset_selected(now_secs);
            println!("Reset {}.", effect_tuner.selected_status_message(now_secs));
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

fn modifier_pressed(keys: &ButtonInput<KeyCode>, key_codes: &[KeyCode]) -> bool {
    key_codes
        .iter()
        .copied()
        .any(|key_code| keys.pressed(key_code))
}
