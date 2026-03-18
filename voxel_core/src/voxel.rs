#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VoxelId(pub u16);

impl VoxelId {
    pub const AIR: Self = Self(0);
    pub const DIRT: Self = Self(1);
    pub const GRASS: Self = Self(2);
    pub const STONE: Self = Self(3);

    #[inline]
    pub fn is_air(self) -> bool {
        self.0 == Self::AIR.0
    }
}

impl Default for VoxelId {
    fn default() -> Self {
        Self::AIR
    }
}
