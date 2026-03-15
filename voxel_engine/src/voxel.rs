// voxel.rs
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct VoxelId(pub u16);

impl VoxelId {
    pub const AIR: Self = Self(0);
    pub fn is_air(self) -> bool {
        self.0 == 0
    }
}
