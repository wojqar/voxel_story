use crate::CHUNK_SIZE;
use crate::chunk::Chunk;
use crate::voxel::VoxelId;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct VoxelWorld {
    chunks: HashMap<IVec3, Chunk>,
    solid_count: usize,
}

impl VoxelWorld {
    pub fn get_chunk(&self, coord: IVec3) -> Option<&Chunk> {
        self.chunks.get(&coord)
    }

    pub fn get_chunk_mut(&mut self, coord: IVec3) -> Option<&mut Chunk> {
        self.chunks.get_mut(&coord)
    }

    pub fn insert_chunk(&mut self, coord: IVec3, chunk: Chunk) {
        let incoming = chunk.count_solid();
        if let Some(old) = self.chunks.insert(coord, chunk) {
            self.solid_count -= old.count_solid();
        }
        self.solid_count += incoming;
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn solid_voxel_count(&self) -> usize {
        self.solid_count
    }

    pub fn set_voxel(&mut self, world_pos: IVec3, id: VoxelId) -> bool {
        let cs = CHUNK_SIZE as i32;
        let chunk_coord = IVec3::new(
            world_pos.x.div_euclid(cs),
            world_pos.y.div_euclid(cs),
            world_pos.z.div_euclid(cs),
        );
        let lx = world_pos.x.rem_euclid(cs) as usize;
        let ly = world_pos.y.rem_euclid(cs) as usize;
        let lz = world_pos.z.rem_euclid(cs) as usize;

        let Some(chunk) = self.chunks.get_mut(&chunk_coord) else {
            return false;
        };

        let old = chunk.get(lx, ly, lz);
        if old == id {
            return false;
        }

        if old.is_air() && !id.is_air() {
            self.solid_count += 1;
        }
        if !old.is_air() && id.is_air() {
            self.solid_count -= 1;
        }

        chunk.set(lx, ly, lz, id);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::VoxelId;

    #[test]
    fn test_solid_count_incremental() {
        let mut world = VoxelWorld::default();
        let mut chunk = Chunk::empty();
        chunk.set(0, 0, 0, VoxelId(1));
        chunk.set(1, 0, 0, VoxelId(1));
        world.insert_chunk(IVec3::ZERO, chunk);
        assert_eq!(world.solid_voxel_count(), 2);

        // nadpisanie chunka — licznik musi być poprawny
        let mut chunk2 = Chunk::empty();
        chunk2.set(0, 0, 0, VoxelId(1));
        world.insert_chunk(IVec3::ZERO, chunk2);
        assert_eq!(world.solid_voxel_count(), 1);
    }
}
