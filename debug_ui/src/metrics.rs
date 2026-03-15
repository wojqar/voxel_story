use bevy::prelude::*;
use voxel_engine::{VoxelWorld, chunk::CHUNK_SIZE};

#[derive(Resource, Default)]
pub struct WorldMetrics {
    pub loaded_chunks: usize,
    pub total_voxels:  usize,
    pub solid_voxels:  usize,
    pub ram_mb:        f64,
}

pub fn update_world_metrics(
    world:      Res<VoxelWorld>,
    mut metrics: ResMut<WorldMetrics>,
) {
    if !world.is_changed() { return; }

    metrics.loaded_chunks = world.chunk_count();
    metrics.total_voxels  = metrics.loaded_chunks * CHUNK_SIZE.pow(3);
    metrics.solid_voxels  = world.count_solid_voxels();
    metrics.ram_mb        = read_ram_mb();
}

fn read_ram_mb() -> f64 {
    #[cfg(target_os = "linux")]
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let kb: f64 = line.split_whitespace()
                    .nth(1)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.0);
                return kb / 1024.0;
            }
        }
    }
    0.0
}