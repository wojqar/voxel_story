use bevy::math::IVec3;

pub struct WorldConfig {
    pub chunks: IVec3,
    pub island_radius: f32,
    pub island_thickness: f32,
    pub noise_scale: f64,
    pub noise_strength: f64,
    pub seed: u32,
}

impl WorldConfig {
    pub fn new() -> Self {
        Self {
            chunks: IVec3::new(50, 8, 50),
            island_radius: 88.0,
            island_thickness: 28.0,
            noise_scale: 40.0,
            noise_strength: 0.3,
            seed: 42,
        }
    }

    pub fn island_center(&self) -> IVec3 {
        IVec3::new(self.chunks.x * 8, self.chunks.y * 8, self.chunks.z * 8)
    }

    pub fn camera_pos(&self) -> bevy::math::Vec3 {
        let c = self.island_center();
        bevy::math::Vec3::new(
            c.x as f32 + self.island_radius * 2.5,
            c.y as f32 + self.island_radius * 0.8,
            c.z as f32 + self.island_radius * 2.5,
        )
    }

    pub fn camera_target(&self) -> bevy::math::Vec3 {
        let c = self.island_center();
        bevy::math::Vec3::new(c.x as f32, c.y as f32, c.z as f32)
    }
}
impl Default for WorldConfig {
    fn default() -> Self {
        Self::new()
    }
}
