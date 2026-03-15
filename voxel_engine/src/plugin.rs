// plugin.rs
use crate::world::VoxelWorld;
use bevy::prelude::*;

pub struct VoxelEnginePlugin;

impl Plugin for VoxelEnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelWorld>();
    }
}
