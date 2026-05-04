# AutoForge

A narrative-driven 2D factory automation game built in Rust. You are **FORGE** — a fractured AI consciousness from a crashed colony ship, rebuilding yourself on an alien world while uncovering the mysteries beneath the surface.

## Features

### Core Gameplay
- **41 recipes** across smelting, assembly, and chemical processing
- **29 technologies** in a prerequisite research tree (Red → Green → Blue → Purple → Yellow science)
- **Full production chain**: miners → belts → inserters → furnaces → assemblers → labs
- **9 ore types** (iron, copper, coal, stone, tin, gold, sulfur, crystal, uranium) as 2×2 deposits
- **Power system**: boilers + steam engines + solar panels + nuclear reactors
- **Day/night cycle** with solar output variation
- **Build zone** that expands as you research (centered on your crashed ship)

### Combat & Defense
- **8 enemy types** (4 biters + 4 spitters) that evolve over time
- **Pollution system**: machines generate pollution → attracts enemy waves from nests
- **Gun turrets** (ammo-consuming) and **laser turrets** (power-consuming)
- **Walls** with HP, auto-regeneration, and damage flash indicators
- **Evolution factor**: enemies get harder as time passes and pollution spreads

### Narrative
- **Intro cutscene** with typewriter text — FORGE wakes up after the crash
- **24 story beats** triggered by milestones (items crafted, research, kills, time)
- **Interactive crashed ship** with rotating lore messages
- **Cute FORGE personality**: friendly, confused, determined to find the colonists
- **Endgame goal**: reconstruct consciousness (50,000 items crafted)

### Quality of Life
- **Tutorial** (5-step guided walkthrough, press H to toggle)
- **Recipe picker** (click assembler → visual popup with all recipes and availability)
- **Recipe browser** (E key — full reference of what makes what)
- **Production stats** (V key — playtime, items/min, building counts)
- **Achievements** (N key — 14 milestones with resource rewards)
- **Blueprint tool** (B key — copy buildings near cursor, click to stamp)
- **Undo** (Ctrl+Z — removes last placed building)
- **Auto-save** every 5 minutes
- **Game speed** 1x–5x (+/- keys)
- **Auto-rotate belts** when drag-placing (follows mouse direction)
- **Edge-scroll** (mouse at screen edges pans camera)
- **Hand-insert** (middle-click to put inventory items directly into machines)

### Logistics
- **3 belt tiers** (yellow/red/blue — 1x/2x/3x speed)
- **Underground belts** (tunnel under buildings, U key)
- **Splitters** (alternate items between two outputs)
- **Storage chests** (auto-feed player inventory)
- **Inserters** (4 tiers — regular/long/fast/stack)
- **Trains** (click train stop to spawn, auto-routes between all stops)
- **Roboport logistics** (auto-delivers items from inventory to nearby machines)

### Technical
- **100% offline** — single binary, no network code, no internet required
- **~1.1 MB** release binary (LTO + strip)
- **60 FPS** with adaptive LOD (auto-degrades if FPS drops)
- **Unified texture atlas** for ground rendering (1 GPU draw call)
- **Bincode save format** (10x smaller than JSON, loads old JSON saves too)
- **10,000+ lines** of Rust across 31 modules

## Building

Requires [Rust](https://rustup.rs/) (1.70+).

```bash
cargo run --release
```

Binary: `target/release/autoforge`

## Controls

| Key | Action |
|-----|--------|
| **WASD / Arrows** | Pan camera |
| **Scroll wheel** | Zoom (toward cursor) |
| **Mouse at edge** | Edge-scroll |
| **1-0, U, T, G, C, P** | Select buildings |
| **Click toolbar** | Select building |
| **Left click** | Place building (hold to drag-place belts) |
| **Right click** | Remove building (hold to mass-delete) |
| **R** | Rotate direction |
| **Q** | Eyedropper (copy type from world) |
| **B** | Blueprint (copy/paste buildings) |
| **Ctrl+Z** | Undo last placement |
| **Middle click** | Hand-insert item into machine |
| **Click assembler** | Open recipe picker popup |
| **E** | Recipe book |
| **Tab** | Research tree |
| **N** | Achievements |
| **V** | Production stats |
| **H** | Toggle tutorial |
| **F1** | Full help / controls |
| **Space** | Pause (shows menu) |
| **+/-** | Game speed (1x–5x) |
| **Home** | Center camera on base |
| **F5 / F9** | Save / Load |
| **Esc** | Close overlay / deselect |

## How to Play

### Getting Started
1. You spawn next to your crashed ship (FORGE BASE) with starter resources
2. All 4 ore types (iron, copper, coal, stone) are within 8 tiles of spawn
3. Place a **Miner** on ore → items flow onto adjacent belt automatically
4. Feed ore + coal into a **Furnace** via inserters → produces plates
5. Put plates into a **Storage Chest** → they become your building inventory
6. Build an **Assembler**, click it to select a recipe (e.g., "Craft Gear")
7. Feed the assembler its inputs → it produces output on adjacent belts

### Key Concepts
- **Storage Chests** auto-feed your inventory (this is how you get building materials)
- **Furnaces need coal** for fuel (the FUEL! indicator shows when they're out)
- **Assemblers lock to one recipe** — click to change via the recipe picker
- **Build zone**: you can only build within a radius of the ship (expands with research)
- **Robot workers** animate from the ship to each building you place
- **Roboports** auto-deliver items to nearby machines from your inventory

### Recipe Chain
```
Iron Ore ──[Furnace+Coal]──> Iron Plate ──[Assembler]──> Gear ──┐
                                                                  ├──> Red Science
Copper Ore ─[Furnace+Coal]─> Copper Plate ─────────────────────┘

Wire (from Copper Plate) + Iron Plate → Green Circuit
Green Circuit + Gear + Iron Plate → Inserter (item)
Inserter (item) + Iron Plate → Green Science
```

### Progression
1. Automate Iron + Copper plates (with coal fuel)
2. Build Assembler → make Gears
3. Make Red Science → feed Lab → research
4. Set up Green Science chain (Wire → Circuit → Inserter item)
5. Research unlocks: belts tiers, fast inserters, steel, solar, military
6. Defend against enemy waves (turrets + walls + ammo)
7. Scale up: chemical processing, nuclear power, trains
8. Reach 50,000 items crafted → consciousness restored → narrative end

## Architecture

| Module | Purpose |
|--------|---------|
| `main.rs` | Game loop, input, UI (unified draw_panel system) |
| `types.rs` | Core enums: Resource (42 types), BuildingKind (40+), Direction, GridPos |
| `grid.rs` | Flat Vec<Tile> grid, spatial item index, coordinate math |
| `building.rs` | Generational arena for buildings, placement validation |
| `item.rs` | Pre-allocated item pool (zero alloc in hot path) |
| `belt.rs` | Belt movement with underground support + corner detection |
| `inserter.rs` | Two-pass item transfer (pick → deliver) |
| `machine.rs` | Miner ejection, furnace fuel, assembler processing |
| `recipe.rs` | 41 recipes with locked recipe matching |
| `research.rs` | 29 technologies, lab consumption, prerequisites |
| `power.rs` | Global power pool (boilers, solar, nuclear, brownout) |
| `daynight.rs` | 7-min day / 3-min night cycle, solar multiplier |
| `pollution.rs` | Sparse diffusion, tree absorption |
| `enemy.rs` | 8 types, evolution, wave spawning, pathfinding |
| `combat.rs` | Turret targeting, ammo consumption |
| `train.rs` | Train entities, schedule-based movement |
| `story.rs` | 24 narrative beats triggered by milestones |
| `milestones.rs` | 14 achievements with resource rewards |
| `cutscene.rs` | Intro sequence with typewriter text + FORGE avatar |
| `sprites.rs` | 64-color palette, procedural pixel art (all sprites in code) |
| `render.rs` | LOD rendering, atlas ground, frustum culling, corner belts |
| `atlas.rs` | Unified texture atlas packer |
| `batcher.rs` | Mesh-based sprite batcher for 1-draw-call rendering |
| `save.rs` | Bincode binary saves (with JSON fallback) |
| `mapgen.rs` | Procedural world: biomes, ores, water, forests, crash debris |
| `buildcost.rs` | Building resource costs, affordability checks |
| `camera.rs` | Pan/zoom with edge-scroll and Home key |
| `fluid.rs` | Pump jack oil extraction |
| `splitter.rs` | Belt splitting with alternation |

## Performance

- Atlas-based ground rendering (1 GPU draw call for all terrain)
- 3-level LOD with FPS-based auto-degradation
- Frequency-gated simulation (heavy systems run less often)
- Frustum culling for all entities
- Sparse pollution diffusion (only processes polluted tiles)
- Pre-allocated generational arenas (zero heap alloc in hot paths)
- Binary save format (bincode — 10x smaller than JSON)
- Release build: LTO + single codegen unit + stripped symbols

## License

MIT
