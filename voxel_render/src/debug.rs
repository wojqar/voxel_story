use bevy::prelude::*;
use std::time::Duration;
use world_api::{DebugEntry, SampleStats};

use crate::components::NeedsRemesh;
use crate::meshing::MeshData;
use crate::region::RegionCoord;
use crate::resources::{InflightTasks, MeshingQueue, RegionMap, VoxelRenderConfig};

#[derive(Debug, Default, Resource)]
pub struct MeshingDebugStats {
    spawn_batch: SampleStats,
    snapshot_ms: SampleStats,
    snapshot_solid_voxels: SampleStats,
    build_ms: SampleStats,
    ready_latency_ms: SampleStats,
    apply_ms: SampleStats,
    apply_batch_size: SampleStats,
    mesh_quads: SampleStats,
    mesh_triangles: SampleStats,
    mesh_vertices: SampleStats,
    mesh_indices: SampleStats,
    spawned_tasks_total: u64,
    completed_tasks_total: u64,
    empty_meshes_total: u64,
    last_completed_region: Option<RegionCoord>,
    slowest_build_region: Option<RegionCoord>,
    slowest_build_ms: f64,
    peak_queue_depth: usize,
    peak_inflight_tasks: usize,
    peak_needs_remesh: usize,
}

impl MeshingDebugStats {
    pub fn record_spawn_batch(&mut self, spawned: usize) {
        if spawned == 0 {
            return;
        }

        self.spawn_batch.record(spawned as f64);
        self.spawned_tasks_total += spawned as u64;
    }

    pub fn record_snapshot(&mut self, duration: Duration, solid_voxels: usize) {
        self.snapshot_ms.record_duration(duration);
        self.snapshot_solid_voxels.record(solid_voxels as f64);
    }

    pub fn record_task_finished(
        &mut self,
        region: RegionCoord,
        build_duration: Duration,
        ready_latency: Duration,
    ) {
        let build_ms = build_duration.as_secs_f64() * 1_000.0;
        self.build_ms.record(build_ms);
        self.ready_latency_ms.record_duration(ready_latency);
        self.completed_tasks_total += 1;
        self.last_completed_region = Some(region);

        if build_ms >= self.slowest_build_ms {
            self.slowest_build_ms = build_ms;
            self.slowest_build_region = Some(region);
        }
    }

    pub fn record_mesh_output(&mut self, mesh: &MeshData) {
        let quads = mesh.positions.len() / 4;
        let triangles = mesh.indices.len() / 3;
        self.mesh_quads.record(quads as f64);
        self.mesh_triangles.record(triangles as f64);
        self.mesh_vertices.record(mesh.positions.len() as f64);
        self.mesh_indices.record(mesh.indices.len() as f64);
    }

    pub fn record_empty_mesh(&mut self) {
        self.empty_meshes_total += 1;
    }

    pub fn record_apply(&mut self, duration: Duration, completed: usize) {
        if completed == 0 {
            return;
        }

        self.apply_ms.record_duration(duration);
        self.apply_batch_size.record(completed as f64);
    }

    pub fn observe_queue_pressure(
        &mut self,
        queue_depth: usize,
        inflight_tasks: usize,
        needs_remesh: usize,
    ) {
        self.peak_queue_depth = self.peak_queue_depth.max(queue_depth);
        self.peak_inflight_tasks = self.peak_inflight_tasks.max(inflight_tasks);
        self.peak_needs_remesh = self.peak_needs_remesh.max(needs_remesh);
    }
}

#[derive(Default)]
pub struct MeshingDebugReportState {
    elapsed: f32,
    previous_spawned_total: u64,
    previous_completed_total: u64,
}

pub fn emit_meshing_debug_entries(
    time: Res<Time>,
    config: Res<VoxelRenderConfig>,
    queue: Res<MeshingQueue>,
    inflight: Res<InflightTasks>,
    region_map: Res<RegionMap>,
    mut stats: ResMut<MeshingDebugStats>,
    needs_remesh_q: Query<(), With<NeedsRemesh>>,
    mut entries: MessageWriter<DebugEntry>,
    mut state: Local<MeshingDebugReportState>,
) {
    let needs_remesh = needs_remesh_q.iter().len();
    stats.observe_queue_pressure(queue.pending.len(), inflight.tasks.len(), needs_remesh);

    state.elapsed += time.delta_secs();
    if state.elapsed < 0.5 {
        return;
    }

    let interval_secs = state.elapsed.max(f32::EPSILON) as f64;
    let spawn_rate =
        (stats.spawned_tasks_total.saturating_sub(state.previous_spawned_total)) as f64 / interval_secs;
    let complete_rate = (stats
        .completed_tasks_total
        .saturating_sub(state.previous_completed_total)) as f64
        / interval_secs;
    state.previous_spawned_total = stats.spawned_tasks_total;
    state.previous_completed_total = stats.completed_tasks_total;
    state.elapsed = 0.0;

    entries.write(DebugEntry::new(
        "Meshing",
        "Config",
        format!(
            "{} spawns/frame | {} inflight max",
            config.max_spawns_per_frame, config.max_inflight_tasks
        ),
    ));
    entries.write(DebugEntry::new("Meshing", "Live regions", region_map.0.len()));
    entries.write(DebugEntry::new(
        "Meshing",
        "Queue depth",
        format!("{} | peak {}", queue.pending.len(), stats.peak_queue_depth),
    ));
    entries.write(DebugEntry::new(
        "Meshing",
        "Inflight",
        format!(
            "{}/{} | peak {}",
            inflight.tasks.len(),
            config.max_inflight_tasks,
            stats.peak_inflight_tasks
        ),
    ));
    entries.write(DebugEntry::new(
        "Meshing",
        "NeedsRemesh",
        format!("{} | peak {}", needs_remesh, stats.peak_needs_remesh),
    ));
    entries.write(DebugEntry::new(
        "Meshing",
        "Spawn rate",
        format!("{spawn_rate:.1}/s | total {}", stats.spawned_tasks_total),
    ));
    entries.write(DebugEntry::new(
        "Meshing",
        "Complete rate",
        format!("{complete_rate:.1}/s | total {}", stats.completed_tasks_total),
    ));
    entries.write(DebugEntry::new(
        "Meshing",
        "Spawn batch",
        stats.spawn_batch.format_summary(1, ""),
    ));
    entries.write(DebugEntry::new(
        "Meshing",
        "Snapshot",
        stats.snapshot_ms.format_summary(3, " ms"),
    ));
    entries.write(DebugEntry::new(
        "Meshing",
        "Snapshot solids",
        stats.snapshot_solid_voxels.format_summary(0, ""),
    ));
    entries.write(DebugEntry::new(
        "Meshing",
        "Build time",
        stats.build_ms.format_summary(3, " ms"),
    ));
    entries.write(DebugEntry::new(
        "Meshing",
        "Ready latency",
        stats.ready_latency_ms.format_summary(3, " ms"),
    ));
    entries.write(DebugEntry::new(
        "Meshing",
        "Apply time",
        stats.apply_ms.format_summary(3, " ms"),
    ));
    entries.write(DebugEntry::new(
        "Meshing",
        "Apply batch",
        stats.apply_batch_size.format_summary(1, ""),
    ));
    entries.write(DebugEntry::new(
        "Meshing",
        "Empty meshes",
        stats.empty_meshes_total,
    ));

    if let Some(region) = stats.last_completed_region {
        entries.write(DebugEntry::new(
            "Meshing",
            "Last region",
            format_region(region),
        ));
    }

    if let Some(region) = stats.slowest_build_region {
        entries.write(DebugEntry::new(
            "Meshing",
            "Slowest build",
            format!("{} ({:.3} ms)", format_region(region), stats.slowest_build_ms),
        ));
    }

    entries.write(DebugEntry::new(
        "Mesh Output",
        "Quads",
        stats.mesh_quads.format_summary(0, ""),
    ));
    entries.write(DebugEntry::new(
        "Mesh Output",
        "Triangles",
        stats.mesh_triangles.format_summary(0, ""),
    ));
    entries.write(DebugEntry::new(
        "Mesh Output",
        "Vertices",
        stats.mesh_vertices.format_summary(0, ""),
    ));
    entries.write(DebugEntry::new(
        "Mesh Output",
        "Indices",
        stats.mesh_indices.format_summary(0, ""),
    ));
}

fn format_region(region: RegionCoord) -> String {
    format!("({}, {}, {})", region.x, region.y, region.z)
}
