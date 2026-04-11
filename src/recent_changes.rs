use bevy::prelude::*;

pub(crate) const RECENT_CHANGE_TIMEOUT_SECS: f32 = 6.0;
const MAX_RECENT_CHANGES: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecentChangeSnapshotRow {
    pub(crate) label: String,
    pub(crate) value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RecentChangesSnapshot {
    pub(crate) timeout_secs: f32,
    pub(crate) rows: Vec<RecentChangeSnapshotRow>,
}

#[derive(Clone, Debug)]
struct RecentChangeEntry {
    label: String,
    value: String,
    changed_at_secs: f32,
}

#[derive(Resource, Clone, Debug, Default)]
pub(crate) struct RecentChangesState {
    entries: Vec<RecentChangeEntry>,
}

impl RecentChangesState {
    pub(crate) fn record(
        &mut self,
        label: impl Into<String>,
        value: impl Into<String>,
        now_secs: f32,
    ) -> bool {
        let label = label.into();
        let value = value.into();
        let Some((label, value)) = normalized_change_entry(label, value) else {
            return false;
        };

        self.upsert(label, value, now_secs);
        true
    }

    pub(crate) fn record_throttled(
        &mut self,
        label: impl Into<String>,
        value: impl Into<String>,
        now_secs: f32,
        min_interval_secs: f32,
    ) -> bool {
        let label = label.into();
        let value = value.into();
        let Some((label, value)) = normalized_change_entry(label, value) else {
            return false;
        };

        if self.is_throttled(&label, now_secs, min_interval_secs.max(0.0)) {
            return false;
        }

        self.upsert(label, value, now_secs);
        true
    }

    pub(crate) fn record_status_message(&mut self, message: impl Into<String>, now_secs: f32) {
        let message = message.into();
        let Some((label, value)) = message.split_once(':') else {
            self.record(message, "", now_secs);
            return;
        };
        let label = label.trim();
        let label = label.strip_prefix("Reset ").unwrap_or(label);
        self.record(label, value.trim(), now_secs);
    }

    pub(crate) fn snapshot(&self, now_secs: f32) -> RecentChangesSnapshot {
        let mut rows = Vec::new();
        for (index, entry) in self.entries.iter().enumerate() {
            let keep_recent = now_secs - entry.changed_at_secs <= RECENT_CHANGE_TIMEOUT_SECS;
            if index == 0 || keep_recent {
                rows.push(RecentChangeSnapshotRow {
                    label: entry.label.clone(),
                    value: entry.value.clone(),
                });
            }
        }

        RecentChangesSnapshot {
            timeout_secs: RECENT_CHANGE_TIMEOUT_SECS,
            rows,
        }
    }

    fn is_throttled(&self, label: &str, now_secs: f32, min_interval_secs: f32) -> bool {
        if min_interval_secs <= 0.0 {
            return false;
        }

        self.entries.iter().any(|entry| {
            entry.label == label
                && now_secs >= entry.changed_at_secs
                && now_secs - entry.changed_at_secs < min_interval_secs
        })
    }

    fn upsert(&mut self, label: String, value: String, now_secs: f32) {
        if let Some(existing_index) = self.entries.iter().position(|entry| entry.label == label) {
            self.entries.remove(existing_index);
        }

        self.entries.insert(
            0,
            RecentChangeEntry {
                label,
                value,
                changed_at_secs: now_secs,
            },
        );
        self.entries.truncate(MAX_RECENT_CHANGES);
    }
}

fn normalized_change_entry(label: String, value: String) -> Option<(String, String)> {
    let label = label.trim().to_string();
    if label.is_empty() {
        return None;
    }

    Some((label, value.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::RecentChangesState;

    #[test]
    fn snapshot_keeps_latest_change_after_timeout() {
        let mut changes = RecentChangesState::default();
        changes.record("Child scale ratio", "0.62", 1.0);
        changes.record("Global object opacity", "80%", 2.0);

        let snapshot = changes.snapshot(20.0);

        assert_eq!(snapshot.rows.len(), 1);
        assert_eq!(snapshot.rows[0].label, "Global object opacity");
        assert_eq!(snapshot.rows[0].value, "80%");
    }

    #[test]
    fn snapshot_keeps_recent_changes_before_timeout() {
        let mut changes = RecentChangesState::default();
        changes.record("Child scale ratio", "0.62", 1.0);
        changes.record("Global object opacity", "80%", 2.0);

        let snapshot = changes.snapshot(3.0);

        assert_eq!(snapshot.rows.len(), 2);
        assert_eq!(snapshot.rows[0].label, "Global object opacity");
        assert_eq!(snapshot.rows[1].label, "Child scale ratio");
    }

    #[test]
    fn recording_the_same_label_updates_and_moves_it_first() {
        let mut changes = RecentChangesState::default();
        changes.record("Child scale ratio", "0.62", 1.0);
        changes.record("Global object opacity", "80%", 2.0);
        changes.record("Child scale ratio", "0.67", 3.0);

        let snapshot = changes.snapshot(3.0);

        assert_eq!(snapshot.rows.len(), 2);
        assert_eq!(snapshot.rows[0].label, "Child scale ratio");
        assert_eq!(snapshot.rows[0].value, "0.67");
    }

    #[test]
    fn status_messages_split_label_and_value_at_colon() {
        let mut changes = RecentChangesState::default();
        changes.record_status_message("Child scale ratio: 0.62", 1.0);

        let snapshot = changes.snapshot(1.0);

        assert_eq!(snapshot.rows[0].label, "Child scale ratio");
        assert_eq!(snapshot.rows[0].value, "0.62");
    }

    #[test]
    fn throttled_recording_keeps_the_previous_value_inside_the_interval() {
        let mut changes = RecentChangesState::default();
        assert!(changes.record_throttled("camera.zoom_velocity", "-0.20", 1.0, 0.25));
        assert!(!changes.record_throttled("camera.zoom_velocity", "-0.40", 1.1, 0.25));

        let snapshot = changes.snapshot(1.1);

        assert_eq!(snapshot.rows.len(), 1);
        assert_eq!(snapshot.rows[0].label, "camera.zoom_velocity");
        assert_eq!(snapshot.rows[0].value, "-0.20");
    }

    #[test]
    fn throttled_recording_updates_after_the_interval() {
        let mut changes = RecentChangesState::default();
        assert!(changes.record_throttled("camera.zoom_velocity", "-0.20", 1.0, 0.25));
        assert!(changes.record_throttled("camera.zoom_velocity", "-0.40", 1.3, 0.25));

        let snapshot = changes.snapshot(1.3);

        assert_eq!(snapshot.rows.len(), 1);
        assert_eq!(snapshot.rows[0].label, "camera.zoom_velocity");
        assert_eq!(snapshot.rows[0].value, "-0.40");
    }
}
