use std::f32::consts::PI;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::catalog::{ShapeCatalog, ShapeGeometry};

#[derive(Clone, Copy, Debug)]
pub(crate) struct SpawnTuning {
    pub(crate) min_scale_ratio: f32,
    pub(crate) max_scale_ratio: f32,
    pub(crate) containment_epsilon: f32,
    pub(crate) twist_per_vertex_radians: f32,
    pub(crate) vertex_offset_ratio: f32,
    pub(crate) vertex_spawn_exclusion_probability: f32,
    pub(crate) spawn_placement_mode: SpawnPlacementMode,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SpawnPlacementMode {
    #[default]
    Vertex,
    Edge,
    Face,
}

impl SpawnPlacementMode {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::Vertex => Self::Edge,
            Self::Edge => Self::Face,
            Self::Face => Self::Vertex,
        }
    }

    pub(crate) fn plural_label(self) -> &'static str {
        match self {
            Self::Vertex => "vertices",
            Self::Edge => "edges",
            Self::Face => "faces",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SpawnAddMode {
    #[default]
    Single,
    FillLevel,
}

impl SpawnAddMode {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::Single => Self::FillLevel,
            Self::FillLevel => Self::Single,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Single => "single object",
            Self::FillLevel => "fill current level",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ShapeKind {
    Cube,
    Tetrahedron,
    Octahedron,
    Dodecahedron,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub(crate) struct SpawnAttachment {
    pub(crate) mode: SpawnPlacementMode,
    pub(crate) index: usize,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct AttachmentOccupancy {
    pub(crate) vertices: Vec<bool>,
    pub(crate) edges: Vec<bool>,
    pub(crate) faces: Vec<bool>,
}

impl AttachmentOccupancy {
    pub(crate) fn new(geometry: &ShapeGeometry) -> Self {
        Self {
            vertices: vec![false; geometry.vertices.len()],
            edges: vec![false; geometry.edges.len()],
            faces: vec![false; geometry.faces.len()],
        }
    }

    pub(crate) fn is_occupied(&self, attachment: SpawnAttachment) -> bool {
        self.occupied(attachment.mode)[attachment.index]
    }

    pub(crate) fn mark_occupied(&mut self, attachment: SpawnAttachment) {
        self.occupied_mut(attachment.mode)[attachment.index] = true;
    }

    fn occupied(&self, mode: SpawnPlacementMode) -> &[bool] {
        match mode {
            SpawnPlacementMode::Vertex => &self.vertices,
            SpawnPlacementMode::Edge => &self.edges,
            SpawnPlacementMode::Face => &self.faces,
        }
    }

    fn occupied_mut(&mut self, mode: SpawnPlacementMode) -> &mut [bool] {
        match mode {
            SpawnPlacementMode::Vertex => &mut self.vertices,
            SpawnPlacementMode::Edge => &mut self.edges,
            SpawnPlacementMode::Face => &mut self.faces,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum NodeOrigin {
    Root,
    Child {
        parent_index: usize,
        attachment: SpawnAttachment,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct ShapeNode {
    pub(crate) kind: ShapeKind,
    pub(crate) level: usize,
    pub(crate) center: Vec3,
    pub(crate) rotation: Quat,
    pub(crate) scale: f32,
    pub(crate) radius: f32,
    pub(crate) occupied_attachments: AttachmentOccupancy,
    pub(crate) origin: NodeOrigin,
}

#[derive(Clone, Debug)]
pub(crate) struct SpawnedShape {
    pub(crate) kind: ShapeKind,
    pub(crate) parent_level: usize,
    pub(crate) node: ShapeNode,
}

#[derive(Clone, Copy, Debug)]
enum SpawnLevelConstraint {
    Any,
    Exact(usize),
}

#[derive(Clone, Debug)]
struct PendingSpawn {
    parent_index: usize,
    parent_level: usize,
    attachment: SpawnAttachment,
    node: ShapeNode,
}

struct SpawnCandidateInput<'a> {
    parent: &'a ShapeNode,
    parent_geometry: &'a ShapeGeometry,
    child_kind: ShapeKind,
    child_geometry: &'a ShapeGeometry,
    parent_index: usize,
    attachment: SpawnAttachment,
    scale_ratio: f32,
    tuning: SpawnTuning,
}

pub(crate) fn root_node(kind: ShapeKind, scale: f32, shapes: &ShapeCatalog) -> ShapeNode {
    let geometry = shapes.geometry(kind);
    ShapeNode {
        kind,
        level: 0,
        center: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale,
        radius: geometry.radius * scale,
        occupied_attachments: AttachmentOccupancy::new(geometry),
        origin: NodeOrigin::Root,
    }
}

pub(crate) fn next_spawn(
    nodes: &mut Vec<ShapeNode>,
    shapes: &ShapeCatalog,
    child_kind: ShapeKind,
    scale_ratio: f32,
    tuning: SpawnTuning,
) -> Option<SpawnedShape> {
    find_next_spawn(
        nodes,
        shapes,
        child_kind,
        scale_ratio,
        tuning,
        SpawnLevelConstraint::Any,
    )
    .map(|pending| apply_spawn(nodes, pending))
}

pub(crate) fn spawn_batch(
    nodes: &mut Vec<ShapeNode>,
    shapes: &ShapeCatalog,
    child_kind: ShapeKind,
    scale_ratio: f32,
    tuning: SpawnTuning,
    add_mode: SpawnAddMode,
) -> Vec<SpawnedShape> {
    let Some(first) = next_spawn(nodes, shapes, child_kind, scale_ratio, tuning) else {
        return Vec::new();
    };

    let mut spawned = vec![first];
    if add_mode != SpawnAddMode::FillLevel {
        return spawned;
    }

    let target_level = spawned[0].node.level;
    while let Some(spawn) =
        next_spawn_at_level(nodes, shapes, child_kind, scale_ratio, tuning, target_level)
    {
        spawned.push(spawn);
    }

    spawned
}

fn next_spawn_at_level(
    nodes: &mut Vec<ShapeNode>,
    shapes: &ShapeCatalog,
    child_kind: ShapeKind,
    scale_ratio: f32,
    tuning: SpawnTuning,
    target_level: usize,
) -> Option<SpawnedShape> {
    find_next_spawn(
        nodes,
        shapes,
        child_kind,
        scale_ratio,
        tuning,
        SpawnLevelConstraint::Exact(target_level),
    )
    .map(|pending| apply_spawn(nodes, pending))
}

fn find_next_spawn(
    nodes: &[ShapeNode],
    shapes: &ShapeCatalog,
    child_kind: ShapeKind,
    scale_ratio: f32,
    tuning: SpawnTuning,
    level_constraint: SpawnLevelConstraint,
) -> Option<PendingSpawn> {
    let scale_ratio = scale_ratio.clamp(tuning.min_scale_ratio, tuning.max_scale_ratio);
    let highest_level = nodes.iter().map(|node| node.level).max().unwrap_or(0);
    let (start_level, end_level) = match level_constraint {
        SpawnLevelConstraint::Any => (0, highest_level),
        SpawnLevelConstraint::Exact(target_level) => {
            let Some(parent_level) = target_level.checked_sub(1) else {
                return None;
            };
            if parent_level > highest_level {
                return None;
            }
            (parent_level, parent_level)
        }
    };

    for level in start_level..=end_level {
        let parent_indices: Vec<usize> = nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| (node.level == level).then_some(index))
            .collect();

        for parent_index in parent_indices {
            let parent = nodes[parent_index].clone();
            let parent_geometry = shapes.geometry(parent.kind);
            let child_geometry = shapes.geometry(child_kind);

            for attachment_index in 0..parent_geometry.attachment_count(tuning.spawn_placement_mode)
            {
                let attachment = SpawnAttachment {
                    mode: tuning.spawn_placement_mode,
                    index: attachment_index,
                };
                if parent.occupied_attachments.is_occupied(attachment) {
                    continue;
                }
                if attachment_is_excluded(
                    parent_index,
                    attachment,
                    tuning.vertex_spawn_exclusion_probability,
                ) {
                    continue;
                }

                let candidate = spawn_candidate(SpawnCandidateInput {
                    parent: &parent,
                    parent_geometry,
                    child_kind,
                    child_geometry,
                    parent_index,
                    attachment,
                    scale_ratio,
                    tuning,
                });

                if is_fully_contained(
                    candidate.center,
                    candidate.radius,
                    nodes,
                    tuning.containment_epsilon,
                ) || contains_existing(
                    candidate.center,
                    candidate.radius,
                    nodes,
                    tuning.containment_epsilon,
                ) {
                    continue;
                }

                return Some(PendingSpawn {
                    parent_index,
                    parent_level: parent.level,
                    attachment,
                    node: candidate,
                });
            }
        }
    }

    None
}

fn apply_spawn(nodes: &mut Vec<ShapeNode>, pending: PendingSpawn) -> SpawnedShape {
    let node = pending.node;
    nodes[pending.parent_index]
        .occupied_attachments
        .mark_occupied(pending.attachment);
    nodes.push(node.clone());

    SpawnedShape {
        kind: node.kind,
        parent_level: pending.parent_level,
        node,
    }
}

pub(crate) fn recompute_spawn_tree(
    nodes: &mut [ShapeNode],
    shapes: &ShapeCatalog,
    twist_per_vertex_radians: f32,
    vertex_offset_ratio: f32,
) {
    for node_index in 1..nodes.len() {
        let (parents, current_and_rest) = nodes.split_at_mut(node_index);
        let node = &mut current_and_rest[0];
        let NodeOrigin::Child {
            parent_index,
            attachment,
        } = node.origin
        else {
            continue;
        };

        let parent = &parents[parent_index];
        let parent_geometry = shapes.geometry(parent.kind);
        let child_geometry = shapes.geometry(node.kind);
        let child_radius = child_geometry.radius * node.scale;
        let (center, rotation) = child_transform(
            parent,
            parent_geometry,
            attachment,
            child_radius,
            twist_per_vertex_radians,
            vertex_offset_ratio,
        );

        node.center = center;
        node.rotation = rotation;
        node.radius = child_radius;
    }
}

fn spawn_candidate(input: SpawnCandidateInput<'_>) -> ShapeNode {
    let scale = input.parent.scale * input.scale_ratio;
    let radius = input.child_geometry.radius * scale;
    let (center, rotation) = child_transform(
        input.parent,
        input.parent_geometry,
        input.attachment,
        radius,
        input.tuning.twist_per_vertex_radians,
        input.tuning.vertex_offset_ratio,
    );

    ShapeNode {
        kind: input.child_kind,
        level: input.parent.level + 1,
        center,
        rotation,
        scale,
        radius,
        occupied_attachments: AttachmentOccupancy::new(input.child_geometry),
        origin: NodeOrigin::Child {
            parent_index: input.parent_index,
            attachment: input.attachment,
        },
    }
}

fn attachment_is_excluded(
    parent_index: usize,
    attachment: SpawnAttachment,
    probability: f32,
) -> bool {
    let probability = probability.clamp(0.0, 1.0);
    if probability <= 0.0 {
        return false;
    }
    if probability >= 1.0 {
        return true;
    }

    attachment_exclusion_sample(parent_index, attachment) < probability
}

fn attachment_exclusion_sample(parent_index: usize, attachment: SpawnAttachment) -> f32 {
    let mode_bits = match attachment.mode {
        SpawnPlacementMode::Vertex => 0xD6E8_FEB8_6659_FD93,
        SpawnPlacementMode::Edge => 0xA076_1D64_78BD_642F,
        SpawnPlacementMode::Face => 0xE703_7ED1_A0B4_28DB,
    };
    let mut state = (parent_index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (attachment.index as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ mode_bits;
    state ^= state >> 33;
    state = state.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    state ^= state >> 33;
    state = state.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    state ^= state >> 33;

    (state as f64 / u64::MAX as f64) as f32
}

fn child_transform(
    parent: &ShapeNode,
    parent_geometry: &ShapeGeometry,
    attachment: SpawnAttachment,
    child_radius: f32,
    twist_per_vertex_radians: f32,
    vertex_offset_ratio: f32,
) -> (Vec3, Quat) {
    let outward_local = parent_geometry.attachment_direction(attachment.mode, attachment.index);
    let outward = parent.rotation * outward_local;
    let world_anchor = parent.center + outward * parent.radius;

    let twist_step = if twist_per_vertex_radians.is_finite() {
        twist_per_vertex_radians
    } else {
        PI / 5.0
    };
    let twist = Quat::from_axis_angle(outward, attachment.index as f32 * twist_step);
    let vertex_offset = child_radius * vertex_offset_ratio.max(0.0);

    (
        world_anchor + outward * vertex_offset,
        twist * parent.rotation,
    )
}

fn is_fully_contained(
    center: Vec3,
    radius: f32,
    nodes: &[ShapeNode],
    containment_epsilon: f32,
) -> bool {
    nodes.iter().any(|node| {
        let distance = center.distance(node.center);
        distance + radius <= node.radius - containment_epsilon
    })
}

fn contains_existing(
    center: Vec3,
    radius: f32,
    nodes: &[ShapeNode],
    containment_epsilon: f32,
) -> bool {
    nodes.iter().any(|node| {
        let distance = center.distance(node.center);
        distance + node.radius <= radius - containment_epsilon
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tuning() -> SpawnTuning {
        SpawnTuning {
            min_scale_ratio: 0.15,
            max_scale_ratio: 1.0,
            containment_epsilon: 0.02,
            twist_per_vertex_radians: PI / 5.0,
            vertex_offset_ratio: 0.0,
            vertex_spawn_exclusion_probability: 0.0,
            spawn_placement_mode: SpawnPlacementMode::Vertex,
        }
    }

    #[test]
    fn root_level_is_exhausted_before_level_one_is_used() {
        let shapes = ShapeCatalog::new();
        let mut nodes = vec![root_node(ShapeKind::Cube, 1.4, &shapes)];

        let mut parent_levels = Vec::new();
        for _ in 0..9 {
            let spawn = next_spawn(&mut nodes, &shapes, ShapeKind::Cube, 0.35, test_tuning())
                .expect("spawn should succeed");
            parent_levels.push(spawn.parent_level);
        }

        assert!(parent_levels[..8].iter().all(|level| *level == 0));
        assert_eq!(parent_levels[8], 1);
    }

    #[test]
    fn fill_level_mode_spawns_remaining_nodes_only_on_the_current_level() {
        let shapes = ShapeCatalog::new();
        let mut nodes = vec![root_node(ShapeKind::Cube, 1.4, &shapes)];

        next_spawn(&mut nodes, &shapes, ShapeKind::Cube, 0.35, test_tuning())
            .expect("initial spawn should succeed");
        let spawned = spawn_batch(
            &mut nodes,
            &shapes,
            ShapeKind::Cube,
            0.35,
            test_tuning(),
            SpawnAddMode::FillLevel,
        );

        assert_eq!(spawned.len(), 7);
        assert!(spawned.iter().all(|spawn| spawn.node.level == 1));
        assert_eq!(nodes.iter().map(|node| node.level).max(), Some(1));
    }

    #[test]
    fn fill_level_mode_stops_before_opening_the_next_level() {
        let shapes = ShapeCatalog::new();
        let mut nodes = vec![root_node(ShapeKind::Cube, 1.4, &shapes)];

        let first_batch = spawn_batch(
            &mut nodes,
            &shapes,
            ShapeKind::Cube,
            0.35,
            test_tuning(),
            SpawnAddMode::FillLevel,
        );
        let second_batch = spawn_batch(
            &mut nodes,
            &shapes,
            ShapeKind::Cube,
            0.35,
            test_tuning(),
            SpawnAddMode::FillLevel,
        );

        assert_eq!(first_batch.len(), 8);
        assert!(first_batch.iter().all(|spawn| spawn.node.level == 1));
        assert!(!second_batch.is_empty());
        assert!(second_batch.iter().all(|spawn| spawn.node.level == 2));
        assert_eq!(nodes.iter().map(|node| node.level).max(), Some(2));
    }

    #[test]
    fn full_containment_is_rejected() {
        let nodes = vec![ShapeNode {
            kind: ShapeKind::Cube,
            level: 0,
            center: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: 1.0,
            radius: 5.0,
            occupied_attachments: AttachmentOccupancy::default(),
            origin: NodeOrigin::Root,
        }];

        assert!(is_fully_contained(
            Vec3::new(0.5, 0.0, 0.0),
            1.0,
            &nodes,
            test_tuning().containment_epsilon,
        ));
        assert!(!is_fully_contained(
            Vec3::new(5.0, 0.0, 0.0),
            1.0,
            &nodes,
            test_tuning().containment_epsilon,
        ));
    }

    #[test]
    fn spawned_child_center_matches_parent_vertex_position() {
        let shapes = ShapeCatalog::new();
        let parent_rotation = Quat::from_euler(EulerRot::YXZ, 0.45, -0.3, 0.2);
        let parent_center = Vec3::new(1.5, -0.75, 2.25);
        let parent_scale = 1.4;
        let parent_kind = ShapeKind::Cube;
        let parent_geometry = shapes.geometry(parent_kind);
        let mut nodes = vec![ShapeNode {
            kind: parent_kind,
            level: 0,
            center: parent_center,
            rotation: parent_rotation,
            scale: parent_scale,
            radius: parent_geometry.radius * parent_scale,
            occupied_attachments: AttachmentOccupancy::new(parent_geometry),
            origin: NodeOrigin::Root,
        }];

        let spawn = next_spawn(
            &mut nodes,
            &shapes,
            ShapeKind::Tetrahedron,
            0.35,
            test_tuning(),
        )
        .expect("spawn should succeed");
        let expected_center = parent_center
            + parent_rotation
                * (parent_geometry.attachment_direction(SpawnPlacementMode::Vertex, 0)
                    * parent_geometry.radius
                    * parent_scale);

        assert!(spawn.node.center.distance(expected_center) <= 1.0e-5);
    }

    #[test]
    fn spawned_child_center_matches_parent_edge_direction() {
        let shapes = ShapeCatalog::new();
        let parent_geometry = shapes.geometry(ShapeKind::Cube);
        let mut nodes = vec![ShapeNode {
            kind: ShapeKind::Cube,
            level: 0,
            center: Vec3::new(1.5, -0.75, 2.25),
            rotation: Quat::IDENTITY,
            scale: 1.4,
            radius: parent_geometry.radius * 1.4,
            occupied_attachments: AttachmentOccupancy::new(parent_geometry),
            origin: NodeOrigin::Root,
        }];

        let spawn = next_spawn(
            &mut nodes,
            &shapes,
            ShapeKind::Tetrahedron,
            0.35,
            SpawnTuning {
                spawn_placement_mode: SpawnPlacementMode::Edge,
                ..test_tuning()
            },
        )
        .expect("spawn should succeed");
        let expected_center = nodes[0].center
            + parent_geometry.attachment_direction(SpawnPlacementMode::Edge, 0) * nodes[0].radius;

        assert!(spawn.node.center.distance(expected_center) <= 1.0e-5);
    }

    #[test]
    fn spawned_child_center_matches_parent_face_direction() {
        let shapes = ShapeCatalog::new();
        let parent_geometry = shapes.geometry(ShapeKind::Cube);
        let mut nodes = vec![ShapeNode {
            kind: ShapeKind::Cube,
            level: 0,
            center: Vec3::new(1.5, -0.75, 2.25),
            rotation: Quat::IDENTITY,
            scale: 1.4,
            radius: parent_geometry.radius * 1.4,
            occupied_attachments: AttachmentOccupancy::new(parent_geometry),
            origin: NodeOrigin::Root,
        }];

        let spawn = next_spawn(
            &mut nodes,
            &shapes,
            ShapeKind::Tetrahedron,
            0.35,
            SpawnTuning {
                spawn_placement_mode: SpawnPlacementMode::Face,
                ..test_tuning()
            },
        )
        .expect("spawn should succeed");
        let expected_center = nodes[0].center
            + parent_geometry.attachment_direction(SpawnPlacementMode::Face, 0) * nodes[0].radius;

        assert!(spawn.node.center.distance(expected_center) <= 1.0e-5);
    }

    #[test]
    fn spawned_child_offset_scales_with_child_radius() {
        let shapes = ShapeCatalog::new();
        let parent_kind = ShapeKind::Cube;
        let parent_scale = 1.4;
        let parent_geometry = shapes.geometry(parent_kind);
        let child_kind = ShapeKind::Tetrahedron;
        let child_geometry = shapes.geometry(child_kind);
        let scale_ratio = 0.35;
        let vertex_offset_ratio = 0.5;
        let mut nodes = vec![ShapeNode {
            kind: parent_kind,
            level: 0,
            center: Vec3::new(1.5, -0.75, 2.25),
            rotation: Quat::IDENTITY,
            scale: parent_scale,
            radius: parent_geometry.radius * parent_scale,
            occupied_attachments: AttachmentOccupancy::new(parent_geometry),
            origin: NodeOrigin::Root,
        }];

        let spawn = next_spawn(
            &mut nodes,
            &shapes,
            child_kind,
            scale_ratio,
            SpawnTuning {
                vertex_offset_ratio,
                ..test_tuning()
            },
        )
        .expect("spawn should succeed");
        let outward = parent_geometry.attachment_direction(SpawnPlacementMode::Vertex, 0);
        let world_anchor = nodes[0].center + outward * nodes[0].radius;
        let child_radius = child_geometry.radius * parent_scale * scale_ratio;
        let expected_center = world_anchor + outward * child_radius * vertex_offset_ratio;

        assert!(spawn.node.center.distance(expected_center) <= 1.0e-5);
    }

    #[test]
    fn full_vertex_exclusion_probability_blocks_spawns() {
        let shapes = ShapeCatalog::new();
        let mut nodes = vec![root_node(ShapeKind::Cube, 1.4, &shapes)];

        let spawn = next_spawn(
            &mut nodes,
            &shapes,
            ShapeKind::Cube,
            0.35,
            SpawnTuning {
                vertex_spawn_exclusion_probability: 1.0,
                ..test_tuning()
            },
        );

        assert!(spawn.is_none());
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn switching_spawn_modes_uses_separate_attachment_occupancy() {
        let shapes = ShapeCatalog::new();
        let mut root = root_node(ShapeKind::Cube, 1.4, &shapes);
        root.occupied_attachments.vertices.fill(true);
        let mut nodes = vec![root];

        let spawn = next_spawn(
            &mut nodes,
            &shapes,
            ShapeKind::Cube,
            0.35,
            SpawnTuning {
                spawn_placement_mode: SpawnPlacementMode::Edge,
                ..test_tuning()
            },
        );

        assert!(spawn.is_some());
    }

    #[test]
    fn add_modes_cycle_between_single_and_fill_level() {
        assert_eq!(SpawnAddMode::Single.next(), SpawnAddMode::FillLevel);
        assert_eq!(SpawnAddMode::FillLevel.next(), SpawnAddMode::Single);
    }

    #[test]
    fn zero_twist_keeps_child_orientation_aligned_with_parent() {
        let shapes = ShapeCatalog::new();
        let parent_rotation = Quat::from_euler(EulerRot::YXZ, 0.45, -0.3, 0.2);
        let parent_kind = ShapeKind::Cube;
        let parent_geometry = shapes.geometry(parent_kind);
        let mut nodes = vec![ShapeNode {
            kind: parent_kind,
            level: 0,
            center: Vec3::new(1.5, -0.75, 2.25),
            rotation: parent_rotation,
            scale: 1.4,
            radius: parent_geometry.radius * 1.4,
            occupied_attachments: AttachmentOccupancy::new(parent_geometry),
            origin: NodeOrigin::Root,
        }];

        let spawn = next_spawn(
            &mut nodes,
            &shapes,
            ShapeKind::Tetrahedron,
            0.35,
            SpawnTuning {
                twist_per_vertex_radians: 0.0,
                ..test_tuning()
            },
        )
        .expect("spawn should succeed");

        assert!(spawn.node.rotation.angle_between(parent_rotation) <= 1.0e-5);
    }

    #[test]
    fn recompute_spawn_tree_updates_existing_children_for_new_twist() {
        let shapes = ShapeCatalog::new();
        let mut nodes = vec![root_node(ShapeKind::Cube, 1.4, &shapes)];
        for _ in 0..2 {
            next_spawn(&mut nodes, &shapes, ShapeKind::Cube, 0.35, test_tuning())
                .expect("spawn should succeed");
        }

        let child_before = nodes[2].rotation * Vec3::X;
        recompute_spawn_tree(&mut nodes, &shapes, 0.0, 0.0);
        let child_after = nodes[2].rotation * Vec3::X;

        assert!(child_after.distance(child_before) > 1.0e-4);
    }

    #[test]
    fn recompute_spawn_tree_updates_existing_children_for_new_vertex_offset() {
        let shapes = ShapeCatalog::new();
        let mut nodes = vec![root_node(ShapeKind::Cube, 1.4, &shapes)];
        next_spawn(&mut nodes, &shapes, ShapeKind::Cube, 0.35, test_tuning())
            .expect("spawn should succeed");

        let center_before = nodes[1].center;
        recompute_spawn_tree(
            &mut nodes,
            &shapes,
            test_tuning().twist_per_vertex_radians,
            0.5,
        );
        let center_after = nodes[1].center;

        assert!(center_after.distance(center_before) > 1.0e-4);
    }
}
