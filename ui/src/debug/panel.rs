use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use world_api::DebugMetrics;

pub fn draw_debug_panel(mut contexts: EguiContexts, metrics: Res<DebugMetrics>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

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
}
