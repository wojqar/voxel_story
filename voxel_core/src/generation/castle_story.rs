use std::f32::consts::TAU;

use crate::chunk::Chunk;
use crate::coords::{IVec3, chunk_to_world};
use crate::generation::WorldGenerator;
use crate::voxel::VoxelId;
use crate::{DEFAULT_CHUNK_SIZE, WorldDimensions};

const CELL_SIZE: f32 = 64.0;
const ISLAND_MARGIN: f32 = 1.08;
const EDGE_FADE_DISTANCE: f32 = 24.0;

#[derive(Debug, Clone, Copy)]
pub struct CastleStoryGenerator {
    seed: u64,
    world_width: i32,
    world_height: i32,
    world_depth: i32,
    min_possible_bottom: i32,
    max_possible_top: i32,
}

#[derive(Debug, Clone, Copy)]
struct ColumnShape {
    bottom: i32,
    top: i32,
    influence: f32,
    dirt_depth: i32,
}

#[derive(Debug, Clone, Copy)]
struct IslandCandidate {
    center_x: f32,
    center_z: f32,
    radius_x: f32,
    radius_z: f32,
    rotation_sin: f32,
    rotation_cos: f32,
    base_altitude: f32,
    crown_height: f32,
    core_depth: f32,
}

impl CastleStoryGenerator {
    pub fn new(seed: u64, dimensions: WorldDimensions) -> Self {
        let world_width = dimensions.x as i32 * DEFAULT_CHUNK_SIZE as i32;
        let world_height = dimensions.y as i32 * DEFAULT_CHUNK_SIZE as i32;
        let world_depth = dimensions.z as i32 * DEFAULT_CHUNK_SIZE as i32;

        let altitude_min = (world_height as f32 * 0.34).max(24.0);
        let altitude_max = (world_height as f32 * 0.52).max(altitude_min + 8.0);
        let max_crown = (world_height as f32 * 0.12).clamp(10.0, 18.0);
        let max_depth = (world_height as f32 * 0.22).clamp(16.0, 30.0);

        Self {
            seed,
            world_width,
            world_height,
            world_depth,
            min_possible_bottom: ((altitude_min - max_depth - 8.0).floor() as i32).max(1),
            max_possible_top: ((altitude_max + max_crown + 6.0).ceil() as i32)
                .min(world_height.saturating_sub(2)),
        }
    }

    pub fn surface_height(&self, world_x: i32, world_z: i32) -> Option<i32> {
        self.sample_column(world_x, world_z)
            .map(|column| column.top)
    }

    fn sample_column(&self, world_x: i32, world_z: i32) -> Option<ColumnShape> {
        if world_x < 0 || world_z < 0 || world_x >= self.world_width || world_z >= self.world_depth
        {
            return None;
        }

        let edge_fade = self.edge_fade(world_x as f32 + 0.5, world_z as f32 + 0.5);
        if edge_fade <= 0.0 {
            return None;
        }

        let cell_x = (world_x as f32 / CELL_SIZE).floor() as i32;
        let cell_z = (world_z as f32 / CELL_SIZE).floor() as i32;
        let sample_x = world_x as f32 + 0.5;
        let sample_z = world_z as f32 + 0.5;

        let mut best: Option<(f32, ColumnShape)> = None;

        for dz in -1..=1 {
            for dx in -1..=1 {
                let Some(island) = self.island_candidate(cell_x + dx, cell_z + dz) else {
                    continue;
                };

                let rel_x = sample_x - island.center_x;
                let rel_z = sample_z - island.center_z;
                let rot_x = rel_x * island.rotation_cos + rel_z * island.rotation_sin;
                let rot_z = -rel_x * island.rotation_sin + rel_z * island.rotation_cos;
                let dist =
                    ((rot_x / island.radius_x).powi(2) + (rot_z / island.radius_z).powi(2)).sqrt();
                if dist >= ISLAND_MARGIN {
                    continue;
                }

                let radial = ((ISLAND_MARGIN - dist) / ISLAND_MARGIN).clamp(0.0, 1.0);
                let mass = smoothstep(radial) * edge_fade;
                if mass <= 0.02 {
                    continue;
                }

                let broad_noise = fbm2(
                    self.seed ^ 0xA4F1_C2D3_B5E6_9182,
                    sample_x * 0.018,
                    sample_z * 0.018,
                    3,
                );
                let detail_noise = fbm2(
                    self.seed ^ 0x37F5_AA01_8C51_D234,
                    sample_x * 0.072,
                    sample_z * 0.072,
                    2,
                );
                let underside_noise = ridged_fbm2(
                    self.seed ^ 0x9EDB_0A5F_54A7_31C9,
                    sample_x * 0.046,
                    sample_z * 0.046,
                    3,
                );

                let crown = island.crown_height * mass.powf(0.42);
                let cliff_drop = (1.0 - mass).powf(1.55) * 5.5;
                let top = island.base_altitude
                    + crown
                    + broad_noise * (4.0 * mass + 1.5)
                    + detail_noise * (1.8 * mass + 0.4)
                    - cliff_drop;
                let thickness = 3.5
                    + island.core_depth * mass.powf(1.45)
                    + underside_noise * (7.0 * mass + 1.0);
                let bottom = top - thickness;
                let dirt_depth = if mass > 0.78 {
                    4
                } else if mass > 0.48 {
                    3
                } else {
                    2
                };

                let column = ColumnShape {
                    bottom: bottom.floor() as i32,
                    top: top.floor() as i32,
                    influence: mass,
                    dirt_depth,
                };
                let score = mass * (1.0 + island.core_depth * 0.01);

                if best
                    .as_ref()
                    .is_none_or(|(best_score, _)| score > *best_score)
                {
                    best = Some((score, column));
                }
            }
        }

        let (_, mut column) = best?;
        column.bottom = column.bottom.max(1);
        column.top = column.top.min(self.world_height.saturating_sub(2));

        if column.top <= column.bottom {
            return None;
        }

        Some(column)
    }

    fn island_candidate(&self, cell_x: i32, cell_z: i32) -> Option<IslandCandidate> {
        let hero_cell = cell_x == 0 && cell_z == 0;
        let spawn_roll = hash01(self.seed ^ 0x5E7A_C1D3_9B42_1180, cell_x, 0, cell_z);
        if !hero_cell && spawn_roll < 0.42 {
            return None;
        }

        let jitter_x = if hero_cell {
            0.62
        } else {
            0.24 + hash01(self.seed ^ 0x1B44_C6D2_EE71_3AF9, cell_x, 0, cell_z) * 0.52
        };
        let jitter_z = if hero_cell {
            0.68
        } else {
            0.24 + hash01(self.seed ^ 0x884A_16F3_4DCE_20A1, cell_x, 1, cell_z) * 0.52
        };
        let center_x = (cell_x as f32 + jitter_x) * CELL_SIZE;
        let center_z = (cell_z as f32 + jitter_z) * CELL_SIZE;

        let angle = hash01(self.seed ^ 0xD054_2BAF_1266_91D4, cell_x, 2, cell_z) * TAU;
        let radius_x = if hero_cell {
            36.0
        } else {
            24.0 + hash01(self.seed ^ 0x72A1_0C4F_90BE_6DD2, cell_x, 3, cell_z) * 18.0
        };
        let radius_z = if hero_cell {
            30.0
        } else {
            20.0 + hash01(self.seed ^ 0xCC15_71B2_4E3A_8DF0, cell_x, 4, cell_z) * 16.0
        };

        let altitude_min = (self.world_height as f32 * 0.34).max(24.0);
        let altitude_span = (self.world_height as f32 * 0.16).clamp(10.0, 22.0);
        let base_altitude = if hero_cell {
            altitude_min + altitude_span * 0.32
        } else {
            altitude_min
                + altitude_span * hash01(self.seed ^ 0x91F2_53E0_77A8_114C, cell_x, 5, cell_z)
        };
        let crown_height = if hero_cell {
            13.0
        } else {
            8.0 + hash01(self.seed ^ 0xA913_7C06_12F0_65AB, cell_x, 6, cell_z) * 8.0
        };
        let core_depth = if hero_cell {
            24.0
        } else {
            16.0 + hash01(self.seed ^ 0x0E51_8E5F_6B63_20DD, cell_x, 7, cell_z) * 12.0
        };

        Some(IslandCandidate {
            center_x,
            center_z,
            radius_x,
            radius_z,
            rotation_sin: angle.sin(),
            rotation_cos: angle.cos(),
            base_altitude,
            crown_height,
            core_depth,
        })
    }

    fn edge_fade(&self, x: f32, z: f32) -> f32 {
        let distance = x
            .min(self.world_width as f32 - x)
            .min(z)
            .min(self.world_depth as f32 - z);
        smoothstep((distance / EDGE_FADE_DISTANCE).clamp(0.0, 1.0))
    }
}

impl<const SIZE: usize> WorldGenerator<SIZE> for CastleStoryGenerator {
    fn generate_chunk(&self, chunk_coord: IVec3) -> Chunk<SIZE> {
        let chunk_min_y = chunk_coord.y * SIZE as i32;
        let chunk_max_y = chunk_min_y + SIZE as i32 - 1;
        if chunk_max_y < self.min_possible_bottom || chunk_min_y > self.max_possible_top {
            return Chunk::default();
        }

        let mut chunk = Chunk::default();
        let origin = chunk_to_world::<SIZE>(chunk_coord, IVec3::ZERO);

        for local_z in 0..SIZE as i32 {
            for local_x in 0..SIZE as i32 {
                let world_x = origin.x + local_x;
                let world_z = origin.z + local_z;
                let Some(column) = self.sample_column(world_x, world_z) else {
                    continue;
                };
                if column.bottom > chunk_max_y || column.top < chunk_min_y {
                    continue;
                }

                let local_bottom = (column.bottom - chunk_min_y).clamp(0, SIZE as i32 - 1);
                let local_top = (column.top - chunk_min_y).clamp(0, SIZE as i32 - 1);

                for local_y in local_bottom..=local_top {
                    let world_y = chunk_min_y + local_y;
                    let depth_from_surface = column.top - world_y;

                    let voxel = if depth_from_surface == 0 {
                        VoxelId::GRASS
                    } else if depth_from_surface <= column.dirt_depth && column.influence > 0.22 {
                        VoxelId::DIRT
                    } else {
                        VoxelId::STONE
                    };

                    chunk.set(IVec3::new(local_x, local_y, local_z), voxel);
                }
            }
        }

        chunk
    }
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn fbm2(seed: u64, x: f32, z: f32, octaves: u32) -> f32 {
    let mut sum = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut norm = 0.0;

    for _ in 0..octaves {
        sum += value_noise2(seed, x * frequency, z * frequency) * amplitude;
        norm += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    if norm == 0.0 { 0.0 } else { sum / norm }
}

fn ridged_fbm2(seed: u64, x: f32, z: f32, octaves: u32) -> f32 {
    let mut sum = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut norm = 0.0;

    for octave in 0..octaves {
        let n = value_noise2(
            seed.wrapping_add(octave as u64),
            x * frequency,
            z * frequency,
        );
        sum += (1.0 - n.abs()) * amplitude;
        norm += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    if norm == 0.0 { 0.0 } else { sum / norm }
}

fn value_noise2(seed: u64, x: f32, z: f32) -> f32 {
    let x0 = x.floor() as i32;
    let z0 = z.floor() as i32;
    let x1 = x0 + 1;
    let z1 = z0 + 1;
    let tx = smoothstep(x - x.floor());
    let tz = smoothstep(z - z.floor());

    let v00 = hash_signed(seed, x0, 0, z0);
    let v10 = hash_signed(seed, x1, 0, z0);
    let v01 = hash_signed(seed, x0, 0, z1);
    let v11 = hash_signed(seed, x1, 0, z1);

    let a = lerp(v00, v10, tx);
    let b = lerp(v01, v11, tx);
    lerp(a, b, tz)
}

fn hash_signed(seed: u64, x: i32, y: i32, z: i32) -> f32 {
    hash01(seed, x, y, z) * 2.0 - 1.0
}

fn hash01(seed: u64, x: i32, y: i32, z: i32) -> f32 {
    let value = mix64(
        seed ^ (x as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (y as i64 as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ (z as i64 as u64).wrapping_mul(0x94D0_49BB_1331_11EB),
    );
    let mantissa = (value >> 40) as u32;
    mantissa as f32 / ((1u32 << 24) - 1) as f32
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::CastleStoryGenerator;
    use crate::generation::WorldGenerator;
    use crate::{DefaultWorld, IVec3, VoxelId, WorldDimensions};

    #[test]
    fn castle_story_generator_creates_floating_islands() {
        let dimensions = WorldDimensions::new(6, 6, 6);
        let generator = CastleStoryGenerator::new(7, dimensions);
        let world = DefaultWorld::from_generator(dimensions, &generator);

        assert!(world.solid_count > 0);

        let max_x = dimensions.x as i32 * crate::DEFAULT_CHUNK_SIZE as i32;
        let max_y = dimensions.y as i32 * crate::DEFAULT_CHUNK_SIZE as i32;
        let max_z = dimensions.z as i32 * crate::DEFAULT_CHUNK_SIZE as i32;

        let mut lowest_solid = i32::MAX;
        let mut has_grass = false;
        let mut has_stone = false;

        for z in 0..max_z {
            for x in 0..max_x {
                for y in 0..max_y {
                    let voxel = world.get_voxel(IVec3::new(x, y, z));
                    if voxel.is_air() {
                        continue;
                    }
                    lowest_solid = lowest_solid.min(y);
                    has_grass |= voxel == VoxelId::GRASS;
                    has_stone |= voxel == VoxelId::STONE;
                }
            }
        }

        assert!(lowest_solid > 0);
        assert!(has_grass);
        assert!(has_stone);
    }

    #[test]
    fn castle_story_generator_is_deterministic_for_a_seed() {
        let dimensions = WorldDimensions::new(6, 6, 6);
        let generator_a = CastleStoryGenerator::new(123, dimensions);
        let generator_b = CastleStoryGenerator::new(123, dimensions);

        let chunk_a: crate::Chunk<16> = generator_a.generate_chunk(IVec3::new(0, 2, 0));
        let chunk_b: crate::Chunk<16> = generator_b.generate_chunk(IVec3::new(0, 2, 0));

        for z in 0..16 {
            for y in 0..16 {
                for x in 0..16 {
                    let local = IVec3::new(x, y, z);
                    assert_eq!(chunk_a.get(local), chunk_b.get(local));
                }
            }
        }
    }
}
