use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Debug)]
pub struct MainTerrainAnchor {
    pub focus: Vec3,
}
