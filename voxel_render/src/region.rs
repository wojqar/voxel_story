use bevy::prelude::*;

pub const REGION_SIZE_CHUNKS: i32 = 4;
pub const CHUNK_SIZE_VOXELS: i32 = 16;
pub const REGION_SIZE_VOXELS: i32 = REGION_SIZE_CHUNKS * CHUNK_SIZE_VOXELS; // 64

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl RegionCoord {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[inline]
pub fn chunk_to_region(chunk: IVec3) -> RegionCoord {
    RegionCoord::new(
        chunk.x.div_euclid(REGION_SIZE_CHUNKS),
        chunk.y.div_euclid(REGION_SIZE_CHUNKS),
        chunk.z.div_euclid(REGION_SIZE_CHUNKS),
    )
}

#[inline]
pub fn region_origin_world_voxel(region: RegionCoord) -> IVec3 {
    IVec3::new(
        region.x * REGION_SIZE_VOXELS,
        region.y * REGION_SIZE_VOXELS,
        region.z * REGION_SIZE_VOXELS,
    )
}

#[inline]
pub fn region_aabb_world(region: RegionCoord) -> (Vec3, Vec3) {
    let min = region_origin_world_voxel(region).as_vec3();
    let max = min + Vec3::splat(REGION_SIZE_VOXELS as f32);
    (min, max)
}

#[inline]
pub fn region_key(region: RegionCoord) -> i64 {
    // Stable key for maps/logging; coordinates are expected to be small.
    ((region.x as i64) & 0x1FFFFF)
        | (((region.y as i64) & 0x1FFFFF) << 21)
        | (((region.z as i64) & 0x1FFFFF) << 42)
}
