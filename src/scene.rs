use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;

use crate::camera::{CameraRig, SceneCamera};
use crate::config::{AppConfig, GenerationConfig, MaterialConfig};
use crate::effects::{camera_effects_from_config, effects_status_messages};
use crate::generation::{SpawnHoldState, twist_status_message, vertex_offset_status_message};
use crate::polyhedra::{
    PolyhedronKind, PolyhedronNode, ShapeCatalog, ShapeGeometry, build_mesh, root_node,
};
use crate::ui::{UiFontSource, load_ui_theme, spawn_help_ui};

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
    pub(crate) twist_per_vertex_radians: f32,
    pub(crate) vertex_offset_ratio: f32,
    pub(crate) spawn_hold: SpawnHoldState,
    pub(crate) twist_decrease_hold: SpawnHoldState,
    pub(crate) twist_increase_hold: SpawnHoldState,
    pub(crate) vertex_offset_decrease_hold: SpawnHoldState,
    pub(crate) vertex_offset_increase_hold: SpawnHoldState,
}

#[derive(Resource)]
pub(crate) struct MaterialState {
    pub(crate) opacity: f32,
}

#[derive(Component)]
pub(crate) struct PolyhedronEntity {
    pub(crate) node_index: usize,
}

pub(crate) fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_config: Res<AppConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_rig: Res<CameraRig>,
) {
    let ui_theme = load_ui_theme(&asset_server, &app_config.ui);
    let shape_assets = ShapeAssets::new(&mut meshes);
    let root = root_generation_node(&shape_assets.catalog, &app_config.generation);
    let initial_opacity = app_config.materials.default_opacity_clamped();

    spawn_polyhedron_entity(
        &mut commands,
        &mut materials,
        shape_assets.mesh(root.kind),
        &root,
        &app_config.materials,
        initial_opacity,
        0,
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
            camera_effects_from_config(&app_config.effects),
        ))
        .id();

    commands.spawn((
        DirectionalLight {
            color: app_config.lighting.directional.color(),
            illuminance: app_config.lighting.directional.illuminance,
            shadows_enabled: app_config.lighting.directional.shadows_enabled,
            ..default()
        },
        Transform::from_translation(app_config.lighting.directional.translation())
            .looking_at(app_config.lighting.directional.look_at(), Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            color: app_config.lighting.point.color(),
            intensity: app_config.lighting.point.intensity,
            range: app_config.lighting.point.range,
            shadows_enabled: app_config.lighting.point.shadows_enabled,
            ..default()
        },
        Transform::from_translation(app_config.lighting.point.translation()),
    ));

    spawn_help_ui(&mut commands, &ui_theme, scene_camera, &app_config.ui);

    let initial_scale_ratio = app_config.generation.default_scale_ratio_clamped();
    let initial_twist = app_config
        .generation
        .default_twist_per_vertex_radians_clamped();
    let initial_vertex_offset = app_config.generation.default_vertex_offset_ratio_clamped();
    commands.insert_resource(ui_theme.clone());
    commands.insert_resource(shape_assets);
    commands.insert_resource(GenerationState {
        nodes: vec![root],
        selected_kind: app_config.generation.default_child_kind,
        scale_ratio: initial_scale_ratio,
        twist_per_vertex_radians: initial_twist,
        vertex_offset_ratio: initial_vertex_offset,
        spawn_hold: SpawnHoldState::default(),
        twist_decrease_hold: SpawnHoldState::default(),
        twist_increase_hold: SpawnHoldState::default(),
        vertex_offset_decrease_hold: SpawnHoldState::default(),
        vertex_offset_increase_hold: SpawnHoldState::default(),
    });
    commands.insert_resource(MaterialState {
        opacity: initial_opacity,
    });

    println!(
        "Controls: F1/H help, F2 FX strip, arrows pitch/yaw, Q/E roll, W/S zoom, Backspace stops camera rotation, hold Space to spawn, R reset scene, 1-4 select shape, F12 screenshot, -/+ adjust child scale ratio, O/P adjust opacity, I reset opacity, hold [/] or ,/. to adjust child twist, T reset twist, hold Z/X to adjust child offset, C reset offset"
    );
    println!(
        "FX strip: Ctrl+Up/Down selects a parameter, Ctrl+Left/Right adjusts the highlighted field, Tab toggles the effect, L toggles the selected parameter LFO, M cycles the highlighted value/amp/freq/shape field, Shift is coarse, Alt is fine, Enter resets the field, Shift+Enter resets all FX settings and LFOs."
    );
    println!(
        "Selected child shape: {:?}, ratio: {:.2}",
        app_config.generation.default_child_kind, initial_scale_ratio
    );
    println!("{}", twist_status_message(initial_twist));
    println!("{}", vertex_offset_status_message(initial_vertex_offset));
    println!("{}", opacity_status_message(initial_opacity));
    for message in effects_status_messages(&app_config.effects) {
        println!("{message}");
    }
    if ui_theme.source == UiFontSource::Fallback {
        eprintln!(
            "Carbon Plus was not found in assets/fonts. Using Bevy's fallback font for UI text."
        );
    }
}

pub(crate) fn root_generation_node(
    shape_catalog: &ShapeCatalog,
    generation_config: &GenerationConfig,
) -> PolyhedronNode {
    root_node(
        generation_config.root_kind,
        generation_config.root_scale,
        shape_catalog,
    )
}

pub(crate) fn reset_generation_state(
    generation_state: &mut GenerationState,
    shape_catalog: &ShapeCatalog,
    generation_config: &GenerationConfig,
) -> PolyhedronNode {
    let root = root_node(
        generation_state.selected_kind,
        generation_config.root_scale,
        shape_catalog,
    );
    generation_state.nodes = vec![root.clone()];
    generation_state.spawn_hold.reset();
    generation_state.twist_decrease_hold.reset();
    generation_state.twist_increase_hold.reset();
    generation_state.vertex_offset_decrease_hold.reset();
    generation_state.vertex_offset_increase_hold.reset();
    root
}

pub(crate) fn spawn_polyhedron_entity(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    mesh: &Handle<Mesh>,
    node: &PolyhedronNode,
    material_config: &MaterialConfig,
    opacity: f32,
    node_index: usize,
) {
    let material = materials.add(polyhedron_material(node, material_config, opacity));

    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material),
        PolyhedronEntity { node_index },
        Transform {
            translation: node.center,
            rotation: node.rotation,
            scale: Vec3::splat(node.scale),
        },
        Visibility::Visible,
    ));
}

pub(crate) fn sync_polyhedron_transforms(
    nodes: &[PolyhedronNode],
    polyhedron_transforms: &mut Query<(&PolyhedronEntity, &mut Transform)>,
) {
    for (polyhedron_entity, mut transform) in polyhedron_transforms.iter_mut() {
        let Some(node) = nodes.get(polyhedron_entity.node_index) else {
            continue;
        };

        transform.translation = node.center;
        transform.rotation = node.rotation;
        transform.scale = Vec3::splat(node.scale);
    }
}

fn polyhedron_material(
    node: &PolyhedronNode,
    material_config: &MaterialConfig,
    opacity: f32,
) -> StandardMaterial {
    let hue = (node.level as f32 * material_config.hue_step_per_level
        + material_config.hue_bias(node.kind))
        % 360.0;
    let opacity = opacity.clamp(0.0, 1.0);

    StandardMaterial {
        base_color: Color::hsl(hue, material_config.saturation, material_config.lightness)
            .with_alpha(opacity),
        alpha_mode: alpha_mode_for_opacity(opacity),
        metallic: material_config.metallic,
        perceptual_roughness: material_config.perceptual_roughness,
        reflectance: material_config.reflectance,
        ..default()
    }
}

pub(crate) fn alpha_mode_for_opacity(opacity: f32) -> AlphaMode {
    if opacity < 0.999 {
        AlphaMode::Blend
    } else {
        AlphaMode::Opaque
    }
}

pub(crate) fn opacity_status_message(opacity: f32) -> String {
    format!(
        "Global object opacity: {:.0}%",
        opacity.clamp(0.0, 1.0) * 100.0
    )
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{AlphaMode, Quat, Vec3};

    use crate::config::GenerationConfig;
    use crate::generation::SpawnHoldState;
    use crate::polyhedra::{NodeOrigin, PolyhedronKind, PolyhedronNode, ShapeCatalog};

    use super::{
        GenerationState, alpha_mode_for_opacity, reset_generation_state, root_generation_node,
    };

    #[test]
    fn reset_generation_state_restores_root_only() {
        let shape_catalog = ShapeCatalog::new();
        let generation_config = GenerationConfig::default();
        let mut root = root_generation_node(&shape_catalog, &generation_config);
        root.occupied_vertices[0] = true;

        let child = PolyhedronNode {
            kind: PolyhedronKind::Tetrahedron,
            level: 1,
            center: Vec3::new(2.0, -1.0, 0.5),
            rotation: Quat::IDENTITY,
            scale: 0.4,
            radius: 0.7,
            occupied_vertices: vec![false; 4],
            origin: NodeOrigin::Child {
                parent_index: 0,
                vertex_index: 0,
            },
        };

        let mut generation_state = GenerationState {
            nodes: vec![root, child],
            selected_kind: PolyhedronKind::Octahedron,
            scale_ratio: 0.42,
            twist_per_vertex_radians: 0.3,
            vertex_offset_ratio: 0.6,
            spawn_hold: SpawnHoldState {
                elapsed_secs: 1.0,
                repeating: true,
            },
            twist_decrease_hold: SpawnHoldState::default(),
            twist_increase_hold: SpawnHoldState::default(),
            vertex_offset_decrease_hold: SpawnHoldState::default(),
            vertex_offset_increase_hold: SpawnHoldState::default(),
        };

        let reset_root =
            reset_generation_state(&mut generation_state, &shape_catalog, &generation_config);

        assert_eq!(generation_state.nodes.len(), 1);
        assert_eq!(generation_state.nodes[0].kind, PolyhedronKind::Octahedron);
        assert_eq!(generation_state.nodes[0].level, 0);
        assert_eq!(generation_state.nodes[0].center, Vec3::ZERO);
        assert_eq!(generation_state.selected_kind, PolyhedronKind::Octahedron);
        assert_eq!(generation_state.scale_ratio, 0.42);
        assert_eq!(generation_state.twist_per_vertex_radians, 0.3);
        assert_eq!(generation_state.vertex_offset_ratio, 0.6);
        assert!(
            generation_state.nodes[0]
                .occupied_vertices
                .iter()
                .all(|occupied| !occupied)
        );
        assert_eq!(reset_root.center, Vec3::ZERO);
        assert_eq!(reset_root.kind, PolyhedronKind::Octahedron);
        assert_eq!(generation_state.spawn_hold.elapsed_secs, 0.0);
        assert!(!generation_state.spawn_hold.repeating);
        assert_eq!(generation_state.twist_decrease_hold.elapsed_secs, 0.0);
        assert!(!generation_state.twist_decrease_hold.repeating);
        assert_eq!(generation_state.twist_increase_hold.elapsed_secs, 0.0);
        assert!(!generation_state.twist_increase_hold.repeating);
        assert_eq!(
            generation_state.vertex_offset_decrease_hold.elapsed_secs,
            0.0
        );
        assert!(!generation_state.vertex_offset_decrease_hold.repeating);
        assert_eq!(
            generation_state.vertex_offset_increase_hold.elapsed_secs,
            0.0
        );
        assert!(!generation_state.vertex_offset_increase_hold.repeating);
    }

    #[test]
    fn transparent_materials_use_blend_mode() {
        assert!(matches!(alpha_mode_for_opacity(0.6), AlphaMode::Blend));
        assert!(matches!(alpha_mode_for_opacity(1.0), AlphaMode::Opaque));
    }
}
