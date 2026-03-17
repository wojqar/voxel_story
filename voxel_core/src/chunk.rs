use crate::coords::{local_to_index, IVec3};
use crate::voxel::VoxelId;

#[derive(Debug, Clone)]
pub struct Chunk<const SIZE: usize = 16> {
    voxels: Vec<VoxelId>,
}

impl<const SIZE: usize> Chunk<SIZE> {
    pub const fn size() -> usize {
        SIZE
    }

    pub fn new_filled(voxel: VoxelId) -> Self {
        Self {
            voxels: vec![voxel; SIZE * SIZE * SIZE],
        }
    }

    #[inline]
    pub fn get(&self, local: IVec3) -> VoxelId {
        let idx = local_to_index::<SIZE>(local);
        self.voxels[idx]
    }

    #[inline]
    pub fn set(&mut self, local: IVec3, voxel: VoxelId) -> bool {
        let idx = local_to_index::<SIZE>(local);
        let prev = self.voxels[idx];
        if prev == voxel {
            return false;
        }
        self.voxels[idx] = voxel;
        true
    }

    pub fn is_empty(&self) -> bool {
        self.voxels.iter().all(|&v| v.is_air())
    }

    pub fn count_solid(&self) -> usize {
        self.voxels.iter().filter(|&&v| !v.is_air()).count()
    }
}

impl<const SIZE: usize> Default for Chunk<SIZE> {
    fn default() -> Self {
        Self::new_filled(VoxelId::AIR)
    }
}

