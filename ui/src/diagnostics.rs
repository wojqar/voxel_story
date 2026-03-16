use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use world_api::DebugEntry;

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Update, debug_performance);
    }
}

fn debug_performance(
    diagnostics: Res<DiagnosticsStore>,
    mut entries: MessageWriter<DebugEntry>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < 1.0 { return; }
    *timer = 0.0;

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    entries.write(DebugEntry::new("Performance", "FPS", format!("{fps:.1}")));
    entries.write(DebugEntry::new("Performance", "Frame time", format!("{frame_ms:.2} ms")));
}