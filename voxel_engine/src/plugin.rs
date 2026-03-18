use bevy::app::{App, Plugin, Startup, Update};
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::prelude::IntoScheduleConfigs;
use std::time::Instant;

use crate::debug::{
    TerrainHeightDebugStats, WorldGenerationDebugStats, emit_engine_debug_entries,
};
use crate::resources::{VoxelWorldResource, WorldConfig};
use voxel_core::generation::CastleStoryGenerator;
use voxel_core::{DefaultWorld, IVec3};
use world_api::{TerrainHeightRequest, TerrainHeightResponse};

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
            .add_systems(Update, emit_engine_debug_entries.after(respond_terrain_height_requests));
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
                let chunk = voxel_core::generation::WorldGenerator::generate_chunk(
                    &generator,
                    chunk_coord,
                );
                world_gen_stats.record_chunk(chunk_coord, chunk_started.elapsed());
                world.replace_chunk(chunk_coord, chunk);
            }
        }
    }

    world_gen_stats.finish(world_started.elapsed(), config.dimensions, world.solid_count);
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

        let surface_height = world
            .0
            .column_height(x, z)
            .map(|surface_y| surface_y as f32 + 1.0);
        let used_fallback = surface_height.is_none();
        let height = surface_height.unwrap_or_else(|| {
                let fallback = world.0.get_voxel(IVec3::new(x, 0, z));
                if fallback.is_air() { 0.0 } else { 1.0 }
            });

        terrain_debug.record_request(
            (x, z),
            height,
            used_fallback,
            request_started.elapsed(),
        );
        responses.write(TerrainHeightResponse { height });
    }

    if batch_len > 0 {
        terrain_debug.record_batch(batch_len, batch_started.elapsed());
    }
}
