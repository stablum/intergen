use bevy::prelude::*;

use crate::polyhedra::{PolyhedronKind, next_spawn, recompute_spawn_tree};
use crate::presets::PresetBrowserState;
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

#[derive(Default)]
pub(crate) struct SpawnHoldState {
    pub(crate) elapsed_secs: f32,
    pub(crate) repeating: bool,
}

impl SpawnHoldState {
    pub(crate) fn update(
        &mut self,
        just_pressed: bool,
        pressed: bool,
        just_released: bool,
        delta_secs: f32,
        hold_delay_secs: f32,
        repeat_interval_secs: f32,
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
            repeat_interval_secs
        } else {
            hold_delay_secs
        };

        if self.elapsed_secs < threshold {
            return false;
        }

        self.elapsed_secs = 0.0;
        self.repeating = true;
        true
    }

    pub(crate) fn reset(&mut self) {
        self.elapsed_secs = 0.0;
        self.repeating = false;
    }
}

pub(crate) fn generation_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    preset_browser: Res<PresetBrowserState>,
    mut scene: GenerationSceneAccess,
) {
    if preset_browser.blocks_input() {
        return;
    }

    if keys.just_pressed(KeyCode::Digit1) {
        scene.generation_state.selected_kind = PolyhedronKind::Cube;
        println!(
            "Selected child shape: {:?}",
            scene.generation_state.selected_kind
        );
    }
    if keys.just_pressed(KeyCode::Digit2) {
        scene.generation_state.selected_kind = PolyhedronKind::Tetrahedron;
        println!(
            "Selected child shape: {:?}",
            scene.generation_state.selected_kind
        );
    }
    if keys.just_pressed(KeyCode::Digit3) {
        scene.generation_state.selected_kind = PolyhedronKind::Octahedron;
        println!(
            "Selected child shape: {:?}",
            scene.generation_state.selected_kind
        );
    }
    if keys.just_pressed(KeyCode::Digit4) {
        scene.generation_state.selected_kind = PolyhedronKind::Dodecahedron;
        println!(
            "Selected child shape: {:?}",
            scene.generation_state.selected_kind
        );
    }

    let (min_scale_ratio, max_scale_ratio) = scene.app_config.generation.scale_bounds();
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        scene.generation_state.scale_ratio = (scene.generation_state.scale_ratio
            - scene.app_config.generation.scale_adjust_step)
            .clamp(min_scale_ratio, max_scale_ratio);
        println!(
            "Child scale ratio: {:.2}",
            scene.generation_state.scale_ratio
        );
    }
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        scene.generation_state.scale_ratio = (scene.generation_state.scale_ratio
            + scene.app_config.generation.scale_adjust_step)
            .clamp(min_scale_ratio, max_scale_ratio);
        println!(
            "Child scale ratio: {:.2}",
            scene.generation_state.scale_ratio
        );
    }

    let (min_opacity, max_opacity) = scene.app_config.materials.opacity_bounds();
    let mut opacity_changed = false;
    if keys.just_pressed(KeyCode::KeyO) {
        scene.material_state.opacity = adjust_clamped_value(
            scene.material_state.opacity,
            -scene.app_config.materials.opacity_adjust_step,
            min_opacity,
            max_opacity,
        );
        opacity_changed = true;
        println!("{}", opacity_status_message(scene.material_state.opacity));
    }
    if keys.just_pressed(KeyCode::KeyP) {
        scene.material_state.opacity = adjust_clamped_value(
            scene.material_state.opacity,
            scene.app_config.materials.opacity_adjust_step,
            min_opacity,
            max_opacity,
        );
        opacity_changed = true;
        println!("{}", opacity_status_message(scene.material_state.opacity));
    }
    if keys.just_pressed(KeyCode::KeyI) {
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

    let (min_twist, max_twist) = scene.app_config.generation.twist_bounds();
    let (min_vertex_offset, max_vertex_offset) = scene.app_config.generation.vertex_offset_bounds();
    let mut transform_changed = false;
    let twist_decrease_requested = scene.generation_state.twist_decrease_hold.update(
        key_group_just_pressed(&keys, &TWIST_DECREASE_KEYS),
        key_group_pressed(&keys, &TWIST_DECREASE_KEYS),
        key_group_just_released(&keys, &TWIST_DECREASE_KEYS),
        time.delta_secs(),
        scene.app_config.generation.twist_hold_delay_secs,
        scene.app_config.generation.twist_repeat_interval_secs,
    );
    if twist_decrease_requested {
        scene.generation_state.twist_per_vertex_radians = adjust_clamped_value(
            scene.generation_state.twist_per_vertex_radians,
            -scene.app_config.generation.twist_adjust_step,
            min_twist,
            max_twist,
        );
        transform_changed = true;
        println!(
            "{}",
            twist_status_message(scene.generation_state.twist_per_vertex_radians)
        );
    }
    let twist_increase_requested = scene.generation_state.twist_increase_hold.update(
        key_group_just_pressed(&keys, &TWIST_INCREASE_KEYS),
        key_group_pressed(&keys, &TWIST_INCREASE_KEYS),
        key_group_just_released(&keys, &TWIST_INCREASE_KEYS),
        time.delta_secs(),
        scene.app_config.generation.twist_hold_delay_secs,
        scene.app_config.generation.twist_repeat_interval_secs,
    );
    if twist_increase_requested {
        scene.generation_state.twist_per_vertex_radians = adjust_clamped_value(
            scene.generation_state.twist_per_vertex_radians,
            scene.app_config.generation.twist_adjust_step,
            min_twist,
            max_twist,
        );
        transform_changed = true;
        println!(
            "{}",
            twist_status_message(scene.generation_state.twist_per_vertex_radians)
        );
    }
    if keys.just_pressed(KeyCode::KeyT) {
        scene.generation_state.twist_per_vertex_radians = scene
            .app_config
            .generation
            .default_twist_per_vertex_radians_clamped();
        transform_changed = true;
        scene.generation_state.twist_decrease_hold.reset();
        scene.generation_state.twist_increase_hold.reset();
        println!(
            "Reset {}",
            twist_status_message(scene.generation_state.twist_per_vertex_radians).to_lowercase()
        );
    }

    let vertex_offset_decrease_requested =
        scene.generation_state.vertex_offset_decrease_hold.update(
            key_group_just_pressed(&keys, &VERTEX_OFFSET_DECREASE_KEYS),
            key_group_pressed(&keys, &VERTEX_OFFSET_DECREASE_KEYS),
            key_group_just_released(&keys, &VERTEX_OFFSET_DECREASE_KEYS),
            time.delta_secs(),
            scene.app_config.generation.vertex_offset_hold_delay_secs,
            scene
                .app_config
                .generation
                .vertex_offset_repeat_interval_secs,
        );
    if vertex_offset_decrease_requested {
        scene.generation_state.vertex_offset_ratio = adjust_clamped_value(
            scene.generation_state.vertex_offset_ratio,
            -scene.app_config.generation.vertex_offset_adjust_step,
            min_vertex_offset,
            max_vertex_offset,
        );
        transform_changed = true;
        println!(
            "{}",
            vertex_offset_status_message(scene.generation_state.vertex_offset_ratio)
        );
    }
    let vertex_offset_increase_requested =
        scene.generation_state.vertex_offset_increase_hold.update(
            key_group_just_pressed(&keys, &VERTEX_OFFSET_INCREASE_KEYS),
            key_group_pressed(&keys, &VERTEX_OFFSET_INCREASE_KEYS),
            key_group_just_released(&keys, &VERTEX_OFFSET_INCREASE_KEYS),
            time.delta_secs(),
            scene.app_config.generation.vertex_offset_hold_delay_secs,
            scene
                .app_config
                .generation
                .vertex_offset_repeat_interval_secs,
        );
    if vertex_offset_increase_requested {
        scene.generation_state.vertex_offset_ratio = adjust_clamped_value(
            scene.generation_state.vertex_offset_ratio,
            scene.app_config.generation.vertex_offset_adjust_step,
            min_vertex_offset,
            max_vertex_offset,
        );
        transform_changed = true;
        println!(
            "{}",
            vertex_offset_status_message(scene.generation_state.vertex_offset_ratio)
        );
    }
    if keys.just_pressed(KeyCode::KeyC) {
        scene.generation_state.vertex_offset_ratio = scene
            .app_config
            .generation
            .default_vertex_offset_ratio_clamped();
        transform_changed = true;
        scene.generation_state.vertex_offset_decrease_hold.reset();
        scene.generation_state.vertex_offset_increase_hold.reset();
        println!(
            "Reset {}",
            vertex_offset_status_message(scene.generation_state.vertex_offset_ratio).to_lowercase()
        );
    }

    if transform_changed {
        let twist_per_vertex_radians = scene.generation_state.twist_per_vertex_radians;
        let vertex_offset_ratio = scene.generation_state.vertex_offset_ratio;
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

    if keys.just_pressed(KeyCode::KeyR) {
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
        keys.just_pressed(KeyCode::Space),
        keys.pressed(KeyCode::Space),
        keys.just_released(KeyCode::Space),
        time.delta_secs(),
        scene.app_config.generation.spawn_hold_delay_secs,
        scene.app_config.generation.spawn_repeat_interval_secs,
    );
    if !spawn_requested {
        return;
    }

    let selected_kind = scene.generation_state.selected_kind;
    let scale_ratio = scene.generation_state.scale_ratio;
    let twist_per_vertex_radians = scene.generation_state.twist_per_vertex_radians;
    let vertex_offset_ratio = scene.generation_state.vertex_offset_ratio;
    let Some(spawn) = next_spawn(
        &mut scene.generation_state.nodes,
        &scene.shape_assets.catalog,
        selected_kind,
        scale_ratio,
        scene
            .app_config
            .generation
            .spawn_tuning(twist_per_vertex_radians, vertex_offset_ratio),
    ) else {
        eprintln!("No valid spawn position is currently available.");
        return;
    };
    let node_index = scene.generation_state.nodes.len() - 1;

    spawn_polyhedron_entity(
        &mut scene.commands,
        &mut scene.materials,
        scene.shape_assets.mesh(spawn.kind),
        &spawn.node,
        &scene.app_config.materials,
        scene.material_state.opacity,
        node_index,
    );
    println!(
        "Spawned {:?} at level {} from parent level {}",
        spawn.kind, spawn.node.level, spawn.parent_level
    );
}

fn key_group_just_pressed(keys: &ButtonInput<KeyCode>, key_codes: &[KeyCode]) -> bool {
    key_codes
        .iter()
        .copied()
        .any(|key_code| keys.just_pressed(key_code))
}

fn key_group_pressed(keys: &ButtonInput<KeyCode>, key_codes: &[KeyCode]) -> bool {
    key_codes
        .iter()
        .copied()
        .any(|key_code| keys.pressed(key_code))
}

fn key_group_just_released(keys: &ButtonInput<KeyCode>, key_codes: &[KeyCode]) -> bool {
    !key_group_pressed(keys, key_codes)
        && key_codes
            .iter()
            .copied()
            .any(|key_code| keys.just_released(key_code))
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
        "Child vertex offset: {:.2}x child radius",
        offset_ratio.max(0.0)
    )
}

#[cfg(test)]
mod tests {
    use super::{
        SpawnHoldState, adjust_clamped_value, twist_status_message, vertex_offset_status_message,
    };
    use crate::config::GenerationConfig;

    #[test]
    fn spawn_hold_repeats_while_space_is_held() {
        let generation_config = GenerationConfig::default();
        let mut spawn_hold = SpawnHoldState::default();

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
        let mut spawn_hold = SpawnHoldState::default();

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
}
