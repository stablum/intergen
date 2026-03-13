use bevy::prelude::*;

use crate::control_page::{
    ControlPageInputMask, just_pressed_unmasked, just_released_unmasked, pressed_unmasked,
};
use crate::parameters::GenerationParameter;
use crate::polyhedra::{
    PolyhedronKind, SpawnAddMode, SpawnPlacementMode, SpawnedNode, recompute_spawn_tree,
    spawn_batch,
};
use crate::runtime_scene::GenerationSceneAccess;
use crate::scene::{
    PolyhedronEntity, alpha_mode_for_opacity, opacity_status_message, reset_generation_state,
    spawn_polyhedron_entity, sync_polyhedron_transforms,
};

const RADIANS_TO_DEGREES: f32 = 180.0 / std::f32::consts::PI;
const TWIST_DECREASE_KEYS: [KeyCode; 2] = [KeyCode::BracketLeft, KeyCode::Comma];
const TWIST_INCREASE_KEYS: [KeyCode; 2] = [KeyCode::BracketRight, KeyCode::Period];
const VERTEX_OFFSET_DECREASE_KEYS: [KeyCode; 1] = [KeyCode::KeyZ];
const VERTEX_OFFSET_INCREASE_KEYS: [KeyCode; 1] = [KeyCode::KeyX];
const VERTEX_EXCLUSION_DECREASE_KEYS: [KeyCode; 1] = [KeyCode::KeyV];
const VERTEX_EXCLUSION_INCREASE_KEYS: [KeyCode; 1] = [KeyCode::KeyB];

pub(crate) fn generation_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    control_page_input_mask: Res<ControlPageInputMask>,
    mut scene: GenerationSceneAccess,
) {
    let input_mask = *control_page_input_mask;

    if just_pressed_unmasked(&keys, input_mask, KeyCode::Digit1) {
        scene.generation_state.selected_kind = PolyhedronKind::Cube;
        println!(
            "Selected child shape: {:?}",
            scene.generation_state.selected_kind
        );
    }
    if just_pressed_unmasked(&keys, input_mask, KeyCode::Digit2) {
        scene.generation_state.selected_kind = PolyhedronKind::Tetrahedron;
        println!(
            "Selected child shape: {:?}",
            scene.generation_state.selected_kind
        );
    }
    if just_pressed_unmasked(&keys, input_mask, KeyCode::Digit3) {
        scene.generation_state.selected_kind = PolyhedronKind::Octahedron;
        println!(
            "Selected child shape: {:?}",
            scene.generation_state.selected_kind
        );
    }
    if just_pressed_unmasked(&keys, input_mask, KeyCode::Digit4) {
        scene.generation_state.selected_kind = PolyhedronKind::Dodecahedron;
        println!(
            "Selected child shape: {:?}",
            scene.generation_state.selected_kind
        );
    }
    if just_pressed_unmasked(&keys, input_mask, KeyCode::KeyG) {
        scene.generation_state.spawn_placement_mode =
            scene.generation_state.spawn_placement_mode.next();
        println!(
            "{}",
            spawn_placement_mode_status_message(scene.generation_state.spawn_placement_mode)
        );
    }

    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ctrl_pressed && just_pressed_unmasked(&keys, input_mask, KeyCode::Space) {
        scene.generation_state.spawn_add_mode = scene.generation_state.spawn_add_mode.next();
        println!(
            "{}",
            spawn_add_mode_status_message(scene.generation_state.spawn_add_mode)
        );
    }

    let scale_spec = scene
        .app_config
        .generation
        .parameter_spec(GenerationParameter::ChildScaleRatio);
    if just_pressed_unmasked(&keys, input_mask, KeyCode::Minus)
        || just_pressed_unmasked(&keys, input_mask, KeyCode::NumpadSubtract)
    {
        let scale_ratio = scene
            .generation_state
            .parameter_mut(GenerationParameter::ChildScaleRatio)
            .adjust_clamped_base_value(-scale_spec.step(), scale_spec);
        println!("Child scale ratio: {:.2}", scale_ratio);
    }
    if just_pressed_unmasked(&keys, input_mask, KeyCode::Equal)
        || just_pressed_unmasked(&keys, input_mask, KeyCode::NumpadAdd)
    {
        let scale_ratio = scene
            .generation_state
            .parameter_mut(GenerationParameter::ChildScaleRatio)
            .adjust_clamped_base_value(scale_spec.step(), scale_spec);
        println!("Child scale ratio: {:.2}", scale_ratio);
    }

    let (min_opacity, max_opacity) = scene.app_config.materials.opacity_bounds();
    let mut opacity_changed = false;
    if just_pressed_unmasked(&keys, input_mask, KeyCode::KeyO) {
        scene.material_state.opacity = adjust_clamped_value(
            scene.material_state.opacity,
            -scene.app_config.materials.opacity_adjust_step,
            min_opacity,
            max_opacity,
        );
        opacity_changed = true;
        println!("{}", opacity_status_message(scene.material_state.opacity));
    }
    if just_pressed_unmasked(&keys, input_mask, KeyCode::KeyP) {
        scene.material_state.opacity = adjust_clamped_value(
            scene.material_state.opacity,
            scene.app_config.materials.opacity_adjust_step,
            min_opacity,
            max_opacity,
        );
        opacity_changed = true;
        println!("{}", opacity_status_message(scene.material_state.opacity));
    }
    if just_pressed_unmasked(&keys, input_mask, KeyCode::KeyI) {
        scene.material_state.opacity = scene.app_config.materials.default_opacity_clamped();
        opacity_changed = true;
        println!(
            "Reset {}",
            opacity_status_message(scene.material_state.opacity).to_lowercase()
        );
    }
    if opacity_changed {
        apply_global_opacity(
            scene.material_state.opacity,
            &mut scene.materials,
            &scene.polyhedron_materials,
        );
    }

    let twist_spec = scene
        .app_config
        .generation
        .parameter_spec(GenerationParameter::ChildTwistPerVertexRadians);
    let offset_spec = scene
        .app_config
        .generation
        .parameter_spec(GenerationParameter::ChildOutwardOffsetRatio);
    let exclusion_spec = scene
        .app_config
        .generation
        .parameter_spec(GenerationParameter::ChildSpawnExclusionProbability);
    let mut transform_changed = false;

    let twist_decrease_requested = scene
        .generation_state
        .parameter_mut(GenerationParameter::ChildTwistPerVertexRadians)
        .input_mut()
        .request_decrease(
            key_group_just_pressed(&keys, input_mask, &TWIST_DECREASE_KEYS),
            key_group_pressed(&keys, input_mask, &TWIST_DECREASE_KEYS),
            key_group_just_released(&keys, input_mask, &TWIST_DECREASE_KEYS),
            time.delta_secs(),
            twist_spec,
        );
    if twist_decrease_requested {
        let twist = scene
            .generation_state
            .parameter_mut(GenerationParameter::ChildTwistPerVertexRadians)
            .adjust_clamped_base_value(-twist_spec.step(), twist_spec);
        transform_changed = true;
        println!("{}", twist_status_message(twist));
    }

    let twist_increase_requested = scene
        .generation_state
        .parameter_mut(GenerationParameter::ChildTwistPerVertexRadians)
        .input_mut()
        .request_increase(
            key_group_just_pressed(&keys, input_mask, &TWIST_INCREASE_KEYS),
            key_group_pressed(&keys, input_mask, &TWIST_INCREASE_KEYS),
            key_group_just_released(&keys, input_mask, &TWIST_INCREASE_KEYS),
            time.delta_secs(),
            twist_spec,
        );
    if twist_increase_requested {
        let twist = scene
            .generation_state
            .parameter_mut(GenerationParameter::ChildTwistPerVertexRadians)
            .adjust_clamped_base_value(twist_spec.step(), twist_spec);
        transform_changed = true;
        println!("{}", twist_status_message(twist));
    }

    if just_pressed_unmasked(&keys, input_mask, KeyCode::KeyT) {
        let twist = scene
            .generation_state
            .parameter_mut(GenerationParameter::ChildTwistPerVertexRadians)
            .reset_to_default(twist_spec);
        transform_changed = true;
        scene
            .generation_state
            .parameter_mut(GenerationParameter::ChildTwistPerVertexRadians)
            .input_mut()
            .reset();
        println!("Reset {}", twist_status_message(twist).to_lowercase());
    }

    let vertex_offset_decrease_requested = scene
        .generation_state
        .parameter_mut(GenerationParameter::ChildOutwardOffsetRatio)
        .input_mut()
        .request_decrease(
            key_group_just_pressed(&keys, input_mask, &VERTEX_OFFSET_DECREASE_KEYS),
            key_group_pressed(&keys, input_mask, &VERTEX_OFFSET_DECREASE_KEYS),
            key_group_just_released(&keys, input_mask, &VERTEX_OFFSET_DECREASE_KEYS),
            time.delta_secs(),
            offset_spec,
        );
    if vertex_offset_decrease_requested {
        let vertex_offset = scene
            .generation_state
            .parameter_mut(GenerationParameter::ChildOutwardOffsetRatio)
            .adjust_clamped_base_value(-offset_spec.step(), offset_spec);
        transform_changed = true;
        println!("{}", vertex_offset_status_message(vertex_offset));
    }

    let vertex_offset_increase_requested = scene
        .generation_state
        .parameter_mut(GenerationParameter::ChildOutwardOffsetRatio)
        .input_mut()
        .request_increase(
            key_group_just_pressed(&keys, input_mask, &VERTEX_OFFSET_INCREASE_KEYS),
            key_group_pressed(&keys, input_mask, &VERTEX_OFFSET_INCREASE_KEYS),
            key_group_just_released(&keys, input_mask, &VERTEX_OFFSET_INCREASE_KEYS),
            time.delta_secs(),
            offset_spec,
        );
    if vertex_offset_increase_requested {
        let vertex_offset = scene
            .generation_state
            .parameter_mut(GenerationParameter::ChildOutwardOffsetRatio)
            .adjust_clamped_base_value(offset_spec.step(), offset_spec);
        transform_changed = true;
        println!("{}", vertex_offset_status_message(vertex_offset));
    }

    if just_pressed_unmasked(&keys, input_mask, KeyCode::KeyC) {
        let vertex_offset = scene
            .generation_state
            .parameter_mut(GenerationParameter::ChildOutwardOffsetRatio)
            .reset_to_default(offset_spec);
        transform_changed = true;
        scene
            .generation_state
            .parameter_mut(GenerationParameter::ChildOutwardOffsetRatio)
            .input_mut()
            .reset();
        println!(
            "Reset {}",
            vertex_offset_status_message(vertex_offset).to_lowercase()
        );
    }

    let vertex_exclusion_decrease_requested = scene
        .generation_state
        .parameter_mut(GenerationParameter::ChildSpawnExclusionProbability)
        .input_mut()
        .request_decrease(
            key_group_just_pressed(&keys, input_mask, &VERTEX_EXCLUSION_DECREASE_KEYS),
            key_group_pressed(&keys, input_mask, &VERTEX_EXCLUSION_DECREASE_KEYS),
            key_group_just_released(&keys, input_mask, &VERTEX_EXCLUSION_DECREASE_KEYS),
            time.delta_secs(),
            exclusion_spec,
        );
    if vertex_exclusion_decrease_requested {
        let exclusion = scene
            .generation_state
            .parameter_mut(GenerationParameter::ChildSpawnExclusionProbability)
            .adjust_clamped_base_value(-exclusion_spec.step(), exclusion_spec);
        println!("{}", vertex_exclusion_status_message(exclusion));
    }

    let vertex_exclusion_increase_requested = scene
        .generation_state
        .parameter_mut(GenerationParameter::ChildSpawnExclusionProbability)
        .input_mut()
        .request_increase(
            key_group_just_pressed(&keys, input_mask, &VERTEX_EXCLUSION_INCREASE_KEYS),
            key_group_pressed(&keys, input_mask, &VERTEX_EXCLUSION_INCREASE_KEYS),
            key_group_just_released(&keys, input_mask, &VERTEX_EXCLUSION_INCREASE_KEYS),
            time.delta_secs(),
            exclusion_spec,
        );
    if vertex_exclusion_increase_requested {
        let exclusion = scene
            .generation_state
            .parameter_mut(GenerationParameter::ChildSpawnExclusionProbability)
            .adjust_clamped_base_value(exclusion_spec.step(), exclusion_spec);
        println!("{}", vertex_exclusion_status_message(exclusion));
    }

    if just_pressed_unmasked(&keys, input_mask, KeyCode::KeyN) {
        let exclusion = scene
            .generation_state
            .parameter_mut(GenerationParameter::ChildSpawnExclusionProbability)
            .reset_to_default(exclusion_spec);
        scene
            .generation_state
            .parameter_mut(GenerationParameter::ChildSpawnExclusionProbability)
            .input_mut()
            .reset();
        println!(
            "Reset {}",
            vertex_exclusion_status_message(exclusion).to_lowercase()
        );
    }

    if transform_changed {
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
        sync_polyhedron_transforms(
            &scene.generation_state.nodes,
            &mut scene.polyhedron_transforms,
        );
    }

    if just_pressed_unmasked(&keys, input_mask, KeyCode::KeyR) {
        for entity in scene.polyhedron_entities.iter() {
            scene.commands.entity(entity).despawn();
        }

        let root = reset_generation_state(
            &mut scene.generation_state,
            &scene.shape_assets.catalog,
            &scene.app_config.generation,
        );
        spawn_polyhedron_entity(
            &mut scene.commands,
            &mut scene.materials,
            scene.shape_assets.mesh(root.kind),
            &root,
            &scene.app_config.materials,
            scene.material_state.opacity,
            0,
        );
        println!("Reset scene to a {:?} root polyhedron.", root.kind);
        return;
    }

    let spawn_requested = scene.generation_state.spawn_hold.update(
        !ctrl_pressed && just_pressed_unmasked(&keys, input_mask, KeyCode::Space),
        !ctrl_pressed && pressed_unmasked(&keys, input_mask, KeyCode::Space),
        !ctrl_pressed && just_released_unmasked(&keys, input_mask, KeyCode::Space),
        time.delta_secs(),
        scene.app_config.generation.spawn_hold_delay_secs,
        scene.app_config.generation.spawn_repeat_interval_secs,
    );
    if !spawn_requested {
        return;
    }

    let selected_kind = scene.generation_state.selected_kind;
    let scale_ratio = scene
        .generation_state
        .scale_ratio(&scene.app_config.generation);
    let spawn_tuning = scene
        .generation_state
        .spawn_tuning(&scene.app_config.generation);
    let add_mode = scene.generation_state.spawn_add_mode;
    let spawned = spawn_batch(
        &mut scene.generation_state.nodes,
        &scene.shape_assets.catalog,
        selected_kind,
        scale_ratio,
        spawn_tuning,
        add_mode,
    );
    if spawned.is_empty() {
        eprintln!("No valid spawn position is currently available.");
        return;
    }

    let first_new_index = scene.generation_state.nodes.len() - spawned.len();
    for (offset, spawn) in spawned.iter().enumerate() {
        spawn_polyhedron_entity(
            &mut scene.commands,
            &mut scene.materials,
            scene.shape_assets.mesh(spawn.kind),
            &spawn.node,
            &scene.app_config.materials,
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

fn apply_global_opacity(
    opacity: f32,
    materials: &mut Assets<StandardMaterial>,
    polyhedron_materials: &Query<&MeshMaterial3d<StandardMaterial>, With<PolyhedronEntity>>,
) {
    for material_handle in polyhedron_materials {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color.set_alpha(opacity);
            material.alpha_mode = alpha_mode_for_opacity(opacity);
        }
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

fn spawn_summary_status_message(spawned: &[SpawnedNode], add_mode: SpawnAddMode) -> String {
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
    use crate::polyhedra::SpawnAddMode;

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
