use bevy::prelude::*;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::fmt::Write as _;

/// Shared resource — stan wyświetlany przez debug panel.
/// Wypełniany przez collect_debug_entries system w ui crate.
/// Każdy system w projekcie może pisać przez DebugEntry event.
#[derive(Resource, Default)]
pub struct DebugMetrics {
    pub sections: BTreeMap<&'static str, BTreeMap<&'static str, String>>,
}

impl DebugMetrics {
    pub fn set(&mut self, section: &'static str, key: &'static str, value: impl Display) {
        let entry = self.sections
            .entry(section)
            .or_default()
            .entry(key)
            .or_default();
        entry.clear();
        write!(entry, "{}", value).unwrap();
    }

    pub fn clear(&mut self) {
        self.sections.clear();
    }
}

/// Event — jeden wpis do debug panelu.
/// Każdy system emituje DebugEntry zamiast pisać bezpośrednio do DebugMetrics.
#[derive(Message, Clone)]
pub struct DebugEntry {
    pub section: &'static str,
    pub key: &'static str,
    pub value: String,
}

impl DebugEntry {
    pub fn new(section: &'static str, key: &'static str, value: impl Display) -> Self {
        Self {
            section,
            key,
            value: value.to_string(),
        }
    }
}