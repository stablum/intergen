mod polyhedra;

use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy::window::PresentMode;

use polyhedra::{
    MAX_SCALE_RATIO, MIN_SCALE_RATIO, PolyhedronKind, PolyhedronNode, ShapeCatalog, build_mesh,
    next_spawn, root_node,
};

const ROOT_SCALE: f32 = 1.9;
const DEFAULT_SCALE_RATIO: f32 = 0.58;
const CAMERA_DISTANCE: f32 = 14.0;
const CAMERA_ROTATION_ACCEL: f32 = 1.9;
const CAMERA_ZOOM_ACCEL: f32 = 24.0;
const ANGULAR_DAMPING: f32 = 2.2;
const ZOOM_DAMPING: f32 = 4.0;
const MIN_CAMERA_DISTANCE: f32 = 4.0;
const MAX_CAMERA_DISTANCE: f32 = 48.0;

pub fn run() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.035, 0.04, 0.06)))
        .insert_resource(CameraRig::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "intergen".into(),
                resolution: (1440, 960).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                camera_input_system,
                camera_motion_system,
                generation_input_system,
            ),
        )
        .run();
}

#[derive(Resource)]
struct ShapeAssets {
    catalog: ShapeCatalog,
    cube: ShapeRuntime,
    tetrahedron: ShapeRuntime,
    octahedron: ShapeRuntime,
    dodecahedron: ShapeRuntime,
}

impl ShapeAssets {
    fn new(meshes: &mut Assets<Mesh>) -> Self {
        let catalog = ShapeCatalog::new();

        Self {
            cube: ShapeRuntime::new(catalog.geometry(PolyhedronKind::Cube), meshes),
            tetrahedron: ShapeRuntime::new(catalog.geometry(PolyhedronKind::Tetrahedron), meshes),
            octahedron: ShapeRuntime::new(catalog.geometry(PolyhedronKind::Octahedron), meshes),
            dodecahedron: ShapeRuntime::new(catalog.geometry(PolyhedronKind::Dodecahedron), meshes),
            catalog,
        }
    }

    fn mesh(&self, kind: PolyhedronKind) -> &Handle<Mesh> {
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
    fn new(geometry: &polyhedra::ShapeGeometry, meshes: &mut Assets<Mesh>) -> Self {
        Self {
            mesh: meshes.add(build_mesh(geometry)),
        }
    }
}

#[derive(Resource)]
struct GenerationState {
    nodes: Vec<PolyhedronNode>,
    selected_kind: PolyhedronKind,
    scale_ratio: f32,
}

#[derive(Resource)]
struct CameraRig {
    orientation: Quat,
    angular_velocity: Vec3,
    distance: f32,
    zoom_velocity: f32,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            orientation: Quat::from_euler(EulerRot::YXZ, -FRAC_PI_4, -0.45, 0.15),
            angular_velocity: Vec3::ZERO,
            distance: CAMERA_DISTANCE,
            zoom_velocity: 0.0,
        }
    }
}

#[derive(Component)]
struct SceneCamera;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_rig: Res<CameraRig>,
) {
    let shape_assets = ShapeAssets::new(&mut meshes);
    let root = root_node(PolyhedronKind::Cube, ROOT_SCALE, &shape_assets.catalog);

    spawn_polyhedron_entity(
        &mut commands,
        &mut materials,
        shape_assets.mesh(root.kind),
        &root,
    );

    let camera_translation = camera_rig.orientation * Vec3::new(0.0, 0.0, camera_rig.distance);
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(camera_translation)
            .looking_at(Vec3::ZERO, camera_rig.orientation * Vec3::Y),
        SceneCamera,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 40_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 50_000_000.0,
            range: 120.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-8.0, 6.0, -7.0),
    ));

    commands.insert_resource(shape_assets);
    commands.insert_resource(GenerationState {
        nodes: vec![root],
        selected_kind: PolyhedronKind::Dodecahedron,
        scale_ratio: DEFAULT_SCALE_RATIO,
    });

    println!(
        "Controls: arrows pitch/yaw, Q/E roll, W/S zoom, Space spawn, 1-4 select shape, -/+ adjust child scale ratio"
    );
    println!(
        "Selected child shape: {:?}, ratio: {:.2}",
        PolyhedronKind::Dodecahedron,
        DEFAULT_SCALE_RATIO
    );
}

fn camera_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_rig: ResMut<CameraRig>,
) {
    let dt = time.delta_secs();
    let torque_step = CAMERA_ROTATION_ACCEL * dt;

    if keys.pressed(KeyCode::ArrowUp) {
        camera_rig.angular_velocity.x += torque_step;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        camera_rig.angular_velocity.x -= torque_step;
    }
    if keys.pressed(KeyCode::ArrowLeft) {
        camera_rig.angular_velocity.y += torque_step;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        camera_rig.angular_velocity.y -= torque_step;
    }
    if keys.pressed(KeyCode::KeyQ) {
        camera_rig.angular_velocity.z += torque_step;
    }
    if keys.pressed(KeyCode::KeyE) {
        camera_rig.angular_velocity.z -= torque_step;
    }
    if keys.pressed(KeyCode::KeyW) {
        camera_rig.zoom_velocity -= CAMERA_ZOOM_ACCEL * dt;
    }
    if keys.pressed(KeyCode::KeyS) {
        camera_rig.zoom_velocity += CAMERA_ZOOM_ACCEL * dt;
    }
}

fn camera_motion_system(
    time: Res<Time>,
    mut camera_rig: ResMut<CameraRig>,
    mut query: Query<&mut Transform, With<SceneCamera>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    if camera_rig.angular_velocity.length_squared() > 0.0 {
        let delta = Quat::from_scaled_axis(camera_rig.angular_velocity * dt);
        camera_rig.orientation = (delta * camera_rig.orientation).normalize();
    }

    camera_rig.angular_velocity *= f32::exp(-ANGULAR_DAMPING * dt);
    camera_rig.zoom_velocity *= f32::exp(-ZOOM_DAMPING * dt);
    camera_rig.distance = (camera_rig.distance + camera_rig.zoom_velocity * dt)
        .clamp(MIN_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE);

    let translation = camera_rig.orientation * Vec3::new(0.0, 0.0, camera_rig.distance);
    *transform = Transform::from_translation(translation)
        .looking_at(Vec3::ZERO, camera_rig.orientation * Vec3::Y);
}

fn generation_input_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    shape_assets: Res<ShapeAssets>,
    mut generation_state: ResMut<GenerationState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        generation_state.selected_kind = PolyhedronKind::Cube;
        println!("Selected child shape: {:?}", generation_state.selected_kind);
    }
    if keys.just_pressed(KeyCode::Digit2) {
        generation_state.selected_kind = PolyhedronKind::Tetrahedron;
        println!("Selected child shape: {:?}", generation_state.selected_kind);
    }
    if keys.just_pressed(KeyCode::Digit3) {
        generation_state.selected_kind = PolyhedronKind::Octahedron;
        println!("Selected child shape: {:?}", generation_state.selected_kind);
    }
    if keys.just_pressed(KeyCode::Digit4) {
        generation_state.selected_kind = PolyhedronKind::Dodecahedron;
        println!("Selected child shape: {:?}", generation_state.selected_kind);
    }

    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        generation_state.scale_ratio =
            (generation_state.scale_ratio - 0.05).clamp(MIN_SCALE_RATIO, MAX_SCALE_RATIO);
        println!("Child scale ratio: {:.2}", generation_state.scale_ratio);
    }
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        generation_state.scale_ratio =
            (generation_state.scale_ratio + 0.05).clamp(MIN_SCALE_RATIO, MAX_SCALE_RATIO);
        println!("Child scale ratio: {:.2}", generation_state.scale_ratio);
    }

    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    let selected_kind = generation_state.selected_kind;
    let scale_ratio = generation_state.scale_ratio;
    let Some(spawn) = next_spawn(
        &mut generation_state.nodes,
        &shape_assets.catalog,
        selected_kind,
        scale_ratio,
    ) else {
        eprintln!("No valid spawn position is currently available.");
        return;
    };

    spawn_polyhedron_entity(
        &mut commands,
        &mut materials,
        shape_assets.mesh(spawn.kind),
        &spawn.node,
    );
    println!(
        "Spawned {:?} at level {} from parent level {}",
        spawn.kind, spawn.node.level, spawn.parent_level
    );
}

fn spawn_polyhedron_entity(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    mesh: &Handle<Mesh>,
    node: &PolyhedronNode,
) {
    let hue = (node.level as f32 * 45.0 + node.kind.hue_bias()) % 360.0;
    let material = materials.add(StandardMaterial {
        base_color: Color::hsl(hue, 0.72, 0.58),
        metallic: 0.82,
        perceptual_roughness: 0.18,
        reflectance: 0.82,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material),
        Transform {
            translation: node.center,
            rotation: node.rotation,
            scale: Vec3::splat(node.scale),
        },
    ));
}
