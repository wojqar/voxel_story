use crate::chunk::Chunk;
use crate::coords::{IVec3, world_to_chunk};
use crate::generation::WorldGenerator;
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
    column_tops: Vec<Option<i32>>,
    pub dimensions: WorldDimensions,
    pub solid_count: usize,
}

impl<const SIZE: usize> VoxelWorld<SIZE> {
    pub fn new(dimensions: WorldDimensions) -> Self {
        let chunks = vec![Chunk::<SIZE>::default(); dimensions.chunk_count()];
        let column_tops = vec![None; (dimensions.x as usize) * (dimensions.z as usize) * SIZE * SIZE];
        Self {
            chunks,
            column_tops,
            dimensions,
            solid_count: 0,
        }
    }

    pub fn from_generator<G>(dimensions: WorldDimensions, generator: &G) -> Self
    where
        G: WorldGenerator<SIZE>,
    {
        let mut world = Self::new(dimensions);
        for z in 0..dimensions.z as i32 {
            for y in 0..dimensions.y as i32 {
                for x in 0..dimensions.x as i32 {
                    let chunk_coord = IVec3::new(x, y, z);
                    let chunk = generator.generate_chunk(chunk_coord);
                    world.replace_chunk(chunk_coord, chunk);
                }
            }
        }
        world
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
        let prev = self
            .get_chunk(chunk_coord)
            .map(|chunk| chunk.get(local))
            .unwrap_or(VoxelId::AIR);
        let Some(chunk) = self.get_chunk_mut(chunk_coord) else {
            return false;
        };

        let changed = chunk.set(local, voxel);
        if changed {
            let prev_solid = !prev.is_air();
            let new_solid = !voxel.is_air();
            match (prev_solid, new_solid) {
                (false, true) => self.solid_count += 1,
                (true, false) => self.solid_count = self.solid_count.saturating_sub(1),
                _ => {}
            }
            self.refresh_column(world_voxel.x, world_voxel.z);
        }
        changed
    }

    pub fn replace_chunk(&mut self, chunk_coord: IVec3, chunk: Chunk<SIZE>) -> bool {
        let Some(idx) = self.chunk_index(chunk_coord) else {
            return false;
        };

        let prev_solid = self.chunks[idx].count_solid();
        let new_solid = chunk.count_solid();
        self.chunks[idx] = chunk;
        self.solid_count = self
            .solid_count
            .saturating_sub(prev_solid)
            .saturating_add(new_solid);
        self.refresh_chunk_columns(chunk_coord);
        true
    }

    pub fn column_height(&self, x: i32, z: i32) -> Option<i32> {
        self.column_index(x, z).and_then(|idx| self.column_tops[idx])
    }

    pub fn snapshot_chunk_aligned_region_u16(
        &self,
        origin_chunk: IVec3,
        chunk_dimensions: IVec3,
    ) -> (Vec<u16>, usize) {
        debug_assert!(chunk_dimensions.x >= 0);
        debug_assert!(chunk_dimensions.y >= 0);
        debug_assert!(chunk_dimensions.z >= 0);

        let voxel_dims = IVec3::new(
            chunk_dimensions.x * SIZE as i32,
            chunk_dimensions.y * SIZE as i32,
            chunk_dimensions.z * SIZE as i32,
        );
        let nx = voxel_dims.x as usize;
        let ny = voxel_dims.y as usize;
        let nz = voxel_dims.z as usize;
        let mut voxels = vec![0u16; nx * ny * nz];
        let mut solid_voxels = 0usize;

        for chunk_z in 0..chunk_dimensions.z {
            for chunk_y in 0..chunk_dimensions.y {
                for chunk_x in 0..chunk_dimensions.x {
                    let chunk_coord = IVec3::new(
                        origin_chunk.x + chunk_x,
                        origin_chunk.y + chunk_y,
                        origin_chunk.z + chunk_z,
                    );
                    let Some(chunk) = self.get_chunk(chunk_coord) else {
                        continue;
                    };

                    solid_voxels += chunk.count_solid();

                    let dst_chunk_x = chunk_x as usize * SIZE;
                    let dst_chunk_y = chunk_y as usize * SIZE;
                    let dst_chunk_z = chunk_z as usize * SIZE;
                    let src = chunk.voxels();

                    for local_z in 0..SIZE {
                        let dst_z = dst_chunk_z + local_z;
                        let src_z = local_z * SIZE * SIZE;

                        for local_y in 0..SIZE {
                            let dst_y = dst_chunk_y + local_y;
                            let dst_index = dst_chunk_x + dst_y * nx + dst_z * nx * ny;
                            let src_index = src_z + local_y * SIZE;

                            for (dst, voxel) in voxels[dst_index..dst_index + SIZE]
                                .iter_mut()
                                .zip(&src[src_index..src_index + SIZE])
                            {
                                *dst = voxel.0;
                            }
                        }
                    }
                }
            }
        }

        (voxels, solid_voxels)
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
        Some(
            (x as usize)
                + (y as usize) * (self.dimensions.x as usize)
                + (z as usize) * (self.dimensions.x as usize) * (self.dimensions.y as usize),
        )
    }

    #[inline]
    fn column_index(&self, x: i32, z: i32) -> Option<usize> {
        if x < 0
            || z < 0
            || x >= (self.dimensions.x as i32) * (SIZE as i32)
            || z >= (self.dimensions.z as i32) * (SIZE as i32)
        {
            return None;
        }

        Some((x as usize) + (z as usize) * (self.dimensions.x as usize) * SIZE)
    }

    fn refresh_chunk_columns(&mut self, chunk_coord: IVec3) {
        let origin_x = chunk_coord.x * SIZE as i32;
        let origin_z = chunk_coord.z * SIZE as i32;

        for local_z in 0..SIZE as i32 {
            for local_x in 0..SIZE as i32 {
                self.refresh_column(origin_x + local_x, origin_z + local_z);
            }
        }
    }

    fn refresh_column(&mut self, x: i32, z: i32) {
        let Some(index) = self.column_index(x, z) else {
            return;
        };

        let chunk_x = x.div_euclid(SIZE as i32);
        let chunk_z = z.div_euclid(SIZE as i32);
        let local_x = x.rem_euclid(SIZE as i32) as usize;
        let local_z = z.rem_euclid(SIZE as i32) as usize;

        let mut top = None;
        for chunk_y in (0..self.dimensions.y as i32).rev() {
            let Some(chunk) = self.get_chunk(IVec3::new(chunk_x, chunk_y, chunk_z)) else {
                continue;
            };

            if let Some(local_y) = chunk.column_height(local_x, local_z) {
                top = Some(chunk_y * SIZE as i32 + local_y as i32);
                break;
            }
        }

        self.column_tops[index] = top;
    }
}

#[cfg(test)]
mod tests {
    use super::{VoxelWorld, WorldDimensions};
    use crate::{Chunk, IVec3, VoxelId};

    #[test]
    fn world_column_height_updates_in_o_one_queries() {
        let mut world = VoxelWorld::<4>::new(WorldDimensions::new(1, 2, 1));

        assert_eq!(world.column_height(0, 0), None);

        assert!(world.set_voxel(IVec3::new(0, 1, 0), VoxelId::DIRT));
        assert_eq!(world.column_height(0, 0), Some(1));

        assert!(world.set_voxel(IVec3::new(0, 6, 0), VoxelId::STONE));
        assert_eq!(world.column_height(0, 0), Some(6));

        assert!(world.set_voxel(IVec3::new(0, 6, 0), VoxelId::AIR));
        assert_eq!(world.column_height(0, 0), Some(1));
    }

    #[test]
    fn region_snapshot_copies_chunk_aligned_data() {
        let mut world = VoxelWorld::<2>::new(WorldDimensions::new(2, 1, 1));

        let mut left = Chunk::<2>::default();
        assert!(left.set(IVec3::new(0, 0, 0), VoxelId::DIRT));
        assert!(left.set(IVec3::new(1, 1, 1), VoxelId::GRASS));

        let mut right = Chunk::<2>::default();
        assert!(right.set(IVec3::new(0, 1, 0), VoxelId::STONE));

        assert!(world.replace_chunk(IVec3::new(0, 0, 0), left));
        assert!(world.replace_chunk(IVec3::new(1, 0, 0), right));

        let (voxels, solid_voxels) =
            world.snapshot_chunk_aligned_region_u16(IVec3::ZERO, IVec3::new(2, 1, 1));

        assert_eq!(solid_voxels, 3);
        assert_eq!(voxels.len(), 16);
        assert_eq!(voxels[0], VoxelId::DIRT.0);
        assert_eq!(voxels[6], VoxelId::STONE.0);
        assert_eq!(voxels[13], VoxelId::GRASS.0);
    }
}
