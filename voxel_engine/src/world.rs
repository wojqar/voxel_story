// world.rs
use crate::chunk::{CHUNK_SIZE, Chunk};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct VoxelWorld {
    chunks: HashMap<IVec3, Chunk>,
}

impl VoxelWorld {
    pub fn get_chunk(&self, coord: IVec3) -> Option<&Chunk> {
        self.chunks.get(&coord)
    }

    pub fn get_chunk_mut(&mut self, coord: IVec3) -> Option<&mut Chunk> {
        self.chunks.get_mut(&coord)
    }

    pub fn insert_chunk(&mut self, coord: IVec3, chunk: Chunk) {
        self.chunks.insert(coord, chunk);
    }

    /// Konwersja world position -> (chunk_coord, local xyz)
    pub fn world_to_chunk(pos: IVec3) -> (IVec3, (usize, usize, usize)) {
        let size = crate::chunk::CHUNK_SIZE as i32;
        let chunk = IVec3::new(
            pos.x.div_euclid(size),
            pos.y.div_euclid(size),
            pos.z.div_euclid(size),
        );
        let local = (
            pos.x.rem_euclid(size) as usize,
            pos.y.rem_euclid(size) as usize,
            pos.z.rem_euclid(size) as usize,
        );
        (chunk, local)
    }
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn count_solid_voxels(&self) -> usize {
        self.chunks
            .values()
            .flat_map(|c| {
                (0..CHUNK_SIZE).flat_map(move |y| {
                    (0..CHUNK_SIZE).flat_map(move |z| (0..CHUNK_SIZE).map(move |x| c.get(x, y, z)))
                })
            })
            .filter(|v| !v.is_air())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_chunk_conversion() {
        let (chunk, local) = VoxelWorld::world_to_chunk(IVec3::new(17, 0, 0));
        assert_eq!(chunk, IVec3::new(1, 0, 0));
        assert_eq!(local, (1, 0, 0));

        // test ujemnych współrzędnych — div_euclid musi działać poprawnie
        let (chunk, local) = VoxelWorld::world_to_chunk(IVec3::new(-1, 0, 0));
        assert_eq!(chunk, IVec3::new(-1, 0, 0));
        assert_eq!(local, (15, 0, 0));
    }
}
