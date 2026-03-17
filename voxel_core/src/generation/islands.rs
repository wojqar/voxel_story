use crate::coords::IVec3;
use crate::voxel::VoxelId;
use crate::world::VoxelWorld;

#[derive(Debug, Clone, Copy)]
pub struct IslandsParams {
    /// How many island "centers" to place.
    pub island_count: u32,
    /// Minimum island radius in voxels.
    pub radius_min: f32,
    /// Maximum island radius in voxels.
    pub radius_max: f32,
    /// Noise frequency (higher => more detail).
    pub noise_freq: f32,
    /// Noise amplitude added to density.
    pub noise_amp: f32,
    /// Density threshold for solid voxels.
    pub threshold: f32,
    /// Voxel id to fill with.
    pub voxel: VoxelId,
}

impl Default for IslandsParams {
    fn default() -> Self {
        Self {
            island_count: 12,
            radius_min: 10.0,
            radius_max: 30.0,
            noise_freq: 0.06,
            noise_amp: 0.9,
            threshold: 0.0,
            voxel: VoxelId(3), // stone
        }
    }
}

#[derive(Debug, Clone)]
pub struct IslandsGenerator {
    pub params: IslandsParams,
}

impl IslandsGenerator {
    pub fn new(params: IslandsParams) -> Self {
        Self { params }
    }

    pub fn generate_into<const SIZE: usize>(&self, world: &mut VoxelWorld<SIZE>, seed: u64) {
        let p = self.params;

        let max_x = (world.dimensions.x as i32) * (SIZE as i32);
        let max_y = (world.dimensions.y as i32) * (SIZE as i32);
        let max_z = (world.dimensions.z as i32) * (SIZE as i32);
        if max_x <= 0 || max_y <= 0 || max_z <= 0 {
            return;
        }

        let mut rng = XorShift64::new(mix64(seed ^ 0xA2E3_92B7_4D18_0B6D));

        // Keep centers away from borders a bit to avoid clipped islands.
        let border = 4.0_f32;
        let fx_min = border;
        let fz_min = border;
        let fx_max = (max_x as f32) - 1.0 - border;
        let fz_max = (max_z as f32) - 1.0 - border;

        // Keep islands in a vertical band so they "float".
        let y_min = (max_y as f32) * 0.25;
        let y_max = (max_y as f32) * 0.85;

        let mut centers = Vec::with_capacity(p.island_count as usize);
        for _ in 0..p.island_count {
            let cx = rng.f32_range(fx_min.max(0.0), fx_max.max(0.0));
            let cz = rng.f32_range(fz_min.max(0.0), fz_max.max(0.0));
            let cy = rng.f32_range(y_min.max(0.0), (y_max.max(0.0)).max(y_min.max(0.0) + 1.0));
            let r = rng.f32_range(p.radius_min.max(1.0), p.radius_max.max(p.radius_min.max(1.0) + 1.0));
            centers.push(IslandCenter {
                cx,
                cy,
                cz,
                radius: r,
            });
        }

        // Fill solids.
        for z in 0..max_z {
            for y in 0..max_y {
                for x in 0..max_x {
                    let density = islands_density(
                        seed,
                        x as f32,
                        y as f32,
                        z as f32,
                        &centers,
                        p.noise_freq,
                        p.noise_amp,
                    );
                    if density > p.threshold {
                        world.set_voxel(IVec3::new(x, y, z), p.voxel);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct IslandCenter {
    cx: f32,
    cy: f32,
    cz: f32,
    radius: f32,
}

fn islands_density(seed: u64, x: f32, y: f32, z: f32, centers: &[IslandCenter], noise_freq: f32, noise_amp: f32) -> f32 {
    let mut best = -1e9_f32;
    for c in centers {
        let dx = x - c.cx;
        let dz = z - c.cz;
        let mut dy = y - c.cy;

        // Organic "blobby" look: slightly flatter tops, steeper undersides.
        let above = dy >= 0.0;
        dy *= if above { 1.15 } else { 1.85 };

        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        let base = 1.0 - (dist / c.radius);
        if base > best {
            best = base;
        }
    }

    if best <= -0.5 {
        return best;
    }

    let n = value_noise_3d(
        seed ^ 0x9E37_79B9_7F4A_7C15,
        x * noise_freq,
        y * noise_freq,
        z * noise_freq,
    );
    best + n * noise_amp
}

// ----------------------------
// Tiny deterministic RNG/noise
// ----------------------------

#[derive(Debug, Clone, Copy)]
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        let mut s = seed;
        if s == 0 {
            s = 0xD1B5_4A32_D192_ED03;
        }
        Self { state: s }
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    #[inline]
    fn next_f32(&mut self) -> f32 {
        // Use top 24 bits for stable f32 mantissa.
        let v = (self.next_u64() >> 40) as u32;
        (v as f32) * (1.0 / ((1u32 << 24) as f32))
    }

    #[inline]
    fn f32_range(&mut self, min: f32, max: f32) -> f32 {
        if max <= min {
            return min;
        }
        min + (max - min) * self.next_f32()
    }
}

#[inline]
fn mix64(mut x: u64) -> u64 {
    // SplitMix64 finalizer (good bit diffusion).
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    x
}

#[inline]
fn hash3_u32(seed: u64, xi: i32, yi: i32, zi: i32) -> u32 {
    let mut x = seed
        ^ (xi as u64).wrapping_mul(0x9E37_79B1)
        ^ (yi as u64).wrapping_mul(0x85EB_CA77)
        ^ (zi as u64).wrapping_mul(0xC2B2_AE3D);
    x = mix64(x);
    (x >> 32) as u32
}

#[inline]
fn fade(t: f32) -> f32 {
    // Smoothstep-ish curve: 6t^5 - 15t^4 + 10t^3
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn value_noise_3d(seed: u64, x: f32, y: f32, z: f32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let z0 = z.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;
    let z1 = z0 + 1;

    let tx = fade(x - (x0 as f32));
    let ty = fade(y - (y0 as f32));
    let tz = fade(z - (z0 as f32));

    let v000 = u32_to_unit(hash3_u32(seed, x0, y0, z0));
    let v100 = u32_to_unit(hash3_u32(seed, x1, y0, z0));
    let v010 = u32_to_unit(hash3_u32(seed, x0, y1, z0));
    let v110 = u32_to_unit(hash3_u32(seed, x1, y1, z0));
    let v001 = u32_to_unit(hash3_u32(seed, x0, y0, z1));
    let v101 = u32_to_unit(hash3_u32(seed, x1, y0, z1));
    let v011 = u32_to_unit(hash3_u32(seed, x0, y1, z1));
    let v111 = u32_to_unit(hash3_u32(seed, x1, y1, z1));

    let x00 = lerp(v000, v100, tx);
    let x10 = lerp(v010, v110, tx);
    let x01 = lerp(v001, v101, tx);
    let x11 = lerp(v011, v111, tx);
    let y0v = lerp(x00, x10, ty);
    let y1v = lerp(x01, x11, ty);

    // Map [0,1] -> [-1,1]
    (lerp(y0v, y1v, tz) * 2.0) - 1.0
}

#[inline]
fn u32_to_unit(v: u32) -> f32 {
    // 24-bit precision in f32 mantissa is enough here.
    let m = v >> 8;
    (m as f32) * (1.0 / ((1u32 << 24) as f32))
}

