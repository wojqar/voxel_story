use crate::chunk::Chunk;
use crate::coords::world_to_chunk;
use crate::voxel::VoxelId;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct VoxelWorld {
    chunks: HashMap<IVec3, Chunk>,
    solid_count: usize,
}

impl VoxelWorld {
    pub fn get_chunk(&self, coord: IVec3) -> Option<&Chunk> {
        self.chunks.get(&coord)
    }

    pub fn get_chunk_mut(&mut self, coord: IVec3) -> Option<&mut Chunk> {
        self.chunks.get_mut(&coord)
    }

    pub fn insert_chunk(&mut self, coord: IVec3, chunk: Chunk) {
        let incoming = chunk.count_solid();
        if let Some(old) = self.chunks.insert(coord, chunk) {
            self.solid_count -= old.count_solid();
        }
        self.solid_count += incoming;
    }

    pub fn set_voxel(&mut self, world_pos: IVec3, id: VoxelId) -> bool {
        let (chunk_coord, lx, ly, lz) = world_to_chunk(world_pos);
        let Some(chunk) = self.chunks.get_mut(&chunk_coord) else { return false };

        let old = chunk.get(lx, ly, lz);
        if old == id { return false; }

        if  old.is_air() && !id.is_air() { self.solid_count += 1; }
        if !old.is_air() &&  id.is_air() { self.solid_count -= 1; }

        chunk.set(lx, ly, lz, id);
        true
    }

    pub fn chunk_count(&self) -> usize { self.chunks.len() }
    pub fn solid_voxel_count(&self) -> usize { self.solid_count }
}