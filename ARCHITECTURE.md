# Architektura voxel_story

## Zasady — NIGDY nie łamać

1. `voxel_core` nie importuje Bevy
2. Zależności tylko w dół — wyższy crate zależy od niższego, nigdy odwrotnie
3. Logika nie zależy od prezentacji — `voxel_interaction` nie importuje `voxel_render`
4. Decyzja o ładowaniu chunków należy do `voxel_engine`, nie do kamery ani renderingu
5. Kosztowne operacje nigdy nie blokują głównego wątku
6. Warstwy komunikują się tylko przez eventy z `world_api` — nigdy przez bezpośrednie importy między sobą
7. Kamera przelicza screen-space na world-space i wysyła gotowe dane — odbiorca nie wie nic o kamerze

---

## Struktura crate'ów

```
world_api           wspólne typy eventów, zero logiki
voxel_core          dane i logika świata, czysty Rust
voxel_engine        Bevy ECS, zarządzanie światem
voxel_render        meshowanie, GPU, frustum culling
voxel_interaction   raycast, edycja bloków
camera              RtsCamera, SpectatorCamera, przełączanie
ui                  HUD, highlight, tooltip
voxel_story         aplikacja, łączy wszystko
```

---

## Hierarchia zależności

```
world_api
    ├── voxel_core
    │       └── voxel_engine
    │               ├── voxel_render
    │               └── voxel_interaction
    ├── camera
    ├── ui
    └── voxel_story → wszystko
```

Każdy crate zależy od `world_api`. `voxel_interaction` zależy od `voxel_engine` (potrzebuje VoxelWorld do raycasta). Poza tym — tylko w dół.

---

## world_api

Cienka warstwa. Zero logiki, zero systemów Bevy. Same definicje typów.

```rust
// events.rs

// Kamera → voxel_interaction
CursorRay { origin: Vec3, direction: Vec3 }

// voxel_interaction → wszyscy
BlockTargeted { pos: IVec3, normal: IVec3 }
BlockTargetCleared

// ui/camera → voxel_interaction
BlockInteract { pos: IVec3, action: InteractAction }
pub enum InteractAction { Remove, Place(VoxelId) }

// voxel_engine → wszyscy
ChunkLoaded(IVec3)
ChunkUnloaded(IVec3)
ChunkModified(IVec3)

// camera → voxel_engine
TerrainHeightRequest { pos: Vec2 }

// voxel_engine → camera
TerrainHeightResponse { height: f32 }

// components.rs
#[derive(Component)]
ActiveCamera        — marker aktywnej kamery, dodawany przez switching system
ChunkObserver { load_distance: u32, unload_distance: u32 }
                    — kto ma ten komponent, wokół niego ładowane są chunki
                    — może być na kamerze, jednostce, cokolwiek
```

---

## voxel_core

Czysty Rust. Zero Bevy, zero GPU, zero async. Testowalny jednostkowo.

```
voxel.rs        VoxelId(u16), is_air(), is_solid(), is_transparent()
                VoxelMaterial — kolor, twardość, właściwości

chunk.rs        Chunk<const SIZE: usize = 16>
                voxels: [VoxelId; SIZE³]
                get(x,y,z), set(x,y,z), is_empty(), count_solid()

coords.rs       world_to_chunk(IVec3) → (chunk_coord, local_coord)
                chunk_to_world(IVec3) → IVec3
                local_to_index(IVec3) → usize

world.rs        WorldDimensions { x, y, z: u32 }
                VoxelWorld
                    chunks: Vec<Chunk>      płaska Vec, O(1) lookup
                    dimensions: WorldDimensions
                    solid_count: usize

                API:
                    new(WorldDimensions) → Self
                    get_voxel(IVec3) → VoxelId
                    set_voxel(IVec3, VoxelId) → bool
                    get_chunk(IVec3) → Option<&Chunk>
                    get_chunk_mut(IVec3) → Option<&mut Chunk>
                    contains(IVec3) → bool

generation/     trait WorldGenerator
                    generate_chunk(IVec3) → Chunk
                IslandGenerator — 3D density function, Perlin noise
```

---

## voxel_engine

Bevy ECS. Opakowuje `voxel_core`. Zero renderingu.

```
plugin.rs       VoxelEnginePlugin
                    rejestruje VoxelWorld jako Resource
                    rejestruje eventy z world_api

resources.rs    WorldConfig — rozmiar świata, seed, parametry generacji

observer.rs     słucha ChunkObserver (zdefiniowany w world_api)
                    system śledzi pozycje obserwatorów
                    ładuje/wyładowuje chunki
                    emituje ChunkLoaded / ChunkUnloaded

editing.rs      słucha BlockInteract
                wywołuje world.set_voxel()
                emituje ChunkModified

generation.rs   async generacja chunków
                priorytet — bliżej obserwatora = wyższy priorytet

terrain.rs      słucha TerrainHeightRequest
                raycast w dół przez VoxelWorld
                odpowiada TerrainHeightResponse
```

Użycie:
```rust
// Headless — zero GPU, zero okna
app.add_plugins(VoxelEnginePlugin::default())
```

---

## voxel_render

Bevy rendering. Reaguje na eventy z `voxel_engine`. Samo dodaje `VoxelEnginePlugin`.

```
plugin.rs       VoxelRenderPlugin
                    dodaje VoxelEnginePlugin jako dependency

region.rs       Region = 4×4×4 chunki = jeden draw call
                30×8×30 chunki → ~112 regionów → ~112 draw calls max

meshing.rs      greedy meshing na poziomie regionu (64³ bloków)
                async, priorytet blisko kamery
                pakowany format wierzchołka (~8 bajtów zamiast 48)

culling.rs      frustum culling per region z AABB

pipeline:
    ChunkLoaded   → oznacz region NeedsRemesh
    NeedsRemesh   → spawn async MeshTask
    MeshTask gotowy → upload do GPU
    ChunkUnloaded → usuń mesh jeśli region pusty
    ChunkModified → oznacz region NeedsRemesh
```

Użycie:
```rust
// Samo wciąga VoxelEnginePlugin
app.add_plugins(VoxelRenderPlugin::default())
```

---

## voxel_interaction

Raycast i input gracza. Tłumaczy intencje gracza na eventy. Zależy od `voxel_engine` (potrzebuje VoxelWorld do raycasta).

```
raycast.rs      słucha CursorRay
                DDA raycast przez VoxelWorld
                emituje BlockTargeted / BlockTargetCleared

input.rs        słucha input gracza (LPM, PPM)
                czyta aktualny BlockTargeted
                wysyła BlockInteract { pos, action }
                — koniec. nie wie co się dzieje dalej.
```

`voxel_engine` słucha `BlockInteract` → wywołuje `set_voxel` → emituje `ChunkModified`.

---

## camera

Zero zależności poza `world_api` i Bevy. Izolowana.

```
spectator.rs    SpectatorCamera — swobodna, WSAD + mysz
                SpectatorActive — marker komponent

rts.rs          RtsCamera — pivot, zoom, pan, Q/E obrót
                RtsActive — marker komponent
                ChunkObserver (z world_api) — dodawany do encji kamery przez voxel_story

                pivot Y:
                    wysyła TerrainHeightRequest
                    słucha TerrainHeightResponse → lerp pivot.y
                    PageUp/PageDown → manual override

switching.rs    Tab → toggle między trybami
                dodaje/usuwa SpectatorActive, RtsActive, ActiveCamera

cursor_ray.rs   słucha Bevy CursorMoved
                przelicza screen-space → world-space ray
                wysyła CursorRay (tylko gdy RtsActive)
```

Użycie:
```rust
// Samo dodaje SpectatorPlugin i RtsCameraPlugin
app.add_plugins(CameraPlugin::default())
```

---

## ui

Zero zależności poza `world_api` i Bevy.

```
highlight.rs    słucha BlockTargeted → podświetlenie bloku
                słucha BlockTargetCleared → chowa highlight

tooltip.rs      słucha BlockTargeted → typ bloku, współrzędne

block_picker.rs wybór materiału (1/2/3)
                Resource z aktualnym VoxelId
                dołączany do BlockInteract
```

---

## Przepływ — edycja bloku

```
mysz się rusza
    → cursor_ray.rs przelicza → wysyła CursorRay { origin, direction }

voxel_interaction/raycast.rs słucha CursorRay
    → DDA raycast → trafiony blok P
    → wysyła BlockTargeted { pos: P, normal: N }

ui słucha BlockTargeted → pokazuje highlight, tooltip

gracz klika LPM
    → voxel_interaction/input.rs wysyła BlockInteract { pos: P, action: Remove }
    — koniec odpowiedzialności voxel_interaction

voxel_engine/editing.rs słucha BlockInteract
    → world.set_voxel(P, AIR)
    → wysyła ChunkModified(chunk_coord)

voxel_render słucha ChunkModified → NeedsRemesh → async remesh
```

---

## Przepływ — ładowanie chunków

```
kamera RTS ma ChunkObserver { load_distance: 8, unload_distance: 12 }

voxel_engine/observer.rs co klatkę:
    → sprawdza pozycje wszystkich ChunkObserver
    → ładuje chunki w zasięgu  → emituje ChunkLoaded
    → wyładowuje poza zasięgiem → emituje ChunkUnloaded

voxel_render słucha:
    ChunkLoaded   → NeedsRemesh na region
    ChunkUnloaded → usuń mesh jeśli region pusty
```

---

## Użycie w main.rs

```rust
// Pełna gra
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VoxelRenderPlugin::default())       // wciąga VoxelEnginePlugin
        .add_plugins(VoxelInteractionPlugin::default())
        .add_plugins(CameraPlugin::default())            // wciąga Spectator + RTS
        .add_plugins(UiPlugin::default())
        .run();
}

// Headless
fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(VoxelEnginePlugin::default())
        .run();
}
```