mod mesh;
mod shapes;
mod spawn;

pub(crate) use mesh::build_mesh;
pub(crate) use shapes::{ShapeCatalog, ShapeGeometry};
pub(crate) use spawn::{
    MAX_SCALE_RATIO, MIN_SCALE_RATIO, PolyhedronKind, PolyhedronNode, next_spawn, root_node,
};
