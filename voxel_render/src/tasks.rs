use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_mesh::Indices;
use bevy_mesh::VertexAttributeValues;
use futures_lite::future;
use std::time::Instant;

use crate::components::NeedsRemesh;
use crate::debug::MeshingDebugStats;
use crate::meshing::{MeshData, build_region_mesh};
use crate::region::{REGION_SIZE_CHUNKS, RegionCoord};
use crate::resources::{
    InflightMeshTask, InflightTasks, MeshTaskOutput, MeshingQueue, RegionMap, RegionMaterial,
    VoxelPalette, VoxelRenderConfig,
};
use voxel_engine::VoxelWorldResource;

pub fn spawn_meshing_tasks(
    mut commands: Commands,
    world: Res<VoxelWorldResource>,
    palette: Res<VoxelPalette>,
    config: Res<VoxelRenderConfig>,
    region_map: Res<RegionMap>,
    mut queue: ResMut<MeshingQueue>,
    mut inflight: ResMut<InflightTasks>,
    mut debug_stats: ResMut<MeshingDebugStats>,
) {
    if inflight.tasks.len() >= config.max_inflight_tasks {
        return;
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

        let snapshot_started = Instant::now();
        let origin_chunk = voxel_core::IVec3::new(
            region.x * REGION_SIZE_CHUNKS,
            region.y * REGION_SIZE_CHUNKS,
            region.z * REGION_SIZE_CHUNKS,
        );
        let chunk_dims =
            voxel_core::IVec3::new(REGION_SIZE_CHUNKS, REGION_SIZE_CHUNKS, REGION_SIZE_CHUNKS);
        if world
            .0
            .chunk_aligned_region_solid_count(origin_chunk, chunk_dims)
            == 0
        {
            if let Some(&entity) = region_map.0.get(&region) {
                commands
                    .entity(entity)
                    .remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>, NeedsRemesh)>();
            }
            continue;
        }

        let (voxels, solid_voxels) = world
            .0
            .snapshot_chunk_aligned_region_u16(origin_chunk, chunk_dims);
        debug_stats.record_snapshot(snapshot_started.elapsed(), solid_voxels);

        let colors = palette.colors.clone();
        let fallback = palette.fallback;
        let task_started = snapshot_started;
        let task: Task<MeshTaskOutput> = pool.spawn(async move {
            let build_started = Instant::now();
            let palette_fn =
                |id: u16| -> [f32; 4] { colors.get(id as usize).copied().unwrap_or(fallback) };
            let mesh = build_region_mesh(region, &voxels, &palette_fn);
            MeshTaskOutput {
                mesh,
                build_duration: build_started.elapsed(),
            }
        });

        inflight.tasks.insert(
            region,
            InflightMeshTask {
                task,
                started_at: task_started,
            },
        );
        spawned += 1;
    }

    debug_stats.record_spawn_batch(spawned);
}

pub fn apply_completed_meshes(
    mut commands: Commands,
    material: Res<RegionMaterial>,
    region_map: Res<RegionMap>,
    queue: Res<MeshingQueue>,
    mut inflight: ResMut<InflightTasks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut debug_stats: ResMut<MeshingDebugStats>,
    needs_remesh_q: Query<Entity, With<NeedsRemesh>>,
) {
    let apply_started = Instant::now();
    let mut finished: Vec<(RegionCoord, MeshTaskOutput, std::time::Duration)> = Vec::new();

    inflight.tasks.retain(|region, task| {
        if let Some(data) = future::block_on(future::poll_once(&mut task.task)) {
            finished.push((*region, data, task.started_at.elapsed()));
            false
        } else {
            true
        }
    });

    let completed_count = finished.len();

    for (region, output, ready_latency) in finished {
        debug_stats.record_task_finished(region, output.build_duration, ready_latency);

        let data = output.mesh;
        let Some(&entity) = region_map.0.get(&region) else {
            continue;
        };

        if data.is_empty() {
            debug_stats.record_empty_mesh();
            commands
                .entity(entity)
                .remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>)>();
        } else {
            debug_stats.record_mesh_output(&data);
            let mesh = mesh_from_data(data);
            let handle = meshes.add(mesh);

            // Keep region entities stable; only swap render payloads.
            commands
                .entity(entity)
                .insert((Mesh3d(handle), MeshMaterial3d(material.0.clone())));
        }

        if needs_remesh_q.get(entity).is_ok() && !queue.pending_set.contains(&region) {
            commands.entity(entity).remove::<NeedsRemesh>();
        }
    }

    debug_stats.record_apply(apply_started.elapsed(), completed_count);
}

fn mesh_from_data(data: MeshData) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::Float32x4(data.colors),
    );
    mesh.insert_indices(Indices::U32(data.indices));
    mesh
}
