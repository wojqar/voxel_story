// generation/flat.rs
use crate::{
    chunk::{CHUNK_SIZE, Chunk},
    generation::WorldGenerator,
    voxel::VoxelId,
};
use bevy::math::IVec3;

pub struct FlatWorldGenerator {
    pub ground_level: i32, // chunk Y poniżej tego = pełny stone
    pub surface_id: VoxelId,
    pub fill_id: VoxelId,
}

impl WorldGenerator for FlatWorldGenerator {
    fn generate_chunk(&self, coord: IVec3) -> Chunk {
        let mut chunk = Chunk::empty();
        let chunk_world_y = coord.y * CHUNK_SIZE as i32;

        for y in 0..CHUNK_SIZE {
            let world_y = chunk_world_y + y as i32;
            let id = if world_y < self.ground_level - 1 {
                self.fill_id
            } else if world_y == self.ground_level - 1 {
                self.surface_id
            } else {
                VoxelId::AIR
            };
            if !id.is_air() {
                for z in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        chunk.set(x, y, z, id);
                    }
                }
            }
        }
        chunk
    }
}
