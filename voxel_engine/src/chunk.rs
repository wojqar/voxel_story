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

    pub fn count_solid(&self) -> usize {
        self.voxels.iter().filter(|v| !v.is_air()).count()
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

    #[test]
    fn test_count_solid() {
        let mut chunk = Chunk::empty();
        assert_eq!(chunk.count_solid(), 0);
        chunk.set(0, 0, 0, VoxelId(1));
        chunk.set(1, 0, 0, VoxelId(1));
        assert_eq!(chunk.count_solid(), 2);
    }
}