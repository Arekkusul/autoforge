# Spritesheet Layout Template

This file documents the exact layout of `spritesheet.png` — the unified sprite atlas.

## How to replace sprites

1. Generate/create a `spritesheet.png` (1024×1024 pixels, PNG, RGBA)
2. Place it in this `assets/` directory
3. The game will embed it at compile time via `include_bytes!`
4. No external files needed at runtime — still a single binary

## Layout (1024×1024 atlas)

All coordinates are top-left corner of each sprite.

### Row 0 (y=0): Ground Tiles — 32×32 each
| X | Sprite | Description |
|---|--------|-------------|
| 0 | Grass | Lush green with blade detail, tiny flowers |
| 32 | Grass Alt | Slightly different shade for variety |
| 64 | Desert | Warm sand with small pebbles |
| 96 | Forest | Deep green canopy with dappled light |
| 128 | Water | Soft blue with foam edges (frame 1) |
| 160 | Water Alt | Water animation frame 2 |

### Row 1 (y=32): Ore Deposits — 32×32 each
| X | Sprite | Description |
|---|--------|-------------|
| 0 | Iron | Rust-brown crystalline rock formation |
| 32 | Copper | Orange-green oxidized rock |
| 64 | Coal | Dark chunky with subtle sheen |
| 96 | Stone | Gray layered sedimentary |
| 128 | Uranium | Green-glowing crystal |
| 160 | Tin | Silver-white metallic |
| 192 | Gold | Brilliant yellow crystal |
| 224 | Sulfur | Bright yellow powder-rock |
| 256 | Crystal | Purple-pink gem formation |
| 288 | Oil | Dark pool with pump machinery |

### Row 2 (y=64): Machines — 32×32 each
| X | Sprite | Description |
|---|--------|-------------|
| 0 | Miner | Cute drill machine, warm coral tones |
| 32 | Stone Furnace | Brick oven, warm amber glow inside |
| 64 | Steel Furnace | Metallic, brighter fire |
| 96 | Assembler | Mechanical arms, periwinkle-blue |
| 128 | Lab | Flask/science equipment, lilac-purple |
| 160 | Boiler | Copper/teal with steam wisps |
| 192 | Steam Engine | Piston machinery, teal body |
| 224 | Solar Panel | Blue reflective panels |
| 256 | Storage Chest | Wooden crate, golden latch |
| 288 | Chemical Plant | Glass tubes, green liquid |
| 320 | Gun Turret | Gray metal base + barrel |
| 352 | Laser Turret | Sleek + blue glow |
| 384 | Wall | Thick stone blocks |
| 416 | Inserter | Arm on circular base |
| 448 | Splitter | Y-junction piece |

### Row 3 (y=96): Belts — 32×32 each (2 frames per tier)
| X | Sprite | Description |
|---|--------|-------------|
| 0 | Yellow Belt F1 | Yellow track, chevrons position 1 |
| 32 | Yellow Belt F2 | Chevrons shifted (animation) |
| 64 | Red Belt F1 | Red/orange track, frame 1 |
| 96 | Red Belt F2 | Frame 2 |
| 128 | Blue Belt F1 | Blue track, frame 1 |
| 160 | Blue Belt F2 | Frame 2 |

### Row 4 (y=128): Items — 16×16 each
| X | Sprite | Description |
|---|--------|-------------|
| 0 | Iron Ore | Rough brown rock chunk |
| 16 | Copper Ore | Orange rock with metallic glint |
| 32 | Coal | Dark black chunk |
| 48 | Stone | Gray rock piece |
| 64 | Iron Plate | Shiny gray flat square |
| 80 | Copper Plate | Warm orange flat square |
| 96 | Steel Plate | Blue-gray shiny |
| 112 | Stone Brick | Small gray brick |
| 128 | Gear | Tiny cogwheel with teeth |
| 144 | Wire | Thin copper squiggle |
| 160 | Pipe | Small tube section |
| 176 | Iron Stick | Thin rod |
| 192 | Green Circuit | Green board with traces |
| 208 | Red Circuit | Red board with more traces |
| 224 | Blue Circuit | Blue board, complex traces |
| 240 | Science Red | Red flask |
| 256 | Science Green | Green flask |
| 272 | Science Blue | Blue flask |
| 288 | Ammo | Brass bullet shape |
| 304 | Battery | Cylinder with + and - |

### Row 5 (y=144): Enemies — 24×24 each
| X | Sprite | Description |
|---|--------|-------------|
| 0 | Small Biter | Cute round bug, pinkish |
| 24 | Medium Biter | Larger, angrier face |
| 48 | Big Biter | Large, dark red, menacing |
| 72 | Spitter | Different body shape, ranged |

### Row 6 (y=168): FORGE Avatar — 48×48 each
| X | Sprite | Description |
|---|--------|-------------|
| 0 | Happy | Big smile, bright eyes |
| 48 | Curious | One eyebrow up, tilted |
| 96 | Worried | Small frown, sweat drop |

## Art Style Guidelines

- **Resolution:** 32×32 for buildings/terrain, 16×16 for items, 24×24 for enemies
- **Aesthetic:** Cute, cozy, rounded shapes. Think "candy factory" not "grimdark"
- **Light source:** Top-left on all sprites
- **Outlines:** Colored (selout), not pure black. 1px on outer edge.
- **Shading:** 2-3 values per surface (base + shadow + highlight)
- **Palette:** Warm pastels, soft gradients. Reference: Core Keeper, Stardew Valley
- **Background:** Transparent (RGBA with alpha=0 for empty space)

## AI Prompt Templates

For PixelLab / Sprite-AI / SEELE:

**Ground:** "pixel art top-down grass tile, 32x32, lush green with tiny flowers, soft lighting, cozy game style, transparent background"

**Machines:** "pixel art top-down cute factory machine, 32x32, [coral/blue/purple] colored, rounded shape, small indicator lights, soft shadows, cozy industrial style"

**Items:** "pixel art game item icon, 16x16, [iron plate/gear/circuit], clean simple design, slight shine, transparent background"

**Enemies:** "pixel art top-down cute bug creature, 24x24, [pink/red], round body, small angry eyes, pixel art style, transparent background"
