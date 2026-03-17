use bevy::{input::keyboard::KeyboardInput, prelude::*};

use crate::camera::SceneCamera;
use crate::control_page::{ControlPage, ControlPageState};
use crate::effects::{CameraEffectsSettings, camera_effects_from_config};
use crate::generation::{apply_global_opacity, recompute_generation_tree};
use crate::runtime_scene::GenerationSceneAccess;

use super::state::{
    AdjustmentModifiers, EffectTunerEditContext, EffectTunerParameter,
    EffectTunerSceneParameter, EffectTunerState, EffectTunerViewContext,
};
use crate::parameters::HoldInput;

pub(crate) fn effect_tuner_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    control_page: Res<ControlPageState>,
    mut keyboard_input_reader: MessageReader<KeyboardInput>,
    mut effect_tuner: ResMut<EffectTunerState>,
    mut scene: GenerationSceneAccess,
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
        if let Some(selected_effect) = effect_tuner.selected_effect_group() {
            if let Some(enabled) = effect_tuner.toggle_selected_effect(now_secs) {
                println!(
                    "{} {}.",
                    selected_effect.label(),
                    if enabled { "enabled" } else { "disabled" }
                );
            }
        }
    }

    if keys.just_pressed(KeyCode::KeyL) {
        let selected_parameter = effect_tuner.selected_parameter();
        if let Some(enabled) = effect_tuner.toggle_selected_lfo(now_secs) {
            println!(
                "LFO for {} {}.",
                selected_parameter.label(),
                if enabled { "enabled" } else { "disabled" }
            );
        }
    }

    if keys.just_pressed(KeyCode::Tab) {
        let direction = if modifiers.shift_pressed { -1 } else { 1 };
        if effect_tuner.step_edit_mode(direction, now_secs) {
            println!("F2 tuner edit mode: {}.", effect_tuner.edit_mode_label());
        }
    }

    if keys.just_pressed(KeyCode::ArrowLeft) {
        if effect_tuner.step_edit_mode(-1, now_secs) {
            println!("F2 tuner edit mode: {}.", effect_tuner.edit_mode_label());
        }
    }

    if keys.just_pressed(KeyCode::ArrowRight) {
        if effect_tuner.step_edit_mode(1, now_secs) {
            println!("F2 tuner edit mode: {}.", effect_tuner.edit_mode_label());
        }
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
        println!("Selected control: {}", effect_tuner.selected_parameter().label());
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
        println!("Selected control: {}", effect_tuner.selected_parameter().label());
    }

    let adjusted_up = {
        let mut context = effect_tuner_edit_context(
            &scene.app_config,
            &mut scene.generation_state,
            &mut scene.material_state,
        );
        effect_tuner.step_adjustment(
            1.0,
            HoldInput {
                just_pressed: !ctrl_pressed && keys.just_pressed(KeyCode::ArrowUp),
                pressed: !ctrl_pressed && keys.pressed(KeyCode::ArrowUp),
                just_released: ctrl_pressed || keys.just_released(KeyCode::ArrowUp),
                delta_secs,
            },
            modifiers,
            &mut context,
            now_secs,
        )
    };
    if adjusted_up {
        apply_selected_parameter_side_effects(effect_tuner.selected_parameter(), &mut scene);
        println!(
            "{}",
            effect_tuner.selected_status_message(
                &effect_tuner_view_context(
                    &scene.app_config,
                    &scene.generation_state,
                    &scene.material_state,
                ),
                now_secs,
            )
        );
    }

    let adjusted_down = {
        let mut context = effect_tuner_edit_context(
            &scene.app_config,
            &mut scene.generation_state,
            &mut scene.material_state,
        );
        effect_tuner.step_adjustment(
            -1.0,
            HoldInput {
                just_pressed: !ctrl_pressed && keys.just_pressed(KeyCode::ArrowDown),
                pressed: !ctrl_pressed && keys.pressed(KeyCode::ArrowDown),
                just_released: ctrl_pressed || keys.just_released(KeyCode::ArrowDown),
                delta_secs,
            },
            modifiers,
            &mut context,
            now_secs,
        )
    };
    if adjusted_down {
        apply_selected_parameter_side_effects(effect_tuner.selected_parameter(), &mut scene);
        println!(
            "{}",
            effect_tuner.selected_status_message(
                &effect_tuner_view_context(
                    &scene.app_config,
                    &scene.generation_state,
                    &scene.material_state,
                ),
                now_secs,
            )
        );
    }

    let enter_pressed =
        keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter);
    if enter_pressed {
        if modifiers.shift_pressed {
            {
                let mut context = effect_tuner_edit_context(
                    &scene.app_config,
                    &mut scene.generation_state,
                    &mut scene.material_state,
                );
                effect_tuner.reset_all(&mut context, now_secs);
            }
            apply_reset_all_side_effects(&mut scene);
            println!("Reset all F2 controls to defaults.");
        } else {
            let selected_parameter = effect_tuner.selected_parameter();
            {
                let mut context = effect_tuner_edit_context(
                    &scene.app_config,
                    &mut scene.generation_state,
                    &mut scene.material_state,
                );
                effect_tuner.reset_selected(&mut context, now_secs);
            }
            apply_selected_parameter_side_effects(selected_parameter, &mut scene);
            println!(
                "Reset {}.",
                effect_tuner.selected_status_message(
                    &effect_tuner_view_context(
                        &scene.app_config,
                        &scene.generation_state,
                        &scene.material_state,
                    ),
                    now_secs,
                )
            );
        }
    }

    if keys.just_pressed(KeyCode::Backspace) {
        let selected_parameter = effect_tuner.selected_parameter();
        let changed = {
            let mut context = effect_tuner_edit_context(
                &scene.app_config,
                &mut scene.generation_state,
                &mut scene.material_state,
            );
            effect_tuner.backspace_numeric_input(&mut context, now_secs)
        };
        if changed {
            apply_selected_parameter_side_effects(selected_parameter, &mut scene);
        }
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
            let selected_parameter = effect_tuner.selected_parameter();
            let changed = {
                let mut context = effect_tuner_edit_context(
                    &scene.app_config,
                    &mut scene.generation_state,
                    &mut scene.material_state,
                );
                effect_tuner.append_numeric_input(character, &mut context, now_secs)
            };
            if changed {
                apply_selected_parameter_side_effects(selected_parameter, &mut scene);
            }
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

fn effect_tuner_view_context<'a>(
    app_config: &'a crate::config::AppConfig,
    generation_state: &'a crate::scene::GenerationState,
    material_state: &'a crate::scene::MaterialState,
) -> EffectTunerViewContext<'a> {
    EffectTunerViewContext {
        generation_config: &app_config.generation,
        generation_state,
        material_config: &app_config.materials,
        material_state,
    }
}

fn effect_tuner_edit_context<'a>(
    app_config: &'a crate::config::AppConfig,
    generation_state: &'a mut crate::scene::GenerationState,
    material_state: &'a mut crate::scene::MaterialState,
) -> EffectTunerEditContext<'a> {
    EffectTunerEditContext {
        generation_config: &app_config.generation,
        generation_state,
        material_config: &app_config.materials,
        material_state,
    }
}

fn apply_selected_parameter_side_effects(
    parameter: EffectTunerParameter,
    scene: &mut GenerationSceneAccess<'_, '_>,
) {
    match parameter {
        EffectTunerParameter::Scene(EffectTunerSceneParameter::ChildTwistPerVertexRadians)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::ChildOutwardOffsetRatio) => {
            recompute_generation_tree(scene);
        }
        EffectTunerParameter::Scene(EffectTunerSceneParameter::GlobalOpacity) => {
            apply_global_opacity(
                scene.material_state.opacity,
                &mut scene.materials,
                &scene.polyhedron_materials,
            );
        }
        EffectTunerParameter::Effect(_)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::ChildScaleRatio)
        | EffectTunerParameter::Scene(
            EffectTunerSceneParameter::ChildSpawnExclusionProbability,
        ) => {}
    }
}

fn apply_reset_all_side_effects(scene: &mut GenerationSceneAccess<'_, '_>) {
    recompute_generation_tree(scene);
    apply_global_opacity(
        scene.material_state.opacity,
        &mut scene.materials,
        &scene.polyhedron_materials,
    );
}
