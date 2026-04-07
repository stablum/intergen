use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use super::browser::PresetIndex;
use crate::scene_snapshot::SceneStateSnapshot;
use crate::timestamp::{current_unix_timestamp, format_utc_timestamp};

const PRESET_DIR: &str = "scene-presets";
const PRESET_FORMAT_VERSION: u32 = 1;

#[derive(Clone)]
pub(super) struct PresetRecord {
    pub(super) path: PathBuf,
    pub(super) stamp: PresetFileStamp,
    pub(super) file: ScenePresetFile,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PresetFileStamp {
    pub(super) len: u64,
    pub(super) modified: Option<SystemTime>,
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
    let mut records = Vec::new();

    for path in preset_paths()? {
        match read_preset_record(&path) {
            Ok(record) => records.push(record),
            Err(error) => eprintln!("{error}"),
        }
    }

    sort_preset_records(&mut records);
    Ok(records)
}

pub(super) fn sync_preset_records(records: &mut Vec<PresetRecord>) -> Result<(), String> {
    let mut cached_records = records
        .drain(..)
        .map(|record| (record.path.clone(), record))
        .collect::<HashMap<_, _>>();
    let mut synced_records = Vec::new();

    for path in preset_paths()? {
        let stamp = match read_preset_file_stamp(&path) {
            Ok(stamp) => stamp,
            Err(error) => {
                eprintln!("{error}");
                cached_records.remove(&path);
                continue;
            }
        };

        match cached_records.remove(&path) {
            Some(record) if record.stamp == stamp => synced_records.push(record),
            _ => match read_preset_record_with_stamp(&path, stamp) {
                Ok(record) => synced_records.push(record),
                Err(error) => eprintln!("{error}"),
            },
        }
    }

    sort_preset_records(&mut synced_records);
    *records = synced_records;
    Ok(())
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

pub(super) fn preset_record_from_file(
    path: PathBuf,
    file: ScenePresetFile,
) -> Result<PresetRecord, String> {
    let stamp = read_preset_file_stamp(&path)?;
    Ok(PresetRecord { path, stamp, file })
}

pub(super) fn write_preset_file(path: &Path, file: &ScenePresetFile) -> Result<(), String> {
    let contents = toml::to_string_pretty(file)
        .map_err(|error| format!("Could not serialize {}: {error}", path.display()))?;
    fs::write(path, contents)
        .map_err(|error| format!("Could not write {}: {error}", path.display()))
}

pub(super) fn unique_preset_path(file_slug: &str) -> Result<PathBuf, String> {
    let preset_dir = ensure_preset_dir()?;
    let timestamp = current_unix_timestamp();
    let timestamp = format!(
        "{}-{:03}",
        format_utc_timestamp(timestamp),
        timestamp.subsec_millis()
    );
    let mut suffix = 0_u32;
    let mut candidate = preset_dir.join(format!(
        "scene-preset-{timestamp}-{suffix:04}-{file_slug}.toml"
    ));
    while candidate.exists() {
        suffix += 1;
        candidate = preset_dir.join(format!(
            "scene-preset-{timestamp}-{suffix:04}-{file_slug}.toml"
        ));
    }
    Ok(candidate)
}

pub(super) fn sort_preset_records(records: &mut [PresetRecord]) {
    records.sort_by(|left, right| right.file.saved_at_unix_ms.cmp(&left.file.saved_at_unix_ms));
}

fn ensure_preset_dir() -> Result<PathBuf, String> {
    let path = Path::new(PRESET_DIR);
    fs::create_dir_all(path)
        .map_err(|error| format!("Could not create {}: {error}", path.display()))?;
    Ok(path.to_path_buf())
}

fn preset_paths() -> Result<Vec<PathBuf>, String> {
    let preset_dir = ensure_preset_dir()?;
    let mut paths = Vec::new();

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
        if path.extension().and_then(|value| value.to_str()) == Some("toml") {
            paths.push(path);
        }
    }

    Ok(paths)
}

fn read_preset_record(path: &Path) -> Result<PresetRecord, String> {
    let stamp = read_preset_file_stamp(path)?;
    read_preset_record_with_stamp(path, stamp)
}

fn read_preset_record_with_stamp(
    path: &Path,
    stamp: PresetFileStamp,
) -> Result<PresetRecord, String> {
    let file = read_preset_file(path)?;
    Ok(PresetRecord {
        path: path.to_path_buf(),
        stamp,
        file,
    })
}

fn read_preset_file_stamp(path: &Path) -> Result<PresetFileStamp, String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Could not stat {}: {error}", path.display()))?;
    Ok(PresetFileStamp {
        len: metadata.len(),
        modified: metadata.modified().ok(),
    })
}

fn current_unix_ms() -> u64 {
    current_unix_timestamp().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::time::Duration;

    use crate::config::{EffectsConfig, LightingConfig, MaterialConfig, RenderingConfig};
    use crate::effect_tuner::{EffectRuntimeSnapshot, EffectTunerState, ParameterLfo};
    use crate::scene_snapshot::{
        CameraRigSnapshot, GenerationSnapshot, MaterialRuntimeSnapshot, SceneStateSnapshot,
    };
    use crate::timestamp::format_utc_timestamp;

    use super::{ScenePresetFile, read_preset_file, write_preset_file};
    use crate::presets::browser::PresetIndex;

    #[derive(Clone, Debug, serde::Deserialize)]
    struct LegacyEffectRuntimeSnapshot {
        current: EffectsConfig,
        lfos: Vec<ParameterLfo>,
    }

    #[derive(Clone, Debug, serde::Deserialize)]
    struct LegacySceneStateSnapshot {
        rendering: RenderingConfig,
        lighting: LightingConfig,
        materials: MaterialConfig,
        camera: CameraRigSnapshot,
        generation: GenerationSnapshot,
        material_state: MaterialRuntimeSnapshot,
        effects: LegacyEffectRuntimeSnapshot,
    }

    impl LegacySceneStateSnapshot {
        fn into_current(self) -> SceneStateSnapshot {
            SceneStateSnapshot {
                rendering: self.rendering,
                lighting: self.lighting,
                materials: self.materials,
                camera: self.camera,
                generation: self.generation,
                material_state: self.material_state,
                effects: EffectRuntimeSnapshot::from_positional_lfos(
                    self.effects.current,
                    self.effects.lfos,
                ),
            }
        }
    }

    #[derive(Clone, Debug, serde::Deserialize)]
    struct LegacyScenePresetFile {
        format_version: u32,
        id: String,
        saved_at_unix_ms: u64,
        summary: String,
        assignment: Option<PresetIndex>,
        scene: LegacySceneStateSnapshot,
    }

    impl LegacyScenePresetFile {
        fn into_current(self) -> ScenePresetFile {
            ScenePresetFile {
                format_version: self.format_version,
                id: self.id,
                saved_at_unix_ms: self.saved_at_unix_ms,
                summary: self.summary,
                assignment: self.assignment,
                scene: self.scene.into_current(),
            }
        }
    }

    fn checked_in_preset_paths() -> Vec<std::path::PathBuf> {
        let mut preset_paths = super::preset_paths().expect("should read checked-in presets");
        preset_paths.sort();
        assert!(
            !preset_paths.is_empty(),
            "expected at least one checked-in scene preset"
        );
        preset_paths
    }

    fn raw_effect_lfo_entries(contents: &str) -> Vec<toml::Value> {
        let value: toml::Value = toml::from_str(contents).expect("preset should parse as raw toml");
        value
            .get("scene")
            .and_then(|value| value.get("effects"))
            .and_then(|value| value.get("lfos"))
            .and_then(toml::Value::as_array)
            .cloned()
            .expect("preset should store effect LFO entries in an array")
    }

    fn raw_runtime_snapshot_lfo_entries(contents: &str) -> Vec<toml::Value> {
        let value: toml::Value =
            toml::from_str(contents).expect("runtime snapshot should parse as raw toml");
        value
            .get("lfos")
            .and_then(toml::Value::as_array)
            .cloned()
            .expect("runtime snapshot should store LFO entries in an array")
    }

    fn lfo_entry_by_parameter<'a>(entries: &'a [toml::Value], parameter: &str) -> &'a toml::Value {
        entries
            .iter()
            .find(|entry| entry.get("parameter").and_then(toml::Value::as_str) == Some(parameter))
            .unwrap_or_else(|| panic!("missing LFO entry for parameter '{parameter}'"))
    }

    #[test]
    fn preset_filename_timestamp_matches_screenshot_style() {
        let timestamp = Duration::new(1_741_018_296, 45_000_000);
        let filename = format!(
            "scene-preset-{}-{:03}-{:04}-example.toml",
            format_utc_timestamp(timestamp),
            timestamp.subsec_millis(),
            0
        );

        assert_eq!(
            filename,
            "scene-preset-2025-03-03_16-11-36-045-0000-example.toml"
        );
    }

    #[test]
    fn checked_in_scene_presets_still_load_and_apply_effect_snapshots() {
        let preset_dir = Path::new("scene-presets");
        assert!(
            preset_dir.exists(),
            "expected checked-in scene-presets directory to exist"
        );
        let default_snapshot =
            EffectTunerState::from_config(&EffectsConfig::default()).runtime_snapshot();
        let current_lfo_layout_len = EffectTunerState::from_config(&EffectsConfig::default())
            .runtime_snapshot()
            .lfos
            .len();
        let checked_in_pre_scale_and_exclusion_len = current_lfo_layout_len - 2;
        let default_encoded =
            toml::to_string(&default_snapshot).expect("default runtime snapshot should serialize");
        let default_lfos = raw_runtime_snapshot_lfo_entries(&default_encoded);

        for path in checked_in_preset_paths() {
            let contents = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("{} should read: {error}", path.display()));
            let stored_lfos = raw_effect_lfo_entries(&contents);
            assert!(
                stored_lfos.len() == checked_in_pre_scale_and_exclusion_len
                    || stored_lfos.len() == current_lfo_layout_len,
                "{} should store either the checked-in pre-scale/exclusion layout or the current keyed LFO layout",
                path.display()
            );
            for stored_lfo in stored_lfos {
                assert!(
                    stored_lfo
                        .get("parameter")
                        .and_then(toml::Value::as_str)
                        .is_some(),
                    "{} should store a stable parameter id for each LFO entry",
                    path.display()
                );
            }

            let file = read_preset_file(&path)
                .unwrap_or_else(|error| panic!("{} should parse: {error}", path.display()));
            file.scene
                .prepare_runtime()
                .unwrap_or_else(|error| panic!("{} should prepare: {error}", path.display()));

            let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
            effect_tuner.apply_runtime_snapshot(&file.scene.effects);
            let restored = effect_tuner.runtime_snapshot();
            let restored_encoded =
                toml::to_string(&restored).expect("restored runtime snapshot should serialize");
            let restored_lfos = raw_runtime_snapshot_lfo_entries(&restored_encoded);

            assert_eq!(
                restored.lfos.len(),
                file.scene.effects.lfos.len(),
                "{} should restore the full keyed LFO layout",
                path.display()
            );

            for (restored_lfo, expected_lfo) in
                restored.lfos.iter().zip(file.scene.effects.lfos.iter())
            {
                assert_eq!(restored_lfo.enabled, expected_lfo.enabled);
                assert_eq!(restored_lfo.shape, expected_lfo.shape);
                assert_eq!(restored_lfo.amplitude, expected_lfo.amplitude);
                assert_eq!(restored_lfo.frequency_hz, expected_lfo.frequency_hz);
            }

            for parameter in [
                "generation.child_scale_ratio",
                "generation.child_spawn_exclusion_probability",
            ] {
                assert_eq!(
                    lfo_entry_by_parameter(&restored_lfos, parameter),
                    lfo_entry_by_parameter(&default_lfos, parameter),
                    "{} should default the newly added generation LFO slot '{}'",
                    path.display(),
                    parameter
                );
            }
        }
    }

    fn rewrite_checked_in_scene_presets_to_current_effect_format_impl() {
        for path in checked_in_preset_paths() {
            let legacy_contents = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("{} should read: {error}", path.display()));
            let legacy_file: LegacyScenePresetFile = toml::from_str(&legacy_contents)
                .unwrap_or_else(|error| {
                    panic!(
                        "{} should parse as a legacy preset: {error}",
                        path.display()
                    )
                });
            let expected_file = legacy_file.into_current();
            let expected_contents = toml::to_string_pretty(&expected_file)
                .unwrap_or_else(|error| panic!("{} should serialize: {error}", path.display()));
            let expected_value: toml::Value =
                toml::from_str(&expected_contents).unwrap_or_else(|error| {
                    panic!("{} expected output should parse: {error}", path.display())
                });

            write_preset_file(&path, &expected_file)
                .unwrap_or_else(|error| panic!("{} should rewrite: {error}", path.display()));

            let rewritten_contents = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("{} should reread: {error}", path.display()));
            let rewritten_value: toml::Value =
                toml::from_str(&rewritten_contents).unwrap_or_else(|error| {
                    panic!("{} rewritten output should parse: {error}", path.display())
                });
            assert_eq!(
                rewritten_value,
                expected_value,
                "{} should rewrite to the expected keyed format",
                path.display()
            );

            let rewritten_file = read_preset_file(&path).unwrap_or_else(|error| {
                panic!("{} should parse after rewrite: {error}", path.display())
            });
            rewritten_file
                .scene
                .prepare_runtime()
                .unwrap_or_else(|error| {
                    panic!(
                        "{} should still prepare after rewrite: {error}",
                        path.display()
                    )
                });

            let mut effect_tuner = EffectTunerState::from_config(&EffectsConfig::default());
            effect_tuner.apply_runtime_snapshot(&rewritten_file.scene.effects);
            let restored = effect_tuner.runtime_snapshot();
            assert_eq!(
                restored.lfos.len(),
                rewritten_file.scene.effects.lfos.len(),
                "{} should preserve the full runtime LFO layout after rewrite",
                path.display()
            );
            for (restored_lfo, expected_lfo) in restored
                .lfos
                .iter()
                .zip(rewritten_file.scene.effects.lfos.iter())
            {
                assert_eq!(restored_lfo.enabled, expected_lfo.enabled);
                assert_eq!(restored_lfo.shape, expected_lfo.shape);
                assert_eq!(restored_lfo.amplitude, expected_lfo.amplitude);
                assert_eq!(restored_lfo.frequency_hz, expected_lfo.frequency_hz);
            }
        }
    }

    #[test]
    #[ignore = "rewrites checked-in scene presets to the current keyed effect snapshot format"]
    fn rewrite_checked_in_scene_presets_to_current_effect_format() {
        rewrite_checked_in_scene_presets_to_current_effect_format_impl();
    }
}
