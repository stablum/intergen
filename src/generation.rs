use bevy::prelude::*;

use crate::control_page::{
    ControlPageInputMask, just_pressed_unmasked, just_released_unmasked, pressed_unmasked,
};
use crate::effect_tuner::{
    EffectTunerEditContext, EffectTunerParameter, EffectTunerSceneParameter, EffectTunerState,
    EffectTunerViewContext,
};
use crate::parameters::GenerationParameter;
use crate::runtime_scene::GenerationSceneAccess;
use crate::scene::{
    ShapeEntity, alpha_mode_for_opacity, material_appearance, opacity_status_message,
    reset_generation_state, spawn_shape_entity, sync_shape_transforms,
};
use crate::shapes::{
    ShapeKind, SpawnAddMode, SpawnPlacementMode, SpawnedShape, recompute_spawn_tree, spawn_batch,
    spawn_batch_with_inputs,
};

const RADIANS_TO_DEGREES: f32 = 180.0 / std::f32::consts::PI;
const TWIST_DECREASE_KEYS: [KeyCode; 1] = [KeyCode::BracketLeft];
const TWIST_INCREASE_KEYS: [KeyCode; 1] = [KeyCode::BracketRight];
const VERTEX_OFFSET_DECREASE_KEYS: [KeyCode; 1] = [KeyCode::KeyZ];
const VERTEX_OFFSET_INCREASE_KEYS: [KeyCode; 1] = [KeyCode::KeyX];
const VERTEX_EXCLUSION_DECREASE_KEYS: [KeyCode; 1] = [KeyCode::KeyV];
const VERTEX_EXCLUSION_INCREASE_KEYS: [KeyCode; 1] = [KeyCode::KeyB];

const SHAPE_SELECTION_KEYS: [(KeyCode, ShapeKind); 4] = [
    (KeyCode::Digit1, ShapeKind::Cube),
    (KeyCode::Digit2, ShapeKind::Tetrahedron),
    (KeyCode::Digit3, ShapeKind::Octahedron),
    (KeyCode::Digit4, ShapeKind::Dodecahedron),
];

pub(crate) fn generation_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    control_page_input_mask: Res<ControlPageInputMask>,
    mut effect_tuner: ResMut<EffectTunerState>,
    mut scene: GenerationSceneAccess,
) {
    let input_mask = *control_page_input_mask;

    handle_shape_selection(&keys, input_mask, &mut scene.generation_state);
    let ctrl_pressed = handle_mode_shortcuts(&keys, input_mask, &mut scene.generation_state);
    handle_scale_input(
        &keys,
        input_mask,
        &scene.app_config.generation,
        &mut scene.generation_state,
    );
    handle_opacity_input(&keys, input_mask, &mut effect_tuner, &mut scene);

    let mut transform_changed = false;
    transform_changed |= handle_generation_parameter_input(
        &keys,
        time.delta_secs(),
        input_mask,
        &scene.app_config.generation,
        &mut scene.generation_state,
        GenerationParameter::ChildTwistPerVertexRadians,
        &TWIST_DECREASE_KEYS,
        &TWIST_INCREASE_KEYS,
        KeyCode::KeyT,
        twist_status_message,
        true,
    );
    transform_changed |= handle_generation_parameter_input(
        &keys,
        time.delta_secs(),
        input_mask,
        &scene.app_config.generation,
        &mut scene.generation_state,
        GenerationParameter::ChildOutwardOffsetRatio,
        &VERTEX_OFFSET_DECREASE_KEYS,
        &VERTEX_OFFSET_INCREASE_KEYS,
        KeyCode::KeyC,
        vertex_offset_status_message,
        true,
    );
    let _ = handle_generation_parameter_input(
        &keys,
        time.delta_secs(),
        input_mask,
        &scene.app_config.generation,
        &mut scene.generation_state,
        GenerationParameter::ChildSpawnExclusionProbability,
        &VERTEX_EXCLUSION_DECREASE_KEYS,
        &VERTEX_EXCLUSION_INCREASE_KEYS,
        KeyCode::KeyN,
        vertex_exclusion_status_message,
        false,
    );

    if transform_changed {
        recompute_generation_tree(&mut scene);
    }

    if handle_scene_reset(&keys, input_mask, &mut scene) {
        return;
    }

    handle_spawn_input(
        &keys,
        time.elapsed_secs(),
        time.delta_secs(),
        input_mask,
        ctrl_pressed,
        &effect_tuner,
        &mut scene,
    );
}

fn handle_shape_selection(
    keys: &ButtonInput<KeyCode>,
    input_mask: ControlPageInputMask,
    generation_state: &mut crate::scene::GenerationState,
) {
    for (key_code, kind) in SHAPE_SELECTION_KEYS {
        if !just_pressed_unmasked(keys, input_mask, key_code) {
            continue;
        }

        generation_state.selected_shape_kind = kind;
        println!(
            "{}",
            selected_child_shape_status_message(generation_state.selected_shape_kind)
        );
    }
}

fn handle_mode_shortcuts(
    keys: &ButtonInput<KeyCode>,
    input_mask: ControlPageInputMask,
    generation_state: &mut crate::scene::GenerationState,
) -> bool {
    if just_pressed_unmasked(keys, input_mask, KeyCode::KeyG) {
        generation_state.spawn_placement_mode = generation_state.spawn_placement_mode.next();
        println!(
            "{}",
            spawn_placement_mode_status_message(generation_state.spawn_placement_mode)
        );
    }

    let ctrl_pressed = control_pressed(keys);
    if ctrl_pressed && just_pressed_unmasked(keys, input_mask, KeyCode::Space) {
        generation_state.spawn_add_mode = generation_state.spawn_add_mode.next();
        println!(
            "{}",
            spawn_add_mode_status_message(generation_state.spawn_add_mode)
        );
    }

    ctrl_pressed
}

fn control_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight)
}

fn handle_scale_input(
    keys: &ButtonInput<KeyCode>,
    input_mask: ControlPageInputMask,
    generation_config: &crate::config::GenerationConfig,
    generation_state: &mut crate::scene::GenerationState,
) {
    let scale_spec = generation_config.parameter_spec(GenerationParameter::ChildScaleRatio);
    if just_pressed_unmasked(keys, input_mask, KeyCode::Minus)
        || just_pressed_unmasked(keys, input_mask, KeyCode::NumpadSubtract)
    {
        let scale_ratio = generation_state
            .parameter_mut(GenerationParameter::ChildScaleRatio)
            .adjust_clamped_base_value(-scale_spec.step(), scale_spec);
        println!("Child scale ratio: {:.2}", scale_ratio);
    }
    if just_pressed_unmasked(keys, input_mask, KeyCode::Equal)
        || just_pressed_unmasked(keys, input_mask, KeyCode::NumpadAdd)
    {
        let scale_ratio = generation_state
            .parameter_mut(GenerationParameter::ChildScaleRatio)
            .adjust_clamped_base_value(scale_spec.step(), scale_spec);
        println!("Child scale ratio: {:.2}", scale_ratio);
    }
}

fn handle_opacity_input(
    keys: &ButtonInput<KeyCode>,
    input_mask: ControlPageInputMask,
    effect_tuner: &mut EffectTunerState,
    scene: &mut GenerationSceneAccess<'_, '_>,
) {
    let (min_opacity, max_opacity) = scene.app_config.materials.opacity_bounds();
    let mut opacity_changed = false;
    let opacity_parameter = EffectTunerParameter::Scene(EffectTunerSceneParameter::GlobalOpacity);
    if just_pressed_unmasked(keys, input_mask, KeyCode::KeyO) {
        {
            let mut context = effect_tuner_edit_context(scene);
            effect_tuner.restore_scene_parameter_base_if_needed(opacity_parameter, &mut context);
        }
        scene.material_state.opacity = adjust_clamped_value(
            scene.material_state.opacity,
            -scene.app_config.materials.opacity_adjust_step,
            min_opacity,
            max_opacity,
        );
        opacity_changed = true;
        println!("{}", opacity_status_message(scene.material_state.opacity));
    }
    if just_pressed_unmasked(keys, input_mask, KeyCode::KeyP) {
        {
            let mut context = effect_tuner_edit_context(scene);
            effect_tuner.restore_scene_parameter_base_if_needed(opacity_parameter, &mut context);
        }
        scene.material_state.opacity = adjust_clamped_value(
            scene.material_state.opacity,
            scene.app_config.materials.opacity_adjust_step,
            min_opacity,
            max_opacity,
        );
        opacity_changed = true;
        println!("{}", opacity_status_message(scene.material_state.opacity));
    }
    if just_pressed_unmasked(keys, input_mask, KeyCode::KeyI) {
        {
            let mut context = effect_tuner_edit_context(scene);
            effect_tuner.restore_scene_parameter_base_if_needed(opacity_parameter, &mut context);
        }
        scene.material_state.opacity = scene.app_config.materials.default_opacity_clamped();
        opacity_changed = true;
        println!(
            "Reset {}",
            opacity_status_message(scene.material_state.opacity).to_lowercase()
        );
    }
    if opacity_changed {
        let view = effect_tuner_view_context(scene);
        effect_tuner.sync_scene_parameter_base_if_needed(opacity_parameter, &view);
        apply_live_material_state(
            &scene.generation_state,
            &scene.app_config.materials,
            &scene.material_state,
            &mut scene.materials,
            &scene.shape_materials,
        );
    }
}

fn effect_tuner_view_context<'a>(
    scene: &'a GenerationSceneAccess<'_, '_>,
) -> EffectTunerViewContext<'a> {
    EffectTunerViewContext {
        camera_config: &scene.app_config.camera,
        camera_rig: &scene.camera_rig,
        generation_config: &scene.app_config.generation,
        generation_state: &scene.generation_state,
        rendering_config: &scene.app_config.rendering,
        rendering_state: &scene.rendering_state,
        lighting_config: &scene.app_config.lighting,
        lighting_state: &scene.lighting_state,
        material_config: &scene.app_config.materials,
        material_state: &scene.material_state,
        stage_state: &scene.stage_state,
    }
}

fn effect_tuner_edit_context<'a>(
    scene: &'a mut GenerationSceneAccess<'_, '_>,
) -> EffectTunerEditContext<'a> {
    EffectTunerEditContext {
        camera_config: &scene.app_config.camera,
        camera_rig: &mut scene.camera_rig,
        generation_config: &scene.app_config.generation,
        generation_state: &mut scene.generation_state,
        rendering_config: &scene.app_config.rendering,
        rendering_state: &mut scene.rendering_state,
        lighting_config: &scene.app_config.lighting,
        lighting_state: &mut scene.lighting_state,
        material_config: &scene.app_config.materials,
        material_state: &mut scene.material_state,
        stage_state: &mut scene.stage_state,
    }
}

fn handle_generation_parameter_input(
    keys: &ButtonInput<KeyCode>,
    delta_secs: f32,
    input_mask: ControlPageInputMask,
    generation_config: &crate::config::GenerationConfig,
    generation_state: &mut crate::scene::GenerationState,
    parameter: GenerationParameter,
    decrease_keys: &[KeyCode],
    increase_keys: &[KeyCode],
    reset_key: KeyCode,
    status_message: fn(f32) -> String,
    recompute_after_change: bool,
) -> bool {
    let spec = generation_config.parameter_spec(parameter);
    let mut transform_changed = false;

    let decrease_requested = generation_state
        .parameter_mut(parameter)
        .input_mut()
        .request_decrease(
            key_group_just_pressed(keys, input_mask, decrease_keys),
            key_group_pressed(keys, input_mask, decrease_keys),
            key_group_just_released(keys, input_mask, decrease_keys),
            delta_secs,
            spec,
        );
    if decrease_requested {
        let value = generation_state
            .parameter_mut(parameter)
            .adjust_clamped_base_value(-spec.step(), spec);
        transform_changed = recompute_after_change;
        println!("{}", status_message(value));
    }

    let increase_requested = generation_state
        .parameter_mut(parameter)
        .input_mut()
        .request_increase(
            key_group_just_pressed(keys, input_mask, increase_keys),
            key_group_pressed(keys, input_mask, increase_keys),
            key_group_just_released(keys, input_mask, increase_keys),
            delta_secs,
            spec,
        );
    if increase_requested {
        let value = generation_state
            .parameter_mut(parameter)
            .adjust_clamped_base_value(spec.step(), spec);
        transform_changed = recompute_after_change;
        println!("{}", status_message(value));
    }

    if just_pressed_unmasked(keys, input_mask, reset_key) {
        let value = generation_state
            .parameter_mut(parameter)
            .reset_to_default(spec);
        generation_state
            .parameter_mut(parameter)
            .input_mut()
            .reset();
        transform_changed = recompute_after_change;
        println!("Reset {}", status_message(value).to_lowercase());
    }

    transform_changed
}

pub(crate) fn recompute_generation_tree(scene: &mut GenerationSceneAccess<'_, '_>) {
    let twist_per_vertex_radians = scene
        .generation_state
        .twist_per_vertex_radians(&scene.app_config.generation);
    let vertex_offset_ratio = scene
        .generation_state
        .vertex_offset_ratio(&scene.app_config.generation);
    recompute_spawn_tree(
        &mut scene.generation_state.nodes,
        &scene.shape_assets.catalog,
        twist_per_vertex_radians,
        vertex_offset_ratio,
    );
    sync_shape_transforms(&scene.generation_state.nodes, &mut scene.shape_transforms);
}

fn handle_scene_reset(
    keys: &ButtonInput<KeyCode>,
    input_mask: ControlPageInputMask,
    scene: &mut GenerationSceneAccess<'_, '_>,
) -> bool {
    if !just_pressed_unmasked(keys, input_mask, KeyCode::KeyR) {
        return false;
    }

    for entity in scene.shape_entities.iter() {
        scene.commands.entity(entity).despawn();
    }

    let root = reset_generation_state(
        &mut scene.generation_state,
        &scene.shape_assets.catalog,
        &scene.app_config.generation,
    );
    let material_config = scene
        .material_state
        .runtime_material_config(&scene.app_config.materials);
    spawn_shape_entity(
        &mut scene.commands,
        &mut scene.materials,
        scene.shape_assets.mesh(root.kind),
        &root,
        &material_config,
        scene.material_state.opacity,
        0,
    );
    println!("Reset scene to a {:?} root shape.", root.kind);
    true
}

fn handle_spawn_input(
    keys: &ButtonInput<KeyCode>,
    now_secs: f32,
    delta_secs: f32,
    input_mask: ControlPageInputMask,
    ctrl_pressed: bool,
    effect_tuner: &EffectTunerState,
    scene: &mut GenerationSceneAccess<'_, '_>,
) {
    let spawn_requested = scene.generation_state.spawn_hold.update(
        !ctrl_pressed && just_pressed_unmasked(keys, input_mask, KeyCode::Space),
        !ctrl_pressed && pressed_unmasked(keys, input_mask, KeyCode::Space),
        !ctrl_pressed && just_released_unmasked(keys, input_mask, KeyCode::Space),
        delta_secs,
        scene.app_config.generation.spawn_hold_delay_secs,
        scene.app_config.generation.spawn_repeat_interval_secs,
    );
    if !spawn_requested {
        return;
    }

    let selected_shape_kind = scene.generation_state.selected_shape_kind;
    let add_mode = scene.generation_state.spawn_add_mode;
    let generation_config = &scene.app_config.generation;
    let spawned = if add_mode == SpawnAddMode::FillLevel {
        let virtual_time_step_secs =
            generation_config.fill_mode_lfo_virtual_time_step_secs_clamped();
        let scale_ratio_base = scene.generation_state.scale_ratio_base();
        let child_axis_scale_base = scene.generation_state.child_axis_scale_base();
        let child_position_offset_base = scene.generation_state.child_position_offset_base();
        let spawn_exclusion_probability_base = scene
            .generation_state
            .vertex_spawn_exclusion_probability_base();
        let twist_per_vertex_radians = scene
            .generation_state
            .twist_per_vertex_radians(generation_config);
        let vertex_offset_ratio = scene
            .generation_state
            .vertex_offset_ratio(generation_config);
        let spawn_placement_mode = scene.generation_state.spawn_placement_mode;

        spawn_batch_with_inputs(
            &mut scene.generation_state.nodes,
            &scene.shape_assets.catalog,
            selected_shape_kind,
            add_mode,
            |spawn_index| {
                let sample_secs = now_secs + virtual_time_step_secs * spawn_index as f32;
                let scale_ratio = effect_tuner.sampled_generation_parameter_value(
                    GenerationParameter::ChildScaleRatio,
                    scale_ratio_base,
                    sample_secs,
                );
                let child_axis_scale = Vec3::new(
                    effect_tuner.sampled_generation_parameter_value(
                        GenerationParameter::ChildAxisScaleX,
                        child_axis_scale_base.x,
                        sample_secs,
                    ),
                    effect_tuner.sampled_generation_parameter_value(
                        GenerationParameter::ChildAxisScaleY,
                        child_axis_scale_base.y,
                        sample_secs,
                    ),
                    effect_tuner.sampled_generation_parameter_value(
                        GenerationParameter::ChildAxisScaleZ,
                        child_axis_scale_base.z,
                        sample_secs,
                    ),
                );
                let child_position_offset = Vec3::new(
                    effect_tuner.sampled_generation_parameter_value(
                        GenerationParameter::ChildPositionOffsetX,
                        child_position_offset_base.x,
                        sample_secs,
                    ),
                    effect_tuner.sampled_generation_parameter_value(
                        GenerationParameter::ChildPositionOffsetY,
                        child_position_offset_base.y,
                        sample_secs,
                    ),
                    effect_tuner.sampled_generation_parameter_value(
                        GenerationParameter::ChildPositionOffsetZ,
                        child_position_offset_base.z,
                        sample_secs,
                    ),
                );
                let spawn_exclusion_probability = effect_tuner.sampled_generation_parameter_value(
                    GenerationParameter::ChildSpawnExclusionProbability,
                    spawn_exclusion_probability_base,
                    sample_secs,
                );
                let spawn_tuning = generation_config.spawn_tuning(
                    child_axis_scale,
                    twist_per_vertex_radians,
                    vertex_offset_ratio,
                    child_position_offset,
                    spawn_exclusion_probability,
                    spawn_placement_mode,
                );

                (scale_ratio, spawn_tuning)
            },
        )
    } else {
        let scale_ratio = scene.generation_state.scale_ratio(generation_config);
        let spawn_tuning = scene.generation_state.spawn_tuning(generation_config);
        spawn_batch(
            &mut scene.generation_state.nodes,
            &scene.shape_assets.catalog,
            selected_shape_kind,
            scale_ratio,
            spawn_tuning,
            add_mode,
        )
    };
    if spawned.is_empty() {
        eprintln!("No valid spawn position is currently available.");
        return;
    }

    let first_new_index = scene.generation_state.nodes.len() - spawned.len();
    let material_config = scene
        .material_state
        .runtime_material_config(&scene.app_config.materials);
    for (offset, spawn) in spawned.iter().enumerate() {
        spawn_shape_entity(
            &mut scene.commands,
            &mut scene.materials,
            scene.shape_assets.mesh(spawn.kind),
            &spawn.node,
            &material_config,
            scene.material_state.opacity,
            first_new_index + offset,
        );
    }
    println!("{}", spawn_summary_status_message(&spawned, add_mode));
}

fn key_group_just_pressed(
    keys: &ButtonInput<KeyCode>,
    input_mask: ControlPageInputMask,
    key_codes: &[KeyCode],
) -> bool {
    key_codes
        .iter()
        .copied()
        .any(|key_code| just_pressed_unmasked(keys, input_mask, key_code))
}

fn key_group_pressed(
    keys: &ButtonInput<KeyCode>,
    input_mask: ControlPageInputMask,
    key_codes: &[KeyCode],
) -> bool {
    key_codes
        .iter()
        .copied()
        .any(|key_code| pressed_unmasked(keys, input_mask, key_code))
}

fn key_group_just_released(
    keys: &ButtonInput<KeyCode>,
    input_mask: ControlPageInputMask,
    key_codes: &[KeyCode],
) -> bool {
    !key_group_pressed(keys, input_mask, key_codes)
        && key_codes
            .iter()
            .copied()
            .any(|key_code| just_released_unmasked(keys, input_mask, key_code))
}

fn adjust_clamped_value(current: f32, delta: f32, min: f32, max: f32) -> f32 {
    (current + delta).clamp(min, max)
}

pub(crate) fn apply_live_material_state(
    generation_state: &crate::scene::GenerationState,
    defaults: &crate::config::MaterialConfig,
    material_state: &crate::scene::MaterialState,
    materials: &mut Assets<StandardMaterial>,
    shape_materials: &Query<(&ShapeEntity, &MeshMaterial3d<StandardMaterial>)>,
) {
    let material_config = material_state.runtime_material_config(defaults);

    for (shape_entity, material_handle) in shape_materials {
        let Some(node) = generation_state.nodes.get(shape_entity.node_index) else {
            continue;
        };
        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };

        let appearance = material_appearance(node, &material_config, material_state.opacity);
        material.base_color = Color::srgba(
            appearance.base_color[0],
            appearance.base_color[1],
            appearance.base_color[2],
            appearance.base_color[3],
        );
        material.alpha_mode = alpha_mode_for_opacity(appearance.base_color[3]);
        material.metallic = appearance.metallic;
        material.perceptual_roughness = appearance.perceptual_roughness;
        material.reflectance = appearance.reflectance;
    }
}

pub(crate) fn twist_status_message(radians: f32) -> String {
    format!(
        "Child twist angle: {:.3} rad ({:.1} deg)",
        radians,
        radians * RADIANS_TO_DEGREES
    )
}

pub(crate) fn vertex_offset_status_message(offset_ratio: f32) -> String {
    format!(
        "Child outward offset: {:.2}x child radius",
        offset_ratio.max(0.0)
    )
}

pub(crate) fn vertex_exclusion_status_message(probability: f32) -> String {
    format!(
        "Global spawn exclusion probability: {:.0}%",
        probability.clamp(0.0, 1.0) * 100.0
    )
}

pub(crate) fn spawn_add_mode_status_message(mode: SpawnAddMode) -> String {
    format!("Object add mode: {}", mode.label())
}

pub(crate) fn spawn_placement_mode_status_message(mode: SpawnPlacementMode) -> String {
    format!("Spawn placement mode: {}", mode.plural_label())
}

pub(crate) fn selected_child_shape_status_message(kind: ShapeKind) -> String {
    format!("Selected child shape: {:?}", kind)
}

fn spawn_summary_status_message(spawned: &[SpawnedShape], add_mode: SpawnAddMode) -> String {
    let first = &spawned[0];
    if spawned.len() == 1 {
        return format!(
            "Spawned {:?} at level {} from parent level {}",
            first.kind, first.node.level, first.parent_level
        );
    }

    format!(
        "Spawned {} {:?} objects at level {} from parent level {} ({})",
        spawned.len(),
        first.kind,
        first.node.level,
        first.parent_level,
        add_mode.label()
    )
}

#[cfg(test)]
mod tests {
    use super::{
        adjust_clamped_value, spawn_add_mode_status_message, twist_status_message,
        vertex_exclusion_status_message, vertex_offset_status_message,
    };
    use crate::config::GenerationConfig;
    use crate::parameters::HoldRepeatState;
    use crate::shapes::SpawnAddMode;

    #[test]
    fn spawn_hold_repeats_while_space_is_held() {
        let generation_config = GenerationConfig::default();
        let mut spawn_hold = HoldRepeatState::default();

        assert!(spawn_hold.update(
            true,
            true,
            false,
            0.0,
            generation_config.spawn_hold_delay_secs,
            generation_config.spawn_repeat_interval_secs,
        ));
        assert!(!spawn_hold.update(
            false,
            true,
            false,
            generation_config.spawn_hold_delay_secs * 0.5,
            generation_config.spawn_hold_delay_secs,
            generation_config.spawn_repeat_interval_secs,
        ));
        assert!(spawn_hold.update(
            false,
            true,
            false,
            generation_config.spawn_hold_delay_secs * 0.5,
            generation_config.spawn_hold_delay_secs,
            generation_config.spawn_repeat_interval_secs,
        ));
        assert!(!spawn_hold.update(
            false,
            true,
            false,
            generation_config.spawn_repeat_interval_secs * 0.5,
            generation_config.spawn_hold_delay_secs,
            generation_config.spawn_repeat_interval_secs,
        ));
        assert!(spawn_hold.update(
            false,
            true,
            false,
            generation_config.spawn_repeat_interval_secs * 0.5,
            generation_config.spawn_hold_delay_secs,
            generation_config.spawn_repeat_interval_secs,
        ));
    }

    #[test]
    fn spawn_hold_resets_after_release() {
        let generation_config = GenerationConfig::default();
        let mut spawn_hold = HoldRepeatState::default();

        assert!(spawn_hold.update(
            true,
            true,
            false,
            0.0,
            generation_config.spawn_hold_delay_secs,
            generation_config.spawn_repeat_interval_secs,
        ));
        assert!(!spawn_hold.update(
            false,
            false,
            true,
            0.0,
            generation_config.spawn_hold_delay_secs,
            generation_config.spawn_repeat_interval_secs,
        ));
        assert!(spawn_hold.update(
            true,
            true,
            false,
            0.0,
            generation_config.spawn_hold_delay_secs,
            generation_config.spawn_repeat_interval_secs,
        ));
    }

    #[test]
    fn twist_adjustment_clamps_to_bounds() {
        assert_eq!(adjust_clamped_value(0.7, 0.2, -0.5, 0.75), 0.75);
        assert_eq!(adjust_clamped_value(-0.4, -0.3, -0.5, 0.75), -0.5);
    }

    #[test]
    fn twist_status_message_includes_radians_and_degrees() {
        let status = twist_status_message(std::f32::consts::FRAC_PI_2);

        assert!(status.contains("1.571 rad"));
        assert!(status.contains("90.0 deg"));
    }

    #[test]
    fn vertex_offset_status_message_mentions_child_radius_scale() {
        let status = vertex_offset_status_message(0.75);

        assert!(status.contains("0.75x child radius"));
    }

    #[test]
    fn vertex_exclusion_status_message_uses_percentage() {
        let status = vertex_exclusion_status_message(0.35);

        assert!(status.contains("35%"));
    }

    #[test]
    fn spawn_add_mode_status_message_mentions_fill_level() {
        let status = spawn_add_mode_status_message(SpawnAddMode::FillLevel);

        assert!(status.contains("fill current level"));
    }
}
