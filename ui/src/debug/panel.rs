use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use world_api::DebugMetrics;

pub fn draw_debug_panel(mut contexts: EguiContexts, metrics: Res<DebugMetrics>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::Window::new("Debug")
        .resizable(true)
        .default_pos([8.0, 8.0])
        .default_size([460.0, 640.0])
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (section, entries) in &metrics.sections {
                        egui::CollapsingHeader::new(*section)
                            .default_open(true)
                            .show(ui, |ui| {
                                egui::Grid::new(*section)
                                    .num_columns(2)
                                    .spacing([16.0, 4.0])
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for (key, val) in entries {
                                            ui.monospace(*key);
                                            ui.monospace(val);
                                            ui.end_row();
                                        }
                                    });
                            });
                        ui.separator();
                    }
                });
        });
}
