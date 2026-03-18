use bevy::diagnostic::{
    DiagnosticPath, DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
    SystemInformationDiagnosticsPlugin,
};
use bevy::prelude::*;
use world_api::DebugEntry;

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(EntityCountDiagnosticsPlugin::default())
            .add_plugins(SystemInformationDiagnosticsPlugin)
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
    if *timer < 0.5 {
        return;
    }
    *timer = 0.0;

    let fps = diagnostic_value(&diagnostics, &FrameTimeDiagnosticsPlugin::FPS).unwrap_or(0.0);
    let frame_ms =
        diagnostic_value(&diagnostics, &FrameTimeDiagnosticsPlugin::FRAME_TIME).unwrap_or(0.0);
    let entities =
        diagnostic_value(&diagnostics, &EntityCountDiagnosticsPlugin::ENTITY_COUNT).unwrap_or(0.0);
    let process_cpu =
        diagnostic_value(&diagnostics, &SystemInformationDiagnosticsPlugin::PROCESS_CPU_USAGE)
            .unwrap_or(0.0);
    let process_mem =
        diagnostic_value(&diagnostics, &SystemInformationDiagnosticsPlugin::PROCESS_MEM_USAGE)
            .unwrap_or(0.0);
    let system_cpu =
        diagnostic_value(&diagnostics, &SystemInformationDiagnosticsPlugin::SYSTEM_CPU_USAGE)
            .unwrap_or(0.0);
    let system_mem =
        diagnostic_value(&diagnostics, &SystemInformationDiagnosticsPlugin::SYSTEM_MEM_USAGE)
            .unwrap_or(0.0);
    let budget_60 = if frame_ms > 0.0 {
        frame_ms / 16.666_667 * 100.0
    } else {
        0.0
    };

    entries.write(DebugEntry::new("Performance", "FPS", format!("{fps:.1}")));
    entries.write(DebugEntry::new(
        "Performance",
        "Frame time",
        format!("{frame_ms:.2} ms"),
    ));
    entries.write(DebugEntry::new(
        "Performance",
        "60 Hz budget",
        format!("{budget_60:.1}%"),
    ));
    entries.write(DebugEntry::new(
        "Performance",
        "Entities",
        format!("{entities:.0}"),
    ));
    entries.write(DebugEntry::new(
        "Performance",
        "Process CPU",
        format!("{process_cpu:.1}%"),
    ));
    entries.write(DebugEntry::new(
        "Performance",
        "Process Mem",
        format!("{process_mem:.2} GiB"),
    ));
    entries.write(DebugEntry::new(
        "Performance",
        "System CPU",
        format!("{system_cpu:.1}%"),
    ));
    entries.write(DebugEntry::new(
        "Performance",
        "System Mem",
        format!("{system_mem:.1}%"),
    ));
}

fn diagnostic_value(diagnostics: &DiagnosticsStore, path: &DiagnosticPath) -> Option<f64> {
    diagnostics
        .get(path)
        .and_then(|diagnostic| diagnostic.smoothed().or_else(|| diagnostic.value()))
}
