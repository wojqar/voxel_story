use crate::{
    chunk::{CHUNK_SIZE, Chunk},
    generation::WorldGenerator,
    voxel::VoxelId,
};
use bevy::math::IVec3;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

pub struct IslandGenerator {
    noise: Fbm<Perlin>,
    /// centrum wyspy w przestrzeni bloków
    pub center: IVec3,
    /// promień wyspy w poziomie (bloki)
    pub radius: f32,
    /// grubość wyspy — połowa powyżej i poniżej center.y
    pub thickness: f32,
    /// siła szumu 3D (im większa tym bardziej skalista bryła)
    pub noise_strength: f64,
    /// skala szumu
    pub noise_scale: f64,
}

impl IslandGenerator {
    pub fn new(seed: u32) -> Self {
        let noise = Fbm::<Perlin>::new(seed)
            .set_octaves(5)
            .set_frequency(1.0)
            .set_lacunarity(2.0)
            .set_persistence(0.5);

        Self {
            noise,
            center: IVec3::new(64, 32, 64),
            radius: 48.0,
            thickness: 14.0,
            noise_strength: 0.55,
            noise_scale: 28.0,
        }
    }

    /// Wartość gęstości dla danego world-space voxela.
    /// > 0.0 → solidny, <= 0.0 → powietrze.
    fn density(&self, wx: i32, wy: i32, wz: i32) -> f64 {
        let cx = self.center.x as f64;
        let cy = self.center.y as f64;
        let cz = self.center.z as f64;

        let dx = (wx as f64 - cx) / self.radius as f64;
        let dz = (wz as f64 - cz) / self.radius as f64;
        let dy = (wy as f64 - cy) / self.thickness as f64;

        // Radialny falloff w poziomie — kółko
        let radial = 1.0 - (dx * dx + dz * dz).sqrt();

        // Pionowy envelope — trapez: płaska góra, stromy dół
        // Górna połowa jest szersza niż dolna → efekt oderwania
        let vertical = if dy >= 0.0 {
            // powyżej centrum: szybki falloff
            1.0 - (dy * 1.4).powi(2)
        } else {
            // poniżej centrum: bardzo stromy — ostre dno wyspy
            1.0 - (dy * 2.2).powi(2)
        };

        // 3D noise — organiczna nieregularność
        let nx = wx as f64 / self.noise_scale;
        let ny = wy as f64 / self.noise_scale;
        let nz = wz as f64 / self.noise_scale;
        let n = self.noise.get([nx, ny, nz]); // ~[-1, 1]

        radial + vertical - 1.0 + n * self.noise_strength
    }

    /// Dla solidnego voxela określa materiał na podstawie tego
    /// co jest bezpośrednio powyżej (cross-chunk nieobsługiwane — uproszczenie).
    fn material(&self, chunk: &Chunk, lx: usize, ly: usize, lz: usize) -> VoxelId {
        // Sprawdź ile warstw solidnych jest powyżej w tym samym chunku
        let mut depth = 0usize;
        let mut y = ly + 1;
        while y < CHUNK_SIZE {
            if chunk.get(lx, y, lz).is_air() {
                break;
            }
            depth += 1;
            y += 1;
        }
        // Jeśli jesteśmy na szczycie chunku i nie natrafiliśmy na powietrze,
        // zakładamy że coś jest powyżej — dajemy STONE
        if y == CHUNK_SIZE && depth > 0 {
            return VoxelId::STONE;
        }

        match depth {
            0 => VoxelId::GRASS,
            1..=2 => VoxelId::DIRT,
            _ => VoxelId::STONE,
        }
    }
}

impl WorldGenerator for IslandGenerator {
    fn generate_chunk(&self, coord: IVec3) -> Chunk {
        let base_x = coord.x * CHUNK_SIZE as i32;
        let base_y = coord.y * CHUNK_SIZE as i32;
        let base_z = coord.z * CHUNK_SIZE as i32;

        let mut chunk = Chunk::empty();

        // Pass 1: wypełnij density — tymczasowo STONE wszędzie gdzie solidny
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let wx = base_x + x as i32;
                    let wy = base_y + y as i32;
                    let wz = base_z + z as i32;
                    if self.density(wx, wy, wz) > 0.0 {
                        chunk.set(x, y, z, VoxelId::STONE);
                    }
                }
            }
        }

        // Pass 2: nadpisz materiały od góry — GRASS/DIRT/STONE
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if chunk.get(x, y, z).is_air() {
                        continue;
                    }
                    let mat = self.material(&chunk, x, y, z);
                    chunk.set(x, y, z, mat);
                }
            }
        }

        chunk
    }
}
