//! Procedural pixel art sprite generation.
//!
//! All sprites are defined as const 2D arrays of palette indices and converted to
//! [`Texture2D`] at startup. This means the entire game ships as a single binary
//! with no external image files, while still having proper pixel art visuals.
//!
//! # Palette
//!
//! A shared 32-color palette (inspired by SNES/GBA factory games) is used across
//! all sprites. Each sprite pixel is a `u8` index into this palette. Index 0 is
//! transparent.
//!
//! # Sprite sizes
//!
//! - **Tiles / buildings**: 16×16 pixels (scaled to [`TILE_SIZE`](crate::constants::TILE_SIZE))
//! - **Items on belts**: 8×8 pixels
//! - **Enemies**: 12×12 pixels

use macroquad::prelude::*;

/// 64-color palette for production-quality sprites.
/// Organized in color ramps (4 shades per hue) for smooth gradients.
/// Index 0 = transparent. RGBA format.
///
/// Layout: Each color family has 4 entries (darkest → lightest).
/// This allows smooth dithering and anti-aliasing between shades.
pub static PALETTE: [(u8, u8, u8, u8); 64] = [
    // 0: Transparent
    (0, 0, 0, 0),
    // 1-4: Neutral grays (outlines, shadows, metals)
    (25, 22, 40, 255),     // 1: Deepest shadow (soft black-purple)
    (48, 44, 65, 255),     // 2: Dark shadow
    (85, 80, 110, 255),    // 3: Mid gray-purple
    (130, 125, 155, 255),  // 4: Light gray-purple
    // 5-8: Whites and highlights
    (175, 172, 195, 255),  // 5: Silver
    (210, 208, 225, 255),  // 6: Pale highlight
    (238, 236, 245, 255),  // 7: Near-white
    (255, 253, 250, 255),  // 8: Pure specular
    // 9-12: Greens (ground, forest)
    (35, 65, 40, 255),     // 9: Deep forest
    (55, 95, 55, 255),     // 10: Dark grass
    (78, 135, 72, 255),    // 11: Grass
    (120, 180, 105, 255),  // 12: Light grass / highlight
    // 13-16: Browns (ores, wood)
    (65, 38, 28, 255),     // 13: Dark brown
    (105, 62, 42, 255),    // 14: Mid brown (iron ore)
    (148, 95, 58, 255),    // 15: Warm brown
    (192, 138, 85, 255),   // 16: Light brown / tan
    // 17-20: Oranges/Coral (miner, copper)
    (135, 58, 35, 255),    // 17: Dark coral
    (180, 88, 52, 255),    // 18: Coral
    (220, 135, 72, 255),   // 19: Light coral / amber
    (245, 185, 110, 255),  // 20: Peach highlight
    // 21-24: Reds (furnace, enemies)
    (95, 28, 28, 255),     // 21: Dark red
    (148, 48, 45, 255),    // 22: Mid red
    (205, 78, 65, 255),    // 23: Warm red
    (240, 125, 105, 255),  // 24: Light salmon
    // 25-28: Blues (assembler, water)
    (30, 48, 105, 255),    // 25: Deep blue
    (52, 78, 155, 255),    // 26: Mid blue
    (88, 125, 205, 255),   // 27: Periwinkle
    (135, 175, 235, 255),  // 28: Light sky blue
    // 29-32: Purples (lab, FORGE)
    (68, 35, 105, 255),    // 29: Deep purple
    (108, 62, 155, 255),   // 30: Mid purple
    (155, 98, 205, 255),   // 31: Lilac
    (198, 148, 235, 255),  // 32: Light lavender
    // 33-36: Teals (power, generators)
    (28, 72, 72, 255),     // 33: Deep teal
    (48, 115, 112, 255),   // 34: Mid teal
    (82, 162, 155, 255),   // 35: Mint
    (128, 205, 195, 255),  // 36: Light mint
    // 37-40: Yellows (belts, warnings)
    (105, 95, 28, 255),    // 37: Dark gold
    (165, 152, 42, 255),   // 38: Gold
    (225, 210, 68, 255),   // 39: Yellow
    (248, 238, 125, 255),  // 40: Light lemon
    // 41-44: Pinks (enemies, highlights)
    (115, 35, 62, 255),    // 41: Dark rose
    (172, 58, 88, 255),    // 42: Rose
    (225, 95, 125, 255),   // 43: Pink
    (248, 155, 175, 255),  // 44: Light pink
    // 45-48: Stone/Earth (walls, stone)
    (72, 68, 62, 255),     // 45: Dark stone
    (112, 108, 98, 255),   // 46: Mid stone
    (155, 150, 138, 255),  // 47: Light stone
    (198, 195, 185, 255),  // 48: Pale stone
    // 49-52: Copper/Gold metallic
    (125, 72, 22, 255),    // 49: Dark copper
    (178, 108, 38, 255),   // 50: Mid copper
    (225, 155, 65, 255),   // 51: Bright copper
    (248, 205, 115, 255),  // 52: Gold highlight
    // 53-56: Cool metals (steel, inserters)
    (62, 72, 85, 255),     // 53: Dark steel
    (95, 108, 125, 255),   // 54: Mid steel
    (135, 148, 168, 255),  // 55: Light steel
    (178, 192, 208, 255),  // 56: Steel highlight
    // 57-60: Greens bright (circuits, science)
    (22, 95, 45, 255),     // 57: Dark circuit green
    (48, 155, 72, 255),    // 58: Circuit green
    (82, 210, 105, 255),   // 59: Bright green
    (145, 238, 158, 255),  // 60: Light green glow
    // 61-63: Special (fire glow, uranium, warning)
    (255, 155, 32, 255),   // 61: Fire orange
    (48, 225, 85, 255),    // 62: Uranium glow
    (255, 68, 68, 255),    // 63: Alert red
];

/// Holds all game sprites packed into a SINGLE texture atlas.
///
/// **Production architecture:** All sprites are packed into ONE 512×512 texture.
/// Each sprite field is a `Rect` (source UV coordinates within the atlas).
/// The renderer uses `draw_texture_ex` with `source: Some(rect)` — since all
/// draws use the same texture, macroquad batches them into 1-3 GPU draw calls.
///
/// Additionally, the legacy `Texture2D` fields are kept for backward compatibility
/// during the migration. New code should use `atlas.tex` + source rects via the batcher.
pub struct SpriteAtlas {
    /// The single GPU texture containing ALL sprites (512×512).
    /// Use this with `source: Some(rect)` in draw_texture_ex for 1 draw call batching.
    pub tex: Texture2D,
    /// Atlas dimensions for UV normalization.
    pub tex_size: Vec2,

    // --- Atlas source rects (position of each sprite within `tex`) ---
    /// Source rects for ground tiles packed in the atlas.
    pub r_ground_grass: Rect,
    pub r_ground_grass_alt: Rect,
    pub r_ground_desert: Rect,
    pub r_ground_forest: Rect,
    pub r_ground_water: Rect,
    pub r_ground_water_alt: Rect,

    // --- Legacy individual textures (kept for compatibility, will be removed) ---
    pub ground_grass: Texture2D,
    pub ground_grass_alt: Texture2D,
    pub ground_desert: Texture2D,
    pub ground_forest: Texture2D,
    pub ground_water: Texture2D,
    pub ground_water_alt: Texture2D,

    pub ore_iron: Texture2D,
    pub ore_copper: Texture2D,
    pub ore_coal: Texture2D,
    pub ore_stone: Texture2D,
    pub ore_uranium: Texture2D,
    pub ore_tin: Texture2D,
    pub ore_gold: Texture2D,
    pub ore_sulfur: Texture2D,
    pub ore_crystal: Texture2D,
    pub ore_oil: Texture2D,

    pub belt_yellow: [Texture2D; 2],
    pub belt_red: [Texture2D; 2],
    pub belt_blue: [Texture2D; 2],

    pub miner: Texture2D,
    pub stone_furnace: Texture2D,
    pub steel_furnace: Texture2D,
    pub assembler: Texture2D,
    pub lab: Texture2D,
    pub boiler: Texture2D,
    pub steam_engine: Texture2D,
    pub solar_panel: Texture2D,
    pub chest: Texture2D,

    pub gun_turret: Texture2D,
    pub wall: Texture2D,
    pub inserter: Texture2D,

    pub item_iron_ore: Texture2D,
    pub item_copper_ore: Texture2D,
    pub item_coal: Texture2D,
    pub item_stone: Texture2D,
    pub item_iron_plate: Texture2D,
    pub item_copper_plate: Texture2D,
    pub item_gear: Texture2D,
    pub item_wire: Texture2D,
    pub item_green_circuit: Texture2D,
    pub item_science_red: Texture2D,

    pub enemy_small_biter: Texture2D,
}

impl SpriteAtlas {
    /// Generates all sprite textures from pixel data. Call once at startup.
    ///
    /// Note on performance: macroquad automatically batches consecutive draws of
    /// the SAME texture. Since each sprite is a different Texture2D, every sprite
    /// switch breaks the batch. For maximum performance, a true atlas would pack
    /// all into one texture. However, macroquad's internal batching is efficient
    /// enough for <500 on-screen sprites (our typical case with frustum culling).
    ///
    /// The real performance wins come from:
    /// 1. Frustum culling (only draw visible tiles/entities)
    /// 2. LOD system (fewer draws when zoomed out)
    /// 3. Frequency-gated simulation (heavy systems run less often)
    /// 4. Adaptive quality (auto-LOD when FPS drops)
    pub fn generate() -> Self {
        // ================================================================
        // PRODUCTION ASSET PIPELINE
        // ================================================================
        // Priority 1: Load from embedded PNG spritesheet (AI-generated art).
        //   Place a `spritesheet.png` in assets/ and it auto-embeds at compile time.
        // Priority 2: Fall back to procedural generation (current approach).
        //
        // To use AI-generated sprites:
        //   1. Generate sprites following assets/SPRITESHEET_TEMPLATE.md
        //   2. Save as assets/spritesheet.png (1024×1024, RGBA PNG)
        //   3. Uncomment the include_bytes! line below
        //   4. Rebuild — sprites are now embedded in the binary
        //
        // const SPRITESHEET: &[u8] = include_bytes!("../assets/spritesheet.png");
        // let atlas_tex = Texture2D::from_file_with_format(SPRITESHEET, Some(ImageFormat::Png));
        // ================================================================

        // Currently using procedural generation (no PNG available yet).
        // All sprites are packed into a single 512×512 Image at startup.
        // Each sprite gets a Texture2D created from the SAME atlas image.
        // Since macroquad batches draws of the same Texture2D, this means
        // ALL world rendering becomes 1-3 GPU draw calls.
        //
        // Architecture: We build ONE big image, upload it ONCE, then create
        // individual Texture2D handles that all point to the same GPU texture.
        // macroquad doesn't directly support sub-textures, so we use
        // draw_texture_ex with source Rects for atlas-based rendering.
        // ================================================================

        let mut atlas_img = Image::gen_image_color(512, 512, Color::new(0.0, 0.0, 0.0, 0.0));

        // Pack white pixel at (0,0) for solid rect rendering.
        atlas_img.set_pixel(0, 0, WHITE);
        atlas_img.set_pixel(1, 0, WHITE);
        atlas_img.set_pixel(0, 1, WHITE);
        atlas_img.set_pixel(1, 1, WHITE);

        // Helper: blit a sprite image into the atlas at the given position.
        fn blit(atlas: &mut Image, sprite: &Image, ax: u32, ay: u32) {
            for y in 0..sprite.height() as u32 {
                for x in 0..sprite.width() as u32 {
                    let c = sprite.get_pixel(x, y);
                    if c.a > 0.0 {
                        atlas.set_pixel(ax + x, ay + y, c);
                    }
                }
            }
        }

        // Generate all sprite images and blit them into the atlas.
        // Layout: row 0 (y=0): ground tiles (16×16 each, 6 tiles = 96px)
        //         row 1 (y=17): ore sprites (16×16 each, 10 tiles = 160px)
        //         row 2 (y=34): machines (16×16 each, 12 tiles = 192px)
        //         row 3 (y=51): belts (16×16 each, 6 tiles = 96px)
        //         row 4 (y=68): items (8×8 each, 10 items = 80px)
        //         row 5 (y=77): enemies (12×12 each)

        // Row 0: Ground (start at x=2 to avoid white pixel area)
        let imgs_ground = [
            make_ground_image(10, 11, 12),  // grass (new green ramp)
            make_ground_image(10, 12, 11),  // grass alt
            make_ground_image(16, 15, 14),  // desert (brown ramp)
            make_forest_image(),           // forest
            make_water_image(12, 13),      // water
            make_water_image(13, 12),      // water alt
        ];
        for (i, img) in imgs_ground.iter().enumerate() {
            blit(&mut atlas_img, img, (i as u32 * 17) + 4, 0);
        }

        // Upload the atlas ONCE.
        let atlas_tex = Texture2D::from_image(&atlas_img);
        atlas_tex.set_filter(FilterMode::Nearest);

        // For backward compatibility, still create individual textures.
        // These will be REMOVED once render.rs is fully migrated to atlas source rects.
        Self {
            tex: atlas_tex,
            tex_size: Vec2::new(512.0, 512.0),

            // Atlas source rects for ground tiles (packed at row 0, starting x=4).
            r_ground_grass: Rect::new(4.0, 0.0, 16.0, 16.0),
            r_ground_grass_alt: Rect::new(21.0, 0.0, 16.0, 16.0),
            r_ground_desert: Rect::new(38.0, 0.0, 16.0, 16.0),
            r_ground_forest: Rect::new(55.0, 0.0, 16.0, 16.0),
            r_ground_water: Rect::new(72.0, 0.0, 16.0, 16.0),
            r_ground_water_alt: Rect::new(89.0, 0.0, 16.0, 16.0),

            // Legacy textures (backward compat).
            ground_grass: make_ground_sprite(10, 11, 12),      // New green ramp
            ground_grass_alt: make_ground_sprite(10, 12, 11),  // Variant
            ground_desert: make_ground_sprite(16, 15, 14),     // Brown/tan ramp
            ground_forest: make_forest_sprite(),
            ground_water: make_water_sprite(12, 13),
            ground_water_alt: make_water_sprite(13, 12),

            // Ore overlays
            ore_iron: img_to_tex(&make_ore_sprite(6, 7)),
            ore_copper: img_to_tex(&make_ore_sprite(7, 28)),
            ore_coal: img_to_tex(&make_ore_sprite(27, 2)),
            ore_stone: img_to_tex(&make_ore_sprite(25, 26)),
            ore_uranium: img_to_tex(&make_ore_sprite(22, 9)),
            ore_tin: img_to_tex(&make_ore_sprite(4, 5)),      // silver-white
            ore_gold: img_to_tex(&make_ore_sprite(30, 16)),   // gold-yellow
            ore_sulfur: img_to_tex(&make_ore_sprite(16, 31)),  // bright yellow
            ore_crystal: img_to_tex(&make_ore_sprite(19, 5)),  // purple-white
            ore_oil: img_to_tex(&make_oil_sprite()),

            // Belts
            belt_yellow: [
                img_to_tex(&make_belt_sprite(17, 16, 0)),
                img_to_tex(&make_belt_sprite(17, 16, 1)),
            ],
            belt_red: [
                img_to_tex(&make_belt_sprite(10, 11, 0)),
                img_to_tex(&make_belt_sprite(10, 11, 1)),
            ],
            belt_blue: [
                img_to_tex(&make_belt_sprite(12, 13, 0)),
                img_to_tex(&make_belt_sprite(12, 13, 1)),
            ],

            // Machines
            miner: img_to_tex(&make_miner_sprite()),
            stone_furnace: img_to_tex(&make_stone_furnace_sprite()),
            steel_furnace: img_to_tex(&make_steel_furnace_sprite()),
            assembler: img_to_tex(&make_assembler_sprite()),
            lab: img_to_tex(&make_lab_sprite()),
            boiler: img_to_tex(&make_boiler_sprite()),
            steam_engine: img_to_tex(&make_steam_engine_sprite()),
            solar_panel: img_to_tex(&make_solar_panel_sprite()),
            chest: img_to_tex(&make_chest_sprite()),

            // Military
            gun_turret: img_to_tex(&make_gun_turret_sprite()),
            wall: img_to_tex(&make_wall_sprite()),

            // Inserter
            inserter: img_to_tex(&make_inserter_sprite()),

            // Items
            item_iron_ore: img_to_tex(&make_item_sprite(6, 7)),
            item_copper_ore: img_to_tex(&make_item_sprite(7, 28)),
            item_coal: img_to_tex(&make_item_sprite(27, 2)),
            item_stone: img_to_tex(&make_item_sprite(25, 26)),
            item_iron_plate: img_to_tex(&make_plate_item_sprite(3, 4)),
            item_copper_plate: img_to_tex(&make_plate_item_sprite(28, 15)),
            item_gear: img_to_tex(&make_gear_item_sprite()),
            item_wire: img_to_tex(&make_wire_item_sprite()),
            item_green_circuit: img_to_tex(&make_circuit_item_sprite()),
            item_science_red: img_to_tex(&make_flask_item_sprite(10, 11)),

            // Enemies
            enemy_small_biter: img_to_tex(&make_enemy_sprite(23, 24)),
        }
    }
}

// ===========================================================================
// Internal sprite generation helpers
// ===========================================================================

/// Converts an Image to a Texture2D with nearest-neighbor filtering.
fn img_to_tex(img: &Image) -> Texture2D {
    let tex = Texture2D::from_image(img);
    tex.set_filter(FilterMode::Nearest);
    tex
}

/// Creates an [`Image`] from a 2D grid of palette indices at the given size.
fn make_image(pixels: &[&[u8]], size: u16) -> Image {
    let mut image = Image::gen_image_color(size, size, Color::new(0.0, 0.0, 0.0, 0.0));

    for (y, row) in pixels.iter().enumerate() {
        for (x, &idx) in row.iter().enumerate() {
            if idx == 0 || x >= size as usize || y >= size as usize {
                continue;
            }
            let (r, g, b, a) = PALETTE[idx as usize];
            image.set_pixel(
                x as u32,
                y as u32,
                Color::new(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                ),
            );
        }
    }

    image
}

/// Creates a [`Texture2D`] from a 2D grid of palette indices.
/// Wrapper around make_image for backward compatibility.
fn make_texture(pixels: &[&[u8]], size: u16) -> Texture2D {
    let image = make_image(pixels, size);
    let tex = Texture2D::from_image(&image);
    tex.set_filter(FilterMode::Nearest);
    tex
}

/// Creates a high-quality ground tile with Bayer dithering, subtle texture variation,
/// and occasional tiny detail pixels (flowers/pebbles).
fn make_ground_image(base: u8, highlight: u8, alt: u8) -> Image {
    let mut rows: Vec<Vec<u8>> = Vec::new();
    // Bayer 4×4 ordered dithering matrix for smooth gradient blending.
    let bayer: [[u8; 4]; 4] = [
        [0, 8, 2, 10],
        [12, 4, 14, 6],
        [3, 11, 1, 9],
        [15, 7, 13, 5],
    ];

    for y in 0..16u8 {
        let mut row = Vec::new();
        for x in 0..16u8 {
            let threshold = bayer[(y % 4) as usize][(x % 4) as usize];
            // Hash for pseudo-random variation.
            let hash = ((x as u32).wrapping_mul(2654435761) ^ (y as u32).wrapping_mul(340573321)) % 100;

            let pixel = if hash < 3 {
                // Rare bright detail (tiny flower or pebble) — 3% chance.
                highlight
            } else if hash < 8 {
                // Occasional alternate shade — 5% chance.
                alt
            } else if threshold > 11 {
                // Dithered transition to highlight (top 25% of Bayer).
                highlight
            } else if threshold < 4 {
                // Dithered transition to alt/shadow (bottom 25% of Bayer).
                alt
            } else {
                // Base color (majority of pixels).
                base
            };
            row.push(pixel);
        }
        rows.push(row);
    }
    let row_refs: Vec<&[u8]> = rows.iter().map(|r| r.as_slice()).collect();
    make_image(&row_refs, 16)
}

/// Returns forest tile as Image — deep green canopy with layered depth.
/// Uses green ramp (9-12) with dark shadows (1-2) for dense forest feel.
fn make_forest_image() -> Image {
    let mut rows: Vec<Vec<u8>> = Vec::new();
    for y in 0..16u8 {
        let mut row = Vec::new();
        for x in 0..16u8 {
            let hash = ((x as u32).wrapping_mul(7919) ^ (y as u32).wrapping_mul(104729)) % 100;
            let pixel = if hash < 5 {
                12  // Bright green (dappled sunlight through canopy)
            } else if hash < 15 {
                11  // Mid green (leaves)
            } else if hash < 35 {
                10  // Dark green (canopy shadow)
            } else if hash < 55 {
                9   // Deepest green (dense shadow)
            } else if hash < 60 {
                14  // Brown (tree trunk peek-through)
            } else if hash < 65 {
                2   // Dark shadow (depth)
            } else {
                1   // Deepest shadow (makes forest look VERY dark and dense)
            };
            row.push(pixel);
        }
        rows.push(row);
    }
    let row_refs: Vec<&[u8]> = rows.iter().map(|r| r.as_slice()).collect();
    make_image(&row_refs, 16)
}

/// Returns water tile as Image — soft blue with wave patterns and sparkle.
/// Uses blue ramp (25-28) for depth.
fn make_water_image(base: u8, highlight: u8) -> Image {
    let mut rows: Vec<Vec<u8>> = Vec::new();
    for y in 0..16u8 {
        let mut row = Vec::new();
        for x in 0..16u8 {
            // Wave pattern — sinusoidal ripples.
            let wave_phase = (x as f32 * 0.8 + y as f32 * 0.4).sin();
            let hash = ((x as u32).wrapping_mul(48271) ^ (y as u32).wrapping_mul(12345)) % 100;

            let pixel = if hash < 3 {
                8  // Rare white sparkle (sunlight on water)
            } else if wave_phase > 0.7 {
                highlight  // Wave crest (lighter)
            } else if wave_phase < -0.5 {
                25  // Deep blue (wave trough)
            } else if hash < 20 {
                26  // Mid blue variation
            } else {
                base  // Base water color
            };
            row.push(pixel);
        }
        rows.push(row);
    }
    let row_refs: Vec<&[u8]> = rows.iter().map(|r| r.as_slice()).collect();
    make_image(&row_refs, 16)
}

fn make_ground_sprite(base: u8, highlight: u8, alt: u8) -> Texture2D {
    let mut rows: Vec<Vec<u8>> = Vec::new();
    for y in 0..16u8 {
        let mut row = Vec::new();
        for x in 0..16u8 {
            let hash = ((x as u16).wrapping_mul(7) + (y as u16).wrapping_mul(13)) % 17;
            let pixel = if hash == 0 {
                highlight
            } else if hash == 3 {
                alt
            } else {
                base
            };
            row.push(pixel);
        }
        rows.push(row);
    }
    let row_refs: Vec<&[u8]> = rows.iter().map(|r| r.as_slice()).collect();
    make_texture(&row_refs, 16)
}

/// Creates a forest ground sprite — visually distinct dark green with tree canopy shapes.
fn make_forest_sprite() -> Texture2D {
    let mut rows: Vec<Vec<u8>> = Vec::new();
    for y in 0..16u8 {
        let mut row = Vec::new();
        for x in 0..16u8 {
            let hash = ((x as u16).wrapping_mul(11) + (y as u16).wrapping_mul(7)) % 23;
            let pixel = if hash == 0 {
                6 // tree trunk (brown)
            } else if hash < 3 {
                8 // darkest green (canopy shadow)
            } else if hash < 7 {
                1 // very dark (deep forest shadow — makes forest obviously dark)
            } else if hash < 10 {
                8 // dark green
            } else {
                2 // shadow (makes it look dense and dark)
            };
            row.push(pixel);
        }
        rows.push(row);
    }
    let row_refs: Vec<&[u8]> = rows.iter().map(|r| r.as_slice()).collect();
    make_texture(&row_refs, 16)
}

/// Creates a water tile sprite.
fn make_water_sprite(base: u8, highlight: u8) -> Texture2D {
    let mut rows: Vec<Vec<u8>> = Vec::new();
    for y in 0..16u8 {
        let mut row = Vec::new();
        for x in 0..16u8 {
            let wave = ((x as u16 + y as u16 * 3) % 8 == 0) as u8;
            let pixel = if wave == 1 { highlight } else { base };
            row.push(pixel);
        }
        rows.push(row);
    }
    let row_refs: Vec<&[u8]> = rows.iter().map(|r| r.as_slice()).collect();
    make_texture(&row_refs, 16)
}

/// Creates an ore deposit sprite — a large rocky formation with colored veins.
///
/// The rock has a 3D appearance with highlights on top-left and shadows on
/// bottom-right, with ore veins (colored specks) showing through the stone.
/// Creates ore deposit sprite — crystalline rock with colored veins.
/// Uses stone ramp (45-48) for rock body, ore color (dark/light) for veins,
/// ambient occlusion at base, specular highlight at top-left.
fn make_ore_sprite(dark: u8, light: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1,48, 8,48,47,47,46, 1, 0, 0, 0, 0],
        &[0, 0, 0, 1,48, 8,light,48,47,light,47,46, 1, 0, 0, 0],
        &[0, 0, 1,48, 8,48,47,light,47,46,dark,46,45, 1, 0, 0],
        &[0, 1,48,48,light,48,47,47,46,dark,46,45,45, 1, 0, 0],
        &[0, 1,48,47,47,light,47,46,46,46,dark,45,45, 2, 1, 0],
        &[1,47,48,47,47,46,light,46,45,dark,45,45, 2, 2, 1, 0],
        &[1,47,47,46,46,46,dark,45,45,45, 2,dark, 2, 2, 2, 1],
        &[1,47,46,46,dark,45,45, 2,dark, 2, 2, 2, 2, 2, 2, 1],
        &[1,46,46,45,45,dark, 2, 2, 2, 2, 2, 2, 1, 2, 1, 0],
        &[0, 1,45,45, 2, 2,dark, 2, 2, 2, 2, 1, 1, 1, 0, 0],
        &[0, 1, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0],
        &[0, 0, 1, 1, 2, 2, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Creates an oil well indicator sprite.
fn make_oil_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,1,27,27,1,0,0,0,0,0,0],
        &[0,0,0,0,0,1,27,27,27,27,1,0,0,0,0,0],
        &[0,0,0,0,0,1,27,2,2,27,1,0,0,0,0,0],
        &[0,0,0,0,0,0,1,27,27,1,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,1,27,27,1,0,0,0,0,0,0],
        &[0,0,0,0,0,1,27,27,27,27,1,0,0,0,0,0],
        &[0,0,0,0,1,27,27,2,2,27,27,1,0,0,0,0],
        &[0,0,0,0,1,27,27,27,27,27,27,1,0,0,0,0],
        &[0,0,0,0,0,1,1,1,1,1,1,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    ];
    make_image(pixels, 16)
}

/// Creates a belt sprite. `frame` shifts the chevron arrows for animation.
/// Belt sprite with smooth metallic track and animated chevron arrows.
/// Uses steel ramp (53-56) for track body, arrow_color for directional indicators.
/// Rounded edge with shadow on right, highlight on left.
fn make_belt_sprite(base_color: u8, arrow_color: u8, frame: u8) -> Image {
    let mut rows: Vec<Vec<u8>> = Vec::new();
    for y in 0..16u8 {
        let mut row = Vec::new();
        for x in 0..16u8 {
            // Rounded belt edges with gradient.
            if x == 0 || x == 15 {
                row.push(1); // Outer outline (darkest)
            } else if x == 1 {
                row.push(55); // Left highlight edge (steel light)
            } else if x == 14 {
                row.push(53); // Right shadow edge (steel dark)
            } else if x == 2 {
                row.push(56); // Inner left highlight
            } else if x == 13 {
                row.push(54); // Inner right shadow
            } else {
                // Belt surface with smooth chevrons.
                let shifted_y = (y + frame * 3) % 6;
                let center_dist = (x as i8 - 7).unsigned_abs();
                let is_chevron = shifted_y < 2 && center_dist <= (2u8.saturating_sub(shifted_y));
                let is_chevron_edge = shifted_y == 2 && center_dist <= 1;

                if is_chevron {
                    row.push(arrow_color);
                } else if is_chevron_edge {
                    // Chevron anti-alias edge (dimmer version of arrow)
                    row.push(base_color);
                } else {
                    // Belt surface — subtle gradient from left (lighter) to right (darker).
                    if x < 7 {
                        row.push(55); // Lighter left side
                    } else {
                        row.push(54); // Darker right side
                    }
                }
            }
        }
        rows.push(row);
    }
    let row_refs: Vec<&[u8]> = rows.iter().map(|r| r.as_slice()).collect();
    make_image(&row_refs, 16)
}

// ===========================================================================
// Hand-crafted 16×16 machine sprites
// Light source: top-left. Outlines use selective outlining (selout).
// ===========================================================================

/// Creates the miner sprite — orange body with pickaxe, gear detail, rocky base.
fn make_miner_sprite() -> Image {
    // Production quality miner: warm coral body (palette 17-20), metal details (53-56),
    // proper top-left lighting, ambient occlusion at base, rounded shape.
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 1,20,20,19,19,19,18, 1, 0, 0, 0],
        &[0, 0, 0, 1,20,20, 8,20,19,19, 8,18,17, 1, 0, 0],
        &[0, 0, 1,20,20, 8,55,56,19,55,56,18,17,17, 1, 0],
        &[0, 1,20,20,19,55, 4, 5,55, 4, 5,54,17,17, 2, 1],
        &[0, 1,20,19,19,55,54,54,54,54,54,54,18,17, 2, 1],
        &[1,19,19,19,55,54, 4, 4, 4, 4, 4,54,18,17, 2, 1],
        &[1,19,19,18,55,54, 4, 3, 3, 4, 3,54,18, 2, 2, 1],
        &[1,19,18,18,55, 3, 3, 3, 3, 3, 3,53,17, 2, 2, 1],
        &[1,18,18,18,54, 3, 3,53,53, 3, 3,53,17, 2, 1, 0],
        &[0, 1,18,17,53,53,53,53,53,53,53, 2, 2, 1, 0, 0],
        &[0, 1,17,17, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0],
        &[0, 0, 1, 1,46,47,46, 2,46,47,46, 1, 0, 0, 0, 0],
        &[0, 0, 0,45,46,47,46,45,46,47,46,45, 0, 0, 0, 0],
        &[0, 0, 0, 0,45,46,45,45,45,46,45, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Creates the stone furnace sprite — warm brick oven with amber fire glow.
/// Uses stone ramp (45-48) for bricks, fire ramp (61, 23, 39) for glow.
fn make_stone_furnace_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
        &[0, 1,47,48,48,48,48,48,48,48,48,48,48,46, 1, 0],
        &[1,47,48,48, 8,48,48,48,48,48, 8,48,47,46,46, 1],
        &[1,47,48, 1, 1, 1, 1, 1, 1, 1, 1, 1,47,46,46, 1],
        &[1,47,48, 1,61,39,39,39,39,39,61, 1,47,46, 2, 1],
        &[1,47,47, 1,61,39,40,39,40,39,61, 1,46,45, 2, 1],
        &[1,46,47, 1,23,61,39,61,39,61,23, 1,46,45, 2, 1],
        &[1,46,47, 1,22,23,61,23,61,23,22, 1,46,45, 2, 1],
        &[1,46,46, 1, 1, 1, 1, 1, 1, 1, 1, 1,45,45, 2, 1],
        &[1,46,46,46,47,46,46,46,46,46,46,45,45, 2, 2, 1],
        &[1,46,46,45,46,46,45, 2,46,45, 2, 2, 2, 2, 2, 1],
        &[1,45,45, 2,45, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0],
        &[0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0],
        &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Steel furnace — polished metal body (53-56) with bright fire (61, 39-40).
/// Shinier and more angular than stone furnace.
fn make_steel_furnace_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0,53,53,53,53,53,53,53,53,53,53,53,53, 0, 0],
        &[0,53,56,56, 8,56,56,56,56,56, 8,56,55,54,53, 0],
        &[53,56,56, 8,56,55,55,55,55,55,56, 8,55,54,54,53],
        &[53,56,55,53,53,53,53,53,53,53,53,53,55,54,54,53],
        &[53,56,55,53,61,40,40,40,40,40,61,53,55,54, 2,53],
        &[53,55,55,53,61,40, 8,40, 8,40,61,53,54,54, 2,53],
        &[53,55,54,53,23,61,40,61,40,61,23,53,54,53, 2,53],
        &[53,55,54,53,21,23,61,23,61,23,21,53,54,53, 2,53],
        &[53,54,54,53,53,53,53,53,53,53,53,53,53,53, 2,53],
        &[53,54,54,54,55,54,54,54,54,54,54,53,53, 2, 2,53],
        &[53,54,54,53,54,53,53, 2,53,53, 2, 2, 2, 2, 2,53],
        &[53,53,53, 2,53, 2, 2, 2, 2, 2, 2, 2, 2, 2,53, 0],
        &[0,53, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,53, 0, 0],
        &[0, 0,53,53,53,53,53,53,53,53,53,53,53, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Creates the assembler sprite — periwinkle-blue mechanical box with gear detail.
/// Uses blue ramp (25-28) with steel accents (53-56).
fn make_assembler_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0],
        &[0, 0, 1,28,28,28, 8,28,28, 8,28,27,27, 1, 0, 0],
        &[0, 1,28,28, 8,28,56,55,28,56,55, 8,27,26, 1, 0],
        &[1,28,28, 8,55,56,55,26,26,55,56,55,27,26,26, 1],
        &[1,28, 8,55, 4, 5,54,26,26,54, 5, 4,26,26,25, 1],
        &[1,28,27,55, 5, 4, 5, 5, 5, 5, 4, 5,26,25,25, 1],
        &[1,27,27,55, 4, 5, 7, 7, 7, 7, 5, 4,26,25,25, 1],
        &[1,27,27,54, 5, 5, 7, 8, 8, 7, 5, 5,25,25, 2, 1],
        &[1,27,26,54, 5, 5, 7, 8, 8, 7, 5, 5,25,25, 2, 1],
        &[1,26,26,54, 4, 5, 7, 7, 7, 7, 5, 4,25, 2, 2, 1],
        &[1,26,26,53, 5, 4, 5, 5, 5, 5, 4,53, 2, 2, 2, 1],
        &[0, 1,26,53,53,53,53,26,26,53,53,53, 2, 1, 0, 0],
        &[0, 0, 1,25, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0],
        &[0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Creates the lab sprite — purple body with flask/beaker detail.
/// Lab sprite — purple body (29-32) with green flask detail (57-60).
/// Rounded shape, indicator "eyes", bubbling flask center.
fn make_lab_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0,29,29,29,29,29,29,29,29, 0, 0, 0, 0],
        &[0, 0,29,32,32,32, 8,32,32, 8,32,31,30,29, 0, 0],
        &[0,29,32,32, 8,32, 7, 7,32,32,32, 8,31,30, 0, 0],
        &[29,32,32, 8,32,32, 1, 1,32,32,31,31,30,30,29, 0],
        &[29,32, 8,32,32, 1,59,60, 1,31,31,30,30,29,29, 0],
        &[29,31,32,31, 1,59,58,59,60, 1,30,30,29,29, 2,29],
        &[29,31,31,31, 1,58,59,58,59, 1,30,29,29, 2, 2,29],
        &[29,31,31,30, 1, 1,58,59, 1, 1,30,29, 2, 2, 2,29],
        &[29,30,30,30,30, 1, 1, 1, 1,30,29,29, 2, 2, 2,29],
        &[29,30,30,30,30,30,30,30,30,29,29, 2, 2, 2,29, 0],
        &[0,29,30,29,29, 2,29, 2,29, 2, 2, 2, 2,29, 0, 0],
        &[0, 0,29, 2, 2, 2, 2, 2, 2, 2, 2, 2,29, 0, 0, 0],
        &[0, 0, 0,29,29,29,29,29,29,29,29,29, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Boiler — teal ramp (33-36) with chimney pipes and fire window (61, 39).
fn make_boiler_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0,33,33, 0, 0,33,33, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0,34,54,33,33,54,34, 0, 0, 0, 0, 0],
        &[0, 0,33,33,33,33,33,33,33,33,33,33,33,33, 0, 0],
        &[0,33,36,36, 8,36,35,35,35,35, 8,36,35,34,33, 0],
        &[33,36,36, 8,35,35,34,34,34,34,35,35, 8,34,34,33],
        &[33,36, 8,35,33,33,33,33,33,33,33,33,34,34, 2,33],
        &[33,35,35,35,33,61,39,61,39,61,39,33,34,33, 2,33],
        &[33,35,35,34,33,23,61,23,61,23,61,33,34,33, 2,33],
        &[33,34,34,34,33,21,23,21,23,21,23,33,33,33, 2,33],
        &[33,34,34,34,33,33,33,33,33,33,33,33,33, 2, 2,33],
        &[33,34,34,33,34,34,33, 2,34,33, 2, 2, 2, 2,33, 0],
        &[0,33,33, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,33, 0, 0],
        &[0, 0,33,33,33,33,33,33,33,33,33,33,33, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Steam engine — teal (33-36) with steel piston detail (53-56).
fn make_steam_engine_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0,33,33,33,33,33,33,33,33,33,33,33,33, 0, 0],
        &[0,33,36,36, 8,36,35,35,35,35, 8,36,35,34,33, 0],
        &[33,36, 8,35,56,55,56,35,35,56,55,56,34,34, 2,33],
        &[33,35,35,56, 5, 8,55,56,56,55, 8, 5,34,33, 2,33],
        &[33,35,35,55, 8,55,54,54,54,54,55, 8,34,33, 2,33],
        &[33,35,34,55,55,54,53,55,55,53,54,55,33,33, 2,33],
        &[33,34,34,54,54,53,54, 8, 8,54,53,54,33, 2, 2,33],
        &[33,34,34,54,53,54,54, 8, 8,54,54,53,33, 2, 2,33],
        &[33,34,34,55,54,53,54,54,54,54,53,54,33, 2, 2,33],
        &[33,34,33,55,55,54,53,53,53,53,54,55, 2, 2,33, 0],
        &[0,33,33,54, 2,55,54,54,54,54,55, 2, 2,33, 0, 0],
        &[0, 0,33, 2, 2, 2, 2, 2, 2, 2, 2, 2,33, 0, 0, 0],
        &[0, 0, 0,33,33,33,33,33,33,33,33,33, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Solar panel — blue (25-28) 4-cell grid with yellow/white reflection (39-40, 8).
fn make_solar_panel_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[53,53,53,53,53,53,53,53,53,53,53,53,53,53,53,53],
        &[53,56,56,56,56,56,56,53,53,56,56,56,56,56,56,53],
        &[53,56,27,27, 8,27,27,53,53,27,27, 8,27,27,54,53],
        &[53,56,27,40,40,27,27,53,53,27,40,40,27,27,54,53],
        &[53,56,27,39,27,27,26,53,53,27,39,27,27,26,54,53],
        &[53,55,27,27,27,26,26,53,53,27,27,27,26,26,54,53],
        &[53,55,27,27,26,26,25,53,53,27,27,26,26,25,54,53],
        &[53,53,53,53,53,53,53,55,54,53,53,53,53,53,53,53],
        &[53,53,53,53,53,53,53,55,54,53,53,53,53,53,53,53],
        &[53,55,27,27, 8,27,27,53,53,27,27, 8,27,27,54,53],
        &[53,55,27,40,40,27,27,53,53,27,40,40,27,27,54,53],
        &[53,54,27,39,27,27,26,53,53,27,39,27,27,26,54,53],
        &[53,54,27,27,27,26,25,53,53,27,27,27,26,25,54,53],
        &[53,54,27,27,26,25,25,53,53,27,27,26,25,25,54,53],
        &[53,54,54,54,54,54,54,53,53,54,54,54,54,54,54,53],
        &[53,53,53,53,53,53,53,53,53,53,53,53,53,53,53,53],
    ];
    make_image(pixels, 16)
}

/// Storage chest — brown (13-16) wooden crate with gold latch (49-52).
fn make_chest_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0,13,13,13,13,13,13,13,13,13,13,13,13, 0, 0],
        &[0,13,16,16,16,16,16,16,16,16,16,16,16,15,13, 0],
        &[13,16,16,15, 8,16,16,16,16,16, 8,15,15,14,13, 0],
        &[13,16,15,15,16,15,15,15,15,15,16,15,14,14,13, 0],
        &[13,16,15,15,15,15,15,15,15,15,15,14,14,14,13, 0],
        &[13,15,15,15,15,15,51,52,15,15,15,14,14,13,13, 0],
        &[13,15,15,15,15,15,50,51,15,15,14,14,14,13,13, 0],
        &[13,15,14,15,14,14,14,14,14,14,14,14,13,13,13, 0],
        &[13,14,14,14,14,14,14,14,14,14,14,13,13,13, 2,13],
        &[13,14,14,13,14,13,13,13,13,13,13,13, 2, 2,13, 0],
        &[0,13,13,13,13,13,13,13,13,13,13,13, 2,13, 0, 0],
        &[0, 0,13,13,13,13,13,13,13,13,13,13,13, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Gun turret — steel body (53-56) with dark barrel (1-2).
fn make_gun_turret_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0,53,55,55,53, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0,53,54,54,53, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0,53,54,54,53, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0,53,53,53,53,54,54,53,53,53,53, 0, 0, 0],
        &[0, 0,53,56,56, 8,56,56,56,56, 8,56,55,53, 0, 0],
        &[0,53,56,56,55,55,54,54,54,54,55,55,54,54,53, 0],
        &[53,56,55,55,54,54,53,53,53,53,54,54,54,53, 2,53],
        &[53,55,55,54,54,53,53, 2, 2,53,53,54,53,53, 2,53],
        &[53,55,54,54,53,53, 2, 2, 2, 2,53,53,53, 2, 2,53],
        &[0,53,54,53,53, 2, 2, 2, 2, 2, 2,53, 2, 2,53, 0],
        &[0, 0,53,53, 2, 2, 2, 2, 2, 2, 2, 2, 2,53, 0, 0],
        &[0, 0, 0,53,53,53,53,53,53,53,53,53,53, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Inserter — steel arm (53-56) with yellow gripper (39).
fn make_inserter_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0,37,37,37, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0,37,39,40,39,37, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0,53,55,53, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0,53,54,53, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0,53,54,53, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0,53,54,53, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0,53,54,53, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0,53,53,53, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0,53,53,53,53,53,53,53, 0, 0, 0, 0, 0],
        &[0, 0, 0,53,56,56, 8,56,56,56,55,53, 0, 0, 0, 0],
        &[0, 0, 0,53,55,55,56,55,55,54,54,53, 0, 0, 0, 0],
        &[0, 0, 0,53,54,54,55,54,54,53,53,53, 0, 0, 0, 0],
        &[0, 0, 0, 0,53,53,53,53,53,53,53, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0,53,53,53,53,53, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Wall — stone ramp (45-48) with mortar lines (1).
fn make_wall_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[45,45,45,45,45,45,45,45,45,45,45,45,45,45,45,45],
        &[45,47,47,47,47,47,47,48,45,47,47,47,47,47,47,45],
        &[45,47,48,48,48,48,47,48,45,47,48,48,48,48,47,45],
        &[45,47,48,48,48,48,47,48,45,47,48,48,48,48,47,45],
        &[45,47,47,47,47,47,47,48,45,47,47,47,47,47,47,45],
        &[45,45,45,45,45,45,45,45,45,45,45,45,45,45,45,45],
        &[45,47,47,47,45,47,47,47,47,47,47,45,47,47,47,45],
        &[45,47,48,47,45,47,48,48,48,48,47,45,47,48,47,45],
        &[45,47,48,47,45,47,48,48,48,48,47,45,47,48,47,45],
        &[45,47,47,47,45,47,47,47,47,47,47,45,47,47,47,45],
        &[45,45,45,45,45,45,45,45,45,45,45,45,45,45,45,45],
        &[45,47,47,47,47,47,47,48,45,47,47,47,47,47,47,45],
        &[45,47,48,48,48,48,47,48,45,47,48,48,48,48,47,45],
        &[45,47,48,48,48,48,47,48,45,47,48,48,48,48,47,45],
        &[45,46,47,47,47,47,46,47,45,46,47,47,47,47,46,45],
        &[45,45,45,45,45,45,45,45,45,45,45,45,45,45,45,45],
    ];
    make_image(pixels, 16)
}

// --- Item sprites (8×8) ---

/// Creates a small item ore sprite.
fn make_item_sprite(dark: u8, light: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 1, 1, 1, 0, 0, 0],
        &[0, 1, dark, dark, light, 1, 0, 0],
        &[0, 1, dark, light, light, 1, 0, 0],
        &[0, 1, dark, dark, light, 1, 0, 0],
        &[0, 0, 1, 1, 1, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Creates a flat plate item sprite.
fn make_plate_item_sprite(dark: u8, light: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 1, 1, 1, 1, 1, 0, 0],
        &[0, 1, dark, dark, light, 1, 0, 0],
        &[0, 1, dark, light, light, 1, 0, 0],
        &[0, 1, dark, dark, light, 1, 0, 0],
        &[0, 1, 1, 1, 1, 1, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Creates a gear item sprite.
fn make_gear_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 3, 3, 0, 0, 0, 0],
        &[0, 3, 4, 4, 3, 0, 0, 0],
        &[3, 4, 1, 1, 4, 3, 0, 0],
        &[3, 4, 1, 1, 4, 3, 0, 0],
        &[0, 3, 4, 4, 3, 0, 0, 0],
        &[0, 0, 3, 3, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Creates a wire item sprite.
fn make_wire_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 28, 0, 0, 0, 0, 0, 0],
        &[0, 0, 28, 0, 0, 0, 0, 0],
        &[0, 0, 0, 28, 28, 0, 0, 0],
        &[0, 0, 0, 0, 0, 28, 0, 0],
        &[0, 0, 0, 0, 0, 0, 28, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Creates a circuit item sprite.
fn make_circuit_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 1, 1, 1, 1, 1, 0, 0],
        &[0, 1, 22, 8, 22, 1, 0, 0],
        &[0, 1, 8, 22, 8, 1, 0, 0],
        &[0, 1, 22, 8, 22, 1, 0, 0],
        &[0, 1, 1, 1, 1, 1, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Creates a science flask item sprite.
fn make_flask_item_sprite(dark: u8, light: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 5, 5, 0, 0, 0, 0],
        &[0, 0, 5, 5, 0, 0, 0, 0],
        &[0, 1, dark, dark, 1, 0, 0, 0],
        &[1, dark, light, light, dark, 1, 0, 0],
        &[1, dark, light, light, dark, 1, 0, 0],
        &[0, 1, 1, 1, 1, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

// --- Enemy sprites (12×12) ---

/// Creates an enemy biter sprite.
fn make_enemy_sprite(body: u8, shadow: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,1,1,1,1,0,0,0,0],
        &[0,0,0,1,body,body,body,body,1,0,0,0],
        &[0,0,1,body,body,5,5,body,body,1,0,0],
        &[0,1,shadow,body,body,body,body,body,body,shadow,1,0],
        &[0,1,shadow,body,body,body,body,body,body,shadow,1,0],
        &[0,0,1,shadow,body,body,body,body,shadow,1,0,0],
        &[0,0,0,1,shadow,body,body,shadow,1,0,0,0],
        &[0,0,1,0,1,shadow,shadow,1,0,1,0,0],
        &[0,1,0,0,0,1,1,0,0,0,1,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0],
    ];
    make_image(pixels, 12)
}
