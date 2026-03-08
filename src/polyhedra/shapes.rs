use std::collections::BTreeSet;

use bevy::prelude::*;

use super::mesh::{centroid, face_normal};
use super::spawn::{PolyhedronKind, SpawnPlacementMode};

#[derive(Clone)]
pub(crate) struct ShapeGeometry {
    pub(crate) vertices: Vec<Vec3>,
    pub(crate) edges: Vec<[usize; 2]>,
    pub(crate) faces: Vec<Vec<usize>>,
    pub(crate) radius: f32,
}

#[derive(Clone)]
pub(crate) struct ShapeCatalog {
    cube: ShapeGeometry,
    tetrahedron: ShapeGeometry,
    octahedron: ShapeGeometry,
    dodecahedron: ShapeGeometry,
}

impl ShapeCatalog {
    pub(crate) fn new() -> Self {
        Self {
            cube: static_geometry(&cube_vertices(), cube_faces()),
            tetrahedron: static_geometry(&tetra_vertices(), tetra_faces()),
            octahedron: static_geometry(&octa_vertices(), octa_faces()),
            dodecahedron: dodecahedron_geometry(),
        }
    }

    pub(crate) fn geometry(&self, kind: PolyhedronKind) -> &ShapeGeometry {
        match kind {
            PolyhedronKind::Cube => &self.cube,
            PolyhedronKind::Tetrahedron => &self.tetrahedron,
            PolyhedronKind::Octahedron => &self.octahedron,
            PolyhedronKind::Dodecahedron => &self.dodecahedron,
        }
    }
}

impl ShapeGeometry {
    pub(crate) fn attachment_count(&self, mode: SpawnPlacementMode) -> usize {
        match mode {
            SpawnPlacementMode::Vertex => self.vertices.len(),
            SpawnPlacementMode::Edge => self.edges.len(),
            SpawnPlacementMode::Face => self.faces.len(),
        }
    }

    pub(crate) fn attachment_anchor(&self, mode: SpawnPlacementMode, index: usize) -> Vec3 {
        match mode {
            SpawnPlacementMode::Vertex => self.vertices[index],
            SpawnPlacementMode::Edge => {
                let [left, right] = self.edges[index];
                (self.vertices[left] + self.vertices[right]) * 0.5
            }
            SpawnPlacementMode::Face => {
                let face_vertices: Vec<Vec3> = self.faces[index]
                    .iter()
                    .map(|face_index| self.vertices[*face_index])
                    .collect();
                centroid(&face_vertices)
            }
        }
    }

    pub(crate) fn attachment_direction(&self, mode: SpawnPlacementMode, index: usize) -> Vec3 {
        let anchor = self.attachment_anchor(mode, index);
        if anchor.length_squared() > 0.0 {
            anchor.normalize()
        } else {
            Vec3::Y
        }
    }
}

fn static_geometry(vertices: &[[f32; 3]], faces: &[&[usize]]) -> ShapeGeometry {
    let vertices: Vec<Vec3> = vertices
        .iter()
        .map(|vertex| Vec3::from_array(*vertex))
        .collect();
    let faces: Vec<Vec<usize>> = faces.iter().map(|face| face.to_vec()).collect();
    let edges = collect_edges(&faces);
    let radius = vertices
        .iter()
        .map(|vertex| vertex.length())
        .fold(0.0, f32::max);

    ShapeGeometry {
        vertices,
        edges,
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
    let edges = collect_edges(&faces);

    ShapeGeometry {
        vertices,
        edges,
        faces,
        radius,
    }
}

fn collect_edges(faces: &[Vec<usize>]) -> Vec<[usize; 2]> {
    let mut edges = BTreeSet::new();

    for face in faces {
        for edge_index in 0..face.len() {
            let left = face[edge_index];
            let right = face[(edge_index + 1) % face.len()];
            let edge = if left < right {
                (left, right)
            } else {
                (right, left)
            };
            edges.insert(edge);
        }
    }

    edges
        .into_iter()
        .map(|(left, right)| [left, right])
        .collect()
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
    use super::dodecahedron_geometry;

    #[test]
    fn dodecahedron_has_expected_counts() {
        let dodecahedron = dodecahedron_geometry();
        assert_eq!(dodecahedron.vertices.len(), 20);
        assert_eq!(dodecahedron.edges.len(), 30);
        assert_eq!(dodecahedron.faces.len(), 12);
        assert!(dodecahedron.faces.iter().all(|face| face.len() == 5));
    }
}
