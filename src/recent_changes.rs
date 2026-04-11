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
    ) {
        let label = label.into();
        let value = value.into();
        if label.trim().is_empty() {
            return;
        }

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
}
