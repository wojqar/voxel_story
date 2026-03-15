// debug_ui/src/plugin.rs
use bevy::prelude::*;
use bevy::diagnostic::{
    DiagnosticsStore, FrameTimeDiagnosticsPlugin, RenderDiagnosticsPlugin,
};
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::metrics::{WorldMetrics, update_world_metrics};

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_plugins(RenderDiagnosticsPlugin)
            .init_resource::<WorldMetrics>()
            .add_systems(Update, update_world_metrics)
            .add_systems(Update, draw_debug_window);
    }
}

fn draw_debug_window(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    metrics: Res<WorldMetrics>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let triangles = diagnostics
        .get(&RenderDiagnosticsPlugin::TOTAL_POLYGONS)
        .and_then(|d| d.value())
        .unwrap_or(0.0);

    let draw_calls = diagnostics
        .get(&RenderDiagnosticsPlugin::TOTAL_DRAW_CALLS)
        .and_then(|d| d.value())
        .unwrap_or(0.0);

    egui::Window::new("Debug")
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Performance");
            ui.label(format!("FPS:        {fps:.1}"));
            ui.label(format!("Frame time: {frame_ms:.2} ms"));
            ui.label(format!("RAM:          {:.1} MB", metrics.ram_mb));
            ui.separator();

            ui.heading("Render");
            ui.label(format!("Triangles:  {}", triangles as u64));
            ui.label(format!("Draw calls: {}", draw_calls as u64));
            ui.separator();

            ui.heading("World");
            ui.label(format!("Chunks:       {}", metrics.loaded_chunks));
            ui.label(format!("Total voxels: {}", metrics.total_voxels));
            ui.label(format!("Solid voxels: {}", metrics.solid_voxels));
        });
}