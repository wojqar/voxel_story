# voxel_story

Gra voxelowa — klon Castle Story w Rust z użyciem Bevy.

## Cel projektu

Budowanie, eksploracja i zarządzanie terenem na archipelagu lewitujących wysp złożonych z bloków. Silnik voxelowy pisany od podstaw z naciskiem na wydajny rendering fixed-size świata, edycję terenu w czasie rzeczywistym i obliczenia pathfindingu dla jednostek.

## Stan projektu

Wczesny etap — fundament silnika.

- Generacja i storage voxeli w chunkach 16×16×16
- Greedy meshing z cullingiem między sąsiednimi chunkami
- Generator archipelagu wysp oparty na 3D density function (Perlin noise)
- Kamera spektatora (WSAD + mysz + scroll)
- Debug UI z metrykami wydajności

## Architektura

Projekt podzielony na workspace crates:

| Crate | Odpowiedzialność |
|---|---|
| `voxel_engine` | Storage, generacja, rendering chunków |
| `debug_ui` | Generyczny overlay z metrykami (niezależny od silnika) |
| `spectator` | Kamera swobodna |
| `voxel_story` | Główna aplikacja, łączy wszystkie crate'y |

## Świat

Fixed-size archipelag lewitujących wysp. Domyślna mapa: **30 × 8 × 30 chunków** (480 × 128 × 480 bloków). Rozmiar ustalany w `WorldConfig` przed kompilacją.

## Roadmap

1. Voxel editing — stawianie i usuwanie bloków z propagacją remesh
2. World coordinate API — czyste przejście między world-space, chunk i local
3. Raycast / picking — selekcja bloków kursorem
4. Pathfinding — nawigacja jednostek po woksylowej siatce
5. Jednostki i surowce — core rozgrywki

## Sterowanie

| Klawisz | Akcja |
|---|---|
| WSAD | Ruch (Spectator: swobodny, RTS: pan) |
| Spacja / Shift | W górę / w dół (tylko Spectator) |
| Mysz | Obrót kamery (tylko Spectator) |
| Scroll | Zmiana prędkości lotu (Spectator) / zoom (RTS) |
| Q / E | Obrót kamery w lewo / prawo (tylko RTS) |
| PageUp / PageDown | Ręczna zmiana wysokości pivota (tylko RTS) |
| LPM | Usuń blok (tylko RTS) |
| PPM | Postaw blok (tylko RTS) |
| 1 / 2 / 3 | Wybór materiału: Stone / Dirt / Grass (tylko RTS) |
| Tab | Przełączenie trybu kamery |
| Escape | Odblokowanie / zablokowanie kursora (tylko Spectator) |

## Uruchomienie

```bash
cargo run --release
```

## Wymagania

- Rust (edycja 2024)
- GPU z obsługą Vulkan (testowane na NVIDIA GTX 1060 + Fedora Linux)

## Inspiracje

- [Castle Story](https://castlestory.net/) — główna inspiracja rozgrywki i stylu świata
- [Minecraft](https://www.minecraft.net/) — model voxelowy i chunki
- [0 FPS — Meshing in a Minecraft Game](https://0fps.net/2012/06/30/meshing-in-a-minecraft-game/) — teoria meshingu