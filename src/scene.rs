use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;

use crate::camera::{CameraRig, SceneCamera};
use crate::generation::SpawnHoldState;
use crate::polyhedra::{
    PolyhedronKind, PolyhedronNode, ShapeCatalog, ShapeGeometry, build_mesh, root_node,
};
use crate::ui::{UiFontSource, load_ui_theme, spawn_help_ui};

const ROOT_KIND: PolyhedronKind = PolyhedronKind::Cube;
const ROOT_SCALE: f32 = 1.9;
const DEFAULT_SCALE_RATIO: f32 = 0.58;

#[derive(Resource)]
pub(crate) struct ShapeAssets {
    pub(crate) catalog: ShapeCatalog,
    cube: ShapeRuntime,
    tetrahedron: ShapeRuntime,
    octahedron: ShapeRuntime,
    dodecahedron: ShapeRuntime,
}

impl ShapeAssets {
    pub(crate) fn new(meshes: &mut Assets<Mesh>) -> Self {
        let catalog = ShapeCatalog::new();

        Self {
            cube: ShapeRuntime::new(catalog.geometry(PolyhedronKind::Cube), meshes),
            tetrahedron: ShapeRuntime::new(catalog.geometry(PolyhedronKind::Tetrahedron), meshes),
            octahedron: ShapeRuntime::new(catalog.geometry(PolyhedronKind::Octahedron), meshes),
            dodecahedron: ShapeRuntime::new(catalog.geometry(PolyhedronKind::Dodecahedron), meshes),
            catalog,
        }
    }

    pub(crate) fn mesh(&self, kind: PolyhedronKind) -> &Handle<Mesh> {
        match kind {
            PolyhedronKind::Cube => &self.cube.mesh,
            PolyhedronKind::Tetrahedron => &self.tetrahedron.mesh,
            PolyhedronKind::Octahedron => &self.octahedron.mesh,
            PolyhedronKind::Dodecahedron => &self.dodecahedron.mesh,
        }
    }
}

struct ShapeRuntime {
    mesh: Handle<Mesh>,
}

impl ShapeRuntime {
    fn new(geometry: &ShapeGeometry, meshes: &mut Assets<Mesh>) -> Self {
        Self {
            mesh: meshes.add(build_mesh(geometry)),
        }
    }
}

#[derive(Resource)]
pub(crate) struct GenerationState {
    pub(crate) nodes: Vec<PolyhedronNode>,
    pub(crate) selected_kind: PolyhedronKind,
    pub(crate) scale_ratio: f32,
    pub(crate) spawn_hold: SpawnHoldState,
}

#[derive(Component)]
pub(crate) struct PolyhedronEntity;

pub(crate) fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_rig: Res<CameraRig>,
) {
    let ui_theme = load_ui_theme(&asset_server);
    let shape_assets = ShapeAssets::new(&mut meshes);
    let root = root_generation_node(&shape_assets.catalog);

    spawn_polyhedron_entity(
        &mut commands,
        &mut materials,
        shape_assets.mesh(root.kind),
        &root,
    );

    let camera_translation = camera_rig.orientation * Vec3::new(0.0, 0.0, camera_rig.distance);
    let scene_camera = commands
        .spawn((
            Camera3d::default(),
            Tonemapping::AcesFitted,
            Transform::from_translation(camera_translation)
                .looking_at(Vec3::ZERO, camera_rig.orientation * Vec3::Y),
            SceneCamera,
            IsDefaultUiCamera,
        ))
        .id();

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.97, 0.93),
            illuminance: 22_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(12.0, 18.0, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.5, 0.6, 0.85),
            intensity: 1_200_000.0,
            range: 60.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-9.0, 5.0, -12.0),
    ));

    spawn_help_ui(&mut commands, &ui_theme, scene_camera);

    commands.insert_resource(ui_theme.clone());
    commands.insert_resource(shape_assets);
    commands.insert_resource(GenerationState {
        nodes: vec![root],
        selected_kind: PolyhedronKind::Dodecahedron,
        scale_ratio: DEFAULT_SCALE_RATIO,
        spawn_hold: SpawnHoldState::default(),
    });

    println!(
        "Controls: F1/H help, arrows pitch/yaw, Q/E roll, W/S zoom, hold Space to spawn, R reset scene, 1-4 select shape, F12 screenshot, -/+ adjust child scale ratio"
    );
    println!(
        "Selected child shape: {:?}, ratio: {:.2}",
        PolyhedronKind::Dodecahedron,
        DEFAULT_SCALE_RATIO
    );
    if ui_theme.source == UiFontSource::Fallback {
        eprintln!(
            "Carbon Plus was not found in assets/fonts. Using Bevy's fallback font for UI text."
        );
    }
}

pub(crate) fn root_generation_node(shape_catalog: &ShapeCatalog) -> PolyhedronNode {
    root_node(ROOT_KIND, ROOT_SCALE, shape_catalog)
}

pub(crate) fn reset_generation_state(
    generation_state: &mut GenerationState,
    shape_catalog: &ShapeCatalog,
) -> PolyhedronNode {
    let root = root_generation_node(shape_catalog);
    generation_state.nodes = vec![root.clone()];
    generation_state.spawn_hold.reset();
    root
}

pub(crate) fn spawn_polyhedron_entity(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    mesh: &Handle<Mesh>,
    node: &PolyhedronNode,
) {
    let hue = (node.level as f32 * 45.0 + node.kind.hue_bias()) % 360.0;
    let material = materials.add(StandardMaterial {
        base_color: Color::hsl(hue, 0.68, 0.56),
        metallic: 0.05,
        perceptual_roughness: 0.86,
        reflectance: 0.24,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material),
        PolyhedronEntity,
        Transform {
            translation: node.center,
            rotation: node.rotation,
            scale: Vec3::splat(node.scale),
        },
        Visibility::Visible,
    ));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{Quat, Vec3};

    use super::{GenerationState, ROOT_KIND, reset_generation_state, root_generation_node};
    use crate::generation::SpawnHoldState;
    use crate::polyhedra::{PolyhedronKind, PolyhedronNode, ShapeCatalog};

    #[test]
    fn reset_generation_state_restores_root_only() {
        let shape_catalog = ShapeCatalog::new();
        let mut root = root_generation_node(&shape_catalog);
        root.occupied_vertices[0] = true;

        let child = PolyhedronNode {
            kind: PolyhedronKind::Tetrahedron,
            level: 1,
            center: Vec3::new(2.0, -1.0, 0.5),
            rotation: Quat::IDENTITY,
            scale: 0.4,
            radius: 0.7,
            occupied_vertices: vec![false; 4],
        };

        let mut generation_state = GenerationState {
            nodes: vec![root, child],
            selected_kind: PolyhedronKind::Octahedron,
            scale_ratio: 0.42,
            spawn_hold: SpawnHoldState {
                elapsed_secs: 1.0,
                repeating: true,
            },
        };

        let reset_root = reset_generation_state(&mut generation_state, &shape_catalog);

        assert_eq!(generation_state.nodes.len(), 1);
        assert_eq!(generation_state.nodes[0].kind, ROOT_KIND);
        assert_eq!(generation_state.nodes[0].level, 0);
        assert_eq!(generation_state.nodes[0].center, Vec3::ZERO);
        assert_eq!(generation_state.selected_kind, PolyhedronKind::Octahedron);
        assert_eq!(generation_state.scale_ratio, 0.42);
        assert!(
            generation_state.nodes[0]
                .occupied_vertices
                .iter()
                .all(|occupied| !occupied)
        );
        assert_eq!(reset_root.center, Vec3::ZERO);
        assert_eq!(generation_state.spawn_hold.elapsed_secs, 0.0);
        assert!(!generation_state.spawn_hold.repeating);
    }
}
