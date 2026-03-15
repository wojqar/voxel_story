use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::mesh::Indices;
use bevy::prelude::*;
use debug_ui::DebugMetrics;
use voxel_engine::{VoxelWorld, chunk::CHUNK_VOLUME};

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Update, update_debug_metrics);
    }
}

fn update_debug_metrics(
    world: Res<VoxelWorld>,
    diagnostics: Res<DiagnosticsStore>,
    mut metrics: ResMut<DebugMetrics>,
    mesh_query: Query<&Mesh3d>,
    mesh_assets: Res<Assets<Mesh>>,
    time: Res<Time>,
    mut ram_cache: Local<(f32, f64)>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let (triangle_count, draw_calls) = mesh_query
        .iter()
        .filter_map(|m| mesh_assets.get(&m.0))
        .fold((0usize, 0usize), |(tris, calls), mesh| {
            let t = match mesh.indices() {
                Some(Indices::U32(v)) => v.len() / 3,
                Some(Indices::U16(v)) => v.len() / 3,
                None => mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .map(|a| a.len() / 3)
                    .unwrap_or(0),
            };
            (tris + t, calls + 1)
        });

    let vertices = mesh_query
        .iter()
        .filter_map(|m| mesh_assets.get(&m.0))
        .map(|m| {
            m.attribute(Mesh::ATTRIBUTE_POSITION)
                .map(|a| a.len())
                .unwrap_or(0)
        })
        .sum::<usize>();

    ram_cache.0 += time.delta_secs();
    if ram_cache.0 >= 1.0 {
        ram_cache.0 = 0.0;
        ram_cache.1 = read_ram_mb();
    }

    metrics.set("Performance", "FPS", format!("{fps:.1}"));
    metrics.set("Performance", "Frame time", format!("{frame_ms:.2} ms"));
    metrics.set("Render", "Triangles", triangle_count);
    metrics.set("Render", "Vertices", vertices);
    metrics.set("Render", "Draw calls", draw_calls);
    metrics.set("World", "Chunks", world.chunk_count());
    metrics.set("World", "Voxels", world.chunk_count() * CHUNK_VOLUME);
    metrics.set("World", "Solid", world.solid_voxel_count());

    #[cfg(target_os = "linux")]
    metrics.set("World", "RAM", format!("{:.1} MB", ram_cache.1));
}

fn read_ram_mb() -> f64 {
    #[cfg(target_os = "linux")]
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let kb: f64 = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.0);
                return kb / 1024.0;
            }
        }
    }
    0.0
}
