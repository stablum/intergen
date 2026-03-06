use bevy::prelude::*;

use crate::config::AppConfig;
use crate::polyhedra::{PolyhedronKind, next_spawn};
use crate::scene::{
    GenerationState, PolyhedronEntity, ShapeAssets, reset_generation_state, spawn_polyhedron_entity,
};

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
    let Some(spawn) = next_spawn(
        &mut generation_state.nodes,
        &shape_assets.catalog,
        selected_kind,
        scale_ratio,
        app_config.generation.spawn_tuning(),
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

#[cfg(test)]
mod tests {
    use super::SpawnHoldState;
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
}
