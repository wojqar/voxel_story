// chunk.rs
use crate::voxel::VoxelId;
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Clone)]
pub struct Chunk {
    voxels: Box<[VoxelId; CHUNK_VOLUME]>,
}

impl Chunk {
    pub fn empty() -> Self {
        Self {
            voxels: Box::new([VoxelId::AIR; CHUNK_VOLUME]),
        }
    }

    #[inline]
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> VoxelId {
        self.voxels[Self::index(x, y, z)]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, id: VoxelId) {
        self.voxels[Self::index(x, y, z)] = id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() {
        let mut chunk = Chunk::empty();
        chunk.set(1, 2, 3, VoxelId(5));
        assert_eq!(chunk.get(1, 2, 3), VoxelId(5));
        assert_eq!(chunk.get(0, 0, 0), VoxelId::AIR);
    }
}
