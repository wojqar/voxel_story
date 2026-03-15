use crate::{
    chunk::{CHUNK_SIZE, Chunk},
    config::WorldConfig,
    generation::WorldGenerator,
    voxel::VoxelId,
};
use bevy::math::IVec3;
use noise::{NoiseFn, Perlin};

pub struct IslandGenerator {
    noise: Perlin,
    center: IVec3,
    radius: f32,
    thickness: f32,
    noise_scale: f64,
    noise_strength: f64,
}

impl IslandGenerator {
    pub fn from_config(cfg: &WorldConfig) -> Self {
        Self {
            noise: Perlin::new(cfg.seed),
            center: cfg.island_center(),
            radius: cfg.island_radius,
            thickness: cfg.island_thickness,
            noise_scale: cfg.noise_scale,
            noise_strength: cfg.noise_strength,
        }
    }

    fn density(&self, wx: i32, wy: i32, wz: i32) -> f64 {
        let dx = (wx as f64 - self.center.x as f64) / self.radius as f64;
        let dy = (wy as f64 - self.center.y as f64) / self.thickness as f64;
        let dz = (wz as f64 - self.center.z as f64) / self.radius as f64;

        // Kółkowy falloff poziomy — 1.0 w centrum, 0.0 na krawędzi
        let r = (dx * dx + dz * dz).sqrt().min(1.0);
        let radial = 1.0 - r;

        // Pionowy envelope asymetryczny:
        // Góra — płaskie cięcie (wyspa ma płaski wierzchołek)
        // Dół — parabola (skaliste stożkowe dno)
        let vertical = if dy >= 0.0 {
            (1.0 - dy * 1.2).max(0.0) // łagodne cięcie góry
        } else {
            (1.0 - dy * dy * 3.0).max(0.0) // parabola — ostre dno
        };

        // Prosty szum 3D bez fBm — surowy Perlin, więcej charakteru
        let n = self.noise.get([
            wx as f64 / self.noise_scale,
            wy as f64 / self.noise_scale,
            wz as f64 / self.noise_scale,
        ]);

        radial * vertical + n * self.noise_strength * vertical - 0.05
    }

    fn material(&self, chunk: &Chunk, lx: usize, ly: usize, lz: usize) -> VoxelId {
        let mut depth = 0usize;
        let mut y = ly + 1;
        while y < CHUNK_SIZE {
            if chunk.get(lx, y, lz).is_air() {
                break;
            }
            depth += 1;
            y += 1;
        }
        if y == CHUNK_SIZE && depth > 0 {
            return VoxelId::STONE;
        }
        match depth {
            0 => VoxelId::GRASS,
            1..=3 => VoxelId::DIRT,
            _ => VoxelId::STONE,
        }
    }
}

impl WorldGenerator for IslandGenerator {
    fn generate_chunk(&self, coord: IVec3) -> Chunk {
        let bx = coord.x * CHUNK_SIZE as i32;
        let by = coord.y * CHUNK_SIZE as i32;
        let bz = coord.z * CHUNK_SIZE as i32;
        let mut chunk = Chunk::empty();

        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if self.density(bx + x as i32, by + y as i32, bz + z as i32) > 0.0 {
                        chunk.set(x, y, z, VoxelId::STONE);
                    }
                }
            }
        }
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if chunk.get(x, y, z).is_air() {
                        continue;
                    }
                    chunk.set(x, y, z, self.material(&chunk, x, y, z));
                }
            }
        }
        chunk
    }
}
