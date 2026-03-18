use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_mesh::Indices;
use bevy_mesh::VertexAttributeValues;
use futures_lite::future;

use crate::components::{NeedsRemesh, RegionMesh};
use crate::meshing::{MeshData, build_region_mesh};
use crate::region::{REGION_SIZE_VOXELS, RegionCoord, region_origin_world_voxel};
use crate::resources::{
    InflightTasks, MeshingQueue, RegionMap, RegionMaterial, VoxelPalette, VoxelRenderConfig,
};
use voxel_engine::VoxelWorldResource;

fn bevy_to_core(v: IVec3) -> voxel_core::IVec3 {
    voxel_core::IVec3::new(v.x, v.y, v.z)
}

pub fn spawn_meshing_tasks(
    world: Res<VoxelWorldResource>,
    palette: Res<VoxelPalette>,
    config: Res<VoxelRenderConfig>,
    mut queue: ResMut<MeshingQueue>,
    mut inflight: ResMut<InflightTasks>,
    mut query: Query<(Entity, &RegionMesh), With<NeedsRemesh>>,
) {
    if inflight.tasks.len() >= config.max_inflight_tasks {
        return;
    }

    // Ensure any "NeedsRemesh" regions end up queued (coalesced via pending_set).
    for (_, rm) in query.iter_mut() {
        queue.enqueue(rm.coord);
    }

    let pool = AsyncComputeTaskPool::get();

    let mut spawned = 0usize;
    while spawned < config.max_spawns_per_frame && inflight.tasks.len() < config.max_inflight_tasks
    {
        let Some(region) = queue.pending.pop_front() else {
            break;
        };
        queue.pending_set.remove(&region);
        if inflight.tasks.contains_key(&region) {
            continue;
        }

        // Snapshot region voxel ids on main thread (bounded copy).
        let origin = region_origin_world_voxel(region);
        let n = REGION_SIZE_VOXELS;
        let mut voxels = vec![0u16; (n * n * n) as usize];
        for z in 0..n {
            for y in 0..n {
                for x in 0..n {
                    let p = IVec3::new(origin.x + x, origin.y + y, origin.z + z);
                    let v = world.0.get_voxel(bevy_to_core(p)).0;
                    voxels[(x + y * n + z * n * n) as usize] = v;
                }
            }
        }

        let colors = palette.colors.clone();
        let fallback = palette.fallback;
        let task: Task<MeshData> = pool.spawn(async move {
            let palette_fn =
                |id: u16| -> [f32; 4] { colors.get(id as usize).copied().unwrap_or(fallback) };
            build_region_mesh(region, &voxels, &palette_fn)
        });

        inflight.tasks.insert(region, task);
        spawned += 1;
    }
}

pub fn apply_completed_meshes(
    mut commands: Commands,
    material: Res<RegionMaterial>,
    mut region_map: ResMut<RegionMap>,
    mut inflight: ResMut<InflightTasks>,
    mut meshes: ResMut<Assets<Mesh>>,
    needs_remesh_q: Query<Entity, With<NeedsRemesh>>,
    transform_q: Query<&Transform>,
) {
    let mut finished: Vec<(RegionCoord, MeshData)> = Vec::new();

    inflight.tasks.retain(|region, task| {
        if let Some(data) = future::block_on(future::poll_once(task)) {
            finished.push((*region, data));
            false
        } else {
            true
        }
    });

    for (region, data) in finished {
        let Some(&entity) = region_map.0.get(&region) else {
            continue;
        };

        if data.is_empty() {
            commands.entity(entity).despawn();
            region_map.0.remove(&region);
            continue;
        }

        let mesh = mesh_from_data(&data);
        let handle = meshes.add(mesh);

        let transform = transform_q.get(entity).copied().unwrap_or_default();
        // Bevy 0.18: render via Mesh3d + MeshMaterial3d.
        commands.entity(entity).insert((
            Mesh3d(handle),
            MeshMaterial3d(material.0.clone()),
            transform,
        ));

        if needs_remesh_q.get(entity).is_ok() {
            commands.entity(entity).remove::<NeedsRemesh>();
        }
    }
}

fn mesh_from_data(data: &MeshData) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals.clone());
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::Float32x4(data.colors.clone()),
    );
    mesh.insert_indices(Indices::U32(data.indices.clone()));
    mesh
}
