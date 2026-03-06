mod mesh;
mod shapes;
mod spawn;

pub(crate) use mesh::build_mesh;
pub(crate) use shapes::{ShapeCatalog, ShapeGeometry};
pub(crate) use spawn::{PolyhedronKind, PolyhedronNode, SpawnTuning, next_spawn, root_node};
