use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::camera::CameraRig;
use crate::config::{AppConfig, LightingConfig, MaterialConfig, RenderingConfig};
use crate::effect_tuner::{EffectRuntimeSnapshot, EffectTunerState};
use crate::scene::{
    GenerationParameters, GenerationState, LightingState, MaterialState, RenderingState,
    SingleSpawnSourceCursor, StageState,
};
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
    #[serde(default = "default_single_attachment_repeat_count")]
    pub(crate) single_attachment_repeat_count: usize,
    pub(crate) scale_ratio: f32,
    #[serde(default = "unit_axis_scale_array")]
    pub(crate) child_axis_scale: [f32; 3],
    pub(crate) twist_per_vertex_radians: f32,
    pub(crate) vertex_offset_ratio: f32,
    #[serde(default = "zero_vec3_array")]
    pub(crate) child_position_offset: [f32; 3],
    #[serde(default)]
    pub(crate) vertex_spawn_exclusion_probability: f32,
    #[serde(default)]
    pub(crate) single_spawn_source_cursor: Option<SingleSpawnSourceCursorSnapshot>,
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
    #[serde(default = "unit_axis_scale_array")]
    pub(crate) axis_scale: [f32; 3],
    #[serde(default = "zero_vec3_array")]
    pub(crate) local_position_offset: [f32; 3],
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SingleSpawnSourceCursorSnapshot {
    pub(crate) parent_index: usize,
    #[serde(default)]
    pub(crate) attachment_mode: SpawnPlacementMode,
    pub(crate) attachment_index: usize,
    pub(crate) successful_spawns: usize,
}

impl SceneStateSnapshot {
    pub(crate) fn capture(
        app_config: &AppConfig,
        camera_rig: &CameraRig,
        generation_state: &GenerationState,
        rendering_state: &RenderingState,
        lighting_state: &LightingState,
        material_state: &MaterialState,
        stage_state: &StageState,
        effect_tuner: &EffectTunerState,
    ) -> Self {
        let runtime_materials = material_state.runtime_material_config(&app_config.materials);
        let runtime_rendering =
            rendering_state.runtime_rendering_config(&app_config.rendering, stage_state);
        let runtime_lighting = lighting_state.runtime_lighting_config(&app_config.lighting);

        Self {
            rendering: runtime_rendering,
            lighting: runtime_lighting,
            materials: runtime_materials,
            camera: CameraRigSnapshot::capture(camera_rig),
            generation: GenerationSnapshot::capture(generation_state),
            material_state: MaterialRuntimeSnapshot::capture(material_state),
            effects: effect_tuner.runtime_snapshot(),
        }
    }

    pub(crate) fn capture_preset(
        app_config: &AppConfig,
        camera_rig: &CameraRig,
        generation_state: &GenerationState,
        rendering_state: &RenderingState,
        lighting_state: &LightingState,
        material_state: &MaterialState,
        stage_state: &StageState,
        effect_tuner: &EffectTunerState,
    ) -> Self {
        let base_camera = effect_tuner.base_camera_rig(&app_config.camera, camera_rig);
        let base_generation =
            effect_tuner.base_generation_state(&app_config.generation, generation_state);
        let base_rendering_state = effect_tuner.base_rendering_state(rendering_state);
        let base_lighting_state = effect_tuner.base_lighting_state(lighting_state);
        let base_material_state =
            effect_tuner.base_material_state(&app_config.materials, material_state);
        let base_stage_state = effect_tuner.base_stage_state(stage_state);
        let runtime_materials = base_material_state.runtime_material_config(&app_config.materials);
        let runtime_rendering =
            base_rendering_state.runtime_rendering_config(&app_config.rendering, &base_stage_state);
        let runtime_lighting = base_lighting_state.runtime_lighting_config(&app_config.lighting);

        Self {
            rendering: runtime_rendering,
            lighting: runtime_lighting,
            materials: runtime_materials,
            camera: CameraRigSnapshot::capture(&base_camera),
            generation: GenerationSnapshot::capture(&base_generation),
            material_state: MaterialRuntimeSnapshot::capture(&base_material_state),
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
            single_attachment_repeat_count: generation_state.single_attachment_repeat_count,
            scale_ratio: generation_state.scale_ratio_base(),
            child_axis_scale: vec3_to_array(generation_state.child_axis_scale_base()),
            twist_per_vertex_radians: generation_state.twist_per_vertex_radians_base(),
            vertex_offset_ratio: generation_state.vertex_offset_ratio_base(),
            child_position_offset: vec3_to_array(generation_state.child_position_offset_base()),
            vertex_spawn_exclusion_probability: generation_state
                .vertex_spawn_exclusion_probability_base(),
            single_spawn_source_cursor: generation_state
                .single_spawn_source_cursor
                .map(SingleSpawnSourceCursorSnapshot::capture),
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
            single_attachment_repeat_count: self.single_attachment_repeat_count,
            single_spawn_source_cursor: self
                .single_spawn_source_cursor
                .as_ref()
                .map(SingleSpawnSourceCursorSnapshot::to_runtime),
            parameters: GenerationParameters::from_base_values_with_axis_scale(
                self.scale_ratio,
                vec3_from_array(self.child_axis_scale),
                self.twist_per_vertex_radians,
                self.vertex_offset_ratio,
                vec3_from_array(self.child_position_offset),
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
            axis_scale: vec3_to_array(node.axis_scale),
            local_position_offset: vec3_to_array(node.local_position_offset),
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
        let mut node = ShapeNode {
            kind: self.shape_kind,
            level: self.level,
            center: vec3_from_array(self.center),
            rotation: quat_from_array(self.rotation),
            scale: self.scale,
            axis_scale: vec3_from_array(self.axis_scale),
            local_position_offset: vec3_from_array(self.local_position_offset),
            radius: 0.0,
            occupied_attachments: AttachmentOccupancy {
                vertices: resize_occupancy(&self.occupied_vertices, geometry.vertices.len()),
                edges: resize_occupancy(&self.occupied_edges, geometry.edges.len()),
                faces: resize_occupancy(&self.occupied_faces, geometry.faces.len()),
            },
            origin: self.origin.to_runtime()?,
        };
        node.radius = node.bounding_radius(geometry);

        Ok(node)
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

impl SingleSpawnSourceCursorSnapshot {
    pub(crate) fn capture(cursor: SingleSpawnSourceCursor) -> Self {
        Self {
            parent_index: cursor.parent_index,
            attachment_mode: cursor.attachment.mode,
            attachment_index: cursor.attachment.index,
            successful_spawns: cursor.successful_spawns,
        }
    }

    pub(crate) fn to_runtime(&self) -> SingleSpawnSourceCursor {
        SingleSpawnSourceCursor {
            parent_index: self.parent_index,
            attachment: SpawnAttachment {
                mode: self.attachment_mode,
                index: self.attachment_index,
            },
            successful_spawns: self.successful_spawns,
        }
    }
}

fn vec3_to_array(vector: Vec3) -> [f32; 3] {
    [vector.x, vector.y, vector.z]
}

fn vec3_from_array(vector: [f32; 3]) -> Vec3 {
    Vec3::new(vector[0], vector[1], vector[2])
}

fn unit_axis_scale_array() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

fn default_single_attachment_repeat_count() -> usize {
    1
}

fn zero_vec3_array() -> [f32; 3] {
    [0.0, 0.0, 0.0]
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
    use bevy::prelude::Vec3;

    use super::{GenerationSnapshot, NodeOriginSnapshot, SceneStateSnapshot, ShapeNodeSnapshot};
    use crate::camera::CameraRig;
    use crate::config::AppConfig;
    use crate::effect_tuner::EffectTunerState;
    use crate::scene::{GenerationState, LightingState, MaterialState, RenderingState, StageState};
    use crate::shapes::{ShapeCatalog, ShapeKind};

    #[test]
    fn capture_uses_runtime_stage_visibility() {
        let app_config = AppConfig::default();
        let camera_rig = CameraRig::from_config(&app_config.camera);
        let generation_state = GenerationState::from_config(&app_config.generation);
        let rendering_state = RenderingState::from_config(&app_config.rendering);
        let lighting_state = LightingState::from_config(&app_config.lighting);
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
            &rendering_state,
            &lighting_state,
            &material_state,
            &stage_state,
            &effect_tuner,
        );

        assert!(snapshot.rendering.stage.enabled);
        assert!(snapshot.rendering.stage.floor.enabled);
        assert!(!snapshot.rendering.stage.backdrop.enabled);
    }

    #[test]
    fn shape_node_snapshot_defaults_axis_scale_to_unit() {
        let shape_catalog = ShapeCatalog::new();
        let snapshot: ShapeNodeSnapshot = toml::from_str(
            r#"
shape_kind = "cube"
level = 0
center = [0.0, 0.0, 0.0]
rotation = [0.0, 0.0, 0.0, 1.0]
scale = 2.0
radius = 999.0
occupied_vertices = []
occupied_edges = []
occupied_faces = []
origin = "Root"
"#,
        )
        .expect("snapshot should parse");

        let node = snapshot
            .to_runtime(&shape_catalog)
            .expect("snapshot should become a runtime node");

        assert_eq!(node.axis_scale, Vec3::ONE);
        assert_eq!(node.local_position_offset, Vec3::ZERO);
        assert!(
            (node.radius - shape_catalog.geometry(ShapeKind::Cube).radius * 2.0).abs() <= 1.0e-6
        );
    }

    #[test]
    fn generation_snapshot_defaults_child_axis_scale_to_unit() {
        let snapshot: GenerationSnapshot = toml::from_str(
            r#"
selected_shape_kind = "cube"
scale_ratio = 0.58
twist_per_vertex_radians = 0.0
vertex_offset_ratio = 0.0
nodes = []
"#,
        )
        .expect("generation snapshot should parse");

        assert_eq!(snapshot.single_attachment_repeat_count, 1);
        assert_eq!(snapshot.child_axis_scale, [1.0, 1.0, 1.0]);
        assert_eq!(snapshot.child_position_offset, [0.0, 0.0, 0.0]);
        assert!(snapshot.single_spawn_source_cursor.is_none());
    }

    #[test]
    fn shape_node_snapshot_radius_uses_largest_axis_scale() {
        let shape_catalog = ShapeCatalog::new();
        let node = ShapeNodeSnapshot {
            shape_kind: ShapeKind::Cube,
            level: 0,
            center: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: 2.0,
            axis_scale: [1.0, 1.5, 0.5],
            local_position_offset: [0.0, 0.0, 0.0],
            radius: 1.0,
            occupied_vertices: Vec::new(),
            occupied_edges: Vec::new(),
            occupied_faces: Vec::new(),
            origin: NodeOriginSnapshot::Root,
        }
        .to_runtime(&shape_catalog)
        .expect("snapshot should become a runtime node");

        assert_eq!(node.axis_scale, Vec3::new(1.0, 1.5, 0.5));
        assert!(
            (node.radius - shape_catalog.geometry(ShapeKind::Cube).radius * 3.0).abs() <= 1.0e-6
        );
    }
}
