pub mod chunk;
pub mod coords;
pub mod generation;
pub mod voxel;
pub mod world;

pub use chunk::Chunk;
pub use coords::IVec3;
pub use voxel::VoxelId;
pub use world::{VoxelWorld, WorldDimensions};

pub const DEFAULT_CHUNK_SIZE: usize = 16;
pub type DefaultChunk = Chunk<DEFAULT_CHUNK_SIZE>;
pub type DefaultWorld = VoxelWorld<DEFAULT_CHUNK_SIZE>;
