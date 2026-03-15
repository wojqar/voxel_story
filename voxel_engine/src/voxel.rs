#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct VoxelId(pub u16);

impl VoxelId {
    pub const AIR: Self = Self(0);
    pub const STONE: Self = Self(1);
    pub const DIRT: Self = Self(2);
    pub const GRASS: Self = Self(3);

    pub fn is_air(self) -> bool {
        self.0 == 0
    }

    pub fn color(self) -> [f32; 4] {
        match self {
            Self::STONE => [0.40, 0.40, 0.40, 1.0],
            Self::DIRT => [0.35, 0.20, 0.08, 1.0],
            Self::GRASS => [0.15, 0.50, 0.08, 1.0],
            _ => [1.0, 0.0, 1.0, 1.0],
        }
    }
}
