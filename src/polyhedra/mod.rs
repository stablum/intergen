mod mesh;
mod shapes;
mod spawn;

pub(crate) use mesh::build_mesh;
pub(crate) use shapes::{ShapeCatalog, ShapeGeometry};
#[allow(unused_imports)]
pub(crate) use spawn::NodeOrigin;
pub(crate) use spawn::{
    AttachmentOccupancy, PolyhedronKind, PolyhedronNode, SpawnAddMode, SpawnAttachment,
    SpawnPlacementMode, SpawnTuning, SpawnedNode, recompute_spawn_tree, root_node, spawn_batch,
};
