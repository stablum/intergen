use bevy::prelude::*;

use crate::config::AppConfig;
use crate::polyhedra::{PolyhedronKind, next_spawn};
use crate::scene::{
    GenerationState, PolyhedronEntity, ShapeAssets, reset_generation_state, spawn_polyhedron_entity,
};

const RADIANS_TO_DEGREES: f32 = 180.0 / std::f32::consts::PI;

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
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    app_config: Res<AppConfig>,
    shape_assets: Res<ShapeAssets>,
    mut generation_state: ResMut<GenerationState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    polyhedron_entities: Query<Entity, With<PolyhedronEntity>>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        generation_state.selected_kind = PolyhedronKind::Cube;
        println!("Selected child shape: {:?}", generation_state.selected_kind);
    }
    if keys.just_pressed(KeyCode::Digit2) {
        generation_state.selected_kind = PolyhedronKind::Tetrahedron;
        println!("Selected child shape: {:?}", generation_state.selected_kind);
    }
    if keys.just_pressed(KeyCode::Digit3) {
        generation_state.selected_kind = PolyhedronKind::Octahedron;
        println!("Selected child shape: {:?}", generation_state.selected_kind);
    }
    if keys.just_pressed(KeyCode::Digit4) {
        generation_state.selected_kind = PolyhedronKind::Dodecahedron;
        println!("Selected child shape: {:?}", generation_state.selected_kind);
    }

    let (min_scale_ratio, max_scale_ratio) = app_config.generation.scale_bounds();
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        generation_state.scale_ratio = (generation_state.scale_ratio
            - app_config.generation.scale_adjust_step)
            .clamp(min_scale_ratio, max_scale_ratio);
        println!("Child scale ratio: {:.2}", generation_state.scale_ratio);
    }
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        generation_state.scale_ratio = (generation_state.scale_ratio
            + app_config.generation.scale_adjust_step)
            .clamp(min_scale_ratio, max_scale_ratio);
        println!("Child scale ratio: {:.2}", generation_state.scale_ratio);
    }

    let (min_twist, max_twist) = app_config.generation.twist_bounds();
    if twist_decrease_requested(&keys) {
        generation_state.twist_per_vertex_radians = adjust_clamped_value(
            generation_state.twist_per_vertex_radians,
            -app_config.generation.twist_adjust_step,
            min_twist,
            max_twist,
        );
        println!(
            "{}",
            twist_status_message(generation_state.twist_per_vertex_radians)
        );
    }
    if twist_increase_requested(&keys) {
        generation_state.twist_per_vertex_radians = adjust_clamped_value(
            generation_state.twist_per_vertex_radians,
            app_config.generation.twist_adjust_step,
            min_twist,
            max_twist,
        );
        println!(
            "{}",
            twist_status_message(generation_state.twist_per_vertex_radians)
        );
    }
    if keys.just_pressed(KeyCode::KeyT) {
        generation_state.twist_per_vertex_radians = app_config
            .generation
            .default_twist_per_vertex_radians_clamped();
        println!(
            "Reset {}",
            twist_status_message(generation_state.twist_per_vertex_radians).to_lowercase()
        );
    }

    if keys.just_pressed(KeyCode::KeyR) {
        for entity in &polyhedron_entities {
            commands.entity(entity).despawn();
        }

        let root = reset_generation_state(
            &mut generation_state,
            &shape_assets.catalog,
            &app_config.generation,
        );
        spawn_polyhedron_entity(
            &mut commands,
            &mut materials,
            shape_assets.mesh(root.kind),
            &root,
            &app_config.materials,
        );
        println!("Reset scene to the root polyhedron.");
        return;
    }

    let spawn_requested = generation_state.spawn_hold.update(
        keys.just_pressed(KeyCode::Space),
        keys.pressed(KeyCode::Space),
        keys.just_released(KeyCode::Space),
        time.delta_secs(),
        app_config.generation.spawn_hold_delay_secs,
        app_config.generation.spawn_repeat_interval_secs,
    );
    if !spawn_requested {
        return;
    }

    let selected_kind = generation_state.selected_kind;
    let scale_ratio = generation_state.scale_ratio;
    let twist_per_vertex_radians = generation_state.twist_per_vertex_radians;
    let Some(spawn) = next_spawn(
        &mut generation_state.nodes,
        &shape_assets.catalog,
        selected_kind,
        scale_ratio,
        app_config.generation.spawn_tuning(twist_per_vertex_radians),
    ) else {
        eprintln!("No valid spawn position is currently available.");
        return;
    };

    spawn_polyhedron_entity(
        &mut commands,
        &mut materials,
        shape_assets.mesh(spawn.kind),
        &spawn.node,
        &app_config.materials,
    );
    println!(
        "Spawned {:?} at level {} from parent level {}",
        spawn.kind, spawn.node.level, spawn.parent_level
    );
}

fn twist_decrease_requested(keys: &ButtonInput<KeyCode>) -> bool {
    keys.just_pressed(KeyCode::BracketLeft) || keys.just_pressed(KeyCode::Comma)
}

fn twist_increase_requested(keys: &ButtonInput<KeyCode>) -> bool {
    keys.just_pressed(KeyCode::BracketRight) || keys.just_pressed(KeyCode::Period)
}

fn adjust_clamped_value(current: f32, delta: f32, min: f32, max: f32) -> f32 {
    (current + delta).clamp(min, max)
}

pub(crate) fn twist_status_message(radians: f32) -> String {
    format!(
        "Child twist angle: {:.3} rad ({:.1} deg)",
        radians,
        radians * RADIANS_TO_DEGREES
    )
}

#[cfg(test)]
mod tests {
    use super::{SpawnHoldState, adjust_clamped_value, twist_status_message};
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
}
