# Architecture — voxel_story

## Zasady niezmienne

1. Zależności tylko w dół hierarchii — nigdy wstecz ani między równorzędnymi crate'ami
2. Warstwy komunikują się wyłącznie przez eventy z `world_api`
3. Kosztowne operacje zawsze async — główny wątek nie blokuje
4. Logika nie zależy od prezentacji

---

## Workspace

```
voxel_story/
├── world_api/        # Wspólne typy eventów i komponentów, zero logiki
├── voxel_core/       # Dane i logika świata, czysty Rust
├── voxel_engine/     # Bevy ECS — zarządzanie światem, generacja
├── voxel_render/     # Meshing, GPU, frustum culling
├── camera/           # RtsCamera, SpectatorCamera, cursor ray
├── ui/               # Debug overlay, diagnostyki
└── voxel_story/      # Binarny entry point, łączy wszystko
```

Bevy: `0.18.1` | Rust edition: `2024` | Chunk size: `16³` voxeli | Region size: `4×4×4` chunki (`64³` voxeli)

---

## Hierarchia zależności

```
world_api
  ├── voxel_core
  │     └── voxel_engine
  │           └── voxel_render
  ├── camera
  ├── ui
  └── voxel_story → all
```

`camera` i `ui` zależą wyłącznie od `world_api` i Bevy — zero cross-coupling z logiką silnika.

---

## world_api

Warstwa kontraktów. Zero logiki, zero systemów Bevy. Kompiluje się jako pierwsza.

**Events (`events.rs`):**

| Event | Kierunek | Opis |
|---|---|---|
| `CursorRay { origin, direction }` | camera → * | Ray w world-space z pozycji kursora |
| `ChunkModified(IVec3)` | voxel_engine → * | Zmiana danych chunka |
| `DebugEntry { section, key, value }` | * → ui | Wpis do debug panelu |

**Components (`components.rs`):**

```rust
ActiveCamera     // Marker — kamera aktywna; używana do viewport_to_world
ChunkObserver {
    load_distance:   u32,
    unload_distance: u32,
}
```

---

## voxel_core

Czysty Rust. Bez Bevy, GPU, async. Zero zewnętrznych zależności.

**`VoxelId(u16)`** — ID voxela. `0` = powietrze. `is_air()` → `id == 0`.

**`Chunk<const SIZE: usize = 16>`** — gęsty array voxeli `SIZE³`. Storage: `Vec<VoxelId>` z indeksowaniem `x + y*SIZE + z*SIZE²`. Operacje: `get(IVec3)`, `set(IVec3, VoxelId) → bool`, `is_empty()`, `count_solid()`.

**`coords.rs`** — konwersje współrzędnych bez alokacji:
- `world_to_chunk::<SIZE>(IVec3) → (chunk_coord, local_coord)` — używa `div_euclid`/`rem_euclid` (poprawne dla ujemnych)
- `chunk_to_world::<SIZE>(chunk, local) → IVec3`
- `local_to_index::<SIZE>(IVec3) → usize`

**`VoxelWorld<const SIZE: usize = 16>`** — płaska `Vec<Chunk>`, lookup O(1). Utrzymuje `solid_count`. API: `get_voxel`, `set_voxel`, `get_chunk`, `get_chunk_mut`, `contains`.

**`WorldGenerator` trait** — `generate_chunk(chunk_coord: IVec3) → Chunk`. Implementacje dostarczane zewnętrznie.

---

## voxel_engine

Bevy ECS nad `voxel_core`. Uruchamia się headless (`MinimalPlugins`).

**Resources:**

```rust
WorldConfig {
    dimensions: WorldDimensions, // Domyślnie 20×8×20 chunków
    seed: u64,
}
VoxelWorldResource(DefaultWorld)
```

**Startup sequence:**
1. `init_voxel_world` — tworzy `VoxelWorldResource` z `WorldConfig::dimensions` (pusty świat)

---

## voxel_render

Bevy rendering. Wciąga `VoxelEnginePlugin` jako dependency.

**Region** — jednostka renderingu: `4×4×4` chunki = `64³` voxeli = jeden draw call. Domyślny świat 20×8×20 chunków → `5×2×5` = 50 regionów, maksymalnie 50 draw calls.

**Pipeline:**

```
seed_initial_regions (PostStartup)
    → spawn Entity per region z NeedsRemesh

handle_chunk_events (Update)
    → ChunkModified → insert NeedsRemesh na region

spawn_meshing_tasks (Update, after handle_chunk_events)
    → coalesce NeedsRemesh → MeshingQueue
    → max_spawns_per_frame = 2, max_inflight_tasks = 8
    → snapshot voxels z VoxelWorld (main thread, bounded copy 64³)
    → spawn async MeshTask do AsyncComputeTaskPool

apply_completed_meshes (Update, after spawn_meshing_tasks)
    → poll tasks (non-blocking)
    → pusty mesh → despawn entity
    → niepusty → upload Mesh → insert Mesh3d + MeshMaterial3d
```

**Greedy meshing** — 3-osiowy algorytm z 2D maską per slice. Dla każdej osi buduje maskę widocznych ścian (boundary solid/air), greedy merge prostokątów po U i V, winding order liczony geometrycznie. Output: `positions`, `normals`, `colors` (RGBA z palety), `indices` (U32).

**VoxelPalette** — `Vec<[f32; 4]>` indeksowany przez `VoxelId.0`. Fallback: magenta. Domyślna paleta: air, dirt, grass, stone.

**MeshingQueue** — `VecDeque` + `HashSet` dla deduplikacji — region trafia do kolejki tylko raz mimo wielokrotnych eventów.

---

## camera

Zależności: tylko `world_api` i Bevy. Dwa tryby kamery na jednej encji.

**Encja kamery** spawned w `setup.rs`:
```rust
Camera3d, Transform,
SpectatorCamera, SpectatorActive,
RtsCamera,
ActiveCamera,
ChunkObserver::default(), // load: 8, unload: 12 chunków
```

**SpectatorCamera** — swobodny lot WSAD + mysz (`AccumulatedMouseMotion`). Scroll zmienia prędkość lotu (1..200).

**RtsCamera** — pivot + offset:
- `pivot: Vec3`, `yaw: f32` (Q/E), `zoom: f32` (scroll, 15..250), pitch stały `45°`
- `translation = pivot + Quat::from_rotation_y(yaw) * Vec3(0, zoom*sin45, zoom*cos45)`
- PageUp/PageDown → ręczna zmiana `pivot.y`

**SwitchingPlugin** — Tab toggleuje tryb. Spectator→RTS: unlock kursor, swap markerów. RTS→Spectator: lock kursor, synchronizuje `yaw/pitch` z `Transform.rotation`.

**CursorRayPlugin** — `camera.viewport_to_world()` → wysyła `CursorRay` każdą klatkę gdy kursor w oknie.

---

## ui

Zależności: `world_api`, Bevy, `bevy_egui`.

**DebugUiPlugin** — egui window `"Debug"` w lewym górnym rogu. `DebugEntry` eventy → `DebugMetrics` Resource (`BTreeMap<section, BTreeMap<key, value>>`) → panel.

**DiagnosticsPlugin** — wysyła `DebugEntry` co 1s: FPS (smoothed), frame time ms.

---

## voxel_story (entry point)

```rust
// Pełna gra
App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin { present_mode: PresentMode::Immediate }))
    .add_plugins(VoxelRenderPlugin::default()) // wciąga VoxelEnginePlugin
    .add_plugins(CameraPlugin)
    .add_plugins(UiPlugin)
    .run();

// Headless
App::new()
    .add_plugins(MinimalPlugins)
    .add_plugins(VoxelEnginePlugin::default())
    .run();
```

`PresentMode::Immediate` — brak V-Sync. Benchmarki: 130 FPS / GTX 1060, 80 FPS / 780M.

---

## Przepływ danych — inicjalizacja świata

```
Startup:
    init_voxel_world         → VoxelWorldResource (pusty świat)

PostStartup:
    seed_initial_regions     → spawn Entity per region z NeedsRemesh

Update (klatka 1+):
    spawn_meshing_tasks      → max 2 taski/klatkę → AsyncComputeTaskPool
    apply_completed_meshes   → poll → Mesh3d upload
```