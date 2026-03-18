use bevy::app::{App, Plugin, Startup, Update};
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::prelude::Vec3;
use bevy::prelude::IntoScheduleConfigs;
use std::time::Instant;

use crate::debug::{TerrainHeightDebugStats, WorldGenerationDebugStats, emit_engine_debug_entries};
use crate::resources::{VoxelWorldResource, WorldConfig};
use voxel_core::generation::CastleStoryGenerator;
use voxel_core::{DEFAULT_CHUNK_SIZE, DefaultWorld, IVec3, WorldDimensions};
use world_api::{MainTerrainAnchor, TerrainHeightRequest, TerrainHeightResponse};

pub struct VoxelEnginePlugin;

impl Default for VoxelEnginePlugin {
    fn default() -> Self {
        Self
    }
}

impl Plugin for VoxelEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TerrainHeightRequest>()
            .add_message::<TerrainHeightResponse>()
            .init_resource::<WorldConfig>()
            .init_resource::<WorldGenerationDebugStats>()
            .init_resource::<TerrainHeightDebugStats>()
            .add_systems(Startup, init_voxel_world)
            .add_systems(Update, respond_terrain_height_requests)
            .add_systems(
                Update,
                emit_engine_debug_entries.after(respond_terrain_height_requests),
            );
    }
}

fn init_voxel_world(
    mut commands: Commands,
    config: Res<WorldConfig>,
    mut world_gen_stats: ResMut<WorldGenerationDebugStats>,
) {
    let generator = CastleStoryGenerator::new(config.seed, config.dimensions);
    let mut world = DefaultWorld::new(config.dimensions);
    let world_started = Instant::now();

    for z in 0..config.dimensions.z as i32 {
        for y in 0..config.dimensions.y as i32 {
            for x in 0..config.dimensions.x as i32 {
                let chunk_coord = IVec3::new(x, y, z);
                let chunk_started = Instant::now();
                let chunk =
                    voxel_core::generation::WorldGenerator::generate_chunk(&generator, chunk_coord);
                world_gen_stats.record_chunk(chunk_coord, chunk_started.elapsed());
                world.replace_chunk(chunk_coord, chunk);
            }
        }
    }

    let main_terrain_anchor = MainTerrainAnchor {
        focus: find_main_terrain_anchor(&world),
    };

    world_gen_stats.finish(
        world_started.elapsed(),
        config.dimensions,
        world.solid_count,
    );
    commands.insert_resource(main_terrain_anchor);
    commands.insert_resource(VoxelWorldResource(world));
}

fn respond_terrain_height_requests(
    world: Res<VoxelWorldResource>,
    mut requests: MessageReader<TerrainHeightRequest>,
    mut responses: MessageWriter<TerrainHeightResponse>,
    mut terrain_debug: ResMut<TerrainHeightDebugStats>,
) {
    let batch_started = Instant::now();
    let mut batch_len = 0usize;

    for request in requests.read() {
        batch_len += 1;
        let x = request.pos.x.floor() as i32;
        let z = request.pos.y.floor() as i32;
        let request_started = Instant::now();

        let height = world
            .0
            .column_height(x, z)
            .map(|surface_y| surface_y as f32 + 1.0);
        let missing_terrain = height.is_none();

        terrain_debug.record_request((x, z), height, missing_terrain, request_started.elapsed());
        responses.write(TerrainHeightResponse { height });
    }

    if batch_len > 0 {
        terrain_debug.record_batch(batch_len, batch_started.elapsed());
    }
}

fn find_main_terrain_anchor(world: &DefaultWorld) -> Vec3 {
    let voxel_dims = voxel_dimensions(world.dimensions);
    let center_x = voxel_dims.x / 2;
    let center_z = voxel_dims.z / 2;
    let max_radius = voxel_dims.x.max(voxel_dims.z) / 2;

    find_nearest_surface(world, center_x, center_z, max_radius)
        .map(|(x, z, height)| Vec3::new(x as f32 + 0.5, height, z as f32 + 0.5))
        .unwrap_or_else(|| Vec3::new(center_x as f32, 0.0, center_z as f32))
}

fn find_nearest_surface(
    world: &DefaultWorld,
    origin_x: i32,
    origin_z: i32,
    max_radius: i32,
) -> Option<(i32, i32, f32)> {
    if let Some(height) = sample_surface_height(world, origin_x, origin_z) {
        return Some((origin_x, origin_z, height));
    }

    for radius in 1..=max_radius {
        let min_x = origin_x - radius;
        let max_x = origin_x + radius;
        let min_z = origin_z - radius;
        let max_z = origin_z + radius;
        let mut best = None;

        for sample_z in min_z..=max_z {
            consider_surface_candidate(world, origin_x, origin_z, min_x, sample_z, &mut best);
            if max_x != min_x {
                consider_surface_candidate(world, origin_x, origin_z, max_x, sample_z, &mut best);
            }
        }

        if min_z != max_z {
            for sample_x in (min_x + 1)..max_x {
                consider_surface_candidate(world, origin_x, origin_z, sample_x, min_z, &mut best);
                consider_surface_candidate(world, origin_x, origin_z, sample_x, max_z, &mut best);
            }
        }

        if let Some((x, z, height, _)) = best {
            return Some((x, z, height));
        }
    }

    None
}

fn consider_surface_candidate(
    world: &DefaultWorld,
    origin_x: i32,
    origin_z: i32,
    sample_x: i32,
    sample_z: i32,
    best: &mut Option<(i32, i32, f32, i32)>,
) {
    let Some(height) = sample_surface_height(world, sample_x, sample_z) else {
        return;
    };

    let distance_sq = (sample_x - origin_x).pow(2) + (sample_z - origin_z).pow(2);
    if best.is_none_or(|(_, _, _, best_distance_sq)| distance_sq < best_distance_sq) {
        *best = Some((sample_x, sample_z, height, distance_sq));
    }
}

fn sample_surface_height(world: &DefaultWorld, x: i32, z: i32) -> Option<f32> {
    world.column_height(x, z).map(|surface_y| surface_y as f32 + 1.0)
}

fn voxel_dimensions(dimensions: WorldDimensions) -> IVec3 {
    IVec3::new(
        dimensions.x as i32 * DEFAULT_CHUNK_SIZE as i32,
        dimensions.y as i32 * DEFAULT_CHUNK_SIZE as i32,
        dimensions.z as i32 * DEFAULT_CHUNK_SIZE as i32,
    )
}
