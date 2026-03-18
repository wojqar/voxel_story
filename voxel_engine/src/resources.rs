use bevy::prelude::Resource;
use voxel_core::{DefaultWorld, WorldDimensions};

#[derive(Debug, Clone, Resource)]
pub struct WorldConfig {
    pub dimensions: WorldDimensions,
    pub seed: u64,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            dimensions: WorldDimensions::new(20, 8, 20),
            seed: 0,
        }
    }
}

#[derive(Debug, Resource)]
pub struct VoxelWorldResource(pub DefaultWorld);
