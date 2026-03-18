use bevy::prelude::*;

#[derive(Component, Default)]
pub struct ActiveCamera;

#[derive(Component)]
pub struct ChunkObserver {
    pub load_distance: u32,
    pub unload_distance: u32,
}

impl Default for ChunkObserver {
    fn default() -> Self {
        Self {
            load_distance: 8,
            unload_distance: 12,
        }
    }
}
