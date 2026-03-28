use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::catalog::ShapeGeometry;

pub(crate) fn build_mesh(geometry: &ShapeGeometry) -> Mesh {
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

pub(super) fn centroid(vertices: &[Vec3]) -> Vec3 {
    vertices.iter().copied().sum::<Vec3>() / vertices.len() as f32
}

pub(super) fn face_normal(vertices: &[Vec3]) -> Vec3 {
    let mut normal = Vec3::ZERO;
    for triangle in 1..vertices.len() - 1 {
        normal += (vertices[triangle] - vertices[0]).cross(vertices[triangle + 1] - vertices[0]);
    }
    normal.normalize()
}
