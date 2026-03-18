# voxel_story

Gra voxelowa — klon Castle Story w Rust z użyciem Bevy.

## Cel projektu

Budowanie, eksploracja i zarządzanie terenem na archipelagu lewitujących wysp złożonych z bloków. Silnik voxelowy pisany od podstaw z naciskiem na wydajny rendering fixed-size świata i edycję terenu w czasie rzeczywistym.

## Stan projektu

Wczesny etap — fundament silnika.

- Storage voxeli w chunkach 16×16×16
- Greedy meshing z async taskami per region (4×4×4 chunki)
- Kamera RTS + Spectator z przełączaniem
- Debug overlay z metrykami wydajności

## Sterowanie

| Klawisz | Akcja |
|---|---|
| WSAD | Ruch (Spectator: swobodny, RTS: pan) |
| Spacja / Shift | W górę / w dół (tylko Spectator) |
| Mysz | Obrót kamery (tylko Spectator) |
| Scroll | Zmiana prędkości lotu (Spectator) / zoom (RTS) |
| Q / E | Obrót kamery (tylko RTS) |
| PageUp / PageDown | Ręczna zmiana wysokości pivota (tylko RTS) |
| Tab | Przełączenie trybu kamery |

## Uruchomienie

```bash
cargo run --release
```

## Wymagania

- Rust (edycja 2024)
- GPU z obsługą Vulkan

## Wyniki testów

- 8745hs, 780M, Windows 11 — avg 600 FPS

## Inspiracje

- [Castle Story](https://castlestory.net/) — główna inspiracja rozgrywki i stylu świata
- [Minecraft](https://www.minecraft.net/) — model voxelowy i chunki
- [0 FPS — Meshing in a Minecraft Game](https://0fps.net/2012/06/30/meshing-in-a-minecraft-game/) — teoria meshingu