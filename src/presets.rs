use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::camera::CameraRig;
use crate::config::{AppConfig, LightingConfig, MaterialConfig, RenderingConfig};
use crate::effect_tuner::{EffectRuntimeSnapshot, EffectTunerState};
use crate::polyhedra::{NodeOrigin, PolyhedronKind, PolyhedronNode};
use crate::runtime_scene::SceneMutationAccess;
use crate::scene::{GenerationState, MaterialState, spawn_polyhedron_entity};

const PRESET_DIR: &str = "scene-presets";
const PRESET_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub(crate) struct PresetIndex {
    pub(crate) bank: u8,
    pub(crate) slot: u8,
}

impl PresetIndex {
    fn code(self) -> String {
        format!("{}{}", self.bank, self.slot)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PresetCommand {
    Load,
    Save,
    Free,
}

impl PresetCommand {
    fn label(self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Save => "save",
            Self::Free => "free",
        }
    }
}

#[derive(Clone)]
struct PresetRecord {
    path: PathBuf,
    file: ScenePresetFile,
}

#[derive(Clone)]
struct CollisionResolutionState {
    index: PresetIndex,
    selected: usize,
    candidates: Vec<PresetRecord>,
    load_after_resolution: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ScenePresetFile {
    format_version: u32,
    id: String,
    saved_at_unix_ms: u64,
    summary: String,
    assignment: Option<PresetIndex>,
    scene: ScenePresetSnapshot,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ScenePresetSnapshot {
    rendering: RenderingConfig,
    lighting: LightingConfig,
    materials: MaterialConfig,
    camera: CameraRigSnapshot,
    generation: GenerationSnapshot,
    material_state: MaterialRuntimeSnapshot,
    effects: EffectRuntimeSnapshot,
}

struct PreparedScenePreset {
    rendering: RenderingConfig,
    lighting: LightingConfig,
    materials: MaterialConfig,
    camera_rig: CameraRig,
    generation: GenerationState,
    material_opacity: f32,
    effects: EffectRuntimeSnapshot,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CameraRigSnapshot {
    orientation: [f32; 4],
    angular_velocity: [f32; 3],
    distance: f32,
    zoom_velocity: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GenerationSnapshot {
    selected_kind: PolyhedronKind,
    scale_ratio: f32,
    twist_per_vertex_radians: f32,
    vertex_offset_ratio: f32,
    #[serde(default)]
    vertex_spawn_exclusion_probability: f32,
    nodes: Vec<PolyhedronNodeSnapshot>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct MaterialRuntimeSnapshot {
    opacity: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PolyhedronNodeSnapshot {
    kind: PolyhedronKind,
    level: usize,
    center: [f32; 3],
    rotation: [f32; 4],
    scale: f32,
    radius: f32,
    occupied_vertices: Vec<bool>,
    origin: NodeOriginSnapshot,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
enum NodeOriginSnapshot {
    Root,
    Child {
        parent_index: usize,
        vertex_index: usize,
    },
}

#[derive(Resource)]
pub(crate) struct PresetBrowserState {
    active: bool,
    command: PresetCommand,
    first_digit: Option<u8>,
    status_message: String,
    records: Vec<PresetRecord>,
    chooser: Option<CollisionResolutionState>,
}

impl Default for PresetBrowserState {
    fn default() -> Self {
        Self {
            active: false,
            command: PresetCommand::Load,
            first_digit: None,
            status_message: String::new(),
            records: Vec::new(),
            chooser: None,
        }
    }
}

impl PresetBrowserState {
    pub(crate) fn load_from_disk() -> Self {
        let mut state = Self::default();
        if let Err(error) = state.refresh() {
            state.status_message = error;
        }
        state
    }

    pub(crate) fn blocks_input(&self) -> bool {
        self.active
    }

    pub(crate) fn is_visible(&self) -> bool {
        self.active
    }

    pub(crate) fn chooser_visible(&self) -> bool {
        self.active && self.chooser.is_some()
    }

    pub(crate) fn strip_text(&self) -> String {
        let target = match self.first_digit {
            Some(bank) => format!(" {}_", bank),
            None => String::new(),
        };
        let banks = (0_u8..10)
            .map(|bank| format!("{}[{}]", bank, self.bank_occupancy(bank)))
            .collect::<Vec<_>>()
            .join(" ");
        let status = if self.status_message.is_empty() {
            String::new()
        } else {
            format!(" | {}", self.status_message)
        };
        format!(
            "PRESETS {}{} {}{}",
            self.command.label(),
            target,
            banks,
            status
        )
    }

    pub(crate) fn chooser_text(&self) -> Option<String> {
        let chooser = self.chooser.as_ref()?;
        let mut lines = vec![format!("Resolve slot {}", chooser.index.code())];
        for (index, candidate) in chooser.candidates.iter().enumerate() {
            let marker = if index == chooser.selected { '>' } else { ' ' };
            lines.push(format!(
                "{} {} {}",
                marker, candidate.file.saved_at_unix_ms, candidate.file.summary
            ));
        }
        lines.push("Up/Down choose  Enter keep  Esc cancel".to_string());
        Some(lines.join("\n"))
    }
}
impl PresetBrowserState {
    fn bank_occupancy(&self, bank: u8) -> String {
        let mut occupancy = String::with_capacity(10);
        for slot in 0_u8..10 {
            let count = self
                .records
                .iter()
                .filter(|record| record.file.assignment == Some(PresetIndex { bank, slot }))
                .count();
            let symbol = match count {
                0 => '.',
                1 => char::from(b'0' + slot),
                _ => '!',
            };
            occupancy.push(symbol);
        }
        occupancy
    }

    fn activate(&mut self) -> Result<(), String> {
        self.active = true;
        self.command = PresetCommand::Load;
        self.first_digit = None;
        self.chooser = None;
        self.status_message.clear();
        self.refresh()
    }

    fn deactivate(&mut self) {
        self.active = false;
        self.command = PresetCommand::Load;
        self.first_digit = None;
        self.chooser = None;
        self.status_message.clear();
    }

    fn arm_save(&mut self) {
        self.command = PresetCommand::Save;
        self.first_digit = None;
        self.chooser = None;
        self.status_message = "type bank+slot".to_string();
    }

    fn arm_free(&mut self) {
        self.command = PresetCommand::Free;
        self.first_digit = None;
        self.chooser = None;
        self.status_message = "type bank+slot".to_string();
    }

    fn cancel_pending(&mut self) {
        self.command = PresetCommand::Load;
        self.first_digit = None;
        self.chooser = None;
        self.status_message = "cancelled".to_string();
    }

    fn push_digit(&mut self, digit: u8) -> Option<PresetIndex> {
        if let Some(bank) = self.first_digit.take() {
            Some(PresetIndex { bank, slot: digit })
        } else {
            self.first_digit = Some(digit);
            self.status_message = format!("{}_", digit);
            None
        }
    }

    fn records_for_index(&self, index: PresetIndex) -> Vec<PresetRecord> {
        let mut records = self
            .records
            .iter()
            .filter(|record| record.file.assignment == Some(index))
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| right.file.saved_at_unix_ms.cmp(&left.file.saved_at_unix_ms));
        records
    }

    fn start_collision_resolution(&mut self, index: PresetIndex, load_after_resolution: bool) {
        let candidates = self.records_for_index(index);
        self.chooser = Some(CollisionResolutionState {
            index,
            selected: 0,
            candidates,
            load_after_resolution,
        });
        self.status_message = format!("resolve {}", index.code());
    }

    fn refresh(&mut self) -> Result<(), String> {
        self.records = load_preset_records()?;
        Ok(())
    }
}

impl CameraRigSnapshot {
    fn capture(camera_rig: &CameraRig) -> Self {
        Self {
            orientation: quat_to_array(camera_rig.orientation),
            angular_velocity: vec3_to_array(camera_rig.angular_velocity),
            distance: camera_rig.distance,
            zoom_velocity: camera_rig.zoom_velocity,
        }
    }

    fn to_runtime(&self) -> CameraRig {
        CameraRig {
            orientation: quat_from_array(self.orientation),
            angular_velocity: vec3_from_array(self.angular_velocity),
            distance: self.distance,
            zoom_velocity: self.zoom_velocity,
        }
    }
}

impl GenerationSnapshot {
    fn capture(generation_state: &GenerationState) -> Self {
        Self {
            selected_kind: generation_state.selected_kind,
            scale_ratio: generation_state.scale_ratio,
            twist_per_vertex_radians: generation_state.twist_per_vertex_radians,
            vertex_offset_ratio: generation_state.vertex_offset_ratio,
            vertex_spawn_exclusion_probability: generation_state.vertex_spawn_exclusion_probability,
            nodes: generation_state
                .nodes
                .iter()
                .map(PolyhedronNodeSnapshot::capture)
                .collect(),
        }
    }

    fn to_runtime(&self) -> Result<GenerationState, String> {
        let nodes = self
            .nodes
            .iter()
            .map(PolyhedronNodeSnapshot::to_runtime)
            .collect::<Result<Vec<_>, _>>()?;
        if nodes.is_empty() {
            return Err("Preset scene has no polyhedron nodes.".to_string());
        }
        Ok(GenerationState {
            nodes,
            selected_kind: self.selected_kind,
            scale_ratio: self.scale_ratio,
            twist_per_vertex_radians: self.twist_per_vertex_radians,
            vertex_offset_ratio: self.vertex_offset_ratio,
            vertex_spawn_exclusion_probability: self.vertex_spawn_exclusion_probability,
            spawn_hold: default(),
            twist_decrease_hold: default(),
            twist_increase_hold: default(),
            vertex_offset_decrease_hold: default(),
            vertex_offset_increase_hold: default(),
            vertex_exclusion_decrease_hold: default(),
            vertex_exclusion_increase_hold: default(),
        })
    }
}

impl MaterialRuntimeSnapshot {
    fn capture(material_state: &MaterialState) -> Self {
        Self {
            opacity: material_state.opacity,
        }
    }
}

impl PolyhedronNodeSnapshot {
    fn capture(node: &PolyhedronNode) -> Self {
        Self {
            kind: node.kind,
            level: node.level,
            center: vec3_to_array(node.center),
            rotation: quat_to_array(node.rotation),
            scale: node.scale,
            radius: node.radius,
            occupied_vertices: node.occupied_vertices.clone(),
            origin: NodeOriginSnapshot::capture(node.origin),
        }
    }

    fn to_runtime(&self) -> Result<PolyhedronNode, String> {
        Ok(PolyhedronNode {
            kind: self.kind,
            level: self.level,
            center: vec3_from_array(self.center),
            rotation: quat_from_array(self.rotation),
            scale: self.scale,
            radius: self.radius,
            occupied_vertices: self.occupied_vertices.clone(),
            origin: self.origin.to_runtime()?,
        })
    }
}

impl NodeOriginSnapshot {
    fn capture(origin: NodeOrigin) -> Self {
        match origin {
            NodeOrigin::Root => Self::Root,
            NodeOrigin::Child {
                parent_index,
                vertex_index,
            } => Self::Child {
                parent_index,
                vertex_index,
            },
        }
    }

    fn to_runtime(&self) -> Result<NodeOrigin, String> {
        Ok(match self {
            Self::Root => NodeOrigin::Root,
            Self::Child {
                parent_index,
                vertex_index,
            } => NodeOrigin::Child {
                parent_index: *parent_index,
                vertex_index: *vertex_index,
            },
        })
    }
}

impl ScenePresetSnapshot {
    fn capture(
        app_config: &AppConfig,
        camera_rig: &CameraRig,
        generation_state: &GenerationState,
        material_state: &MaterialState,
        effect_tuner: &EffectTunerState,
    ) -> Self {
        Self {
            rendering: app_config.rendering.clone(),
            lighting: app_config.lighting.clone(),
            materials: app_config.materials.clone(),
            camera: CameraRigSnapshot::capture(camera_rig),
            generation: GenerationSnapshot::capture(generation_state),
            material_state: MaterialRuntimeSnapshot::capture(material_state),
            effects: effect_tuner.runtime_snapshot(),
        }
    }

    fn summary(&self) -> String {
        let root_kind = self
            .generation
            .nodes
            .first()
            .map(|node| format!("{:?}", node.kind))
            .unwrap_or_else(|| "Unknown".to_string());
        format!("{} root, {} nodes", root_kind, self.generation.nodes.len())
    }

    fn prepare_runtime(&self) -> Result<PreparedScenePreset, String> {
        Ok(PreparedScenePreset {
            rendering: self.rendering.clone(),
            lighting: self.lighting.clone(),
            materials: self.materials.clone(),
            camera_rig: self.camera.to_runtime(),
            generation: self.generation.to_runtime()?,
            material_opacity: self.material_state.opacity.clamp(0.0, 1.0),
            effects: self.effects.clone(),
        })
    }
}

fn vec3_to_array(vector: Vec3) -> [f32; 3] {
    [vector.x, vector.y, vector.z]
}

fn vec3_from_array(vector: [f32; 3]) -> Vec3 {
    Vec3::new(vector[0], vector[1], vector[2])
}

fn quat_to_array(quat: Quat) -> [f32; 4] {
    [quat.x, quat.y, quat.z, quat.w]
}

fn quat_from_array(quat: [f32; 4]) -> Quat {
    Quat::from_xyzw(quat[0], quat[1], quat[2], quat[3]).normalize()
}

impl ScenePresetFile {
    fn new(index: PresetIndex, scene: ScenePresetSnapshot) -> Self {
        let saved_at_unix_ms = current_unix_ms();
        let summary = scene.summary();
        Self {
            format_version: PRESET_FORMAT_VERSION,
            id: format!("preset-{saved_at_unix_ms}"),
            saved_at_unix_ms,
            summary,
            assignment: Some(index),
            scene,
        }
    }
}

pub(crate) fn preset_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut preset_browser: ResMut<PresetBrowserState>,
    mut scene: SceneMutationAccess,
) {
    if keys.just_pressed(KeyCode::F3) {
        if preset_browser.active {
            preset_browser.deactivate();
            println!("Scene preset mode closed.");
        } else {
            match preset_browser.activate() {
                Ok(()) => println!("Scene preset mode open. Type two digits to recall a slot."),
                Err(error) => eprintln!("{error}"),
            }
        }
        return;
    }

    if !preset_browser.active {
        return;
    }

    if let Some(chooser) = preset_browser.chooser.as_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            preset_browser.cancel_pending();
            return;
        }
        if keys.just_pressed(KeyCode::ArrowUp) && chooser.selected > 0 {
            chooser.selected -= 1;
        }
        if keys.just_pressed(KeyCode::ArrowDown) && chooser.selected + 1 < chooser.candidates.len()
        {
            chooser.selected += 1;
        }
        if keys.just_pressed(KeyCode::Enter) {
            match resolve_collision(&mut preset_browser, &mut scene) {
                Ok(Some(message)) => println!("{message}"),
                Ok(None) => {}
                Err(error) => eprintln!("{error}"),
            }
        }
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        preset_browser.cancel_pending();
        return;
    }

    if keys.just_pressed(KeyCode::KeyS) {
        preset_browser.arm_save();
        return;
    }

    if keys.just_pressed(KeyCode::Delete) {
        preset_browser.arm_free();
        return;
    }

    let Some(digit) = just_pressed_digit(&keys) else {
        return;
    };
    let Some(index) = preset_browser.push_digit(digit) else {
        return;
    };

    let result = match preset_browser.command {
        PresetCommand::Load => load_assigned_preset(&mut preset_browser, index, &mut scene),
        PresetCommand::Save => save_scene_preset(
            &mut preset_browser,
            index,
            &scene.app_config,
            &scene.camera_rig,
            &scene.generation_state,
            &scene.material_state,
            &scene.effect_tuner,
        ),
        PresetCommand::Free => free_assigned_slot(&mut preset_browser, index),
    };

    match result {
        Ok(Some(message)) => println!("{message}"),
        Ok(None) => {}
        Err(error) => eprintln!("{error}"),
    }
}

fn load_assigned_preset(
    preset_browser: &mut PresetBrowserState,
    index: PresetIndex,
    scene: &mut SceneMutationAccess<'_, '_>,
) -> Result<Option<String>, String> {
    let records = preset_browser.records_for_index(index);
    if records.is_empty() {
        preset_browser.command = PresetCommand::Load;
        preset_browser.first_digit = None;
        preset_browser.status_message = format!("slot {} is empty", index.code());
        return Ok(None);
    }

    if records.len() > 1 {
        preset_browser.command = PresetCommand::Load;
        preset_browser.first_digit = None;
        preset_browser.start_collision_resolution(index, true);
        return Ok(Some(format!(
            "Slot {} has multiple assigned presets. Resolve the chooser.",
            index.code()
        )));
    }

    let record = &records[0];
    apply_scene_preset(&record.file.scene, scene)?;
    let message = format!(
        "Loaded scene preset {}: {}",
        index.code(),
        record.file.summary
    );
    preset_browser.command = PresetCommand::Load;
    preset_browser.first_digit = None;
    preset_browser.status_message = message.clone();
    Ok(Some(message))
}

fn save_scene_preset(
    preset_browser: &mut PresetBrowserState,
    index: PresetIndex,
    app_config: &AppConfig,
    camera_rig: &CameraRig,
    generation_state: &GenerationState,
    material_state: &MaterialState,
    effect_tuner: &EffectTunerState,
) -> Result<Option<String>, String> {
    let scene = ScenePresetSnapshot::capture(
        app_config,
        camera_rig,
        generation_state,
        material_state,
        effect_tuner,
    );
    let file = ScenePresetFile::new(index, scene.clone());
    let path = unique_preset_path(scene.file_slug().as_str())?;
    write_preset_file(&path, &file)?;
    preset_browser.refresh()?;

    if preset_browser.records_for_index(index).len() > 1 {
        preset_browser.command = PresetCommand::Load;
        preset_browser.first_digit = None;
        preset_browser.start_collision_resolution(index, false);
        return Ok(Some(format!(
            "Stored a new scene preset in slot {}. Resolve which preset stays assigned.",
            index.code()
        )));
    }

    let message = format!("Stored scene preset {}: {}", index.code(), file.summary);
    preset_browser.command = PresetCommand::Load;
    preset_browser.first_digit = None;
    preset_browser.status_message = message.clone();
    Ok(Some(message))
}

fn free_assigned_slot(
    preset_browser: &mut PresetBrowserState,
    index: PresetIndex,
) -> Result<Option<String>, String> {
    let records = preset_browser.records_for_index(index);
    if records.is_empty() {
        preset_browser.command = PresetCommand::Load;
        preset_browser.first_digit = None;
        preset_browser.status_message = format!("slot {} is already empty", index.code());
        return Ok(None);
    }

    for mut record in records {
        record.file.assignment = None;
        write_preset_file(&record.path, &record.file)?;
    }

    preset_browser.refresh()?;
    let message = format!("Freed scene preset slot {}.", index.code());
    preset_browser.command = PresetCommand::Load;
    preset_browser.first_digit = None;
    preset_browser.status_message = message.clone();
    Ok(Some(message))
}

fn resolve_collision(
    preset_browser: &mut PresetBrowserState,
    scene: &mut SceneMutationAccess<'_, '_>,
) -> Result<Option<String>, String> {
    let Some(chooser) = preset_browser.chooser.take() else {
        return Ok(None);
    };
    let Some(chosen) = chooser.candidates.get(chooser.selected).cloned() else {
        preset_browser.status_message = "chooser selection was invalid".to_string();
        return Ok(None);
    };

    for (candidate_index, mut candidate) in chooser.candidates.into_iter().enumerate() {
        candidate.file.assignment = if candidate_index == chooser.selected {
            Some(chooser.index)
        } else {
            None
        };
        write_preset_file(&candidate.path, &candidate.file)?;
    }

    preset_browser.refresh()?;

    if chooser.load_after_resolution {
        apply_scene_preset(&chosen.file.scene, scene)?;
    }

    let message = format!(
        "Slot {} now points to {}.",
        chooser.index.code(),
        chosen.file.summary
    );
    preset_browser.command = PresetCommand::Load;
    preset_browser.first_digit = None;
    preset_browser.status_message = message.clone();
    Ok(Some(message))
}

fn apply_scene_preset(
    scene: &ScenePresetSnapshot,
    runtime: &mut SceneMutationAccess<'_, '_>,
) -> Result<(), String> {
    let prepared = scene.prepare_runtime()?;

    runtime.app_config.rendering = prepared.rendering;
    runtime.app_config.lighting = prepared.lighting;
    runtime.app_config.materials = prepared.materials;

    runtime.clear_color.0 = runtime.app_config.rendering.clear_color();
    runtime.ambient_light.color = runtime.app_config.rendering.ambient_light_color();
    runtime.ambient_light.brightness = runtime.app_config.rendering.ambient_light_brightness;

    for (mut light, mut transform) in runtime.directional_lights.iter_mut() {
        light.color = runtime.app_config.lighting.directional.color();
        light.illuminance = runtime.app_config.lighting.directional.illuminance;
        light.shadows_enabled = runtime.app_config.lighting.directional.shadows_enabled;
        *transform =
            Transform::from_translation(runtime.app_config.lighting.directional.translation())
                .looking_at(runtime.app_config.lighting.directional.look_at(), Vec3::Y);
    }

    for (mut light, mut transform) in runtime.point_lights.iter_mut() {
        light.color = runtime.app_config.lighting.point.color();
        light.intensity = runtime.app_config.lighting.point.intensity;
        light.range = runtime.app_config.lighting.point.range;
        light.shadows_enabled = runtime.app_config.lighting.point.shadows_enabled;
        *transform = Transform::from_translation(runtime.app_config.lighting.point.translation());
    }

    *runtime.camera_rig = prepared.camera_rig;
    runtime
        .effect_tuner
        .apply_runtime_snapshot(&prepared.effects);
    *runtime.generation_state = prepared.generation;
    runtime.material_state.opacity = prepared.material_opacity;

    for entity in runtime.polyhedron_entities.iter() {
        runtime.commands.entity(entity).despawn();
    }

    for (node_index, node) in runtime.generation_state.nodes.iter().enumerate() {
        spawn_polyhedron_entity(
            &mut runtime.commands,
            &mut runtime.materials,
            runtime.shape_assets.mesh(node.kind),
            node,
            &runtime.app_config.materials,
            runtime.material_state.opacity,
            node_index,
        );
    }

    Ok(())
}

fn load_preset_records() -> Result<Vec<PresetRecord>, String> {
    let preset_dir = ensure_preset_dir()?;
    let mut records = Vec::new();

    let entries = fs::read_dir(&preset_dir)
        .map_err(|error| format!("Could not read {}: {error}", preset_dir.display()))?;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                eprintln!("Skipping preset directory entry: {error}");
                continue;
            }
        };
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("toml") {
            continue;
        }
        match read_preset_file(&path) {
            Ok(file) => records.push(PresetRecord { path, file }),
            Err(error) => eprintln!("{error}"),
        }
    }

    records.sort_by(|left, right| right.file.saved_at_unix_ms.cmp(&left.file.saved_at_unix_ms));
    Ok(records)
}

fn ensure_preset_dir() -> Result<PathBuf, String> {
    let path = Path::new(PRESET_DIR);
    fs::create_dir_all(path)
        .map_err(|error| format!("Could not create {}: {error}", path.display()))?;
    Ok(path.to_path_buf())
}

fn read_preset_file(path: &Path) -> Result<ScenePresetFile, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    let file: ScenePresetFile = toml::from_str(&contents)
        .map_err(|error| format!("Could not parse {}: {error}", path.display()))?;
    if file.format_version != PRESET_FORMAT_VERSION {
        return Err(format!(
            "Skipping {}: preset format {} is unsupported.",
            path.display(),
            file.format_version
        ));
    }
    Ok(file)
}

fn write_preset_file(path: &Path, file: &ScenePresetFile) -> Result<(), String> {
    let contents = toml::to_string_pretty(file)
        .map_err(|error| format!("Could not serialize {}: {error}", path.display()))?;
    fs::write(path, contents)
        .map_err(|error| format!("Could not write {}: {error}", path.display()))
}

fn unique_preset_path(file_slug: &str) -> Result<PathBuf, String> {
    let preset_dir = ensure_preset_dir()?;
    let timestamp = current_unix_ms();
    let base = format!("scene-preset-{timestamp}-{file_slug}");
    let mut candidate = preset_dir.join(format!("{base}.toml"));
    let mut suffix = 1_u32;
    while candidate.exists() {
        candidate = preset_dir.join(format!("{base}-{suffix}.toml"));
        suffix += 1;
    }
    Ok(candidate)
}

fn current_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn just_pressed_digit(keys: &ButtonInput<KeyCode>) -> Option<u8> {
    const DIGIT_KEYS: [(KeyCode, u8); 10] = [
        (KeyCode::Digit0, 0),
        (KeyCode::Digit1, 1),
        (KeyCode::Digit2, 2),
        (KeyCode::Digit3, 3),
        (KeyCode::Digit4, 4),
        (KeyCode::Digit5, 5),
        (KeyCode::Digit6, 6),
        (KeyCode::Digit7, 7),
        (KeyCode::Digit8, 8),
        (KeyCode::Digit9, 9),
    ];
    const NUMPAD_KEYS: [(KeyCode, u8); 10] = [
        (KeyCode::Numpad0, 0),
        (KeyCode::Numpad1, 1),
        (KeyCode::Numpad2, 2),
        (KeyCode::Numpad3, 3),
        (KeyCode::Numpad4, 4),
        (KeyCode::Numpad5, 5),
        (KeyCode::Numpad6, 6),
        (KeyCode::Numpad7, 7),
        (KeyCode::Numpad8, 8),
        (KeyCode::Numpad9, 9),
    ];

    DIGIT_KEYS
        .into_iter()
        .chain(NUMPAD_KEYS)
        .find_map(|(key_code, digit)| keys.just_pressed(key_code).then_some(digit))
}

#[cfg(test)]
mod tests {
    use super::{PresetBrowserState, PresetIndex, PresetRecord, ScenePresetFile};

    #[test]
    fn bank_occupancy_marks_collisions() {
        let mut state = PresetBrowserState::default();
        state.records = vec![
            PresetRecord {
                path: "one.toml".into(),
                file: ScenePresetFile {
                    format_version: 1,
                    id: "one".to_string(),
                    saved_at_unix_ms: 1,
                    summary: "a".to_string(),
                    assignment: Some(PresetIndex { bank: 2, slot: 3 }),
                    scene: dummy_scene(),
                },
            },
            PresetRecord {
                path: "two.toml".into(),
                file: ScenePresetFile {
                    format_version: 1,
                    id: "two".to_string(),
                    saved_at_unix_ms: 2,
                    summary: "b".to_string(),
                    assignment: Some(PresetIndex { bank: 2, slot: 3 }),
                    scene: dummy_scene(),
                },
            },
        ];

        assert_eq!(state.bank_occupancy(2).chars().nth(3), Some('!'));
    }

    fn dummy_scene() -> super::ScenePresetSnapshot {
        super::ScenePresetSnapshot {
            rendering: crate::config::RenderingConfig::default(),
            lighting: crate::config::LightingConfig::default(),
            materials: crate::config::MaterialConfig::default(),
            camera: super::CameraRigSnapshot {
                orientation: [0.0, 0.0, 0.0, 1.0],
                angular_velocity: [0.0, 0.0, 0.0],
                distance: 1.0,
                zoom_velocity: 0.0,
            },
            generation: super::GenerationSnapshot {
                selected_kind: crate::polyhedra::PolyhedronKind::Cube,
                scale_ratio: 0.5,
                twist_per_vertex_radians: 0.0,
                vertex_offset_ratio: 0.0,
                vertex_spawn_exclusion_probability: 0.0,
                nodes: vec![super::PolyhedronNodeSnapshot {
                    kind: crate::polyhedra::PolyhedronKind::Cube,
                    level: 0,
                    center: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: 1.0,
                    radius: 1.0,
                    occupied_vertices: vec![],
                    origin: super::NodeOriginSnapshot::Root,
                }],
            },
            material_state: super::MaterialRuntimeSnapshot { opacity: 1.0 },
            effects: crate::effect_tuner::EffectRuntimeSnapshot {
                current: crate::config::EffectsConfig::default(),
                lfos: Vec::new(),
            },
        }
    }
}

impl ScenePresetSnapshot {
    fn file_slug(&self) -> String {
        self.generation
            .nodes
            .first()
            .map(|node| format!("{:?}", node.kind).to_ascii_lowercase())
            .unwrap_or_else(|| "scene".to_string())
    }
}
