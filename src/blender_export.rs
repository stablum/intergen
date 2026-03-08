use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::app::AppExit;
use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use serde::Serialize;

use crate::camera::CameraRig;
use crate::config::{
    AppConfig, EffectsConfig, LightingConfig, MaterialConfig, RenderingConfig, WindowConfig,
};
use crate::effect_tuner::{EffectRuntimeSnapshot, EffectTunerState};
use crate::polyhedra::{PolyhedronKind, PolyhedronNode, SpawnPlacementMode};
use crate::presets::PresetBrowserState;
use crate::runtime_scene::SceneSnapshotAccess;
use crate::scene::{GenerationState, MaterialState, ShapeAssets};

const BLEND_EXPORT_DIR: &str = "blend-exports";
const BLEND_EXPORT_FORMAT_VERSION: u32 = 1;
const BLENDER_IMPORT_SCRIPT: &str = "tools/blender/import_intergen_scene.py";
const BLENDER_EXECUTABLE_ENV: &str = "BLENDER_EXECUTABLE";

#[derive(Resource)]
pub(crate) struct AutomatedBlendExport {
    path: PathBuf,
    requested: bool,
    trigger_frame: u32,
}

impl AutomatedBlendExport {
    pub(crate) fn new(path: PathBuf, trigger_frame: u32) -> Self {
        Self {
            path,
            requested: false,
            trigger_frame,
        }
    }
}

#[derive(Debug, Serialize)]
struct BlendExportFile {
    format_version: u32,
    app_version: String,
    exported_at_unix_ms: u64,
    world: BlendWorld,
    camera: BlendCamera,
    lights: BlendLights,
    objects: Vec<BlendObject>,
    state: BlendStateMetadata,
    evaluated_effects: EffectsConfig,
    effects: EffectRuntimeSnapshot,
}

#[derive(Debug, Serialize)]
struct BlendWorld {
    clear_color: [f32; 4],
    ambient_color: [f32; 3],
    ambient_brightness: f32,
}

#[derive(Debug, Serialize)]
struct BlendCamera {
    position: [f32; 3],
    forward: [f32; 3],
    up: [f32; 3],
}

#[derive(Debug, Serialize)]
struct BlendLights {
    directional: BlendDirectionalLight,
    point: BlendPointLight,
}

#[derive(Debug, Serialize)]
struct BlendDirectionalLight {
    position: [f32; 3],
    forward: [f32; 3],
    color: [f32; 3],
    illuminance: f32,
    shadows_enabled: bool,
}

#[derive(Debug, Serialize)]
struct BlendPointLight {
    position: [f32; 3],
    color: [f32; 3],
    intensity: f32,
    range: f32,
    shadows_enabled: bool,
}

#[derive(Debug, Serialize)]
struct BlendObject {
    name: String,
    node_index: usize,
    kind: PolyhedronKind,
    level: usize,
    center: [f32; 3],
    vertices: Vec<[f32; 3]>,
    faces: Vec<Vec<usize>>,
    material: BlendMaterial,
    parent_index: Option<usize>,
    parent_attachment_mode: Option<SpawnPlacementMode>,
    parent_attachment_index: Option<usize>,
    parent_vertex_index: Option<usize>,
}

#[derive(Debug, Serialize)]
struct BlendMaterial {
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    reflectance: f32,
    opacity: f32,
}

#[derive(Debug, Serialize)]
struct BlendStateMetadata {
    window: WindowConfig,
    rendering: RenderingConfig,
    lighting: LightingConfig,
    materials: MaterialConfig,
    camera_rig: BlendCameraRigMetadata,
    generation: BlendGenerationMetadata,
    material_state: BlendMaterialStateMetadata,
}

#[derive(Debug, Serialize)]
struct BlendCameraRigMetadata {
    orientation: [f32; 4],
    angular_velocity: [f32; 3],
    distance: f32,
    zoom_velocity: f32,
}

#[derive(Debug, Serialize)]
struct BlendGenerationMetadata {
    selected_kind: PolyhedronKind,
    spawn_placement_mode: SpawnPlacementMode,
    scale_ratio: f32,
    twist_per_vertex_radians: f32,
    vertex_offset_ratio: f32,
    vertex_spawn_exclusion_probability: f32,
    nodes: Vec<BlendNodeMetadata>,
}

#[derive(Debug, Serialize)]
struct BlendMaterialStateMetadata {
    opacity: f32,
}

#[derive(Debug, Serialize)]
struct BlendNodeMetadata {
    kind: PolyhedronKind,
    level: usize,
    center: [f32; 3],
    rotation: [f32; 4],
    scale: f32,
    radius: f32,
    occupied_vertices: Vec<bool>,
    occupied_edges: Vec<bool>,
    occupied_faces: Vec<bool>,
    origin: BlendNodeOrigin,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum BlendNodeOrigin {
    Root,
    Child {
        parent_index: usize,
        attachment_mode: SpawnPlacementMode,
        attachment_index: usize,
    },
}

impl BlendExportFile {
    fn capture(
        app_config: &AppConfig,
        camera_rig: &CameraRig,
        effect_tuner: &EffectTunerState,
        shape_assets: &ShapeAssets,
        generation_state: &GenerationState,
        material_state: &MaterialState,
        now_secs: f32,
    ) -> Self {
        let evaluated_effects = effect_tuner.evaluated_effects(now_secs);
        let effects = effect_tuner.runtime_snapshot();
        let camera_position_bevy =
            camera_rig.orientation * Vec3::new(0.0, 0.0, camera_rig.distance);
        let camera_forward_bevy = safe_normalize(-camera_position_bevy, Vec3::NEG_Z);
        let camera_up_bevy = safe_normalize(camera_rig.orientation * Vec3::Y, Vec3::Y);

        let directional_translation = app_config.lighting.directional.translation();
        let directional_forward = safe_normalize(
            app_config.lighting.directional.look_at() - directional_translation,
            Vec3::NEG_Y,
        );
        let point_translation = app_config.lighting.point.translation();

        Self {
            format_version: BLEND_EXPORT_FORMAT_VERSION,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            exported_at_unix_ms: current_unix_ms(),
            world: BlendWorld {
                clear_color: [
                    app_config.rendering.clear_color[0],
                    app_config.rendering.clear_color[1],
                    app_config.rendering.clear_color[2],
                    1.0,
                ],
                ambient_color: app_config.rendering.ambient_light_color,
                ambient_brightness: app_config.rendering.ambient_light_brightness,
            },
            camera: BlendCamera {
                position: bevy_point_to_blender_array(camera_position_bevy),
                forward: bevy_direction_to_blender_array(camera_forward_bevy),
                up: bevy_direction_to_blender_array(camera_up_bevy),
            },
            lights: BlendLights {
                directional: BlendDirectionalLight {
                    position: bevy_point_to_blender_array(directional_translation),
                    forward: bevy_direction_to_blender_array(directional_forward),
                    color: app_config.lighting.directional.color,
                    illuminance: app_config.lighting.directional.illuminance,
                    shadows_enabled: app_config.lighting.directional.shadows_enabled,
                },
                point: BlendPointLight {
                    position: bevy_point_to_blender_array(point_translation),
                    color: app_config.lighting.point.color,
                    intensity: app_config.lighting.point.intensity,
                    range: app_config.lighting.point.range,
                    shadows_enabled: app_config.lighting.point.shadows_enabled,
                },
            },
            objects: generation_state
                .nodes
                .iter()
                .enumerate()
                .map(|(node_index, node)| {
                    BlendObject::capture(
                        node_index,
                        node,
                        shape_assets,
                        &app_config.materials,
                        material_state.opacity,
                    )
                })
                .collect(),
            state: BlendStateMetadata {
                window: app_config.window.clone(),
                rendering: app_config.rendering.clone(),
                lighting: app_config.lighting.clone(),
                materials: app_config.materials.clone(),
                camera_rig: BlendCameraRigMetadata::capture(camera_rig),
                generation: BlendGenerationMetadata::capture(generation_state),
                material_state: BlendMaterialStateMetadata {
                    opacity: material_state.opacity,
                },
            },
            evaluated_effects,
            effects,
        }
    }
}

impl BlendObject {
    fn capture(
        node_index: usize,
        node: &PolyhedronNode,
        shape_assets: &ShapeAssets,
        material_config: &MaterialConfig,
        opacity: f32,
    ) -> Self {
        let geometry = shape_assets.catalog.geometry(node.kind);
        let vertices = geometry
            .vertices
            .iter()
            .map(|vertex| node.center + node.rotation * (*vertex * node.scale))
            .map(bevy_point_to_blender_array)
            .collect();
        let (parent_index, parent_attachment_mode, parent_attachment_index, parent_vertex_index) =
            match node.origin {
                crate::polyhedra::NodeOrigin::Root => (None, None, None, None),
                crate::polyhedra::NodeOrigin::Child {
                    parent_index,
                    attachment,
                } => (
                    Some(parent_index),
                    Some(attachment.mode),
                    Some(attachment.index),
                    (attachment.mode == SpawnPlacementMode::Vertex).then_some(attachment.index),
                ),
            };

        Self {
            name: format!(
                "intergen_{node_index:04}_{}_l{:02}",
                kind_slug(node.kind),
                node.level
            ),
            node_index,
            kind: node.kind,
            level: node.level,
            center: bevy_point_to_blender_array(node.center),
            vertices,
            faces: geometry.faces.clone(),
            material: BlendMaterial::capture(node, material_config, opacity),
            parent_index,
            parent_attachment_mode,
            parent_attachment_index,
            parent_vertex_index,
        }
    }
}

impl BlendMaterial {
    fn capture(node: &PolyhedronNode, material_config: &MaterialConfig, opacity: f32) -> Self {
        let hue = (node.level as f32 * material_config.hue_step_per_level
            + material_config.hue_bias(node.kind))
        .rem_euclid(360.0);
        let rgb = hsl_to_rgb(
            hue,
            material_config.saturation.clamp(0.0, 1.0),
            material_config.lightness.clamp(0.0, 1.0),
        );
        let opacity = opacity.clamp(0.0, 1.0);

        Self {
            base_color: [rgb[0], rgb[1], rgb[2], opacity],
            metallic: material_config.metallic.clamp(0.0, 1.0),
            roughness: material_config.perceptual_roughness.clamp(0.0, 1.0),
            reflectance: material_config.reflectance.clamp(0.0, 1.0),
            opacity,
        }
    }
}

impl BlendCameraRigMetadata {
    fn capture(camera_rig: &CameraRig) -> Self {
        Self {
            orientation: quat_to_array(camera_rig.orientation),
            angular_velocity: vec3_to_array(camera_rig.angular_velocity),
            distance: camera_rig.distance,
            zoom_velocity: camera_rig.zoom_velocity,
        }
    }
}

impl BlendGenerationMetadata {
    fn capture(generation_state: &GenerationState) -> Self {
        Self {
            selected_kind: generation_state.selected_kind,
            spawn_placement_mode: generation_state.spawn_placement_mode,
            scale_ratio: generation_state.scale_ratio_base(),
            twist_per_vertex_radians: generation_state.twist_per_vertex_radians_base(),
            vertex_offset_ratio: generation_state.vertex_offset_ratio_base(),
            vertex_spawn_exclusion_probability: generation_state
                .vertex_spawn_exclusion_probability_base(),
            nodes: generation_state
                .nodes
                .iter()
                .map(BlendNodeMetadata::capture)
                .collect(),
        }
    }
}

impl BlendNodeMetadata {
    fn capture(node: &PolyhedronNode) -> Self {
        Self {
            kind: node.kind,
            level: node.level,
            center: vec3_to_array(node.center),
            rotation: quat_to_array(node.rotation),
            scale: node.scale,
            radius: node.radius,
            occupied_vertices: node.occupied_attachments.vertices.clone(),
            occupied_edges: node.occupied_attachments.edges.clone(),
            occupied_faces: node.occupied_attachments.faces.clone(),
            origin: BlendNodeOrigin::capture(node.origin),
        }
    }
}

impl BlendNodeOrigin {
    fn capture(origin: crate::polyhedra::NodeOrigin) -> Self {
        match origin {
            crate::polyhedra::NodeOrigin::Root => Self::Root,
            crate::polyhedra::NodeOrigin::Child {
                parent_index,
                attachment,
            } => Self::Child {
                parent_index,
                attachment_mode: attachment.mode,
                attachment_index: attachment.index,
            },
        }
    }
}

pub(crate) fn blender_export_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    preset_browser: Res<PresetBrowserState>,
    scene: SceneSnapshotAccess,
) {
    if preset_browser.blocks_input() || !keys.just_pressed(KeyCode::F4) {
        return;
    }

    let path = match default_blend_export_path(&scene.generation_state) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    match export_current_scene(&path, &scene, time.elapsed_secs()) {
        Ok(message) => println!("{message}"),
        Err(error) => eprintln!("{error}"),
    }
}

pub(crate) fn automated_blend_export_system(
    time: Res<Time>,
    frame_count: Res<FrameCount>,
    automated_blend_export: Option<ResMut<AutomatedBlendExport>>,
    scene: SceneSnapshotAccess,
    mut app_exit: MessageWriter<AppExit>,
) {
    let Some(mut automated_blend_export) = automated_blend_export else {
        return;
    };

    if automated_blend_export.requested || frame_count.0 < automated_blend_export.trigger_frame {
        return;
    }

    automated_blend_export.requested = true;
    match export_current_scene(&automated_blend_export.path, &scene, time.elapsed_secs()) {
        Ok(message) => println!("{message}"),
        Err(error) => eprintln!("{error}"),
    }
    app_exit.write(AppExit::Success);
}

fn export_current_scene(
    output_path: &Path,
    scene: &SceneSnapshotAccess<'_, '_>,
    now_secs: f32,
) -> Result<String, String> {
    ensure_parent_dir(output_path)?;
    let snapshot = BlendExportFile::capture(
        scene.app_config.as_ref(),
        scene.camera_rig.as_ref(),
        scene.effect_tuner.as_ref(),
        scene.shape_assets.as_ref(),
        scene.generation_state.as_ref(),
        scene.material_state.as_ref(),
        now_secs,
    );
    let snapshot_path = snapshot_sidecar_path(output_path);
    let snapshot_json = serde_json::to_string_pretty(&snapshot)
        .map_err(|error| format!("Could not serialize Blender export snapshot: {error}"))?;
    fs::write(&snapshot_path, snapshot_json).map_err(|error| {
        format!(
            "Could not write Blender export snapshot {}: {error}",
            snapshot_path.display()
        )
    })?;

    if let Err(error) = run_blender_import(&snapshot_path, output_path) {
        return Err(format!(
            "{error}\nSnapshot kept at {} for inspection.",
            snapshot_path.display()
        ));
    }

    if let Err(error) = fs::remove_file(&snapshot_path) {
        eprintln!(
            "Could not remove temporary Blender export snapshot {}: {error}",
            snapshot_path.display()
        );
    }

    Ok(format!("Saved Blender scene to {}.", output_path.display()))
}

fn run_blender_import(snapshot_path: &Path, output_path: &Path) -> Result<(), String> {
    let current_dir = std::env::current_dir()
        .map_err(|error| format!("Could not resolve the current working directory: {error}"))?;
    let script_path = current_dir.join(BLENDER_IMPORT_SCRIPT);
    if !script_path.is_file() {
        return Err(format!(
            "Blender import script was not found at {}.",
            script_path.display()
        ));
    }

    let blender_executable = blender_executable();
    let output = Command::new(&blender_executable)
        .arg("--background")
        .arg("--factory-startup")
        .arg("--python-exit-code")
        .arg("1")
        .arg("--python")
        .arg(&script_path)
        .arg("--")
        .arg(snapshot_path)
        .arg(output_path)
        .output()
        .map_err(|error| {
            format!(
                "Could not launch Blender via {:?}: {error}. Set {} to override the executable path if needed.",
                blender_executable,
                BLENDER_EXECUTABLE_ENV
            )
        })?;

    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let mut details = Vec::new();
    if !stdout.is_empty() {
        details.push(format!("stdout:\n{stdout}"));
    }
    if !stderr.is_empty() {
        details.push(format!("stderr:\n{stderr}"));
    }
    let joined = if details.is_empty() {
        String::new()
    } else {
        format!("\n{}", details.join("\n\n"))
    };

    Err(format!(
        "Blender failed to build {} using {} (exit status: {}).{}",
        output_path.display(),
        script_path.display(),
        output.status,
        joined
    ))
}

fn default_blend_export_path(generation_state: &GenerationState) -> Result<PathBuf, String> {
    let current_dir = std::env::current_dir()
        .map_err(|error| format!("Could not resolve the current working directory: {error}"))?;
    let export_dir = current_dir.join(BLEND_EXPORT_DIR);
    fs::create_dir_all(&export_dir).map_err(|error| {
        format!(
            "Could not create Blender export directory {}: {error}",
            export_dir.display()
        )
    })?;

    let root_kind = generation_state
        .nodes
        .first()
        .map(|node| node.kind)
        .unwrap_or(generation_state.selected_kind);
    let timestamp = current_unix_ms();
    let mut suffix = 0_u32;
    loop {
        let candidate = export_dir.join(blend_export_filename(root_kind, timestamp, suffix));
        if !candidate.exists() {
            return Ok(candidate);
        }
        suffix += 1;
    }
}

fn blend_export_filename(root_kind: PolyhedronKind, timestamp_ms: u64, suffix: u32) -> String {
    if suffix == 0 {
        format!("intergen-{timestamp_ms}-{}.blend", kind_slug(root_kind))
    } else {
        format!(
            "intergen-{timestamp_ms}-{}-{suffix}.blend",
            kind_slug(root_kind)
        )
    }
}

fn snapshot_sidecar_path(output_path: &Path) -> PathBuf {
    let stem = output_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("intergen-export");
    output_path.with_file_name(format!("{stem}.snapshot.json"))
}

fn blender_executable() -> OsString {
    std::env::var_os(BLENDER_EXECUTABLE_ENV).unwrap_or_else(|| OsString::from("blender"))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return Ok(());
    };

    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "Could not create Blender export directory {}: {error}",
            parent.display()
        )
    })
}

fn kind_slug(kind: PolyhedronKind) -> &'static str {
    match kind {
        PolyhedronKind::Cube => "cube",
        PolyhedronKind::Tetrahedron => "tetrahedron",
        PolyhedronKind::Octahedron => "octahedron",
        PolyhedronKind::Dodecahedron => "dodecahedron",
    }
}

fn safe_normalize(vector: Vec3, fallback: Vec3) -> Vec3 {
    if vector.length_squared() <= f32::EPSILON {
        fallback
    } else {
        vector.normalize()
    }
}

fn bevy_point_to_blender_array(point: Vec3) -> [f32; 3] {
    [point.x, point.z, -point.y]
}

fn bevy_direction_to_blender_array(direction: Vec3) -> [f32; 3] {
    [direction.x, direction.z, -direction.y]
}

fn vec3_to_array(vector: Vec3) -> [f32; 3] {
    [vector.x, vector.y, vector.z]
}

fn quat_to_array(quat: Quat) -> [f32; 4] {
    [quat.x, quat.y, quat.z, quat.w]
}

fn current_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn hsl_to_rgb(hue_degrees: f32, saturation: f32, lightness: f32) -> [f32; 3] {
    if saturation <= f32::EPSILON {
        return [lightness, lightness, lightness];
    }

    let hue = hue_degrees.rem_euclid(360.0) / 360.0;
    let q = if lightness < 0.5 {
        lightness * (1.0 + saturation)
    } else {
        lightness + saturation - lightness * saturation
    };
    let p = 2.0 * lightness - q;

    [
        hue_to_rgb(p, q, hue + 1.0 / 3.0),
        hue_to_rgb(p, q, hue),
        hue_to_rgb(p, q, hue - 1.0 / 3.0),
    ]
}

fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    let mut t = t;
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 0.5 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec3;

    use super::{bevy_point_to_blender_array, blend_export_filename, hsl_to_rgb};
    use crate::polyhedra::PolyhedronKind;

    #[test]
    fn bevy_axes_convert_to_blender_axes() {
        assert_eq!(
            bevy_point_to_blender_array(Vec3::new(1.0, 2.0, 3.0)),
            [1.0, 3.0, -2.0]
        );
    }

    #[test]
    fn export_filename_uses_root_kind_slug() {
        let file_name = blend_export_filename(PolyhedronKind::Octahedron, 42, 0);

        assert!(file_name.contains("octahedron"));
        assert!(file_name.ends_with(".blend"));
    }

    #[test]
    fn hsl_to_rgb_matches_red_primary() {
        let rgb = hsl_to_rgb(0.0, 1.0, 0.5);

        assert!((rgb[0] - 1.0).abs() < 1.0e-6);
        assert!(rgb[1].abs() < 1.0e-6);
        assert!(rgb[2].abs() < 1.0e-6);
    }
}
