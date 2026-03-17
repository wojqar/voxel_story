use bevy::app::{App, Plugin, Startup};
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::prelude::IntoScheduleConfigs;

use crate::resources::{VoxelWorldResource, WorldConfig};
use voxel_core::DefaultWorld;
use voxel_core::generation::{IslandsGenerator, IslandsParams};

pub struct VoxelEnginePlugin;

impl Default for VoxelEnginePlugin {
    fn default() -> Self {
        Self
    }
}

impl Plugin for VoxelEnginePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldConfig::default())
            .add_systems(Startup, init_voxel_world)
            .add_systems(Startup, generate_startup_world.after(init_voxel_world));
    }
}

fn init_voxel_world(mut commands: Commands, config: Res<WorldConfig>) {
    commands.insert_resource(VoxelWorldResource(DefaultWorld::new(config.dimensions)));
}

fn generate_startup_world(mut world: ResMut<VoxelWorldResource>, config: Res<WorldConfig>) {
    let generator = IslandsGenerator::new(IslandsParams::default());
    generator.generate_into(&mut world.0, config.seed);
}

