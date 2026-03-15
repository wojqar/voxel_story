use crate::chunk::Chunk;
use bevy::math::IVec3;

pub mod heightmap;

pub trait WorldGenerator: Send + Sync {
    fn generate_chunk(&self, coord: IVec3) -> Chunk;
}