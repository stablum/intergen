mod polyhedra;

use std::f32::consts::FRAC_PI_4;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use bevy::app::AppExit;
use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
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
const SPAWN_HOLD_DELAY_SECS: f32 = 0.24;
const SPAWN_REPEAT_INTERVAL_SECS: f32 = 0.07;
const SCREENSHOT_OUTPUT_DIR: &str = "screenshots";
const AUTO_CAPTURE_FRAME_DELAY: u32 = 8;
const CARBON_PLUS_FONT_CANDIDATES: &[&str] = &[
    "fonts/CarbonPlus-Regular.ttf",
    "fonts/CarbonPlus-Regular.otf",
    "fonts/Carbon Plus Regular.ttf",
    "fonts/Carbon Plus Regular.otf",
    "fonts/CarbonPlus.ttf",
    "fonts/Carbon Plus.ttf",
];

pub fn run() {
    let launch_config = match LaunchConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.035, 0.04, 0.06)))
        .insert_resource(AmbientLight {
            color: Color::srgb(0.7, 0.74, 0.82),
            brightness: 12.0,
            ..default()
        })
        .insert_resource(CameraRig::default())
        .insert_resource(HelpOverlayState::default())
        .insert_resource(ScreenshotCounter::default())
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
                toggle_help_overlay_system,
                camera_input_system,
                camera_motion_system,
                generation_input_system,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (manual_screenshot_input_system, automated_capture_system),
        );

    if let Some(path) = launch_config.capture_path {
        app.insert_resource(AutomatedCapture::new(
            path,
            launch_config.capture_delay_frames,
        ));
    }

    app.run();
}

#[derive(Default)]
struct LaunchConfig {
    capture_path: Option<PathBuf>,
    capture_delay_frames: u32,
}

impl LaunchConfig {
    fn from_env() -> Result<Self, String> {
        parse_launch_config(std::env::args_os().skip(1))
    }
}

fn parse_launch_config(args: impl IntoIterator<Item = OsString>) -> Result<LaunchConfig, String> {
    let mut capture_path = None;
    let mut capture_delay_frames = AUTO_CAPTURE_FRAME_DELAY;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--capture" => {
                let Some(path) = args.next() else {
                    return Err("Missing path after --capture".to_string());
                };
                capture_path = Some(PathBuf::from(path));
            }
            "--capture-delay-frames" => {
                let Some(frame_count) = args.next() else {
                    return Err("Missing frame count after --capture-delay-frames".to_string());
                };
                let frame_count = frame_count.to_string_lossy();
                capture_delay_frames = frame_count.parse::<u32>().map_err(|_| {
                    format!("Invalid frame count for --capture-delay-frames: {frame_count}")
                })?;
            }
            "--help" | "-h" => {
                return Err(
                    "Usage: cargo run -- [--capture <output.png>] [--capture-delay-frames <n>]\nF12 saves a screenshot during normal interactive runs."
                        .to_string(),
                );
            }
            other => {
                return Err(format!("Unknown argument: {other}"));
            }
        }
    }

    Ok(LaunchConfig {
        capture_path,
        capture_delay_frames,
    })
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
    spawn_hold: SpawnHoldState,
}

#[derive(Default)]
struct SpawnHoldState {
    elapsed_secs: f32,
    repeating: bool,
}

impl SpawnHoldState {
    fn update(
        &mut self,
        just_pressed: bool,
        pressed: bool,
        just_released: bool,
        delta_secs: f32,
    ) -> bool {
        if just_released || !pressed {
            self.reset();
            return false;
        }

        if just_pressed {
            self.reset();
            return true;
        }

        self.elapsed_secs += delta_secs;
        let threshold = if self.repeating {
            SPAWN_REPEAT_INTERVAL_SECS
        } else {
            SPAWN_HOLD_DELAY_SECS
        };

        if self.elapsed_secs < threshold {
            return false;
        }

        self.elapsed_secs = 0.0;
        self.repeating = true;
        true
    }

    fn reset(&mut self) {
        self.elapsed_secs = 0.0;
        self.repeating = false;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UiFontSource {
    CarbonPlus,
    Fallback,
}

#[derive(Clone, Resource)]
struct UiTheme {
    font: Handle<Font>,
    source: UiFontSource,
}

impl UiTheme {
    fn text_font(&self, font_size: f32) -> TextFont {
        TextFont {
            font: self.font.clone(),
            font_size,
            ..default()
        }
    }
}

#[derive(Resource)]
struct CameraRig {
    orientation: Quat,
    angular_velocity: Vec3,
    distance: f32,
    zoom_velocity: f32,
}

#[derive(Resource, Default)]
struct HelpOverlayState {
    visible: bool,
}

#[derive(Resource, Default)]
struct ScreenshotCounter {
    next_index: u32,
}

#[derive(Resource)]
struct AutomatedCapture {
    path: PathBuf,
    requested: bool,
    trigger_frame: u32,
}

impl AutomatedCapture {
    fn new(path: PathBuf, trigger_frame: u32) -> Self {
        Self {
            path,
            requested: false,
            trigger_frame,
        }
    }
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

#[derive(Component)]
struct HelpOverlay;

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_rig: Res<CameraRig>,
) {
    let ui_theme = load_ui_theme(&asset_server);
    let shape_assets = ShapeAssets::new(&mut meshes);
    let root = root_node(PolyhedronKind::Cube, ROOT_SCALE, &shape_assets.catalog);

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
            bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
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
        "Controls: F1/H help, arrows pitch/yaw, Q/E roll, W/S zoom, hold Space to spawn, 1-4 select shape, F12 screenshot, -/+ adjust child scale ratio"
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
    time: Res<Time>,
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

    let spawn_requested = generation_state.spawn_hold.update(
        keys.just_pressed(KeyCode::Space),
        keys.pressed(KeyCode::Space),
        keys.just_released(KeyCode::Space),
        time.delta_secs(),
    );
    if !spawn_requested {
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

fn toggle_help_overlay_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut help_overlay: ResMut<HelpOverlayState>,
    mut overlay_query: Query<&mut Visibility, With<HelpOverlay>>,
) {
    if !(keys.just_pressed(KeyCode::F1) || keys.just_pressed(KeyCode::KeyH)) {
        return;
    }

    help_overlay.visible = !help_overlay.visible;

    let Ok(mut visibility) = overlay_query.single_mut() else {
        return;
    };

    *visibility = if help_overlay.visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

fn manual_screenshot_input_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut screenshot_counter: ResMut<ScreenshotCounter>,
) {
    if !keys.just_pressed(KeyCode::F12) {
        return;
    }

    let path = PathBuf::from(SCREENSHOT_OUTPUT_DIR)
        .join(format!("intergen-{:04}.png", screenshot_counter.next_index));
    screenshot_counter.next_index += 1;
    request_screenshot(&mut commands, path, false);
}

fn automated_capture_system(
    mut commands: Commands,
    frame_count: Res<FrameCount>,
    automated_capture: Option<ResMut<AutomatedCapture>>,
) {
    let Some(mut automated_capture) = automated_capture else {
        return;
    };

    if automated_capture.requested || frame_count.0 < automated_capture.trigger_frame {
        return;
    }

    automated_capture.requested = true;
    request_screenshot(&mut commands, automated_capture.path.clone(), true);
}

fn request_screenshot(commands: &mut Commands, path: PathBuf, exit_after_capture: bool) {
    if !ensure_parent_dir(&path) {
        return;
    }

    println!("Saving screenshot to {}", path.display());
    let mut entity = commands.spawn(Screenshot::primary_window());
    entity.observe(save_to_disk(path));
    if exit_after_capture {
        entity.observe(exit_after_screenshot_capture);
    }
}

fn ensure_parent_dir(path: &Path) -> bool {
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return true;
    };

    if let Err(error) = fs::create_dir_all(parent) {
        eprintln!(
            "Could not create screenshot directory {}: {error}",
            parent.display()
        );
        return false;
    }

    true
}

fn exit_after_screenshot_capture(_: On<ScreenshotCaptured>, mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}

fn load_ui_theme(asset_server: &AssetServer) -> UiTheme {
    if let Some(font_asset) = carbon_plus_font_asset() {
        return UiTheme {
            font: asset_server.load(font_asset),
            source: UiFontSource::CarbonPlus,
        };
    }

    UiTheme {
        font: default(),
        source: UiFontSource::Fallback,
    }
}

fn carbon_plus_font_asset() -> Option<&'static str> {
    CARBON_PLUS_FONT_CANDIDATES
        .iter()
        .copied()
        .find(|path| Path::new("assets").join(path).is_file())
}

fn spawn_help_ui(commands: &mut Commands, ui_theme: &UiTheme, scene_camera: Entity) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(18),
                left: px(18),
                padding: UiRect::axes(px(12), px(8)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.08, 0.13, 0.86)),
            BorderRadius::MAX,
            GlobalZIndex(20),
            UiTargetCamera(scene_camera),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("F1 / H: controls"),
                ui_theme.text_font(14.0),
                TextColor(Color::srgb(0.93, 0.95, 0.99)),
            ));
        });

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                padding: UiRect::all(px(24)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.02, 0.04, 0.72)),
            GlobalZIndex(30),
            Visibility::Hidden,
            HelpOverlay,
            UiTargetCamera(scene_camera),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100),
                        max_width: px(460),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(12),
                        padding: UiRect::all(px(20)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.1, 0.16, 0.95)),
                    BorderRadius::all(px(20)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Keybindings"),
                        ui_theme.text_font(28.0),
                        TextColor(Color::srgb(0.98, 0.99, 1.0)),
                    ));
                    panel.spawn((
                        Text::new(controls_overlay_text(ui_theme.source)),
                        ui_theme.text_font(16.0),
                        TextColor(Color::srgb(0.89, 0.92, 0.96)),
                        TextLayout::new_with_justify(Justify::Left),
                        Node {
                            max_width: px(420),
                            ..default()
                        },
                    ));
                });
        });
}

fn controls_overlay_text(font_source: UiFontSource) -> String {
    format!(
        concat!(
            "F1 / H: Toggle this overlay\n",
            "Arrow Up / Down: Pitch camera\n",
            "Arrow Left / Right: Yaw camera\n",
            "Q / E: Roll camera\n",
            "W / S: Zoom in / out\n",
            "Space: Spawn polyhedra (hold to repeat)\n",
            "1: Select cube\n",
            "2: Select tetrahedron\n",
            "3: Select octahedron\n",
            "4: Select dodecahedron\n",
            "F12: Save a screenshot\n",
            "- / +: Adjust child scale ratio\n",
            "\n",
            "{}"
        ),
        font_status_line(font_source)
    )
}

fn font_status_line(font_source: UiFontSource) -> &'static str {
    match font_source {
        UiFontSource::CarbonPlus => "Font: Carbon Plus",
        UiFontSource::Fallback => {
            "Font: fallback active. Add a Carbon Plus .ttf or .otf under assets/fonts/."
        }
    }
}

fn spawn_polyhedron_entity(
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
    use std::ffi::OsString;
    use std::path::Path;

    use super::{
        AUTO_CAPTURE_FRAME_DELAY, SPAWN_HOLD_DELAY_SECS, SPAWN_REPEAT_INTERVAL_SECS,
        SpawnHoldState, UiFontSource, controls_overlay_text, font_status_line, parse_launch_config,
    };

    #[test]
    fn overlay_text_lists_help_and_spawn_controls() {
        let text = controls_overlay_text(UiFontSource::CarbonPlus);

        assert!(text.contains("F1 / H: Toggle this overlay"));
        assert!(text.contains("Space: Spawn polyhedra (hold to repeat)"));
        assert!(text.contains("F12: Save a screenshot"));
        assert!(text.contains("4: Select dodecahedron"));
    }

    #[test]
    fn fallback_font_status_mentions_assets_directory() {
        let status = font_status_line(UiFontSource::Fallback);

        assert!(status.contains("assets/fonts"));
        assert!(status.contains("Carbon Plus"));
    }

    #[test]
    fn launch_config_parses_capture_path() {
        let config = parse_launch_config([
            OsString::from("--capture"),
            OsString::from("screenshots/test.png"),
        ])
        .expect("capture path should parse");

        assert_eq!(
            config.capture_path.as_deref(),
            Some(Path::new("screenshots/test.png"))
        );
        assert_eq!(config.capture_delay_frames, AUTO_CAPTURE_FRAME_DELAY);
    }
    #[test]
    fn launch_config_parses_capture_delay_frames() {
        let config = parse_launch_config([
            OsString::from("--capture-delay-frames"),
            OsString::from("64"),
        ])
        .expect("capture delay should parse");
        assert_eq!(config.capture_delay_frames, 64);
        assert_eq!(config.capture_path, None);
    }

    #[test]
    fn spawn_hold_repeats_while_space_is_held() {
        let mut spawn_hold = SpawnHoldState::default();

        assert!(spawn_hold.update(true, true, false, 0.0));
        assert!(!spawn_hold.update(false, true, false, SPAWN_HOLD_DELAY_SECS * 0.5));
        assert!(spawn_hold.update(false, true, false, SPAWN_HOLD_DELAY_SECS * 0.5));
        assert!(!spawn_hold.update(false, true, false, SPAWN_REPEAT_INTERVAL_SECS * 0.5,));
        assert!(spawn_hold.update(false, true, false, SPAWN_REPEAT_INTERVAL_SECS * 0.5,));
    }

    #[test]
    fn spawn_hold_resets_after_release() {
        let mut spawn_hold = SpawnHoldState::default();

        assert!(spawn_hold.update(true, true, false, 0.0));
        assert!(!spawn_hold.update(false, false, true, 0.0));
        assert!(spawn_hold.update(true, true, false, 0.0));
    }
}
