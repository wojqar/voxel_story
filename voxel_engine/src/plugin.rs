use bevy::app::{App, Plugin, Startup, Update};
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::ecs::system::{Commands, Res};

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
            .add_systems(Startup, init_voxel_world)
            .add_systems(Update, respond_terrain_height_requests);
    }
}

fn init_voxel_world(mut commands: Commands, config: Res<WorldConfig>) {
    let generator = CastleStoryGenerator::new(config.seed, config.dimensions);
    let world = DefaultWorld::from_generator(config.dimensions, &generator);
    commands.insert_resource(VoxelWorldResource(world));
}

fn respond_terrain_height_requests(
    world: Res<VoxelWorldResource>,
    mut requests: MessageReader<TerrainHeightRequest>,
    mut responses: MessageWriter<TerrainHeightResponse>,
) {
    for request in requests.read() {
        let x = request.pos.x.floor() as i32;
        let z = request.pos.y.floor() as i32;

        let height = world
            .0
            .column_height(x, z)
            .map(|surface_y| surface_y as f32 + 1.0)
            .unwrap_or_else(|| {
                let fallback = world.0.get_voxel(IVec3::new(x, 0, z));
                if fallback.is_air() { 0.0 } else { 1.0 }
            });

        responses.write(TerrainHeightResponse { height });
    }
}
