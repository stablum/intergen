#[path = "shapes.rs"]
mod catalog;
mod mesh;
mod spawn;

pub(crate) use catalog::{ShapeCatalog, ShapeGeometry};
pub(crate) use mesh::build_mesh;
#[allow(unused_imports)]
pub(crate) use spawn::NodeOrigin;
pub(crate) use spawn::{
    AttachmentOccupancy, ShapeKind, ShapeNode, SpawnAddMode, SpawnAttachment, SpawnPlacementMode,
    SpawnTuning, SpawnedShape, recompute_spawn_tree, root_node_with_axis_scale, spawn_batch,
    spawn_batch_with_inputs,
};
