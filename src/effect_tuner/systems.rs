use bevy::{
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseScrollUnit, MouseWheel},
    },
    prelude::*,
};

use crate::camera::SceneCamera;
use crate::control_page::{ControlPage, ControlPageState};
use crate::effects::{CameraEffectsSettings, camera_effects_from_config};
use crate::generation::{apply_live_material_state, recompute_generation_tree};
use crate::runtime_scene::GenerationSceneAccess;
use crate::scene::sync_stage_entities;

use super::state::{
    AdjustmentModifiers, EffectTunerEditContext, EffectTunerParameter, EffectTunerSceneParameter,
    EffectTunerState, EffectTunerViewContext,
};
use crate::parameters::HoldInput;

pub(crate) fn effect_tuner_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    control_page: Res<ControlPageState>,
    mut keyboard_input_reader: MessageReader<KeyboardInput>,
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    mut mouse_wheel_selection_remainder: Local<f32>,
    mut effect_tuner: ResMut<EffectTunerState>,
    mut scene: GenerationSceneAccess,
) {
    if !control_page.page_has_focus(ControlPage::EffectTuner) {
        *mouse_wheel_selection_remainder = 0.0;
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
        let view = effect_tuner_view_context(
            &scene.app_config,
            &scene.generation_state,
            &scene.material_state,
            &scene.stage_state,
        );
        if let Some(enabled) = effect_tuner.toggle_selected_lfo(&view, now_secs) {
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
        println!(
            "Selected control: {}",
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
            "Selected control: {}",
            effect_tuner.selected_parameter().label()
        );
    }

    let mut scrolled_selection = false;
    for mouse_wheel in mouse_wheel_reader.read() {
        *mouse_wheel_selection_remainder +=
            mouse_wheel_selection_lines(mouse_wheel.y, mouse_wheel.unit);
    }
    let selection_steps = mouse_wheel_selection_whole_steps(*mouse_wheel_selection_remainder);
    if selection_steps != 0 {
        *mouse_wheel_selection_remainder -= selection_steps as f32;
        let direction = if selection_steps > 0 { -1 } else { 1 };
        for _ in 0..selection_steps.unsigned_abs() {
            if effect_tuner.scroll_selection(direction, now_secs) {
                scrolled_selection = true;
            }
        }
    }
    if scrolled_selection {
        println!(
            "Selected control: {}",
            effect_tuner.selected_parameter().label()
        );
    }

    let adjusted_up = {
        restore_selected_scene_parameter_base_if_needed(&mut effect_tuner, &mut scene);
        let mut context = effect_tuner_edit_context(
            &scene.app_config,
            &mut scene.generation_state,
            &mut scene.material_state,
            &mut scene.stage_state,
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
        sync_selected_scene_parameter_base_if_needed(&mut effect_tuner, &scene);
        println!(
            "{}",
            effect_tuner.selected_status_message(
                &effect_tuner_view_context(
                    &scene.app_config,
                    &scene.generation_state,
                    &scene.material_state,
                    &scene.stage_state,
                ),
                now_secs,
            )
        );
    }

    let adjusted_down = {
        restore_selected_scene_parameter_base_if_needed(&mut effect_tuner, &mut scene);
        let mut context = effect_tuner_edit_context(
            &scene.app_config,
            &mut scene.generation_state,
            &mut scene.material_state,
            &mut scene.stage_state,
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
        sync_selected_scene_parameter_base_if_needed(&mut effect_tuner, &scene);
        println!(
            "{}",
            effect_tuner.selected_status_message(
                &effect_tuner_view_context(
                    &scene.app_config,
                    &scene.generation_state,
                    &scene.material_state,
                    &scene.stage_state,
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
                    &mut scene.stage_state,
                );
                effect_tuner.reset_all(&mut context, now_secs);
            }
            effect_tuner.sync_material_scene_lfo_bases(&scene.material_state);
            apply_reset_all_side_effects(&mut scene);
            println!("Reset all F2 controls to defaults.");
        } else if effect_tuner.finalize_numeric_entry(now_secs) {
            println!(
                "Set {}.",
                effect_tuner.selected_status_message(
                    &effect_tuner_view_context(
                        &scene.app_config,
                        &scene.generation_state,
                        &scene.material_state,
                        &scene.stage_state,
                    ),
                    now_secs,
                )
            );
        } else {
            let selected_parameter = effect_tuner.selected_parameter();
            restore_selected_scene_parameter_base_if_needed(&mut effect_tuner, &mut scene);
            {
                let mut context = effect_tuner_edit_context(
                    &scene.app_config,
                    &mut scene.generation_state,
                    &mut scene.material_state,
                    &mut scene.stage_state,
                );
                effect_tuner.reset_selected(&mut context, now_secs);
            }
            apply_selected_parameter_side_effects(selected_parameter, &mut scene);
            effect_tuner
                .sync_scene_parameter_base_if_needed(selected_parameter, &scene.material_state);
            println!(
                "Reset {}.",
                effect_tuner.selected_status_message(
                    &effect_tuner_view_context(
                        &scene.app_config,
                        &scene.generation_state,
                        &scene.material_state,
                        &scene.stage_state,
                    ),
                    now_secs,
                )
            );
        }
    }

    if keys.just_pressed(KeyCode::Backspace) {
        let selected_parameter = effect_tuner.selected_parameter();
        let changed = {
            restore_selected_scene_parameter_base_if_needed(&mut effect_tuner, &mut scene);
            let mut context = effect_tuner_edit_context(
                &scene.app_config,
                &mut scene.generation_state,
                &mut scene.material_state,
                &mut scene.stage_state,
            );
            effect_tuner.backspace_numeric_input(&mut context, now_secs)
        };
        if changed {
            apply_selected_parameter_side_effects(selected_parameter, &mut scene);
            effect_tuner
                .sync_scene_parameter_base_if_needed(selected_parameter, &scene.material_state);
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
                restore_selected_scene_parameter_base_if_needed(&mut effect_tuner, &mut scene);
                let mut context = effect_tuner_edit_context(
                    &scene.app_config,
                    &mut scene.generation_state,
                    &mut scene.material_state,
                    &mut scene.stage_state,
                );
                effect_tuner.append_numeric_input(character, &mut context, now_secs)
            };
            if changed {
                apply_selected_parameter_side_effects(selected_parameter, &mut scene);
                effect_tuner
                    .sync_scene_parameter_base_if_needed(selected_parameter, &scene.material_state);
            }
        }
    }
}

pub(crate) fn apply_effect_tuner_system(
    time: Res<Time>,
    mut effect_tuner: ResMut<EffectTunerState>,
    mut scene: GenerationSceneAccess,
    mut camera_effects: Query<&mut CameraEffectsSettings, With<SceneCamera>>,
) {
    let should_update_effects = effect_tuner.is_changed() || effect_tuner.has_active_effect_lfos();
    if effect_tuner.needs_scene_lfo_application() {
        let lfo_result = effect_tuner.apply_scene_lfos(
            time.elapsed_secs(),
            &scene.app_config.generation,
            &mut scene.generation_state,
            &scene.app_config.materials,
            &mut scene.material_state,
        );
        if lfo_result.generation_changed {
            recompute_generation_tree(&mut scene);
        }
        if lfo_result.materials_changed {
            apply_live_material_state(
                &scene.generation_state,
                &scene.app_config.materials,
                &scene.material_state,
                &mut scene.materials,
                &scene.shape_materials,
            );
        }
    }

    if !should_update_effects {
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

fn mouse_wheel_selection_lines(delta_y: f32, unit: MouseScrollUnit) -> f32 {
    match unit {
        MouseScrollUnit::Line => delta_y,
        MouseScrollUnit::Pixel => delta_y / 40.0,
    }
}

fn mouse_wheel_selection_whole_steps(lines: f32) -> isize {
    if lines > 0.0 {
        lines.floor() as isize
    } else if lines < 0.0 {
        lines.ceil() as isize
    } else {
        0
    }
}

fn is_numeric_entry_char(character: char) -> bool {
    matches!(character, '0'..='9' | '.' | ',' | '-' | '+')
}

fn restore_selected_scene_parameter_base_if_needed(
    effect_tuner: &mut EffectTunerState,
    scene: &mut GenerationSceneAccess<'_, '_>,
) {
    let selected_parameter = effect_tuner.selected_parameter();
    effect_tuner.restore_scene_parameter_base_if_needed(
        selected_parameter,
        &scene.app_config.materials,
        &mut scene.material_state,
    );
}

fn sync_selected_scene_parameter_base_if_needed(
    effect_tuner: &mut EffectTunerState,
    scene: &GenerationSceneAccess<'_, '_>,
) {
    effect_tuner.sync_scene_parameter_base_if_needed(
        effect_tuner.selected_parameter(),
        &scene.material_state,
    );
}

fn effect_tuner_view_context<'a>(
    app_config: &'a crate::config::AppConfig,
    generation_state: &'a crate::scene::GenerationState,
    material_state: &'a crate::scene::MaterialState,
    stage_state: &'a crate::scene::StageState,
) -> EffectTunerViewContext<'a> {
    EffectTunerViewContext {
        generation_config: &app_config.generation,
        generation_state,
        material_config: &app_config.materials,
        material_state,
        stage_state,
    }
}

fn effect_tuner_edit_context<'a>(
    app_config: &'a crate::config::AppConfig,
    generation_state: &'a mut crate::scene::GenerationState,
    material_state: &'a mut crate::scene::MaterialState,
    stage_state: &'a mut crate::scene::StageState,
) -> EffectTunerEditContext<'a> {
    EffectTunerEditContext {
        generation_config: &app_config.generation,
        generation_state,
        material_config: &app_config.materials,
        material_state,
        stage_config: &app_config.rendering.stage,
        stage_state,
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
        EffectTunerParameter::Scene(EffectTunerSceneParameter::GlobalOpacity)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialHueStepPerLevel)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialSaturation)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialLightness)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialMetallic)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialPerceptualRoughness)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialReflectance)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialCubeHueBias)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialTetrahedronHueBias)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialOctahedronHueBias)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialDodecahedronHueBias)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialSurfaceMode)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialBaseSurface)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialRootSurface)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialAccentSurface)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialAccentEveryNLevels)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialLevelLightnessShift)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialLevelSaturationShift)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialLevelMetallicShift)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialLevelRoughnessShift)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::MaterialLevelReflectanceShift) => {
            apply_live_material_state(
                &scene.generation_state,
                &scene.app_config.materials,
                &scene.material_state,
                &mut scene.materials,
                &scene.shape_materials,
            );
        }
        EffectTunerParameter::Scene(EffectTunerSceneParameter::StageEnabled)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::StageFloorEnabled)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::StageBackdropEnabled) => {
            sync_stage_entities(
                &mut scene.commands,
                &mut scene.meshes,
                &mut scene.materials,
                &scene.app_config.rendering,
                &scene.stage_state,
                &scene.stage_entities,
            );
        }
        EffectTunerParameter::Effect(_)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::ChildKind)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::SpawnPlacementMode)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::SpawnAddMode)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::ChildScaleRatio)
        | EffectTunerParameter::Scene(EffectTunerSceneParameter::ChildSpawnExclusionProbability) => {
        }
    }
}

fn apply_reset_all_side_effects(scene: &mut GenerationSceneAccess<'_, '_>) {
    recompute_generation_tree(scene);
    apply_live_material_state(
        &scene.generation_state,
        &scene.app_config.materials,
        &scene.material_state,
        &mut scene.materials,
        &scene.shape_materials,
    );
    sync_stage_entities(
        &mut scene.commands,
        &mut scene.meshes,
        &mut scene.materials,
        &scene.app_config.rendering,
        &scene.stage_state,
        &scene.stage_entities,
    );
}

#[cfg(test)]
mod tests {
    use bevy::input::mouse::MouseScrollUnit;

    use super::{mouse_wheel_selection_lines, mouse_wheel_selection_whole_steps};

    #[test]
    fn mouse_wheel_selection_lines_handles_line_and_pixel_units() {
        assert_eq!(mouse_wheel_selection_lines(1.0, MouseScrollUnit::Line), 1.0);
        assert_eq!(
            mouse_wheel_selection_lines(-2.0, MouseScrollUnit::Line),
            -2.0
        );
        assert_eq!(
            mouse_wheel_selection_lines(80.0, MouseScrollUnit::Pixel),
            2.0
        );
        assert_eq!(
            mouse_wheel_selection_lines(-20.0, MouseScrollUnit::Pixel),
            -0.5
        );
    }

    #[test]
    fn mouse_wheel_selection_whole_steps_preserves_partial_scroll() {
        assert_eq!(mouse_wheel_selection_whole_steps(0.8), 0);
        assert_eq!(mouse_wheel_selection_whole_steps(1.0), 1);
        assert_eq!(mouse_wheel_selection_whole_steps(2.4), 2);
        assert_eq!(mouse_wheel_selection_whole_steps(-0.8), 0);
        assert_eq!(mouse_wheel_selection_whole_steps(-1.0), -1);
        assert_eq!(mouse_wheel_selection_whole_steps(-1.6), -1);
    }
}
