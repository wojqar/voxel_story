use bevy::prelude::*;
use std::time::Duration;
use voxel_core::{IVec3, WorldDimensions};
use world_api::{DebugEntry, SampleStats};

const CHUNK_SIZE_VOXELS: u64 = voxel_core::DEFAULT_CHUNK_SIZE as u64;

#[derive(Debug, Default, Resource)]
pub struct WorldGenerationDebugStats {
    total_ms: f64,
    generated_chunks: u64,
    chunk_time_ms: SampleStats,
    slowest_chunk: Option<IVec3>,
    slowest_chunk_ms: f64,
    dimensions: Option<WorldDimensions>,
    solid_voxels: usize,
}

impl WorldGenerationDebugStats {
    pub fn record_chunk(&mut self, chunk_coord: IVec3, duration: Duration) {
        let duration_ms = duration.as_secs_f64() * 1_000.0;
        self.generated_chunks += 1;
        self.chunk_time_ms.record(duration_ms);

        if duration_ms >= self.slowest_chunk_ms {
            self.slowest_chunk = Some(chunk_coord);
            self.slowest_chunk_ms = duration_ms;
        }
    }

    pub fn finish(&mut self, total: Duration, dimensions: WorldDimensions, solid_voxels: usize) {
        self.total_ms = total.as_secs_f64() * 1_000.0;
        self.dimensions = Some(dimensions);
        self.solid_voxels = solid_voxels;
    }
}

#[derive(Debug, Default, Resource)]
pub struct TerrainHeightDebugStats {
    request_time_ms: SampleStats,
    batch_time_ms: SampleStats,
    batch_size: SampleStats,
    requests_total: u64,
    misses_total: u64,
    last_query: Option<((i32, i32), Option<f32>)>,
    slowest_query: Option<(i32, i32)>,
    slowest_query_ms: f64,
}

impl TerrainHeightDebugStats {
    pub fn record_request(
        &mut self,
        query_pos: (i32, i32),
        height: Option<f32>,
        missing_terrain: bool,
        duration: Duration,
    ) {
        let duration_ms = duration.as_secs_f64() * 1_000.0;
        self.request_time_ms.record(duration_ms);
        self.requests_total += 1;
        self.last_query = Some((query_pos, height));

        if missing_terrain {
            self.misses_total += 1;
        }

        if duration_ms >= self.slowest_query_ms {
            self.slowest_query = Some(query_pos);
            self.slowest_query_ms = duration_ms;
        }
    }

    pub fn record_batch(&mut self, batch_len: usize, duration: Duration) {
        self.batch_size.record(batch_len as f64);
        self.batch_time_ms.record_duration(duration);
    }
}

#[derive(Default)]
pub struct EngineDebugReportState {
    elapsed: f32,
    previous_requests_total: u64,
}

pub fn emit_engine_debug_entries(
    time: Res<Time>,
    config: Res<crate::resources::WorldConfig>,
    world: Option<Res<crate::resources::VoxelWorldResource>>,
    world_gen: Res<WorldGenerationDebugStats>,
    terrain: Res<TerrainHeightDebugStats>,
    mut entries: MessageWriter<DebugEntry>,
    mut state: Local<EngineDebugReportState>,
) {
    state.elapsed += time.delta_secs();
    if state.elapsed < 0.5 {
        return;
    }

    let interval_secs = state.elapsed.max(f32::EPSILON) as f64;
    let request_rate = (terrain
        .requests_total
        .saturating_sub(state.previous_requests_total)) as f64
        / interval_secs;
    state.previous_requests_total = terrain.requests_total;
    state.elapsed = 0.0;

    entries.write(DebugEntry::new("World", "Seed", config.seed));
    entries.write(DebugEntry::new(
        "World",
        "Chunk dims",
        format_dimensions(config.dimensions),
    ));
    entries.write(DebugEntry::new(
        "World",
        "Voxel dims",
        format_voxel_dimensions(config.dimensions),
    ));
    entries.write(DebugEntry::new(
        "World",
        "Chunks built",
        world_gen.generated_chunks,
    ));
    entries.write(DebugEntry::new(
        "World",
        "Generate total",
        format!("{:.2} ms", world_gen.total_ms),
    ));
    entries.write(DebugEntry::new(
        "World",
        "Chunk build",
        world_gen.chunk_time_ms.format_summary(2, " ms"),
    ));

    let chunks_per_sec = if world_gen.total_ms > 0.0 {
        world_gen.generated_chunks as f64 / (world_gen.total_ms / 1_000.0)
    } else {
        0.0
    };
    entries.write(DebugEntry::new(
        "World",
        "Chunk throughput",
        format!("{chunks_per_sec:.1} chunks/s"),
    ));

    let total_voxels = total_voxel_capacity(config.dimensions);
    let solid_fill = if total_voxels > 0 {
        world_gen.solid_voxels as f64 / total_voxels as f64 * 100.0
    } else {
        0.0
    };
    entries.write(DebugEntry::new(
        "World",
        "Solid voxels",
        world_gen.solid_voxels,
    ));
    entries.write(DebugEntry::new(
        "World",
        "World fill",
        format!("{solid_fill:.2}% of {total_voxels} voxels"),
    ));

    if let Some(chunk) = world_gen.slowest_chunk {
        entries.write(DebugEntry::new(
            "World",
            "Slowest chunk",
            format!(
                "{} ({:.2} ms)",
                format_ivec3(chunk),
                world_gen.slowest_chunk_ms
            ),
        ));
    }

    if let Some(world) = world {
        entries.write(DebugEntry::new(
            "World",
            "Loaded solids",
            world.0.solid_count,
        ));
    }

    entries.write(DebugEntry::new(
        "Terrain",
        "Requests/sec",
        format!("{request_rate:.1}"),
    ));
    entries.write(DebugEntry::new(
        "Terrain",
        "Requests total",
        terrain.requests_total,
    ));
    entries.write(DebugEntry::new(
        "Terrain",
        "Misses total",
        terrain.misses_total,
    ));
    entries.write(DebugEntry::new(
        "Terrain",
        "Query time",
        terrain.request_time_ms.format_summary(3, " ms"),
    ));
    entries.write(DebugEntry::new(
        "Terrain",
        "Batch time",
        terrain.batch_time_ms.format_summary(3, " ms"),
    ));
    entries.write(DebugEntry::new(
        "Terrain",
        "Batch size",
        terrain.batch_size.format_summary(1, ""),
    ));

    if let Some((pos, height)) = terrain.last_query {
        entries.write(DebugEntry::new(
            "Terrain",
            "Last sample",
            format!(
                "{} -> {}",
                format_ivec2(pos),
                height
                    .map(|value| format!("{value:.2}"))
                    .unwrap_or_else(|| "none".to_string())
            ),
        ));
    }

    if let Some(pos) = terrain.slowest_query {
        entries.write(DebugEntry::new(
            "Terrain",
            "Slowest query",
            format!("{} ({:.3} ms)", format_ivec2(pos), terrain.slowest_query_ms),
        ));
    }
}

fn total_voxel_capacity(dimensions: WorldDimensions) -> u64 {
    dimensions.x as u64
        * dimensions.y as u64
        * dimensions.z as u64
        * CHUNK_SIZE_VOXELS
        * CHUNK_SIZE_VOXELS
        * CHUNK_SIZE_VOXELS
}

fn format_dimensions(dimensions: WorldDimensions) -> String {
    format!("{} x {} x {}", dimensions.x, dimensions.y, dimensions.z)
}

fn format_voxel_dimensions(dimensions: WorldDimensions) -> String {
    format!(
        "{} x {} x {}",
        dimensions.x as u64 * CHUNK_SIZE_VOXELS,
        dimensions.y as u64 * CHUNK_SIZE_VOXELS,
        dimensions.z as u64 * CHUNK_SIZE_VOXELS
    )
}

fn format_ivec2(value: (i32, i32)) -> String {
    format!("({}, {})", value.0, value.1)
}

fn format_ivec3(value: IVec3) -> String {
    format!("({}, {}, {})", value.x, value.y, value.z)
}
