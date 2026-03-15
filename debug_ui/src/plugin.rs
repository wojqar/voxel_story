use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use crate::metrics::{WorldMetrics, update_world_metrics};

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(EguiPlugin::default())
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<WorldMetrics>()
            .add_systems(Update, update_world_metrics)
            .add_systems(EguiPrimaryContextPass, draw_debug_window);
    }
}

fn draw_debug_window(
    mut contexts: EguiContexts,
    diagnostics:  Res<DiagnosticsStore>,
    metrics:      Res<WorldMetrics>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    egui::Window::new("Debug")
        .resizable(false)
        .default_pos([8.0, 8.0])
        .show(ctx, |ui| {
            ui.heading("Performance");
            ui.monospace(format!("FPS        {:>8.1}", fps));
            ui.monospace(format!("Frame time {:>7.2} ms", frame_ms));
            ui.separator();
            ui.heading("World");
            ui.monospace(format!("Chunks     {:>8}", metrics.loaded_chunks));
            ui.monospace(format!("Voxels     {:>8}", metrics.total_voxels));
            ui.monospace(format!("Solid      {:>8}", metrics.solid_voxels));
            ui.monospace(format!("RAM        {:>6.1} MB", metrics.ram_mb));
        });

    Ok(())
}