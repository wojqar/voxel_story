use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use std::collections::BTreeMap;

#[derive(Resource, Default)]
pub struct DebugMetrics {
    sections: BTreeMap<String, BTreeMap<String, String>>,
}

impl DebugMetrics {
    pub fn set(&mut self, section: impl Into<String>, key: impl Into<String>, value: impl ToString) {
        self.sections
            .entry(section.into())
            .or_default()
            .insert(key.into(), value.to_string());
    }
}

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(EguiPlugin::default())
            .init_resource::<DebugMetrics>()
            .add_systems(EguiPrimaryContextPass, draw_debug_window);
    }
}

fn draw_debug_window(
    mut contexts: EguiContexts,
    metrics:      Res<DebugMetrics>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::Window::new("Debug")
        .resizable(false)
        .default_pos([8.0, 8.0])
        .show(ctx, |ui| {
            for (section, entries) in &metrics.sections {
                ui.heading(section);
                for (key, val) in entries {
                    ui.monospace(format!("{key:<16} {val}"));
                }
                ui.separator();
            }
        });

    Ok(())
}