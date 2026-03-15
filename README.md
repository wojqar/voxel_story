# voxel_story

Gra voxelowa inspirowana [Castle Story](https://castlestory.net/) — budowanie, eksploracja i zarządzanie terenem w świecie złożonym z bloków.

## Cel projektu

Celem jest zbudowanie od podstaw silnika voxelowego w Rust z użyciem Bevy, a następnie oparcie na nim rozgrywki w stylu Castle Story: dynamiczny teren, budowanie struktur, jednostki i surowce.

## Stan projektu

Wczesny etap — fundament silnika.

- Generacja i storage voxeli w chunkach 16×16×16
- Naive face culling z uwzględnieniem sąsiednich chunków
- Heightmap generator terenu oparty na fBm (Perlin noise)
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

## Sterowanie

| Klawisz | Akcja |
|---|---|
| WSAD | Ruch |
| Spacja / Shift | W górę / w dół |
| Mysz | Obrót kamery |
| Scroll | Zmiana prędkości lotu |
| Escape | Odblokowanie / zablokowanie kursora |

## Uruchomienie

```bash
cargo run --release
```

## Wymagania

- Rust (edycja 2024)
- GPU z obsługą Vulkan (testowane na NVIDIA GTX 1060 + Fedora Linux)

## Inspiracje

- [Castle Story](https://castlestory.net/) — główna inspiracja rozgrywki
- [Minecraft](https://www.minecraft.net/) — model voxelowy i chunki
- [0 FPS — Meshing in a Minecraft Game](https://0fps.net/2012/06/30/meshing-in-a-minecraft-game/) — teoria meshingu
