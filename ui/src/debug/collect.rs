use bevy::prelude::*;
use world_api::{DebugEntry, DebugMetrics};

pub fn collect_debug_entries(
    mut entries: MessageReader<DebugEntry>,
    mut metrics: ResMut<DebugMetrics>,
) {
    for entry in entries.read() {
        metrics.set(entry.section, entry.key, &entry.value);
    }
}
