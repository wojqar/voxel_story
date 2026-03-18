use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use crate::chunk::Chunk;
use crate::coords::{IVec3, chunk_to_world};
use crate::generation::WorldGenerator;
use crate::voxel::VoxelId;
use crate::{DEFAULT_CHUNK_SIZE, WorldDimensions};

#[derive(Debug, Clone)]
pub struct CastleStoryGenerator {
    noise: Fbm<Perlin>,
    scale: f64,
    threshold: f32,
    center_x: f32,
    center_y: f32,
    center_z: f32,
    radius: f32,
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
    min_z: i32,
    max_z: i32,
}

impl CastleStoryGenerator {
    pub fn new(seed: u64, dimensions: WorldDimensions) -> Self {
        let world_width = dimensions.x as i32 * DEFAULT_CHUNK_SIZE as i32;
        let world_height = dimensions.y as i32 * DEFAULT_CHUNK_SIZE as i32;
        let world_depth = dimensions.z as i32 * DEFAULT_CHUNK_SIZE as i32;

        let center_x = world_width as f32 * 0.5;
        let center_y = world_height as f32 * 0.52;
        let center_z = world_depth as f32 * 0.5;
        let span = world_width.min(world_height).min(world_depth) as f32;
        let radius = (span * 0.48).clamp(40.0, span * 0.72);

        Self {
            noise: Fbm::<Perlin>::new(seed as u32)
                .set_octaves(2)
                .set_persistence(0.5)
                .set_lacunarity(2.0),
            scale: 3.0,
            threshold: 0.45,
            center_x,
            center_y,
            center_z,
            radius,
            min_x: (center_x - radius).floor() as i32,
            max_x: (center_x + radius).ceil() as i32,
            min_y: (center_y - radius).floor() as i32,
            max_y: (center_y + radius).ceil() as i32,
            min_z: (center_z - radius).floor() as i32,
            max_z: (center_z + radius).ceil() as i32,
        }
    }

    fn distance_mask(distance: f32) -> f32 {
        if distance <= 0.7 {
            return 1.0;
        }
        if distance >= 1.0 {
            return 0.0;
        }
        1.0 - (distance - 0.7) / 0.7
    }

    fn vertical_band_mask(vertical: f32) -> f32 {
        if vertical <= 0.0 {
            return 0.0;
        }
        if vertical < 0.4 {
            return vertical / 0.4;
        }
        if vertical < 0.6 {
            return 1.0 - (vertical - 0.4) / 0.2;
        }
        0.0
    }

    fn local_xz(&self, sample_x: f32, sample_z: f32) -> Option<(f32, f32)> {
        let local_x = (sample_x - self.center_x) / self.radius;
        let local_z = (sample_z - self.center_z) / self.radius;

        if local_x.abs() > 1.0 || local_z.abs() > 1.0 {
            return None;
        }

        Some((local_x, local_z))
    }

    #[inline]
    fn local_y(&self, sample_y: f32) -> f32 {
        (sample_y - self.center_y) / self.radius
    }

    fn density(&self, local_x: f32, local_y: f32, local_z: f32) -> f32 {
        let distance = local_x.abs().max(local_y.abs()).max(local_z.abs());
        let distance_mask = Self::distance_mask(distance);
        if distance_mask <= 0.0 {
            return 0.0;
        }

        let vertical = ((local_y + 1.0) * 0.5).clamp(0.0, 1.0);
        let vertical_mask = Self::vertical_band_mask(vertical);
        if vertical_mask <= 0.0 {
            return 0.0;
        }

        let noise = self.noise.get([
            local_x as f64 * self.scale,
            local_y as f64 * self.scale,
            local_z as f64 * self.scale,
        ]) as f32;
        let noise = (noise + 1.0) * 0.5;

        distance_mask * vertical_mask * noise
    }

    #[inline]
    fn is_solid(&self, local_x: f32, local_y: f32, local_z: f32) -> bool {
        self.density(local_x, local_y, local_z) >= self.threshold
    }

    fn chunk_intersects_volume(&self, chunk_coord: IVec3, size: i32) -> bool {
        let chunk_min_x = chunk_coord.x * size;
        let chunk_min_y = chunk_coord.y * size;
        let chunk_min_z = chunk_coord.z * size;
        let chunk_max_x = chunk_min_x + size - 1;
        let chunk_max_y = chunk_min_y + size - 1;
        let chunk_max_z = chunk_min_z + size - 1;

        !(chunk_max_x < self.min_x
            || chunk_min_x > self.max_x
            || chunk_max_y < self.min_y
            || chunk_min_y > self.max_y
            || chunk_max_z < self.min_z
            || chunk_min_z > self.max_z)
    }
}

impl<const SIZE: usize> WorldGenerator<SIZE> for CastleStoryGenerator {
    fn generate_chunk(&self, chunk_coord: IVec3) -> Chunk<SIZE> {
        let size = SIZE as i32;
        if !self.chunk_intersects_volume(chunk_coord, size) {
            return Chunk::default();
        }

        let chunk_min_y = chunk_coord.y * size;
        let chunk_max_y = chunk_min_y + size - 1;
        let scan_floor = self.min_y.max(chunk_min_y);
        let origin = chunk_to_world::<SIZE>(chunk_coord, IVec3::ZERO);
        let mut chunk = Chunk::default();

        for local_z in 0..SIZE {
            let world_z = origin.z + local_z as i32;
            if world_z < self.min_z || world_z > self.max_z {
                continue;
            }

            for local_x in 0..SIZE {
                let world_x = origin.x + local_x as i32;
                if world_x < self.min_x || world_x > self.max_x {
                    continue;
                }

                let Some((shape_x, shape_z)) =
                    self.local_xz(world_x as f32 + 0.5, world_z as f32 + 0.5)
                else {
                    continue;
                };

                let mut solids = [false; SIZE];
                let mut surface_y = None;

                for world_y in (scan_floor..=self.max_y).rev() {
                    let shape_y = self.local_y(world_y as f32 + 0.5);
                    if shape_y.abs() > 1.0 {
                        continue;
                    }

                    let solid = self.is_solid(shape_x, shape_y, shape_z);
                    if solid && surface_y.is_none() {
                        surface_y = Some(world_y);
                    }

                    if world_y >= chunk_min_y && world_y <= chunk_max_y {
                        solids[(world_y - chunk_min_y) as usize] = solid;
                    }
                }

                let Some(surface_y) = surface_y else { continue };
                if surface_y < chunk_min_y {
                    continue;
                }

                let local_top = (surface_y.min(chunk_max_y) - chunk_min_y) as usize;
                let local_bottom = (scan_floor - chunk_min_y) as usize;

                for local_y in local_bottom..=local_top {
                    if !solids[local_y] {
                        continue;
                    }

                    let depth_from_surface = surface_y - (chunk_min_y + local_y as i32);
                    let voxel = if depth_from_surface == 0 {
                        VoxelId::GRASS
                    } else if depth_from_surface <= 2 {
                        VoxelId::DIRT
                    } else {
                        VoxelId::STONE
                    };

                    chunk.set(
                        IVec3::new(local_x as i32, local_y as i32, local_z as i32),
                        voxel,
                    );
                }
            }
        }

        chunk
    }
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
