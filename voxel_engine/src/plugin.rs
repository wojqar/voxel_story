use bevy::app::{App, Plugin, Startup};
use bevy::ecs::system::{Commands, Res};

use crate::resources::{VoxelWorldResource, WorldConfig};
use voxel_core::DefaultWorld;

pub struct VoxelEnginePlugin;

impl Default for VoxelEnginePlugin {
    fn default() -> Self {
        Self
    }
}

impl Plugin for VoxelEnginePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldConfig::default())
            .add_systems(Startup, init_voxel_world);
    }
}

fn init_voxel_world(mut commands: Commands, config: Res<WorldConfig>) {
    commands.insert_resource(VoxelWorldResource(DefaultWorld::new(config.dimensions)));
}

