use bevy::camera::visibility::RenderLayers;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;

use crate::camera::SceneCamera;
use crate::scene::{GenerationState, ShapeAssets};

pub(crate) const SPAWN_DEBUG_RENDER_LAYER: usize = 1;
pub(crate) const SPAWN_DEBUG_CAMERA_ORDER: isize = 1;

pub(crate) struct SpawnDebugOverlayPlugin;

impl Plugin for SpawnDebugOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnDebugOverlayState>();
    }
}

#[derive(Resource)]
pub(crate) struct SpawnDebugOverlayState {
    enabled: bool,
    parent_node_index: Option<usize>,
    child_node_index: Option<usize>,
    material: Option<Handle<StandardMaterial>>,
}

impl Default for SpawnDebugOverlayState {
    fn default() -> Self {
        Self {
            enabled: false,
            parent_node_index: Some(0),
            child_node_index: None,
            material: None,
        }
    }
}

impl SpawnDebugOverlayState {
    pub(crate) fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }

    pub(crate) fn focus_parent(&mut self, parent_node_index: Option<usize>) {
        self.parent_node_index = parent_node_index;
        self.child_node_index = None;
    }

    pub(crate) fn track_spawn(&mut self, parent_node_index: usize, child_node_index: usize) {
        self.parent_node_index = Some(parent_node_index);
        self.child_node_index = Some(child_node_index);
    }

    fn desired_node_index(&self, role: SpawnDebugOverlayRole) -> Option<usize> {
        if !self.enabled {
            return None;
        }

        match role {
            SpawnDebugOverlayRole::Parent => self.parent_node_index,
            SpawnDebugOverlayRole::Child => self.child_node_index,
        }
    }
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
enum SpawnDebugOverlayRole {
    Parent,
    Child,
}

impl SpawnDebugOverlayRole {
    const ALL: [Self; 2] = [Self::Parent, Self::Child];

    fn slot(self) -> usize {
        match self {
            Self::Parent => 0,
            Self::Child => 1,
        }
    }
}

#[derive(Component)]
pub(crate) struct SpawnDebugOverlayEntity {
    role: SpawnDebugOverlayRole,
    node_index: usize,
}

#[derive(Component)]
pub(crate) struct SpawnDebugOverlayCamera;

pub(crate) fn sync_spawn_debug_overlay_system(
    mut commands: Commands,
    generation_state: Res<GenerationState>,
    shape_assets: Res<ShapeAssets>,
    mut overlay_state: ResMut<SpawnDebugOverlayState>,
    mut overlay_materials: ResMut<Assets<StandardMaterial>>,
    scene_camera: Query<
        (Entity, &Transform),
        (With<SceneCamera>, Without<SpawnDebugOverlayCamera>),
    >,
    mut overlay_camera: Query<
        (Entity, &mut Camera, &mut Transform),
        (With<SpawnDebugOverlayCamera>, Without<SceneCamera>),
    >,
    mut ui_target_cameras: Query<&mut UiTargetCamera>,
    mut overlays: Query<
        (
            Entity,
            &SpawnDebugOverlayEntity,
            &mut Mesh3d,
            &mut Transform,
        ),
        (Without<SceneCamera>, Without<SpawnDebugOverlayCamera>),
    >,
) {
    if let (
        Ok((scene_camera_entity, scene_transform)),
        Ok((overlay_camera_entity, mut camera, mut camera_transform)),
    ) = (scene_camera.single(), overlay_camera.single_mut())
    {
        camera.is_active = overlay_state.enabled;
        *camera_transform = *scene_transform;
        let desired_ui_camera = ui_camera_for_debug_state(
            overlay_state.enabled,
            scene_camera_entity,
            overlay_camera_entity,
        );
        for mut ui_target_camera in &mut ui_target_cameras {
            if ui_target_camera.0 == scene_camera_entity
                || ui_target_camera.0 == overlay_camera_entity
            {
                ui_target_camera.0 = desired_ui_camera;
            }
        }
    }

    let material = overlay_state
        .material
        .get_or_insert_with(|| {
            overlay_materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0),
                emissive: LinearRgba::rgb(4.0, 0.0, 0.0),
                unlit: true,
                cull_mode: None,
                ..default()
            })
        })
        .clone();
    let mut found = [false; 2];

    for (entity, overlay, mut mesh, mut transform) in &mut overlays {
        let desired_node_index = overlay_state.desired_node_index(overlay.role);
        let Some(node) = desired_node_index
            .filter(|node_index| *node_index == overlay.node_index)
            .and_then(|node_index| generation_state.nodes.get(node_index))
        else {
            commands.entity(entity).despawn();
            continue;
        };

        mesh.0 = shape_assets.debug_wireframe_mesh(node.kind).clone();
        *transform = overlay_transform(node);
        found[overlay.role.slot()] = true;
    }

    for role in SpawnDebugOverlayRole::ALL {
        if found[role.slot()] {
            continue;
        }
        let Some(node_index) = overlay_state.desired_node_index(role) else {
            continue;
        };
        let Some(node) = generation_state.nodes.get(node_index) else {
            continue;
        };

        commands.spawn((
            SpawnDebugOverlayEntity { role, node_index },
            Mesh3d(shape_assets.debug_wireframe_mesh(node.kind).clone()),
            MeshMaterial3d(material.clone()),
            overlay_transform(node),
            RenderLayers::layer(SPAWN_DEBUG_RENDER_LAYER),
            NotShadowCaster,
            NotShadowReceiver,
            Visibility::Visible,
        ));
    }
}

fn ui_camera_for_debug_state(
    enabled: bool,
    scene_camera: Entity,
    overlay_camera: Entity,
) -> Entity {
    if enabled {
        overlay_camera
    } else {
        scene_camera
    }
}

fn overlay_transform(node: &crate::shapes::ShapeNode) -> Transform {
    Transform {
        translation: node.center,
        rotation: node.rotation,
        scale: node.combined_scale(),
    }
}

pub(crate) fn spawn_debug_overlay_status_message(enabled: bool) -> &'static str {
    if enabled {
        "Spawn debug overlay: on"
    } else {
        "Spawn debug overlay: off"
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Entity;

    use super::{
        SpawnDebugOverlayState, spawn_debug_overlay_status_message, ui_camera_for_debug_state,
    };

    #[test]
    fn ui_tracks_the_camera_that_finishes_the_scene() {
        let scene_camera = Entity::from_raw_u32(1).expect("scene camera entity should be valid");
        let overlay_camera =
            Entity::from_raw_u32(2).expect("overlay camera entity should be valid");

        assert_eq!(
            ui_camera_for_debug_state(false, scene_camera, overlay_camera),
            scene_camera
        );
        assert_eq!(
            ui_camera_for_debug_state(true, scene_camera, overlay_camera),
            overlay_camera
        );
    }

    #[test]
    fn overlay_starts_hidden_and_focused_on_the_root() {
        let mut state = SpawnDebugOverlayState::default();

        assert!(state.toggle());
        assert_eq!(state.parent_node_index, Some(0));
        assert_eq!(state.child_node_index, None);
    }

    #[test]
    fn tracking_a_sibling_replaces_only_the_child_focus() {
        let mut state = SpawnDebugOverlayState::default();
        state.track_spawn(3, 7);
        state.track_spawn(3, 8);

        assert_eq!(state.parent_node_index, Some(3));
        assert_eq!(state.child_node_index, Some(8));
    }

    #[test]
    fn changing_parent_replaces_the_pair_and_rewind_clears_the_child() {
        let mut state = SpawnDebugOverlayState::default();
        state.track_spawn(3, 7);
        state.track_spawn(7, 11);

        assert_eq!(state.parent_node_index, Some(7));
        assert_eq!(state.child_node_index, Some(11));

        state.focus_parent(Some(0));
        assert_eq!(state.parent_node_index, Some(0));
        assert_eq!(state.child_node_index, None);
    }

    #[test]
    fn status_message_reports_both_toggle_states() {
        assert!(spawn_debug_overlay_status_message(true).ends_with("on"));
        assert!(spawn_debug_overlay_status_message(false).ends_with("off"));
    }
}
