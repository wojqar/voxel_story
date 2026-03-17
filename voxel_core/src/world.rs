use crate::chunk::Chunk;
use crate::coords::{world_to_chunk, IVec3};
use crate::voxel::VoxelId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldDimensions {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl WorldDimensions {
    pub const fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    pub const fn chunk_count(self) -> usize {
        (self.x as usize) * (self.y as usize) * (self.z as usize)
    }
}

#[derive(Debug, Clone)]
pub struct VoxelWorld<const SIZE: usize = 16> {
    chunks: Vec<Chunk<SIZE>>,
    pub dimensions: WorldDimensions,
    pub solid_count: usize,
}

impl<const SIZE: usize> VoxelWorld<SIZE> {
    pub fn new(dimensions: WorldDimensions) -> Self {
        let chunks = vec![Chunk::<SIZE>::default(); dimensions.chunk_count()];
        Self {
            chunks,
            dimensions,
            solid_count: 0,
        }
    }

    #[inline]
    pub fn contains(&self, world_voxel: IVec3) -> bool {
        let max_x = (self.dimensions.x as i32) * (SIZE as i32);
        let max_y = (self.dimensions.y as i32) * (SIZE as i32);
        let max_z = (self.dimensions.z as i32) * (SIZE as i32);
        world_voxel.x >= 0
            && world_voxel.y >= 0
            && world_voxel.z >= 0
            && world_voxel.x < max_x
            && world_voxel.y < max_y
            && world_voxel.z < max_z
    }

    #[inline]
    pub fn get_chunk(&self, chunk_coord: IVec3) -> Option<&Chunk<SIZE>> {
        let idx = self.chunk_index(chunk_coord)?;
        self.chunks.get(idx)
    }

    #[inline]
    pub fn get_chunk_mut(&mut self, chunk_coord: IVec3) -> Option<&mut Chunk<SIZE>> {
        let idx = self.chunk_index(chunk_coord)?;
        self.chunks.get_mut(idx)
    }

    pub fn get_voxel(&self, world_voxel: IVec3) -> VoxelId {
        if !self.contains(world_voxel) {
            return VoxelId::AIR;
        }
        let (chunk_coord, local) = world_to_chunk::<SIZE>(world_voxel);
        self.get_chunk(chunk_coord)
            .map(|c| c.get(local))
            .unwrap_or(VoxelId::AIR)
    }

    pub fn set_voxel(&mut self, world_voxel: IVec3, voxel: VoxelId) -> bool {
        if !self.contains(world_voxel) {
            return false;
        }

        let (chunk_coord, local) = world_to_chunk::<SIZE>(world_voxel);
        let Some(chunk) = self.get_chunk_mut(chunk_coord) else {
            return false;
        };

        let prev = chunk.get(local);
        if prev == voxel {
            return false;
        }

        let changed = chunk.set(local, voxel);
        if changed {
            let prev_solid = !prev.is_air();
            let new_solid = !voxel.is_air();
            match (prev_solid, new_solid) {
                (false, true) => self.solid_count += 1,
                (true, false) => self.solid_count = self.solid_count.saturating_sub(1),
                _ => {}
            }
        }
        changed
    }

    #[inline]
    fn chunk_index(&self, chunk_coord: IVec3) -> Option<usize> {
        if chunk_coord.x < 0 || chunk_coord.y < 0 || chunk_coord.z < 0 {
            return None;
        }
        let x = chunk_coord.x as u32;
        let y = chunk_coord.y as u32;
        let z = chunk_coord.z as u32;
        if x >= self.dimensions.x || y >= self.dimensions.y || z >= self.dimensions.z {
            return None;
        }
        Some((x as usize) + (y as usize) * (self.dimensions.x as usize)
            + (z as usize) * (self.dimensions.x as usize) * (self.dimensions.y as usize))
    }
}

