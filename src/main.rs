use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::mesh::Indices;
use debug_ui::{DebugMetrics, DebugUiPlugin};
use voxel_engine::{
    RenderingPlugin, VoxelEnginePlugin, VoxelWorld,
    chunk::CHUNK_SIZE,
    generation::{WorldGenerator, heightmap::HeightmapGenerator},
    rendering::{ChunkEntity, NeedsRemesh},
};
use spectator::{SpectatorPlugin, SpectatorCamera};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((VoxelEnginePlugin, RenderingPlugin, DebugUiPlugin, SpectatorPlugin))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update_debug_metrics)
        .run();
}

fn update_debug_metrics(
    world:        Res<VoxelWorld>,
    diagnostics:  Res<DiagnosticsStore>,
    mut metrics:  ResMut<DebugMetrics>,
    mesh_query:   Query<&Mesh3d>,
    mesh_assets:  Res<Assets<Mesh>>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let (triangle_count, draw_calls) = mesh_query.iter()
        .filter_map(|m| mesh_assets.get(&m.0))
        .fold((0usize, 0usize), |(tris, calls), mesh| {
            let t = match mesh.indices() {
                Some(Indices::U32(v))  => v.len() / 3,
                Some(Indices::U16(v))  => v.len() / 3,
                None => mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                    .map(|a| a.len() / 3)
                    .unwrap_or(0),
            };
            (tris + t, calls + 1)
        });

    metrics.set("Render", "Triangles",  triangle_count);
    metrics.set("Render", "Draw calls", draw_calls);
    metrics.set("Render", "Vertices",
        mesh_query.iter()
            .filter_map(|m| mesh_assets.get(&m.0))
            .map(|m| m.attribute(Mesh::ATTRIBUTE_POSITION)
                .map(|a| a.len()).unwrap_or(0))
            .sum::<usize>()
    );
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
    Transform::from_xyz(-120., 40., -120.)
        .looking_at(Vec3::new(64., 16., 64.), Vec3::Y),
    SpectatorCamera::default(),
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

    for cx in 0..50i32 {
        for cz in 0..50i32 {
            for cy in 0..3i32 {
                let coord = IVec3::new(cx, cy, cz);
                world.insert_chunk(coord, generator.generate_chunk(coord));
                commands.spawn((ChunkEntity(coord), NeedsRemesh));
            }
        }
    }
}
