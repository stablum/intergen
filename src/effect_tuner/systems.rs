use bevy::{
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseScrollUnit, MouseWheel},
    },
    prelude::*,
};

use crate::camera::{SceneCamera, sync_scene_camera_transform};
use crate::control_page::{ControlPage, ControlPageState};
use crate::effects::{CameraEffectsSettings, camera_effects_from_config};
use crate::generation::{apply_live_material_state, recompute_generation_tree};
use crate::recent_changes::RecentChangesState;
use crate::runtime_scene::GenerationSceneAccess;
use crate::scene::{apply_live_lighting_state, apply_live_rendering_state, sync_stage_entities};

use super::EffectOverlayField;
use super::state::{
    AdjustmentModifiers, EffectTunerEditContext, EffectTunerParameter, EffectTunerState,
    EffectTunerViewContext, SceneChangeTarget, SceneLfoApplicationResult,
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
    mut recent_changes: ResMut<RecentChangesState>,
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
    let enter_pressed =
        keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter);

    if effect_tuner.page_mode() == super::state::EffectTunerPageMode::GroupSelect {
        if effect_tuner.step_selection(
            -1,
            HoldInput {
                just_pressed: keys.just_pressed(KeyCode::ArrowUp),
                pressed: keys.pressed(KeyCode::ArrowUp),
                just_released: keys.just_released(KeyCode::ArrowUp),
                delta_secs,
            },
            now_secs,
        ) {
            println!("Selected group: {}", effect_tuner.selected_group_label());
        }

        if effect_tuner.step_selection(
            1,
            HoldInput {
                just_pressed: keys.just_pressed(KeyCode::ArrowDown),
                pressed: keys.pressed(KeyCode::ArrowDown),
                just_released: keys.just_released(KeyCode::ArrowDown),
                delta_secs,
            },
            now_secs,
        ) {
            println!("Selected group: {}", effect_tuner.selected_group_label());
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
            println!("Selected group: {}", effect_tuner.selected_group_label());
        }

        if !modifiers.shift_pressed && (enter_pressed || keys.just_pressed(KeyCode::Space)) {
            let selected_group = effect_tuner.selected_group_label();
            effect_tuner.show_selected_group_list_page(now_secs);
            println!("F2 {} parameter list page pinned open.", selected_group);
        }

        return;
    }

    if keys.just_pressed(KeyCode::Space) {
        if let Some(selected_effect) = effect_tuner.selected_effect_group() {
            if let Some(enabled) = effect_tuner.toggle_selected_effect(now_secs) {
                recent_changes.record(
                    format!("{} effect", selected_effect.label()),
                    if enabled { "ON" } else { "OFF" },
                    now_secs,
                );
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
            &scene.camera_rig,
            &scene.generation_state,
            &scene.rendering_state,
            &scene.lighting_state,
            &scene.material_state,
            &scene.stage_state,
        );
        if let Some(enabled) = effect_tuner.toggle_selected_lfo(&view, now_secs) {
            recent_changes.record(
                format!("{} LFO", selected_parameter.label()),
                if enabled { "ON" } else { "OFF" },
                now_secs,
            );
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
            just_pressed: !ctrl_pressed && keys.just_pressed(KeyCode::ArrowUp),
            pressed: !ctrl_pressed && keys.pressed(KeyCode::ArrowUp),
            just_released: ctrl_pressed || keys.just_released(KeyCode::ArrowUp),
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
            just_pressed: !ctrl_pressed && keys.just_pressed(KeyCode::ArrowDown),
            pressed: !ctrl_pressed && keys.pressed(KeyCode::ArrowDown),
            just_released: ctrl_pressed || keys.just_released(KeyCode::ArrowDown),
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

    let editing_selected_value = selected_field_edits_parameter_value(&effect_tuner);

    let adjusted_up = {
        if editing_selected_value {
            restore_selected_scene_parameter_base_if_needed(&mut effect_tuner, &mut scene);
        }
        let mut context = effect_tuner_edit_context(
            &scene.app_config,
            &mut scene.camera_rig,
            &mut scene.generation_state,
            &mut scene.rendering_state,
            &mut scene.lighting_state,
            &mut scene.material_state,
            &mut scene.stage_state,
        );
        effect_tuner.step_adjustment(
            1.0,
            HoldInput {
                just_pressed: ctrl_pressed && keys.just_pressed(KeyCode::ArrowUp),
                pressed: ctrl_pressed && keys.pressed(KeyCode::ArrowUp),
                just_released: !ctrl_pressed || keys.just_released(KeyCode::ArrowUp),
                delta_secs,
            },
            modifiers,
            &mut context,
            now_secs,
        )
    };
    if adjusted_up {
        if editing_selected_value {
            apply_selected_parameter_side_effects(effect_tuner.selected_parameter(), &mut scene);
            sync_selected_scene_parameter_base_if_needed(&mut effect_tuner, &scene);
        }
        record_selected_change(&effect_tuner, &scene, &mut recent_changes, now_secs);
        println!(
            "{}",
            effect_tuner.selected_status_message(
                &effect_tuner_view_context(
                    &scene.app_config,
                    &scene.camera_rig,
                    &scene.generation_state,
                    &scene.rendering_state,
                    &scene.lighting_state,
                    &scene.material_state,
                    &scene.stage_state,
                ),
                now_secs,
            )
        );
    }

    let adjusted_down = {
        if editing_selected_value {
            restore_selected_scene_parameter_base_if_needed(&mut effect_tuner, &mut scene);
        }
        let mut context = effect_tuner_edit_context(
            &scene.app_config,
            &mut scene.camera_rig,
            &mut scene.generation_state,
            &mut scene.rendering_state,
            &mut scene.lighting_state,
            &mut scene.material_state,
            &mut scene.stage_state,
        );
        effect_tuner.step_adjustment(
            -1.0,
            HoldInput {
                just_pressed: ctrl_pressed && keys.just_pressed(KeyCode::ArrowDown),
                pressed: ctrl_pressed && keys.pressed(KeyCode::ArrowDown),
                just_released: !ctrl_pressed || keys.just_released(KeyCode::ArrowDown),
                delta_secs,
            },
            modifiers,
            &mut context,
            now_secs,
        )
    };
    if adjusted_down {
        if editing_selected_value {
            apply_selected_parameter_side_effects(effect_tuner.selected_parameter(), &mut scene);
            sync_selected_scene_parameter_base_if_needed(&mut effect_tuner, &scene);
        }
        record_selected_change(&effect_tuner, &scene, &mut recent_changes, now_secs);
        println!(
            "{}",
            effect_tuner.selected_status_message(
                &effect_tuner_view_context(
                    &scene.app_config,
                    &scene.camera_rig,
                    &scene.generation_state,
                    &scene.rendering_state,
                    &scene.lighting_state,
                    &scene.material_state,
                    &scene.stage_state,
                ),
                now_secs,
            )
        );
    }

    if enter_pressed {
        if modifiers.shift_pressed {
            {
                let mut context = effect_tuner_edit_context(
                    &scene.app_config,
                    &mut scene.camera_rig,
                    &mut scene.generation_state,
                    &mut scene.rendering_state,
                    &mut scene.lighting_state,
                    &mut scene.material_state,
                    &mut scene.stage_state,
                );
                effect_tuner.reset_all(&mut context, now_secs);
            }
            effect_tuner.sync_scene_lfo_bases(&effect_tuner_view_context(
                &scene.app_config,
                &scene.camera_rig,
                &scene.generation_state,
                &scene.rendering_state,
                &scene.lighting_state,
                &scene.material_state,
                &scene.stage_state,
            ));
            apply_reset_all_side_effects(&mut scene);
            recent_changes.record("F2 controls", "defaults", now_secs);
            println!("Reset all F2 controls to defaults.");
        } else if effect_tuner.finalize_numeric_entry(now_secs) {
            record_selected_change(&effect_tuner, &scene, &mut recent_changes, now_secs);
            println!(
                "Set {}.",
                effect_tuner.selected_status_message(
                    &effect_tuner_view_context(
                        &scene.app_config,
                        &scene.camera_rig,
                        &scene.generation_state,
                        &scene.rendering_state,
                        &scene.lighting_state,
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
            if editing_selected_value {
                restore_selected_scene_parameter_base_if_needed(&mut effect_tuner, &mut scene);
            }
            let mut context = effect_tuner_edit_context(
                &scene.app_config,
                &mut scene.camera_rig,
                &mut scene.generation_state,
                &mut scene.rendering_state,
                &mut scene.lighting_state,
                &mut scene.material_state,
                &mut scene.stage_state,
            );
            effect_tuner.backspace_numeric_input(&mut context, now_secs)
        };
        if changed {
            if editing_selected_value {
                apply_selected_parameter_side_effects(selected_parameter, &mut scene);
                sync_selected_scene_parameter_base_if_needed(&mut effect_tuner, &scene);
            }
            record_selected_change(&effect_tuner, &scene, &mut recent_changes, now_secs);
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
                if editing_selected_value {
                    restore_selected_scene_parameter_base_if_needed(&mut effect_tuner, &mut scene);
                }
                let mut context = effect_tuner_edit_context(
                    &scene.app_config,
                    &mut scene.camera_rig,
                    &mut scene.generation_state,
                    &mut scene.rendering_state,
                    &mut scene.lighting_state,
                    &mut scene.material_state,
                    &mut scene.stage_state,
                );
                effect_tuner.append_numeric_input(character, &mut context, now_secs)
            };
            if changed {
                if editing_selected_value {
                    apply_selected_parameter_side_effects(selected_parameter, &mut scene);
                    sync_selected_scene_parameter_base_if_needed(&mut effect_tuner, &scene);
                }
                record_selected_change(&effect_tuner, &scene, &mut recent_changes, now_secs);
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
        let lfo_result = {
            let mut context = effect_tuner_edit_context(
                &scene.app_config,
                &mut scene.camera_rig,
                &mut scene.generation_state,
                &mut scene.rendering_state,
                &mut scene.lighting_state,
                &mut scene.material_state,
                &mut scene.stage_state,
            );
            effect_tuner.apply_scene_lfos(time.elapsed_secs(), &mut context)
        };
        apply_scene_lfo_side_effects(lfo_result, &mut scene);
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

fn selected_field_edits_parameter_value(effect_tuner: &EffectTunerState) -> bool {
    effect_tuner.active_field() == EffectOverlayField::Value
}

fn record_selected_change(
    effect_tuner: &EffectTunerState,
    scene: &GenerationSceneAccess<'_, '_>,
    recent_changes: &mut RecentChangesState,
    now_secs: f32,
) {
    let view = effect_tuner_view_context(
        &scene.app_config,
        &scene.camera_rig,
        &scene.generation_state,
        &scene.rendering_state,
        &scene.lighting_state,
        &scene.material_state,
        &scene.stage_state,
    );
    let (label, value) = effect_tuner.selected_change_entry(&view);
    recent_changes.record(label, value, now_secs);
}

fn restore_selected_scene_parameter_base_if_needed(
    effect_tuner: &mut EffectTunerState,
    scene: &mut GenerationSceneAccess<'_, '_>,
) {
    let selected_parameter = effect_tuner.selected_parameter();
    let mut context = effect_tuner_edit_context(
        &scene.app_config,
        &mut scene.camera_rig,
        &mut scene.generation_state,
        &mut scene.rendering_state,
        &mut scene.lighting_state,
        &mut scene.material_state,
        &mut scene.stage_state,
    );
    effect_tuner.restore_scene_parameter_base_if_needed(selected_parameter, &mut context);
}

fn sync_selected_scene_parameter_base_if_needed(
    effect_tuner: &mut EffectTunerState,
    scene: &GenerationSceneAccess<'_, '_>,
) {
    let view = effect_tuner_view_context(
        &scene.app_config,
        &scene.camera_rig,
        &scene.generation_state,
        &scene.rendering_state,
        &scene.lighting_state,
        &scene.material_state,
        &scene.stage_state,
    );
    effect_tuner.sync_scene_parameter_base_if_needed(effect_tuner.selected_parameter(), &view);
}

fn effect_tuner_view_context<'a>(
    app_config: &'a crate::config::AppConfig,
    camera_rig: &'a crate::camera::CameraRig,
    generation_state: &'a crate::scene::GenerationState,
    rendering_state: &'a crate::scene::RenderingState,
    lighting_state: &'a crate::scene::LightingState,
    material_state: &'a crate::scene::MaterialState,
    stage_state: &'a crate::scene::StageState,
) -> EffectTunerViewContext<'a> {
    EffectTunerViewContext {
        camera_config: &app_config.camera,
        camera_rig,
        generation_config: &app_config.generation,
        generation_state,
        rendering_config: &app_config.rendering,
        rendering_state,
        lighting_config: &app_config.lighting,
        lighting_state,
        material_config: &app_config.materials,
        material_state,
        stage_state,
    }
}

fn effect_tuner_edit_context<'a>(
    app_config: &'a crate::config::AppConfig,
    camera_rig: &'a mut crate::camera::CameraRig,
    generation_state: &'a mut crate::scene::GenerationState,
    rendering_state: &'a mut crate::scene::RenderingState,
    lighting_state: &'a mut crate::scene::LightingState,
    material_state: &'a mut crate::scene::MaterialState,
    stage_state: &'a mut crate::scene::StageState,
) -> EffectTunerEditContext<'a> {
    EffectTunerEditContext {
        camera_config: &app_config.camera,
        camera_rig,
        generation_config: &app_config.generation,
        generation_state,
        rendering_config: &app_config.rendering,
        rendering_state,
        lighting_config: &app_config.lighting,
        lighting_state,
        material_config: &app_config.materials,
        material_state,
        stage_state,
    }
}

fn apply_selected_parameter_side_effects(
    parameter: EffectTunerParameter,
    scene: &mut GenerationSceneAccess<'_, '_>,
) {
    let EffectTunerParameter::Scene(parameter) = parameter else {
        return;
    };

    match parameter.change_target() {
        SceneChangeTarget::Generation => {
            recompute_generation_tree(scene);
        }
        SceneChangeTarget::Materials => {
            apply_live_material_state(
                &scene.generation_state,
                &scene.app_config.materials,
                &scene.material_state,
                &mut scene.materials,
                &scene.shape_materials,
            );
        }
        SceneChangeTarget::Stage => {
            let runtime_rendering = scene
                .rendering_state
                .runtime_rendering_config(&scene.app_config.rendering, &scene.stage_state);
            sync_stage_entities(
                &mut scene.commands,
                &mut scene.meshes,
                &mut scene.materials,
                &runtime_rendering,
                &scene.stage_entities,
            );
        }
        SceneChangeTarget::Rendering => {
            let runtime_rendering = scene
                .rendering_state
                .runtime_rendering_config(&scene.app_config.rendering, &scene.stage_state);
            apply_live_rendering_state(
                &runtime_rendering,
                &mut scene.clear_color,
                &mut scene.ambient_light,
            );
        }
        SceneChangeTarget::Camera => {
            sync_scene_camera_transform(&scene.camera_rig, &mut scene.camera_transforms);
        }
        SceneChangeTarget::Lighting => {
            let runtime_lighting = scene
                .lighting_state
                .runtime_lighting_config(&scene.app_config.lighting);
            apply_live_lighting_state(
                &runtime_lighting,
                &mut scene.directional_lights,
                &mut scene.point_lights,
                &mut scene.accent_lights,
            );
        }
        SceneChangeTarget::None => {}
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
    let runtime_rendering = scene
        .rendering_state
        .runtime_rendering_config(&scene.app_config.rendering, &scene.stage_state);
    apply_live_rendering_state(
        &runtime_rendering,
        &mut scene.clear_color,
        &mut scene.ambient_light,
    );
    sync_stage_entities(
        &mut scene.commands,
        &mut scene.meshes,
        &mut scene.materials,
        &runtime_rendering,
        &scene.stage_entities,
    );
    let runtime_lighting = scene
        .lighting_state
        .runtime_lighting_config(&scene.app_config.lighting);
    apply_live_lighting_state(
        &runtime_lighting,
        &mut scene.directional_lights,
        &mut scene.point_lights,
        &mut scene.accent_lights,
    );
    sync_scene_camera_transform(&scene.camera_rig, &mut scene.camera_transforms);
}

fn apply_scene_lfo_side_effects(
    side_effects: SceneLfoApplicationResult,
    scene: &mut GenerationSceneAccess<'_, '_>,
) {
    if side_effects.generation_changed {
        recompute_generation_tree(scene);
    }
    if side_effects.materials_changed {
        apply_live_material_state(
            &scene.generation_state,
            &scene.app_config.materials,
            &scene.material_state,
            &mut scene.materials,
            &scene.shape_materials,
        );
    }
    let runtime_rendering =
        (side_effects.rendering_changed || side_effects.stage_changed).then(|| {
            scene
                .rendering_state
                .runtime_rendering_config(&scene.app_config.rendering, &scene.stage_state)
        });
    if let Some(runtime_rendering) = runtime_rendering.as_ref() {
        if side_effects.rendering_changed {
            apply_live_rendering_state(
                runtime_rendering,
                &mut scene.clear_color,
                &mut scene.ambient_light,
            );
        }
        if side_effects.stage_changed {
            sync_stage_entities(
                &mut scene.commands,
                &mut scene.meshes,
                &mut scene.materials,
                runtime_rendering,
                &scene.stage_entities,
            );
        }
    }
    if side_effects.lighting_changed {
        let runtime_lighting = scene
            .lighting_state
            .runtime_lighting_config(&scene.app_config.lighting);
        apply_live_lighting_state(
            &runtime_lighting,
            &mut scene.directional_lights,
            &mut scene.point_lights,
            &mut scene.accent_lights,
        );
    }
    if side_effects.camera_changed {
        sync_scene_camera_transform(&scene.camera_rig, &mut scene.camera_transforms);
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::mouse::MouseScrollUnit;

    use super::{
        mouse_wheel_selection_lines, mouse_wheel_selection_whole_steps,
        selected_field_edits_parameter_value,
    };
    use crate::config::EffectsConfig;
    use crate::effect_tuner::EffectTunerState;

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

    #[test]
    fn only_value_field_edits_trigger_direct_scene_restore_path() {
        let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());

        assert!(selected_field_edits_parameter_value(&effect_tuner));

        assert!(effect_tuner.step_edit_mode(1, 0.0));
        assert!(!selected_field_edits_parameter_value(&effect_tuner));
    }
}
