use bevy::prelude::*;

use crate::polyhedra::{MAX_SCALE_RATIO, MIN_SCALE_RATIO, PolyhedronKind, next_spawn};
use crate::scene::{
    GenerationState, PolyhedronEntity, ShapeAssets, reset_generation_state, spawn_polyhedron_entity,
};

const SPAWN_HOLD_DELAY_SECS: f32 = 0.24;
const SPAWN_REPEAT_INTERVAL_SECS: f32 = 0.07;

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
            SPAWN_REPEAT_INTERVAL_SECS
        } else {
            SPAWN_HOLD_DELAY_SECS
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

    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        generation_state.scale_ratio =
            (generation_state.scale_ratio - 0.05).clamp(MIN_SCALE_RATIO, MAX_SCALE_RATIO);
        println!("Child scale ratio: {:.2}", generation_state.scale_ratio);
    }
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        generation_state.scale_ratio =
            (generation_state.scale_ratio + 0.05).clamp(MIN_SCALE_RATIO, MAX_SCALE_RATIO);
        println!("Child scale ratio: {:.2}", generation_state.scale_ratio);
    }

    if keys.just_pressed(KeyCode::KeyR) {
        for entity in &polyhedron_entities {
            commands.entity(entity).despawn();
        }

        let root = reset_generation_state(&mut generation_state, &shape_assets.catalog);
        spawn_polyhedron_entity(
            &mut commands,
            &mut materials,
            shape_assets.mesh(root.kind),
            &root,
        );
        println!("Reset scene to the root polyhedron.");
        return;
    }

    let spawn_requested = generation_state.spawn_hold.update(
        keys.just_pressed(KeyCode::Space),
        keys.pressed(KeyCode::Space),
        keys.just_released(KeyCode::Space),
        time.delta_secs(),
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
    ) else {
        eprintln!("No valid spawn position is currently available.");
        return;
    };

    spawn_polyhedron_entity(
        &mut commands,
        &mut materials,
        shape_assets.mesh(spawn.kind),
        &spawn.node,
    );
    println!(
        "Spawned {:?} at level {} from parent level {}",
        spawn.kind, spawn.node.level, spawn.parent_level
    );
}

#[cfg(test)]
mod tests {
    use super::{SPAWN_HOLD_DELAY_SECS, SPAWN_REPEAT_INTERVAL_SECS, SpawnHoldState};

    #[test]
    fn spawn_hold_repeats_while_space_is_held() {
        let mut spawn_hold = SpawnHoldState::default();

        assert!(spawn_hold.update(true, true, false, 0.0));
        assert!(!spawn_hold.update(false, true, false, SPAWN_HOLD_DELAY_SECS * 0.5));
        assert!(spawn_hold.update(false, true, false, SPAWN_HOLD_DELAY_SECS * 0.5));
        assert!(!spawn_hold.update(false, true, false, SPAWN_REPEAT_INTERVAL_SECS * 0.5));
        assert!(spawn_hold.update(false, true, false, SPAWN_REPEAT_INTERVAL_SECS * 0.5));
    }

    #[test]
    fn spawn_hold_resets_after_release() {
        let mut spawn_hold = SpawnHoldState::default();

        assert!(spawn_hold.update(true, true, false, 0.0));
        assert!(!spawn_hold.update(false, false, true, 0.0));
        assert!(spawn_hold.update(true, true, false, 0.0));
    }
}
