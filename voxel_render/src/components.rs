use bevy::prelude::*;

use crate::region::RegionCoord;

#[derive(Component, Debug, Clone, Copy)]
pub struct RegionMesh {
    pub coord: RegionCoord,
}

#[derive(Component, Debug, Default)]
pub struct NeedsRemesh;

