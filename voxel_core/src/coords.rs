#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IVec3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl IVec3 {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[inline]
pub fn local_to_index<const SIZE: usize>(local: IVec3) -> usize {
    debug_assert!(local.x >= 0 && local.x < SIZE as i32);
    debug_assert!(local.y >= 0 && local.y < SIZE as i32);
    debug_assert!(local.z >= 0 && local.z < SIZE as i32);

    (local.x as usize) + (local.y as usize) * SIZE + (local.z as usize) * SIZE * SIZE
}

#[inline]
pub fn world_to_chunk<const SIZE: usize>(world: IVec3) -> (IVec3, IVec3) {
    let s = SIZE as i32;
    let chunk = IVec3::new(
        world.x.div_euclid(s),
        world.y.div_euclid(s),
        world.z.div_euclid(s),
    );
    let local = IVec3::new(
        world.x.rem_euclid(s),
        world.y.rem_euclid(s),
        world.z.rem_euclid(s),
    );
    (chunk, local)
}

#[inline]
pub fn chunk_to_world<const SIZE: usize>(chunk: IVec3, local: IVec3) -> IVec3 {
    let s = SIZE as i32;
    IVec3::new(
        chunk.x * s + local.x,
        chunk.y * s + local.y,
        chunk.z * s + local.z,
    )
}
