use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::camera::CameraRig;
use crate::config::{AppConfig, LightingConfig, MaterialConfig, RenderingConfig};
use crate::effect_tuner::{EffectRuntimeSnapshot, EffectTunerState};
use crate::scene::{GenerationParameters, GenerationState, MaterialState, StageState};
use crate::shapes::{
    AttachmentOccupancy, NodeOrigin, ShapeKind, ShapeNode, SpawnAddMode, SpawnAttachment,
    SpawnPlacementMode,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SceneStateSnapshot {
    pub(crate) rendering: RenderingConfig,
    pub(crate) lighting: LightingConfig,
    pub(crate) materials: MaterialConfig,
    pub(crate) camera: CameraRigSnapshot,
    pub(crate) generation: GenerationSnapshot,
    pub(crate) material_state: MaterialRuntimeSnapshot,
    pub(crate) effects: EffectRuntimeSnapshot,
}

pub(crate) struct PreparedSceneState {
    pub(crate) rendering: RenderingConfig,
    pub(crate) lighting: LightingConfig,
    pub(crate) materials: MaterialConfig,
    pub(crate) camera_rig: CameraRig,
    pub(crate) generation: GenerationState,
    pub(crate) material_opacity: f32,
    pub(crate) effects: EffectRuntimeSnapshot,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct CameraRigSnapshot {
    pub(crate) orientation: [f32; 4],
    pub(crate) angular_velocity: [f32; 3],
    pub(crate) distance: f32,
    pub(crate) zoom_velocity: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct GenerationSnapshot {
    #[serde(alias = "selected_kind")]
    pub(crate) selected_shape_kind: ShapeKind,
    #[serde(default)]
    pub(crate) spawn_placement_mode: SpawnPlacementMode,
    #[serde(default)]
    pub(crate) spawn_add_mode: SpawnAddMode,
    pub(crate) scale_ratio: f32,
    pub(crate) twist_per_vertex_radians: f32,
    pub(crate) vertex_offset_ratio: f32,
    #[serde(default)]
    pub(crate) vertex_spawn_exclusion_probability: f32,
    pub(crate) nodes: Vec<ShapeNodeSnapshot>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct MaterialRuntimeSnapshot {
    pub(crate) opacity: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ShapeNodeSnapshot {
    #[serde(alias = "kind")]
    pub(crate) shape_kind: ShapeKind,
    pub(crate) level: usize,
    pub(crate) center: [f32; 3],
    pub(crate) rotation: [f32; 4],
    pub(crate) scale: f32,
    pub(crate) radius: f32,
    pub(crate) occupied_vertices: Vec<bool>,
    #[serde(default)]
    pub(crate) occupied_edges: Vec<bool>,
    #[serde(default)]
    pub(crate) occupied_faces: Vec<bool>,
    pub(crate) origin: NodeOriginSnapshot,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) enum NodeOriginSnapshot {
    Root,
    Child {
        parent_index: usize,
        #[serde(default)]
        attachment_mode: SpawnPlacementMode,
        #[serde(alias = "vertex_index")]
        attachment_index: usize,
    },
}

impl SceneStateSnapshot {
    pub(crate) fn capture(
        app_config: &AppConfig,
        camera_rig: &CameraRig,
        generation_state: &GenerationState,
        material_state: &MaterialState,
        stage_state: &StageState,
        effect_tuner: &EffectTunerState,
    ) -> Self {
        let runtime_materials = material_state.runtime_material_config(&app_config.materials);
        let mut runtime_rendering = app_config.rendering.clone();
        runtime_rendering.stage = stage_state.runtime_stage_config(&app_config.rendering.stage);

        Self {
            rendering: runtime_rendering,
            lighting: app_config.lighting.clone(),
            materials: runtime_materials,
            camera: CameraRigSnapshot::capture(camera_rig),
            generation: GenerationSnapshot::capture(generation_state),
            material_state: MaterialRuntimeSnapshot::capture(material_state),
            effects: effect_tuner.runtime_snapshot(),
        }
    }

    pub(crate) fn summary(&self) -> String {
        let root_shape_kind = self
            .generation
            .nodes
            .first()
            .map(|node| format!("{:?}", node.shape_kind))
            .unwrap_or_else(|| "Unknown".to_string());
        format!(
            "{} root, {} nodes",
            root_shape_kind,
            self.generation.nodes.len()
        )
    }

    pub(crate) fn prepare_runtime(&self) -> Result<PreparedSceneState, String> {
        Ok(PreparedSceneState {
            rendering: self.rendering.clone(),
            lighting: self.lighting.clone(),
            materials: self.materials.clone(),
            camera_rig: self.camera.to_runtime(),
            generation: self.generation.to_runtime()?,
            material_opacity: self.material_state.opacity.clamp(0.0, 1.0),
            effects: self.effects.clone(),
        })
    }

    pub(crate) fn file_slug(&self) -> String {
        self.generation
            .nodes
            .first()
            .map(|node| format!("{:?}", node.shape_kind).to_ascii_lowercase())
            .unwrap_or_else(|| "scene".to_string())
    }
}

impl CameraRigSnapshot {
    pub(crate) fn capture(camera_rig: &CameraRig) -> Self {
        Self {
            orientation: quat_to_array(camera_rig.orientation),
            angular_velocity: vec3_to_array(camera_rig.angular_velocity),
            distance: camera_rig.distance,
            zoom_velocity: camera_rig.zoom_velocity,
        }
    }

    pub(crate) fn to_runtime(&self) -> CameraRig {
        CameraRig {
            orientation: quat_from_array(self.orientation),
            angular_velocity: vec3_from_array(self.angular_velocity),
            distance: self.distance,
            zoom_velocity: self.zoom_velocity,
        }
    }
}

impl GenerationSnapshot {
    pub(crate) fn capture(generation_state: &GenerationState) -> Self {
        Self {
            selected_shape_kind: generation_state.selected_shape_kind,
            spawn_placement_mode: generation_state.spawn_placement_mode,
            spawn_add_mode: generation_state.spawn_add_mode,
            scale_ratio: generation_state.scale_ratio_base(),
            twist_per_vertex_radians: generation_state.twist_per_vertex_radians_base(),
            vertex_offset_ratio: generation_state.vertex_offset_ratio_base(),
            vertex_spawn_exclusion_probability: generation_state
                .vertex_spawn_exclusion_probability_base(),
            nodes: generation_state
                .nodes
                .iter()
                .map(ShapeNodeSnapshot::capture)
                .collect(),
        }
    }

    pub(crate) fn to_runtime(&self) -> Result<GenerationState, String> {
        let shape_catalog = crate::shapes::ShapeCatalog::new();
        let nodes = self
            .nodes
            .iter()
            .map(|node| node.to_runtime(&shape_catalog))
            .collect::<Result<Vec<_>, _>>()?;
        if nodes.is_empty() {
            return Err("Preset scene has no shape nodes.".to_string());
        }
        Ok(GenerationState {
            nodes,
            selected_shape_kind: self.selected_shape_kind,
            spawn_placement_mode: self.spawn_placement_mode,
            spawn_add_mode: self.spawn_add_mode,
            parameters: GenerationParameters::from_base_values(
                self.scale_ratio,
                self.twist_per_vertex_radians,
                self.vertex_offset_ratio,
                self.vertex_spawn_exclusion_probability,
            ),
            spawn_hold: default(),
        })
    }
}

impl MaterialRuntimeSnapshot {
    pub(crate) fn capture(material_state: &MaterialState) -> Self {
        Self {
            opacity: material_state.opacity,
        }
    }
}

impl ShapeNodeSnapshot {
    pub(crate) fn capture(node: &ShapeNode) -> Self {
        Self {
            shape_kind: node.kind,
            level: node.level,
            center: vec3_to_array(node.center),
            rotation: quat_to_array(node.rotation),
            scale: node.scale,
            radius: node.radius,
            occupied_vertices: node.occupied_attachments.vertices.clone(),
            occupied_edges: node.occupied_attachments.edges.clone(),
            occupied_faces: node.occupied_attachments.faces.clone(),
            origin: NodeOriginSnapshot::capture(node.origin),
        }
    }

    pub(crate) fn to_runtime(
        &self,
        shape_catalog: &crate::shapes::ShapeCatalog,
    ) -> Result<ShapeNode, String> {
        let geometry = shape_catalog.geometry(self.shape_kind);

        Ok(ShapeNode {
            kind: self.shape_kind,
            level: self.level,
            center: vec3_from_array(self.center),
            rotation: quat_from_array(self.rotation),
            scale: self.scale,
            radius: self.radius,
            occupied_attachments: AttachmentOccupancy {
                vertices: resize_occupancy(&self.occupied_vertices, geometry.vertices.len()),
                edges: resize_occupancy(&self.occupied_edges, geometry.edges.len()),
                faces: resize_occupancy(&self.occupied_faces, geometry.faces.len()),
            },
            origin: self.origin.to_runtime()?,
        })
    }
}

impl NodeOriginSnapshot {
    pub(crate) fn capture(origin: NodeOrigin) -> Self {
        match origin {
            NodeOrigin::Root => Self::Root,
            NodeOrigin::Child {
                parent_index,
                attachment,
            } => Self::Child {
                parent_index,
                attachment_mode: attachment.mode,
                attachment_index: attachment.index,
            },
        }
    }

    pub(crate) fn to_runtime(&self) -> Result<NodeOrigin, String> {
        Ok(match self {
            Self::Root => NodeOrigin::Root,
            Self::Child {
                parent_index,
                attachment_mode,
                attachment_index,
            } => NodeOrigin::Child {
                parent_index: *parent_index,
                attachment: SpawnAttachment {
                    mode: *attachment_mode,
                    index: *attachment_index,
                },
            },
        })
    }
}

fn vec3_to_array(vector: Vec3) -> [f32; 3] {
    [vector.x, vector.y, vector.z]
}

fn vec3_from_array(vector: [f32; 3]) -> Vec3 {
    Vec3::new(vector[0], vector[1], vector[2])
}

fn quat_to_array(quat: Quat) -> [f32; 4] {
    [quat.x, quat.y, quat.z, quat.w]
}

fn quat_from_array(quat: [f32; 4]) -> Quat {
    Quat::from_xyzw(quat[0], quat[1], quat[2], quat[3]).normalize()
}

fn resize_occupancy(values: &[bool], len: usize) -> Vec<bool> {
    let mut resized = values.to_vec();
    resized.resize(len, false);
    resized.truncate(len);
    resized
}

#[cfg(test)]
mod tests {
    use super::SceneStateSnapshot;
    use crate::camera::CameraRig;
    use crate::config::AppConfig;
    use crate::effect_tuner::EffectTunerState;
    use crate::scene::{GenerationState, MaterialState, StageState};

    #[test]
    fn capture_uses_runtime_stage_visibility() {
        let app_config = AppConfig::default();
        let camera_rig = CameraRig::from_config(&app_config.camera);
        let generation_state = GenerationState::from_config(&app_config.generation);
        let material_state = MaterialState::from_config(&app_config.materials);
        let mut stage_state = StageState::from_config(&app_config.rendering.stage);
        let effect_tuner = EffectTunerState::from_config(&app_config.effects);

        stage_state.enabled = true;
        stage_state.floor_enabled = true;
        stage_state.backdrop_enabled = false;

        let snapshot = SceneStateSnapshot::capture(
            &app_config,
            &camera_rig,
            &generation_state,
            &material_state,
            &stage_state,
            &effect_tuner,
        );

        assert!(snapshot.rendering.stage.enabled);
        assert!(snapshot.rendering.stage.floor.enabled);
        assert!(!snapshot.rendering.stage.backdrop.enabled);
    }
}
