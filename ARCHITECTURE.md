# Architecture - voxel_story

## Invariants

1. Dependencies only point downward through the workspace graph.
2. Cross-crate communication goes through `world_api` messages and shared marker types.
3. Expensive render work stays off the main thread when possible.
4. The world is fixed-size at runtime. There is no chunk load/unload system.

---

## Workspace

```text
voxel_story/
|- world_api/        # Shared messages, debug contracts, marker components
|- voxel_core/       # Pure Rust voxel data structures and generation
|- voxel_engine/     # Bevy ECS world bootstrap and gameplay-facing queries
|- voxel_render/     # Region meshing, async mesh tasks, GPU upload
|- camera/           # Spectator + RTS camera logic
|- ui/               # Debug overlay and diagnostics
\- voxel_story/      # Binary entry point
```

Bevy: `0.18.1`
Rust edition: `2024`
Chunk size: `16^3` voxels
Region size: `4 x 4 x 4` chunks = `64^3` voxels

---

## Dependency Graph

```text
world_api
  |- voxel_core
  |   \- voxel_engine
  |       \- voxel_render
  |- camera
  |- ui
  \- voxel_story -> all
```

`camera` and `ui` do not depend on engine or render internals.

---

## world_api

Contract crate only. No systems, no engine logic.

### Messages

| Message | Direction | Purpose |
|---|---|---|
| `CursorRay { origin, direction }` | camera -> * | Cursor ray in world space |
| `TerrainHeightRequest { pos }` | camera -> voxel_engine | Query terrain height for RTS camera |
| `TerrainHeightResponse { height }` | voxel_engine -> camera | Height response |
| `BlockTargeted { pos, normal }` | gameplay -> * | Targeted voxel feedback |
| `BlockTargetCleared` | gameplay -> * | Clears target feedback |
| `BlockInteract { pos, action }` | gameplay -> voxel_engine | Terrain edit request |
| `ChunkModified(IVec3)` | voxel_engine -> voxel_render | Marks a chunk's region dirty |
| `DebugEntry { section, key, value }` | * -> ui | Debug panel entry |

### Components

```rust
ActiveCamera // marker for the camera used by cursor-ray systems
```

There is no observer component for chunk streaming because the world stays fully resident.

---

## voxel_core

Pure Rust world representation. No Bevy, no rendering.

### Core Types

- `VoxelId(u16)`: voxel identifier, `0` is air.
- `Chunk<const SIZE: usize = 16>`: dense voxel storage plus cached `solid_count` and per-column top voxel.
- `WorldDimensions { x, y, z }`: world size in chunks.
- `VoxelWorld<const SIZE: usize = 16>`: flat chunk array with O(1) lookup.
- `WorldGenerator`: trait for chunk generation.

### Coordinate Utilities

`coords.rs` provides:

- `world_to_chunk::<SIZE>(IVec3) -> (chunk_coord, local_coord)`
- `chunk_to_world::<SIZE>(chunk, local) -> IVec3`
- `local_to_index::<SIZE>(IVec3) -> usize`

All conversions use Euclidean division so negative coordinates behave correctly even though the current world bounds are non-negative.

### World Data Model

`VoxelWorld` stores:

- `chunks: Vec<Chunk<SIZE>>`
- `column_tops: Vec<Option<i32>>`
- `solid_count: usize`
- `dimensions: WorldDimensions`

Important operations:

- `get_voxel` / `set_voxel`
- `get_chunk` / `get_chunk_mut`
- `replace_chunk`
- `column_height`
- `snapshot_chunk_aligned_region_u16`
- `chunk_aligned_region_solid_count`

`column_height` is O(1) after updates because both `Chunk` and `VoxelWorld` maintain cached top-of-column data.

---

## voxel_engine

Bevy ECS layer over `voxel_core`. Responsible for building the fixed-size world and answering engine-side queries.

### Resources

```rust
WorldConfig {
    dimensions: WorldDimensions, // default: 20 x 8 x 20 chunks
    seed: u64,
}

VoxelWorldResource(DefaultWorld)
```

### Startup Flow

`init_voxel_world`:

1. Creates `CastleStoryGenerator` from `WorldConfig`.
2. Allocates an empty `DefaultWorld`.
3. Iterates every chunk coordinate in bounds.
4. Generates each chunk synchronously.
5. Stores it with `replace_chunk`.
6. Inserts `VoxelWorldResource`.

The whole world is generated up front. Nothing streams in later.

### Runtime Queries

`respond_terrain_height_requests` answers RTS camera height probes using cached column heights, with a fallback check at `y = 0` when the column is empty.

---

## voxel_render

Rendering layer for the fixed-size voxel world.

### Region Model

A region is `4 x 4 x 4` chunks. Regions are the meshing and draw-call unit.

For the default world size `20 x 8 x 20`, the maximum region grid is `5 x 2 x 5`, so at most 50 region entities ever need render payloads.

### Resources

- `RegionMap`: maps `RegionCoord` to a stable entity
- `MeshingQueue`: deduplicated FIFO queue of dirty regions
- `InflightTasks`: async mesh tasks keyed by region
- `VoxelPalette`: voxel id -> RGBA lookup table
- `VoxelRenderConfig`: meshing throttles
- `RegionMaterial`: shared material handle

### Pipeline

```text
PostStartup:
    seed_initial_regions
        -> scan all region bounds
        -> spawn entities only for regions with solid voxels
        -> mark them NeedsRemesh

Update:
    handle_chunk_modifications
        -> ChunkModified
        -> map chunk to region
        -> enqueue region and mark NeedsRemesh

    spawn_meshing_tasks
        -> drain queue within max_spawns_per_frame
        -> skip regions that are now fully empty
        -> snapshot non-empty 64^3 voxel regions on main thread
        -> spawn async greedy meshing tasks

    apply_completed_meshes
        -> poll finished tasks
        -> empty mesh: remove Mesh3d + material components
        -> non-empty mesh: upload mesh and attach render components
        -> clear NeedsRemesh when no requeue is pending
```

### Meshing

`build_region_mesh` performs 3-axis greedy meshing over a dense `u16` voxel snapshot.

Implementation details:

- Builds a 2D face mask for each slice.
- Greedily merges rectangles with matching voxel id and face direction.
- Emits positions, normals, colors, and indices.
- Uses a direct winding rule per axis instead of a geometric normal test per quad.

### Optimization Notes

The current render path avoids a few obvious fixed-world costs:

- No chunk load/unload messages.
- No startup meshing for fully empty regions.
- No async task spawn for empty dirty regions.
- Mesh upload moves vertex/index buffers into Bevy instead of cloning them.

---

## camera

Depends only on `world_api` and Bevy.

The camera entity contains:

```rust
Camera3d
Transform
SpectatorCamera
SpectatorActive
RtsCamera
ActiveCamera
```

### Modes

- `SpectatorCamera`: free-fly movement.
- `RtsCamera`: pivot-based RTS camera with terrain height sampling.
- `SwitchingPlugin`: toggles between the two modes.
- `CursorRayPlugin`: emits `CursorRay` each frame while the cursor is inside the viewport.

There is no render-distance or chunk-observer state on the camera.

---

## ui

Depends on `world_api`, Bevy, and `bevy_egui`.

- `DebugUiPlugin`: displays grouped `DebugEntry` values.
- `DiagnosticsPlugin`: publishes FPS and frame-time metrics.

---

## voxel_story

Entry-point crate that wires everything together.

### Full App

```rust
App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        present_mode: PresentMode::Immediate,
        ..default()
    }))
    .add_plugins(VoxelRenderPlugin::default())
    .add_plugins(CameraPlugin)
    .add_plugins(UiPlugin)
    .run();
```

### Headless Engine

```rust
App::new()
    .add_plugins(MinimalPlugins)
    .add_plugins(VoxelEnginePlugin::default())
    .run();
```

---

## World Initialization Flow

```text
Startup:
    init_voxel_world
        -> generate every chunk in bounds
        -> build VoxelWorldResource

PostStartup:
    seed_initial_regions
        -> create render entities only for non-empty regions
        -> queue initial remesh work

Update:
    spawn_meshing_tasks
    apply_completed_meshes
    emit debug metrics
```

This architecture is optimized for a small, always-loaded world rather than an infinite streamed one.
