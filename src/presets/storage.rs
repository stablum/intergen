use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::browser::PresetIndex;
use crate::scene_snapshot::SceneStateSnapshot;

const PRESET_DIR: &str = "scene-presets";
const PRESET_FORMAT_VERSION: u32 = 1;

#[derive(Clone)]
pub(super) struct PresetRecord {
    pub(super) path: PathBuf,
    pub(super) file: ScenePresetFile,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct ScenePresetFile {
    pub(super) format_version: u32,
    pub(super) id: String,
    pub(super) saved_at_unix_ms: u64,
    pub(super) summary: String,
    pub(super) assignment: Option<PresetIndex>,
    pub(super) scene: SceneStateSnapshot,
}

impl ScenePresetFile {
    pub(super) fn new(index: PresetIndex, scene: SceneStateSnapshot) -> Self {
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

pub(super) fn load_preset_records() -> Result<Vec<PresetRecord>, String> {
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

pub(super) fn read_preset_file(path: &Path) -> Result<ScenePresetFile, String> {
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

pub(super) fn write_preset_file(path: &Path, file: &ScenePresetFile) -> Result<(), String> {
    let contents = toml::to_string_pretty(file)
        .map_err(|error| format!("Could not serialize {}: {error}", path.display()))?;
    fs::write(path, contents)
        .map_err(|error| format!("Could not write {}: {error}", path.display()))
}

pub(super) fn unique_preset_path(file_slug: &str) -> Result<PathBuf, String> {
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

fn ensure_preset_dir() -> Result<PathBuf, String> {
    let path = Path::new(PRESET_DIR);
    fs::create_dir_all(path)
        .map_err(|error| format!("Could not create {}: {error}", path.display()))?;
    Ok(path.to_path_buf())
}

fn current_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
