use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use debug_ui::{DebugMetrics, DebugUiPlugin};
use voxel_engine::{
    RenderingPlugin, VoxelEnginePlugin, VoxelWorld,
    chunk::CHUNK_SIZE,
    generation::{WorldGenerator, heightmap::HeightmapGenerator},
    rendering::{ChunkEntity, NeedsRemesh},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((VoxelEnginePlugin, RenderingPlugin, DebugUiPlugin))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update_debug_metrics)
        .run();
}

fn update_debug_metrics(
    world: Res<VoxelWorld>,
    diagnostics: Res<DiagnosticsStore>,
    mut metrics: ResMut<DebugMetrics>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    metrics.set("Performance", "FPS", format!("{fps:.1}"));
    metrics.set("Performance", "Frame time", format!("{frame_ms:.2} ms"));
    metrics.set("World", "Chunks", world.chunk_count());
    metrics.set("World", "Voxels", world.chunk_count() * CHUNK_SIZE.pow(3));
    metrics.set("World", "Solid", world.count_solid_voxels());
    metrics.set("World", "RAM", format!("{:.1} MB", read_ram_mb()));
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

fn setup(mut commands: Commands, mut world: ResMut<VoxelWorld>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-120., 100., -120.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.)),
    ));

    let generator = HeightmapGenerator::new(42);

    for cx in 0..8i32 {
        for cz in 0..8i32 {
            for cy in 0..3i32 {
                let coord = IVec3::new(cx, cy, cz);
                world.insert_chunk(coord, generator.generate_chunk(coord));
                commands.spawn((ChunkEntity(coord), NeedsRemesh));
            }
        }
    }
}
