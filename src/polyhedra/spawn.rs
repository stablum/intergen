use std::f32::consts::PI;

use bevy::prelude::*;
use serde::Deserialize;

use super::shapes::{ShapeCatalog, ShapeGeometry};

#[derive(Clone, Copy, Debug)]
pub(crate) struct SpawnTuning {
    pub(crate) min_scale_ratio: f32,
    pub(crate) max_scale_ratio: f32,
    pub(crate) containment_epsilon: f32,
    pub(crate) twist_per_vertex_radians: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PolyhedronKind {
    Cube,
    Tetrahedron,
    Octahedron,
    Dodecahedron,
}

#[derive(Clone, Debug)]
pub(crate) struct PolyhedronNode {
    pub(crate) kind: PolyhedronKind,
    pub(crate) level: usize,
    pub(crate) center: Vec3,
    pub(crate) rotation: Quat,
    pub(crate) scale: f32,
    pub(crate) radius: f32,
    pub(crate) occupied_vertices: Vec<bool>,
}

#[derive(Clone, Debug)]
pub(crate) struct SpawnedNode {
    pub(crate) kind: PolyhedronKind,
    pub(crate) parent_level: usize,
    pub(crate) node: PolyhedronNode,
}

pub(crate) fn root_node(kind: PolyhedronKind, scale: f32, shapes: &ShapeCatalog) -> PolyhedronNode {
    let geometry = shapes.geometry(kind);
    PolyhedronNode {
        kind,
        level: 0,
        center: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale,
        radius: geometry.radius * scale,
        occupied_vertices: vec![false; geometry.vertices.len()],
    }
}

pub(crate) fn next_spawn(
    nodes: &mut Vec<PolyhedronNode>,
    shapes: &ShapeCatalog,
    child_kind: PolyhedronKind,
    scale_ratio: f32,
    tuning: SpawnTuning,
) -> Option<SpawnedNode> {
    let scale_ratio = scale_ratio.clamp(tuning.min_scale_ratio, tuning.max_scale_ratio);
    let highest_level = nodes.iter().map(|node| node.level).max().unwrap_or(0);

    for level in 0..=highest_level {
        let parent_indices: Vec<usize> = nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| (node.level == level).then_some(index))
            .collect();

        for parent_index in parent_indices {
            let parent = nodes[parent_index].clone();
            let parent_geometry = shapes.geometry(parent.kind);
            let child_geometry = shapes.geometry(child_kind);

            for vertex_index in 0..parent_geometry.vertices.len() {
                if parent.occupied_vertices[vertex_index] {
                    continue;
                }

                let candidate = spawn_candidate(
                    &parent,
                    parent_geometry,
                    child_kind,
                    child_geometry,
                    vertex_index,
                    scale_ratio,
                    tuning.twist_per_vertex_radians,
                );

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

                nodes[parent_index].occupied_vertices[vertex_index] = true;
                nodes.push(candidate.clone());

                return Some(SpawnedNode {
                    kind: child_kind,
                    parent_level: parent.level,
                    node: candidate,
                });
            }
        }
    }

    None
}

fn spawn_candidate(
    parent: &PolyhedronNode,
    parent_geometry: &ShapeGeometry,
    child_kind: PolyhedronKind,
    child_geometry: &ShapeGeometry,
    vertex_index: usize,
    scale_ratio: f32,
    twist_per_vertex_radians: f32,
) -> PolyhedronNode {
    let local_vertex = parent_geometry.vertices[vertex_index] * parent.scale;
    let world_vertex = parent.center + parent.rotation * local_vertex;
    let direction = parent.rotation * parent_geometry.vertices[vertex_index];
    let outward = if direction.length_squared() > 0.0 {
        direction.normalize()
    } else {
        Vec3::Y
    };

    let scale = parent.scale * scale_ratio;
    let radius = child_geometry.radius * scale;
    let center = world_vertex;

    let align = Quat::from_rotation_arc(Vec3::Y, outward);
    let twist_step = if twist_per_vertex_radians.is_finite() {
        twist_per_vertex_radians
    } else {
        PI / 5.0
    };
    let twist = Quat::from_axis_angle(outward, vertex_index as f32 * twist_step);

    PolyhedronNode {
        kind: child_kind,
        level: parent.level + 1,
        center,
        rotation: twist * align,
        scale,
        radius,
        occupied_vertices: vec![false; child_geometry.vertices.len()],
    }
}

fn is_fully_contained(
    center: Vec3,
    radius: f32,
    nodes: &[PolyhedronNode],
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
    nodes: &[PolyhedronNode],
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
        }
    }

    #[test]
    fn root_level_is_exhausted_before_level_one_is_used() {
        let shapes = ShapeCatalog::new();
        let mut nodes = vec![root_node(PolyhedronKind::Cube, 1.4, &shapes)];

        let mut parent_levels = Vec::new();
        for _ in 0..9 {
            let spawn = next_spawn(
                &mut nodes,
                &shapes,
                PolyhedronKind::Cube,
                0.35,
                test_tuning(),
            )
            .expect("spawn should succeed");
            parent_levels.push(spawn.parent_level);
        }

        assert!(parent_levels[..8].iter().all(|level| *level == 0));
        assert_eq!(parent_levels[8], 1);
    }

    #[test]
    fn full_containment_is_rejected() {
        let nodes = vec![PolyhedronNode {
            kind: PolyhedronKind::Cube,
            level: 0,
            center: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: 1.0,
            radius: 5.0,
            occupied_vertices: vec![false; 8],
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
        let parent_kind = PolyhedronKind::Cube;
        let parent_geometry = shapes.geometry(parent_kind);
        let mut nodes = vec![PolyhedronNode {
            kind: parent_kind,
            level: 0,
            center: parent_center,
            rotation: parent_rotation,
            scale: parent_scale,
            radius: parent_geometry.radius * parent_scale,
            occupied_vertices: vec![false; parent_geometry.vertices.len()],
        }];

        let spawn = next_spawn(
            &mut nodes,
            &shapes,
            PolyhedronKind::Tetrahedron,
            0.35,
            test_tuning(),
        )
        .expect("spawn should succeed");
        let expected_center =
            parent_center + parent_rotation * (parent_geometry.vertices[0] * parent_scale);

        assert!(spawn.node.center.distance(expected_center) <= 1.0e-5);
    }
}
