// generation/heightmap.rs
use crate::{
    chunk::{CHUNK_SIZE, Chunk},
    generation::WorldGenerator,
    voxel::VoxelId,
};
use bevy::math::IVec3;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

pub struct HeightmapGenerator {
    noise: Fbm<Perlin>,
    /// bazowa wysokość terenu w blokach
    pub base_height: i32,
    /// amplituda — max odchylenie od base_height
    pub amplitude: f64,
    /// skala horyzontalna (większa = łagodniejsze wzgórza)
    pub scale: f64,
}

impl HeightmapGenerator {
    pub fn new(seed: u32) -> Self {
        let noise = Fbm::<Perlin>::new(seed)
            .set_octaves(5)
            .set_frequency(1.0)
            .set_lacunarity(2.0)
            .set_persistence(0.5);

        Self {
            noise,
            base_height: 16,
            amplitude: 12.0,
            scale: 80.0,
        }
    }

    fn surface_height(&self, world_x: i32, world_z: i32) -> i32 {
        let nx = world_x as f64 / self.scale;
        let nz = world_z as f64 / self.scale;
        let n = self.noise.get([nx, nz]); // zakres ~[-1, 1]
        (self.base_height as f64 + n * self.amplitude).round() as i32
    }
}

impl WorldGenerator for HeightmapGenerator {
    fn generate_chunk(&self, coord: IVec3) -> Chunk {
        let mut chunk = Chunk::empty();
        let chunk_base_x = coord.x * CHUNK_SIZE as i32;
        let chunk_base_y = coord.y * CHUNK_SIZE as i32;
        let chunk_base_z = coord.z * CHUNK_SIZE as i32;

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x = chunk_base_x + x as i32;
                let world_z = chunk_base_z + z as i32;
                let surface = self.surface_height(world_x, world_z);

                for y in 0..CHUNK_SIZE {
                    let world_y = chunk_base_y + y as i32;
                    if world_y <= surface {
                        chunk.set(x, y, z, VoxelId(1)); // trawa / solid
                    }
                }
            }
        }

        chunk
    }
}
