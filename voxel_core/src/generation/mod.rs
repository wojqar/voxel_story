use crate::chunk::Chunk;
use crate::coords::IVec3;

pub mod islands;
pub use islands::{IslandsGenerator, IslandsParams};

pub trait WorldGenerator<const SIZE: usize = 16> {
    fn generate_chunk(&self, chunk_coord: IVec3) -> Chunk<SIZE>;
}

