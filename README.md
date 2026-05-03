# AutoForge

A 2D top-down factory automation game built in Rust. Mine ores, smelt plates, assemble products, research technologies, and defend your factory from hostile creatures.

Inspired by Factorio and Shapez — the core loop is **build, optimize, expand, research, defend**.

## Features

- **Procedural world generation** — biomes, ore deposits (9 types), water, forests, enemy nests
- **Full production chain** — miners, smelters, assemblers, belts, inserters
- **18+ recipes** — from iron plates to red science packs and beyond
- **Research tree** — 17 technologies across multiple science tiers
- **Enemies & combat** — pollution attracts enemy waves, defend with turrets and walls
- **Pixel art sprites** — all generated procedurally at startup, no external assets
- **Save/Load** — F5 to save, F9 to load (JSON format)
- **100% offline** — single binary, no internet required

## Building

Requires [Rust](https://rustup.rs/) (1.70+).

```bash
cargo run --release
```

The release binary is at `target/release/autoforge`.

## Controls

| Key | Action |
|-----|--------|
| **WASD / Arrows** | Pan camera |
| **Scroll wheel** | Zoom (toward cursor) |
| **1-8** | Select building from toolbar |
| **Left click** | Place building (hold to drag-place belts) |
| **Right click** | Remove building |
| **R** | Rotate placement direction |
| **Q** | Eyedropper (pick building type from world) |
| **Space** | Pause / unpause |
| **Tab** | Toggle research screen |
| **F5** | Save game |
| **F9** | Load game |
| **Esc** | Deselect building |

## Gameplay Guide

### Getting Started

1. Pan to find **iron ore** (brown rocks) and **copper ore** (orange rocks) near the center
2. Place a **Miner** (key 2) on an ore deposit — it will extract ore automatically
3. Place **Belts** (key 1) leading away from the miner to transport ore
4. Place an **Inserter** (key 4) next to a **Furnace** (key 3) to feed ore into it
5. The furnace smelts ore into plates — use another inserter + belt to collect output
6. Build **Assemblers** (key 5) to craft components from plates

### Recipe Chain

```
Iron Ore ──[Smelter]──> Iron Plate ──[Assembler]──> Gear ─────────┐
                                                                    ├──> Red Science Pack
Copper Ore ─[Smelter]─> Copper Plate ─────────────────────────────┘

Iron Plate ──[Assembler]──> Gear ──────────────────────────┐
Copper Plate ─[Assembler]─> Wire ──> Green Circuit ────────┤
                                                            ├──> Green Science Pack
Iron Plate ─────────────────────────────────────────────────┘
```

### Power (Stone Furnaces)

Stone furnaces burn **coal** as fuel. Place coal on a belt leading to the furnace via an inserter — the furnace will automatically consume it.

### Research

Press **Tab** to open the research screen. Click a technology to start researching. Build **Labs** (key 8) and feed them science packs via inserters to advance research.

### Defense

Your factory generates **pollution** which spreads across the map. When pollution reaches enemy nests, they send attack waves. Build **Gun Turrets** and **Walls** to defend. Feed ammo to turrets via inserters.

## Architecture

| Module | Purpose |
|--------|---------|
| `main.rs` | Game loop (fixed 20 TPS timestep), input, UI |
| `constants.rs` | All tuning numbers |
| `types.rs` | Core enums (Resource, BuildingKind, Direction, etc.) |
| `grid.rs` | Flat tile grid with spatial item index |
| `mapgen.rs` | Procedural world generation |
| `building.rs` | Building arena with generational indices |
| `item.rs` | Item pool (pre-allocated, zero-alloc hot path) |
| `belt.rs` | Belt item movement with smooth interpolation |
| `inserter.rs` | Item transfer between belts and machines |
| `machine.rs` | Production: miners, smelters, assemblers |
| `recipe.rs` | 18 recipes with matching logic |
| `research.rs` | Tech tree (17 technologies) and lab processing |
| `pollution.rs` | Pollution generation and diffusion |
| `enemy.rs` | Enemy spawning, pathfinding, attacks |
| `combat.rs` | Turret targeting and damage |
| `sprites.rs` | Procedural pixel art (32-color palette) |
| `render.rs` | Frustum-culled world rendering |
| `save.rs` | JSON save/load |
| `camera.rs` | Pan/zoom camera |

## Performance

- Fixed 20 TPS simulation decoupled from rendering
- Flat array grid for O(1) tile lookups
- Generational arenas for buildings and items (no per-entity heap allocation)
- Frustum culling — only visible tiles are drawn
- Pollution diffusion runs every 5 ticks
- Release build with LTO and single codegen unit for maximum optimization

## License

MIT
