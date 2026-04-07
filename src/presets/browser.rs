use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::storage::{PresetRecord, load_preset_records, sort_preset_records, sync_preset_records};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub(crate) struct PresetIndex {
    pub(crate) bank: u8,
    pub(crate) slot: u8,
}

impl PresetIndex {
    pub(super) fn code(self) -> String {
        format!("{}{}", self.bank, self.slot)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PresetCommand {
    Load,
    Save,
    Free,
}

impl PresetCommand {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Save => "save",
            Self::Free => "free",
        }
    }
}

pub(crate) struct PresetStripSegments {
    pub(crate) command: &'static str,
    pub(crate) target: String,
    pub(crate) banks: Vec<PresetStripBankSegment>,
    pub(crate) status: String,
    pub(crate) emphasize_command: bool,
    pub(crate) emphasize_target: bool,
}

pub(crate) struct PresetStripBankSegment {
    pub(crate) bank: u8,
    pub(crate) label: String,
    pub(crate) prefix: String,
    pub(crate) selected_slot: String,
    pub(crate) suffix: String,
    pub(crate) emphasize_bank: bool,
    pub(crate) emphasize_selected_slot: bool,
}

#[derive(Clone)]
pub(super) struct CollisionResolutionState {
    pub(super) index: PresetIndex,
    pub(super) selected: usize,
    pub(super) candidates: Vec<PresetRecord>,
    pub(super) load_after_resolution: bool,
}

#[derive(Resource)]
pub(crate) struct PresetBrowserState {
    pub(super) command: PresetCommand,
    pub(super) first_digit: Option<u8>,
    pub(super) status_message: String,
    pub(super) records: Vec<PresetRecord>,
    pub(super) chooser: Option<CollisionResolutionState>,
    pub(super) highlighted_index: Option<PresetIndex>,
}

#[derive(Resource)]
pub(crate) struct AutomatedScenePresetLoad {
    pub(super) path: PathBuf,
}

impl AutomatedScenePresetLoad {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Default for PresetBrowserState {
    fn default() -> Self {
        Self {
            command: PresetCommand::Load,
            first_digit: None,
            status_message: String::new(),
            records: Vec::new(),
            chooser: None,
            highlighted_index: None,
        }
    }
}

impl PresetBrowserState {
    pub(crate) fn load_from_disk() -> Self {
        let mut state = Self::default();
        match load_preset_records() {
            Ok(records) => state.records = records,
            Err(error) => state.status_message = error,
        }
        state
    }

    pub(crate) fn chooser_visible(&self) -> bool {
        self.chooser.is_some()
    }

    pub(crate) fn strip_segments(&self) -> PresetStripSegments {
        let target = match self.first_digit {
            Some(bank) => format!(" {}_", bank),
            None => String::new(),
        };
        let banks = (0_u8..10)
            .map(|bank| self.strip_bank_segment(bank))
            .collect::<Vec<_>>();
        let status = if self.status_message.is_empty() {
            String::new()
        } else {
            format!(" | {}", self.status_message)
        };
        PresetStripSegments {
            command: self.command.label(),
            target,
            banks,
            status,
            emphasize_command: self.command == PresetCommand::Save,
            emphasize_target: self.first_digit.is_some(),
        }
    }

    fn strip_bank_segment(&self, bank: u8) -> PresetStripBankSegment {
        let occupancy = self.bank_occupancy(bank);
        let Some(highlighted_index) = self.highlighted_index.filter(|index| index.bank == bank)
        else {
            return PresetStripBankSegment {
                bank,
                label: bank.to_string(),
                prefix: format!("[{}]", occupancy),
                selected_slot: String::new(),
                suffix: String::new(),
                emphasize_bank: false,
                emphasize_selected_slot: false,
            };
        };

        let slot_index = highlighted_index.slot as usize;
        let prefix = format!("[{}", &occupancy[..slot_index]);
        let selected_slot = occupancy
            .as_bytes()
            .get(slot_index)
            .map(|character| (*character as char).to_string())
            .unwrap_or_default();
        let suffix = if slot_index < occupancy.len() {
            format!("{}]", &occupancy[slot_index + 1..])
        } else {
            "]".to_string()
        };

        PresetStripBankSegment {
            bank,
            label: bank.to_string(),
            prefix,
            selected_slot: selected_slot.clone(),
            suffix,
            emphasize_bank: true,
            emphasize_selected_slot: !selected_slot.is_empty(),
        }
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
        lines.push("Up/Down choose  Enter keep  Esc close".to_string());
        Some(lines.join("\n"))
    }

    pub(crate) fn open_page(&mut self) -> Result<(), String> {
        self.command = PresetCommand::Load;
        self.first_digit = None;
        self.chooser = None;
        self.status_message.clear();
        self.refresh()
    }

    pub(crate) fn close_page(&mut self) {
        self.command = PresetCommand::Load;
        self.first_digit = None;
        self.chooser = None;
        self.status_message.clear();
    }

    pub(super) fn bank_occupancy(&self, bank: u8) -> String {
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

    pub(super) fn arm_save(&mut self) {
        self.command = PresetCommand::Save;
        self.first_digit = None;
        self.chooser = None;
        self.status_message = "type bank+slot".to_string();
    }

    pub(super) fn arm_free(&mut self) {
        self.command = PresetCommand::Free;
        self.first_digit = None;
        self.chooser = None;
        self.status_message = "type bank+slot".to_string();
    }

    pub(super) fn push_digit(&mut self, digit: u8) -> Option<PresetIndex> {
        if let Some(bank) = self.first_digit.take() {
            Some(PresetIndex { bank, slot: digit })
        } else {
            self.first_digit = Some(digit);
            self.status_message = format!("{}_", digit);
            None
        }
    }

    pub(super) fn records_for_index(&self, index: PresetIndex) -> Vec<PresetRecord> {
        let mut records = self
            .records
            .iter()
            .filter(|record| record.file.assignment == Some(index))
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| right.file.saved_at_unix_ms.cmp(&left.file.saved_at_unix_ms));
        records
    }

    pub(super) fn start_collision_resolution(
        &mut self,
        index: PresetIndex,
        load_after_resolution: bool,
    ) {
        let candidates = self.records_for_index(index);
        self.chooser = Some(CollisionResolutionState {
            index,
            selected: 0,
            candidates,
            load_after_resolution,
        });
        self.status_message = format!("resolve {}", index.code());
    }

    pub(super) fn refresh(&mut self) -> Result<(), String> {
        sync_preset_records(&mut self.records)?;
        Ok(())
    }

    pub(super) fn upsert_record(&mut self, record: PresetRecord) {
        if let Some(index) = self
            .records
            .iter()
            .position(|existing| existing.path == record.path)
        {
            self.records[index] = record;
        } else {
            self.records.push(record);
        }
        sort_preset_records(&mut self.records);
    }

    pub(super) fn highlight_index(&mut self, index: PresetIndex) {
        self.highlighted_index = Some(index);
    }
}

#[cfg(test)]
mod tests {
    use super::{PresetBrowserState, PresetIndex};
    use crate::config::{EffectsConfig, LightingConfig, MaterialConfig, RenderingConfig};
    use crate::presets::storage::{PresetFileStamp, ScenePresetFile};
    use crate::scene_snapshot::{
        CameraRigSnapshot, GenerationSnapshot, MaterialRuntimeSnapshot, NodeOriginSnapshot,
        SceneStateSnapshot, ShapeNodeSnapshot,
    };
    use crate::shapes::{ShapeKind, SpawnAddMode, SpawnPlacementMode};

    #[test]
    fn bank_occupancy_marks_collisions() {
        let mut state = PresetBrowserState::default();
        state.records = vec![
            super::PresetRecord {
                path: "one.toml".into(),
                stamp: PresetFileStamp {
                    len: 0,
                    modified: None,
                },
                file: ScenePresetFile {
                    format_version: 1,
                    id: "one".to_string(),
                    saved_at_unix_ms: 1,
                    summary: "a".to_string(),
                    assignment: Some(PresetIndex { bank: 2, slot: 3 }),
                    scene: dummy_scene(),
                },
            },
            super::PresetRecord {
                path: "two.toml".into(),
                stamp: PresetFileStamp {
                    len: 0,
                    modified: None,
                },
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

    #[test]
    fn upsert_record_replaces_matching_path_without_duplication() {
        let mut state = PresetBrowserState::default();
        state.records = vec![record_with_saved_at("one.toml", 1)];

        state.upsert_record(record_with_saved_at("one.toml", 2));

        assert_eq!(state.records.len(), 1);
        assert_eq!(state.records[0].file.saved_at_unix_ms, 2);
    }

    #[test]
    fn upsert_record_keeps_newest_record_first() {
        let mut state = PresetBrowserState::default();
        state.records = vec![record_with_saved_at("old.toml", 1)];

        state.upsert_record(record_with_saved_at("new.toml", 2));

        assert_eq!(state.records[0].path, std::path::PathBuf::from("new.toml"));
        assert_eq!(state.records[1].path, std::path::PathBuf::from("old.toml"));
    }

    #[test]
    fn strip_segments_highlight_save_and_selected_target() {
        let mut state = PresetBrowserState::default();
        state.arm_save();
        state.first_digit = Some(4);

        let segments = state.strip_segments();

        assert_eq!(segments.command, "save");
        assert_eq!(segments.target, " 4_");
        assert!(segments.emphasize_command);
        assert!(segments.emphasize_target);
    }

    #[test]
    fn strip_segments_highlight_loaded_bank_and_slot() {
        let mut state = PresetBrowserState::default();
        state.records = vec![super::PresetRecord {
            path: "loaded.toml".into(),
            stamp: PresetFileStamp {
                len: 0,
                modified: None,
            },
            file: ScenePresetFile {
                format_version: 1,
                id: "loaded".to_string(),
                saved_at_unix_ms: 1,
                summary: "loaded".to_string(),
                assignment: Some(PresetIndex { bank: 4, slot: 2 }),
                scene: dummy_scene(),
            },
        }];
        state.highlight_index(PresetIndex { bank: 4, slot: 2 });

        let segments = state.strip_segments();
        let bank = segments
            .banks
            .iter()
            .find(|bank| bank.bank == 4)
            .expect("bank 4 should be present");

        assert!(bank.emphasize_bank);
        assert!(bank.emphasize_selected_slot);
        assert_eq!(bank.selected_slot, "2");
    }

    fn record_with_saved_at(path: &str, saved_at_unix_ms: u64) -> super::PresetRecord {
        super::PresetRecord {
            path: path.into(),
            stamp: PresetFileStamp {
                len: saved_at_unix_ms,
                modified: None,
            },
            file: ScenePresetFile {
                format_version: 1,
                id: path.to_string(),
                saved_at_unix_ms,
                summary: path.to_string(),
                assignment: None,
                scene: dummy_scene(),
            },
        }
    }

    fn dummy_scene() -> SceneStateSnapshot {
        SceneStateSnapshot {
            rendering: RenderingConfig::default(),
            lighting: LightingConfig::default(),
            materials: MaterialConfig::default(),
            camera: CameraRigSnapshot {
                orientation: [0.0, 0.0, 0.0, 1.0],
                angular_velocity: [0.0, 0.0, 0.0],
                distance: 1.0,
                zoom_velocity: 0.0,
            },
            generation: GenerationSnapshot {
                selected_shape_kind: ShapeKind::Cube,
                spawn_placement_mode: SpawnPlacementMode::Vertex,
                spawn_add_mode: SpawnAddMode::Single,
                scale_ratio: 0.5,
                twist_per_vertex_radians: 0.0,
                vertex_offset_ratio: 0.0,
                vertex_spawn_exclusion_probability: 0.0,
                nodes: vec![ShapeNodeSnapshot {
                    shape_kind: ShapeKind::Cube,
                    level: 0,
                    center: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: 1.0,
                    axis_scale: [1.0, 1.0, 1.0],
                    radius: 1.0,
                    occupied_vertices: vec![],
                    occupied_edges: vec![],
                    occupied_faces: vec![],
                    origin: NodeOriginSnapshot::Root,
                }],
            },
            material_state: MaterialRuntimeSnapshot { opacity: 1.0 },
            effects: crate::effect_tuner::EffectRuntimeSnapshot {
                current: EffectsConfig::default(),
                lfos: Vec::new(),
            },
        }
    }
}
