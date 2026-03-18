use bevy::app::PostStartup;
use bevy::prelude::*;

use crate::components::{NeedsRemesh, RegionMesh};
use crate::debug::{MeshingDebugStats, emit_meshing_debug_entries};
use crate::region::{REGION_SIZE_CHUNKS, RegionCoord};
use crate::region::{chunk_to_region, region_origin_world_voxel};
use crate::resources::{
    InflightTasks, MeshingQueue, RegionMap, RegionMaterial, VoxelPalette, VoxelRenderConfig,
};
use crate::tasks::{apply_completed_meshes, spawn_meshing_tasks};
use voxel_engine::VoxelEnginePlugin;
use voxel_engine::VoxelWorldResource;
use world_api::{ChunkLoaded, ChunkModified, ChunkUnloaded};

pub struct VoxelRenderPlugin;

impl Default for VoxelRenderPlugin {
    fn default() -> Self {
        Self
    }
}

impl Plugin for VoxelRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VoxelEnginePlugin::default())
            .add_message::<ChunkLoaded>()
            .add_message::<ChunkUnloaded>()
            .add_message::<ChunkModified>()
            .init_resource::<RegionMap>()
            .init_resource::<MeshingQueue>()
            .init_resource::<InflightTasks>()
            .init_resource::<MeshingDebugStats>()
            .init_resource::<VoxelPalette>()
            .init_resource::<VoxelRenderConfig>()
            .add_systems(Startup, init_material)
            .add_systems(PostStartup, seed_initial_regions)
            .add_systems(Update, handle_chunk_events)
            .add_systems(Update, spawn_meshing_tasks.after(handle_chunk_events))
            .add_systems(Update, apply_completed_meshes.after(spawn_meshing_tasks))
            .add_systems(Update, emit_meshing_debug_entries.after(apply_completed_meshes));
    }
}

fn init_material(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let material = materials.add(StandardMaterial {
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    });
    commands.insert_resource(RegionMaterial(material));
}

fn handle_chunk_events(
    mut commands: Commands,
    mut region_map: ResMut<RegionMap>,
    mut loaded: MessageReader<ChunkLoaded>,
    mut unloaded: MessageReader<ChunkUnloaded>,
    mut modified: MessageReader<ChunkModified>,
) {
    for ChunkLoaded(chunk) in loaded.read() {
        ensure_region_entity(&mut commands, &mut region_map, chunk_to_region(*chunk));
    }
    for ChunkUnloaded(chunk) in unloaded.read() {
        ensure_region_entity(&mut commands, &mut region_map, chunk_to_region(*chunk));
    }
    for ChunkModified(chunk) in modified.read() {
        ensure_region_entity(&mut commands, &mut region_map, chunk_to_region(*chunk));
    }
}

fn seed_initial_regions(
    mut commands: Commands,
    mut region_map: ResMut<RegionMap>,
    world: Res<VoxelWorldResource>,
) {
    let dims = world.0.dimensions;

    let rx = ((dims.x as i32) + REGION_SIZE_CHUNKS - 1) / REGION_SIZE_CHUNKS;
    let ry = ((dims.y as i32) + REGION_SIZE_CHUNKS - 1) / REGION_SIZE_CHUNKS;
    let rz = ((dims.z as i32) + REGION_SIZE_CHUNKS - 1) / REGION_SIZE_CHUNKS;

    for z in 0..rz {
        for y in 0..ry {
            for x in 0..rx {
                ensure_region_entity(&mut commands, &mut region_map, RegionCoord::new(x, y, z));
            }
        }
    }
}

fn ensure_region_entity(
    commands: &mut Commands,
    region_map: &mut RegionMap,
    region: crate::region::RegionCoord,
) {
    if let Some(&entity) = region_map.0.get(&region) {
        commands.entity(entity).insert(NeedsRemesh);
        return;
    }

    let origin = region_origin_world_voxel(region).as_vec3();
    let entity = commands
        .spawn((
            RegionMesh { coord: region },
            NeedsRemesh,
            Transform::from_translation(origin),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id();
    region_map.0.insert(region, entity);
}
