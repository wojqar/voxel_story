use bevy::prelude::*;

#[derive(Message, Clone)]
pub struct CursorRay {
    pub origin: Vec3,
    pub direction: Vec3,
}

#[derive(Message, Clone)]
pub struct TerrainHeightRequest {
    pub pos: Vec2,
}

#[derive(Message, Clone)]
pub struct TerrainHeightResponse {
    pub height: f32,
}

#[derive(Message, Clone)]
pub struct BlockTargeted {
    pub pos: IVec3,
    pub normal: IVec3,
}

#[derive(Message, Clone)]
pub struct BlockTargetCleared;

#[derive(Message, Clone)]
pub struct BlockInteract {
    pub pos: IVec3,
    pub action: InteractAction,
}

#[derive(Clone, Copy, Debug)]
pub enum InteractAction {
    Remove,
    Place(u16),
}

#[derive(Message, Clone)]
pub struct ChunkLoaded(pub IVec3);

#[derive(Message, Clone)]
pub struct ChunkUnloaded(pub IVec3);

#[derive(Message, Clone)]
pub struct ChunkModified(pub IVec3);
