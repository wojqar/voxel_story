use bevy::math::IVec3;
use crate::chunk::CHUNK_SIZE;

/// Przelicza pozycję world-space na współrzędne chunka i lokalne xyz.
#[inline]
pub fn world_to_chunk(pos: IVec3) -> (IVec3, usize, usize, usize) {
    let cs = CHUNK_SIZE as i32;
    let chunk = IVec3::new(
        pos.x.div_euclid(cs),
        pos.y.div_euclid(cs),
        pos.z.div_euclid(cs),
    );
    let lx = pos.x.rem_euclid(cs) as usize;
    let ly = pos.y.rem_euclid(cs) as usize;
    let lz = pos.z.rem_euclid(cs) as usize;
    (chunk, lx, ly, lz)
}