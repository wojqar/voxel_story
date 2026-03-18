use std::collections::{HashMap, HashSet, VecDeque};

use bevy::prelude::*;
use bevy::tasks::Task;

use crate::meshing::MeshData;
use crate::region::RegionCoord;

#[derive(Debug, Default, Resource)]
pub struct RegionMap(pub HashMap<RegionCoord, Entity>);

#[derive(Debug, Resource)]
pub struct VoxelPalette {
    // indexed by voxel id (u16); fallback is used for out-of-range ids.
    pub colors: Vec<[f32; 4]>,
    pub fallback: [f32; 4],
}

impl Default for VoxelPalette {
    fn default() -> Self {
        let mut colors = vec![[1.0, 0.0, 1.0, 1.0]; 256];
        colors[0] = [0.0, 0.0, 0.0, 0.0]; // air
        colors[1] = [0.55, 0.35, 0.20, 1.0]; // dirt
        colors[2] = [0.20, 0.65, 0.25, 1.0]; // grass
        colors[3] = [0.5, 0.5, 0.55, 1.0]; // stone
        Self {
            colors,
            fallback: [1.0, 0.0, 1.0, 1.0],
        }
    }
}

impl VoxelPalette {
    #[inline]
    pub fn color_rgba(&self, voxel: u16) -> [f32; 4] {
        self.colors
            .get(voxel as usize)
            .copied()
            .unwrap_or(self.fallback)
    }
}

#[derive(Debug, Clone, Copy, Resource)]
pub struct VoxelRenderConfig {
    pub max_inflight_tasks: usize,
    pub max_spawns_per_frame: usize,
}

impl Default for VoxelRenderConfig {
    fn default() -> Self {
        Self {
            max_inflight_tasks: 8,
            max_spawns_per_frame: 2,
        }
    }
}

#[derive(Default, Resource)]
pub struct MeshingQueue {
    pub pending: VecDeque<RegionCoord>,
    pub pending_set: HashSet<RegionCoord>,
}

impl MeshingQueue {
    pub fn enqueue(&mut self, region: RegionCoord) {
        if self.pending_set.insert(region) {
            self.pending.push_back(region);
        }
    }
}

#[derive(Resource, Default)]
pub struct InflightTasks {
    pub tasks: HashMap<RegionCoord, Task<MeshData>>,
}

#[derive(Resource)]
pub struct RegionMaterial(pub Handle<StandardMaterial>);
