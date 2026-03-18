use crate::coords::{IVec3, local_to_index};
use crate::voxel::VoxelId;

#[derive(Debug, Clone)]
pub struct Chunk<const SIZE: usize = 16> {
    voxels: Vec<VoxelId>,
    solid_count: usize,
    column_tops: Vec<Option<u8>>,
}

impl<const SIZE: usize> Chunk<SIZE> {
    pub const fn size() -> usize {
        SIZE
    }

    pub fn new_filled(voxel: VoxelId) -> Self {
        let solid_count = if voxel.is_air() { 0 } else { SIZE * SIZE * SIZE };
        let column_top = if voxel.is_air() {
            None
        } else {
            Some((SIZE - 1) as u8)
        };
        Self {
            voxels: vec![voxel; SIZE * SIZE * SIZE],
            solid_count,
            column_tops: vec![column_top; SIZE * SIZE],
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
        match (prev.is_air(), voxel.is_air()) {
            (true, false) => self.solid_count += 1,
            (false, true) => self.solid_count = self.solid_count.saturating_sub(1),
            _ => {}
        }

        let x = local.x as usize;
        let y = local.y as usize;
        let z = local.z as usize;
        let column_idx = column_index::<SIZE>(x, z);
        match self.column_tops[column_idx] {
            Some(top) if voxel.is_air() && top as usize == y => self.refresh_column_top(x, z),
            Some(top) if !voxel.is_air() && y > top as usize => {
                self.column_tops[column_idx] = Some(y as u8);
            }
            None if !voxel.is_air() => {
                self.column_tops[column_idx] = Some(y as u8);
            }
            _ => {}
        }

        true
    }

    #[inline]
    pub fn voxels(&self) -> &[VoxelId] {
        &self.voxels
    }

    #[inline]
    pub fn column_height(&self, x: usize, z: usize) -> Option<usize> {
        debug_assert!(x < SIZE);
        debug_assert!(z < SIZE);
        self.column_tops[column_index::<SIZE>(x, z)].map(|y| y as usize)
    }

    pub fn is_empty(&self) -> bool {
        self.solid_count == 0
    }

    pub fn count_solid(&self) -> usize {
        self.solid_count
    }

    fn refresh_column_top(&mut self, x: usize, z: usize) {
        let column_idx = column_index::<SIZE>(x, z);

        for y in (0..SIZE).rev() {
            let idx = x + y * SIZE + z * SIZE * SIZE;
            if !self.voxels[idx].is_air() {
                self.column_tops[column_idx] = Some(y as u8);
                return;
            }
        }

        self.column_tops[column_idx] = None;
    }
}

impl<const SIZE: usize> Default for Chunk<SIZE> {
    fn default() -> Self {
        Self::new_filled(VoxelId::AIR)
    }
}

#[inline]
fn column_index<const SIZE: usize>(x: usize, z: usize) -> usize {
    x + z * SIZE
}

#[cfg(test)]
mod tests {
    use super::Chunk;
    use crate::{IVec3, VoxelId};

    #[test]
    fn chunk_tracks_solid_count_and_column_top() {
        let mut chunk = Chunk::<4>::default();
        let column = IVec3::new(1, 0, 2);

        assert!(chunk.is_empty());
        assert_eq!(chunk.count_solid(), 0);
        assert_eq!(chunk.column_height(1, 2), None);

        assert!(chunk.set(column, VoxelId::STONE));
        assert_eq!(chunk.count_solid(), 1);
        assert_eq!(chunk.column_height(1, 2), Some(0));

        assert!(chunk.set(IVec3::new(1, 3, 2), VoxelId::GRASS));
        assert_eq!(chunk.count_solid(), 2);
        assert_eq!(chunk.column_height(1, 2), Some(3));

        assert!(chunk.set(IVec3::new(1, 3, 2), VoxelId::AIR));
        assert_eq!(chunk.count_solid(), 1);
        assert_eq!(chunk.column_height(1, 2), Some(0));
    }
}
