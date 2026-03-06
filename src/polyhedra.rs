use std::f32::consts::PI;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub const MIN_SCALE_RATIO: f32 = 0.15;
pub const MAX_SCALE_RATIO: f32 = 1.0;

const SPAWN_GAP_FACTOR: f32 = 0.12;
const CONTAINMENT_EPSILON: f32 = 0.02;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PolyhedronKind {
    Cube,
    Tetrahedron,
    Octahedron,
    Dodecahedron,
}

impl PolyhedronKind {
    pub fn hue_bias(self) -> f32 {
        match self {
            Self::Cube => 35.0,
            Self::Tetrahedron => 110.0,
            Self::Octahedron => 205.0,
            Self::Dodecahedron => 290.0,
        }
    }
}

#[derive(Clone)]
pub struct ShapeGeometry {
    pub vertices: Vec<Vec3>,
    pub faces: Vec<Vec<usize>>,
    pub radius: f32,
}

#[derive(Clone)]
pub struct ShapeCatalog {
    cube: ShapeGeometry,
    tetrahedron: ShapeGeometry,
    octahedron: ShapeGeometry,
    dodecahedron: ShapeGeometry,
}

impl ShapeCatalog {
    pub fn new() -> Self {
        Self {
            cube: static_geometry(&cube_vertices(), cube_faces()),
            tetrahedron: static_geometry(&tetra_vertices(), tetra_faces()),
            octahedron: static_geometry(&octa_vertices(), octa_faces()),
            dodecahedron: dodecahedron_geometry(),
        }
    }

    pub fn geometry(&self, kind: PolyhedronKind) -> &ShapeGeometry {
        match kind {
            PolyhedronKind::Cube => &self.cube,
            PolyhedronKind::Tetrahedron => &self.tetrahedron,
            PolyhedronKind::Octahedron => &self.octahedron,
            PolyhedronKind::Dodecahedron => &self.dodecahedron,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PolyhedronNode {
    pub kind: PolyhedronKind,
    pub level: usize,
    pub center: Vec3,
    pub rotation: Quat,
    pub scale: f32,
    pub radius: f32,
    pub occupied_vertices: Vec<bool>,
}

#[derive(Clone, Debug)]
pub struct SpawnedNode {
    pub kind: PolyhedronKind,
    pub parent_level: usize,
    pub node: PolyhedronNode,
}

pub fn root_node(kind: PolyhedronKind, scale: f32, shapes: &ShapeCatalog) -> PolyhedronNode {
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

pub fn next_spawn(
    nodes: &mut Vec<PolyhedronNode>,
    shapes: &ShapeCatalog,
    child_kind: PolyhedronKind,
    scale_ratio: f32,
) -> Option<SpawnedNode> {
    let scale_ratio = scale_ratio.clamp(MIN_SCALE_RATIO, MAX_SCALE_RATIO);
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
                );

                if is_fully_contained(candidate.center, candidate.radius, nodes)
                    || contains_existing(candidate.center, candidate.radius, nodes)
                {
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

pub fn build_mesh(geometry: &ShapeGeometry) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    for face in &geometry.faces {
        let vertices = outward_face_vertices(geometry, face);
        let normal = face_normal(&vertices);
        let base = positions.len() as u32;
        for vertex in &vertices {
            positions.push([vertex.x, vertex.y, vertex.z]);
            normals.push([normal.x, normal.y, normal.z]);
        }
        for triangle_index in 1..vertices.len() - 1 {
            indices.extend_from_slice(&[
                base,
                base + triangle_index as u32,
                base + triangle_index as u32 + 1,
            ]);
        }
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
fn outward_face_vertices(geometry: &ShapeGeometry, face: &[usize]) -> Vec<Vec3> {
    let mut vertices: Vec<Vec3> = face.iter().map(|index| geometry.vertices[*index]).collect();
    if face_normal(&vertices).dot(centroid(&vertices)) < 0.0 {
        vertices.reverse();
    }
    vertices
}

fn spawn_candidate(
    parent: &PolyhedronNode,
    parent_geometry: &ShapeGeometry,
    child_kind: PolyhedronKind,
    child_geometry: &ShapeGeometry,
    vertex_index: usize,
    scale_ratio: f32,
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
    let separation = scale.max(0.2) * SPAWN_GAP_FACTOR;
    let center = world_vertex + outward * (radius + separation);

    let align = Quat::from_rotation_arc(Vec3::Y, outward);
    let twist = Quat::from_axis_angle(outward, vertex_index as f32 * PI / 5.0);

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

fn is_fully_contained(center: Vec3, radius: f32, nodes: &[PolyhedronNode]) -> bool {
    nodes.iter().any(|node| {
        let distance = center.distance(node.center);
        distance + radius <= node.radius - CONTAINMENT_EPSILON
    })
}

fn contains_existing(center: Vec3, radius: f32, nodes: &[PolyhedronNode]) -> bool {
    nodes.iter().any(|node| {
        let distance = center.distance(node.center);
        distance + node.radius <= radius - CONTAINMENT_EPSILON
    })
}

fn static_geometry(vertices: &[[f32; 3]], faces: &[&[usize]]) -> ShapeGeometry {
    let vertices: Vec<Vec3> = vertices
        .iter()
        .map(|vertex| Vec3::from_array(*vertex))
        .collect();
    let faces: Vec<Vec<usize>> = faces.iter().map(|face| face.to_vec()).collect();
    let radius = vertices
        .iter()
        .map(|vertex| vertex.length())
        .fold(0.0, f32::max);

    ShapeGeometry {
        vertices,
        faces,
        radius,
    }
}

fn cube_vertices() -> [[f32; 3]; 8] {
    [
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [-1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
    ]
}

fn cube_faces() -> &'static [&'static [usize]] {
    &[
        &[0, 1, 3, 2],
        &[4, 6, 7, 5],
        &[0, 4, 5, 1],
        &[2, 3, 7, 6],
        &[0, 2, 6, 4],
        &[1, 5, 7, 3],
    ]
}

fn tetra_vertices() -> [[f32; 3]; 4] {
    [
        [1.0, 1.0, 1.0],
        [-1.0, -1.0, 1.0],
        [-1.0, 1.0, -1.0],
        [1.0, -1.0, -1.0],
    ]
}

fn tetra_faces() -> &'static [&'static [usize]] {
    &[&[0, 2, 1], &[0, 1, 3], &[0, 3, 2], &[1, 2, 3]]
}

fn octa_vertices() -> [[f32; 3]; 6] {
    [
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
    ]
}

fn octa_faces() -> &'static [&'static [usize]] {
    &[
        &[0, 2, 4],
        &[4, 2, 1],
        &[1, 2, 5],
        &[5, 2, 0],
        &[4, 3, 0],
        &[1, 3, 4],
        &[5, 3, 1],
        &[0, 3, 5],
    ]
}

fn dodecahedron_geometry() -> ShapeGeometry {
    let icosa_vertices = icosahedron_vertices();
    let icosa_faces = icosahedron_faces();

    let vertices = icosa_faces
        .iter()
        .map(|face| {
            let [a, b, c] = *face;
            (icosa_vertices[a] + icosa_vertices[b] + icosa_vertices[c]) / 3.0
        })
        .collect::<Vec<_>>();

    let mut faces = Vec::new();
    for (vertex_index, vertex) in icosa_vertices.iter().enumerate() {
        let mut incident_faces: Vec<usize> = icosa_faces
            .iter()
            .enumerate()
            .filter_map(|(face_index, face)| face.contains(&vertex_index).then_some(face_index))
            .collect();

        sort_face_indices(vertex.normalize(), &vertices, &mut incident_faces);
        let face_vertices = incident_faces
            .iter()
            .map(|index| vertices[*index])
            .collect::<Vec<_>>();
        let normal = face_normal(&face_vertices);
        let face_center = centroid(&face_vertices);

        if normal.dot(face_center) < 0.0 {
            incident_faces.reverse();
        }

        faces.push(incident_faces);
    }

    let radius = vertices
        .iter()
        .map(|vertex| vertex.length())
        .fold(0.0, f32::max);

    ShapeGeometry {
        vertices,
        faces,
        radius,
    }
}

fn sort_face_indices(normal: Vec3, vertices: &[Vec3], face_indices: &mut [usize]) {
    let reference = if normal.y.abs() < 0.95 {
        Vec3::Y
    } else {
        Vec3::X
    };
    let tangent = normal.cross(reference).normalize();
    let bitangent = normal.cross(tangent).normalize();

    face_indices.sort_by(|left, right| {
        let left_vertex = vertices[*left];
        let left_projected = left_vertex - normal * left_vertex.dot(normal);
        let right_vertex = vertices[*right];
        let right_projected = right_vertex - normal * right_vertex.dot(normal);

        let left_angle = left_projected
            .dot(bitangent)
            .atan2(left_projected.dot(tangent));
        let right_angle = right_projected
            .dot(bitangent)
            .atan2(right_projected.dot(tangent));
        left_angle.total_cmp(&right_angle)
    });
}

fn centroid(vertices: &[Vec3]) -> Vec3 {
    vertices.iter().copied().sum::<Vec3>() / vertices.len() as f32
}

fn face_normal(vertices: &[Vec3]) -> Vec3 {
    let mut normal = Vec3::ZERO;
    for triangle in 1..vertices.len() - 1 {
        normal += (vertices[triangle] - vertices[0]).cross(vertices[triangle + 1] - vertices[0]);
    }
    normal.normalize()
}

fn icosahedron_vertices() -> [Vec3; 12] {
    let phi = (1.0 + 5.0_f32.sqrt()) * 0.5;
    [
        Vec3::new(-1.0, phi, 0.0),
        Vec3::new(1.0, phi, 0.0),
        Vec3::new(-1.0, -phi, 0.0),
        Vec3::new(1.0, -phi, 0.0),
        Vec3::new(0.0, -1.0, phi),
        Vec3::new(0.0, 1.0, phi),
        Vec3::new(0.0, -1.0, -phi),
        Vec3::new(0.0, 1.0, -phi),
        Vec3::new(phi, 0.0, -1.0),
        Vec3::new(phi, 0.0, 1.0),
        Vec3::new(-phi, 0.0, -1.0),
        Vec3::new(-phi, 0.0, 1.0),
    ]
}

fn icosahedron_faces() -> [[usize; 3]; 20] {
    [
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dodecahedron_has_expected_counts() {
        let dodecahedron = dodecahedron_geometry();
        assert_eq!(dodecahedron.vertices.len(), 20);
        assert_eq!(dodecahedron.faces.len(), 12);
        assert!(dodecahedron.faces.iter().all(|face| face.len() == 5));
    }

    #[test]
    fn root_level_is_exhausted_before_level_one_is_used() {
        let shapes = ShapeCatalog::new();
        let mut nodes = vec![root_node(PolyhedronKind::Cube, 1.4, &shapes)];

        let mut parent_levels = Vec::new();
        for _ in 0..9 {
            let spawn = next_spawn(&mut nodes, &shapes, PolyhedronKind::Cube, 0.35)
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

        assert!(is_fully_contained(Vec3::new(0.5, 0.0, 0.0), 1.0, &nodes));
        assert!(!is_fully_contained(Vec3::new(5.0, 0.0, 0.0), 1.0, &nodes));
    }
}
