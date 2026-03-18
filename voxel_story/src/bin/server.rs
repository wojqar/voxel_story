use bevy::prelude::*;
use voxel_engine::VoxelEnginePlugin;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(VoxelEnginePlugin::default())
        .run();
}
