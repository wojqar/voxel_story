use std::collections::BTreeMap;
use std::fmt::Write as _;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

#[derive(Resource, Default)]
pub struct DebugMetrics {
    sections: BTreeMap<&'static str, BTreeMap<&'static str, String>>,
}

impl DebugMetrics {
    pub fn set(&mut self, section: &'static str, key: &'static str, value: impl std::fmt::Display) {
        let entry = self.sections
            .entry(section)
            .or_default()
            .entry(key)
            .or_default();
        entry.clear();
        write!(entry, "{}", value).unwrap();
    }
}

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<DebugMetrics>()
            .add_systems(EguiPrimaryContextPass, draw_debug_window);
    }
}

fn draw_debug_window(mut contexts: EguiContexts, metrics: Res<DebugMetrics>) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Debug")
        .resizable(false)
        .default_pos([8.0, 8.0])
        .show(ctx, |ui| {
            for (section, entries) in &metrics.sections {
                ui.heading(*section);
                for (key, val) in entries {
                    ui.monospace(format!("{key:<16} {val}"));
                }
                ui.separator();
            }
        });
    Ok(())
}