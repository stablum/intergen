use bevy::{input::keyboard::KeyboardInput, prelude::*};

use crate::camera::SceneCamera;
use crate::control_page::{ControlPage, ControlPageState};
use crate::effects::{CameraEffectsSettings, camera_effects_from_config};

use super::state::{AdjustmentModifiers, EffectTunerState};
use crate::parameters::HoldInput;

pub(crate) fn effect_tuner_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    control_page: Res<ControlPageState>,
    mut keyboard_input_reader: MessageReader<KeyboardInput>,
    mut effect_tuner: ResMut<EffectTunerState>,
) {
    if !control_page.page_has_focus(ControlPage::EffectTuner) {
        return;
    }

    let now_secs = time.elapsed_secs();
    let ctrl_pressed = modifier_pressed(&keys, &[KeyCode::ControlLeft, KeyCode::ControlRight]);
    let modifiers = AdjustmentModifiers {
        shift_pressed: modifier_pressed(&keys, &[KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        alt_pressed: modifier_pressed(&keys, &[KeyCode::AltLeft, KeyCode::AltRight]),
    };
    let delta_secs = time.delta_secs();

    if keys.just_pressed(KeyCode::Space) {
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

    if keys.just_pressed(KeyCode::Tab) {
        let direction = if modifiers.shift_pressed { -1 } else { 1 };
        effect_tuner.step_edit_mode(direction, now_secs);
        println!("FX tuner edit mode: {}.", effect_tuner.edit_mode_label());
    }

    if keys.just_pressed(KeyCode::ArrowLeft) {
        effect_tuner.step_edit_mode(-1, now_secs);
        println!("FX tuner edit mode: {}.", effect_tuner.edit_mode_label());
    }

    if keys.just_pressed(KeyCode::ArrowRight) {
        effect_tuner.step_edit_mode(1, now_secs);
        println!("FX tuner edit mode: {}.", effect_tuner.edit_mode_label());
    }

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
        1.0,
        HoldInput {
            just_pressed: !ctrl_pressed && keys.just_pressed(KeyCode::ArrowUp),
            pressed: !ctrl_pressed && keys.pressed(KeyCode::ArrowUp),
            just_released: ctrl_pressed || keys.just_released(KeyCode::ArrowUp),
            delta_secs,
        },
        modifiers,
        now_secs,
    ) {
        println!("{}", effect_tuner.selected_status_message(now_secs));
    }

    if effect_tuner.step_adjustment(
        -1.0,
        HoldInput {
            just_pressed: !ctrl_pressed && keys.just_pressed(KeyCode::ArrowDown),
            pressed: !ctrl_pressed && keys.pressed(KeyCode::ArrowDown),
            just_released: ctrl_pressed || keys.just_released(KeyCode::ArrowDown),
            delta_secs,
        },
        modifiers,
        now_secs,
    ) {
        println!("{}", effect_tuner.selected_status_message(now_secs));
    }

    let enter_pressed =
        keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter);
    if enter_pressed {
        if modifiers.shift_pressed {
            effect_tuner.reset_all(now_secs);
            println!("Reset all FX settings and LFOs to defaults.");
        } else {
            effect_tuner.reset_selected(now_secs);
            println!("Reset {}.", effect_tuner.selected_status_message(now_secs));
        }
    }

    if keys.just_pressed(KeyCode::Backspace) {
        let _ = effect_tuner.backspace_numeric_input(now_secs);
    }

    if ctrl_pressed || modifiers.alt_pressed {
        return;
    }

    for keyboard_input in keyboard_input_reader.read() {
        if !keyboard_input.state.is_pressed() || keyboard_input.repeat {
            continue;
        }

        let Some(inserted_text) = &keyboard_input.text else {
            continue;
        };

        for character in inserted_text
            .chars()
            .filter(|character| is_numeric_entry_char(*character))
        {
            let _ = effect_tuner.append_numeric_input(character, now_secs);
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

fn is_numeric_entry_char(character: char) -> bool {
    matches!(character, '0'..='9' | '.' | '-' | '+')
}
