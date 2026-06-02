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
    /// All rendering uses this texture with source Rects for zero texture switches.
    pub tex: Texture2D,
    /// Atlas dimensions for UV normalization.
    pub tex_size: Vec2,

    // --- Atlas source rects (position of each sprite within `tex`) ---
    // Ground tiles (16×16)
    pub r_ground_grass: Rect,
    pub r_ground_grass_alt: Rect,
    pub r_ground_desert: Rect,
    pub r_ground_forest: Rect,
    pub r_ground_water: Rect,
    pub r_ground_water_alt: Rect,

    // Ore deposits (16×16)
    pub r_ore_iron: Rect,
    pub r_ore_copper: Rect,
    pub r_ore_coal: Rect,
    pub r_ore_stone: Rect,
    pub r_ore_uranium: Rect,
    pub r_ore_tin: Rect,
    pub r_ore_gold: Rect,
    pub r_ore_sulfur: Rect,
    pub r_ore_crystal: Rect,
    pub r_ore_oil: Rect,

    // Belts (16×16, 2 animation frames each)
    pub r_belt_yellow: [Rect; 2],
    pub r_belt_red: [Rect; 2],
    pub r_belt_blue: [Rect; 2],
    pub r_belt_corner_left_yellow: [Rect; 2],
    pub r_belt_corner_left_red: [Rect; 2],
    pub r_belt_corner_left_blue: [Rect; 2],
    pub r_belt_corner_right_yellow: [Rect; 2],
    pub r_belt_corner_right_red: [Rect; 2],
    pub r_belt_corner_right_blue: [Rect; 2],

    // Machines & structures (16×16, animated ones have 2 frames)
    pub r_miner: [Rect; 2],
    pub r_stone_furnace: [Rect; 2],
    pub r_steel_furnace: [Rect; 2],
    pub r_assembler: [Rect; 2],
    pub r_lab: [Rect; 2],
    pub r_boiler: Rect,
    pub r_steam_engine: Rect,
    pub r_solar_panel: Rect,
    pub r_chest: Rect,
    pub r_gun_turret: Rect,
    pub r_laser_turret: Rect,
    pub r_wall: Rect,
    pub r_inserter: Rect,
    pub r_underground_belt: Rect,
    pub r_splitter: Rect,
    pub r_chemical_plant: Rect,
    pub r_water_pump: Rect,
    pub r_oil_refinery: Rect,
    pub r_nuclear_reactor: Rect,
    pub r_accumulator: Rect,
    pub r_radar: Rect,
    pub r_pipe: Rect,
    pub r_roboport: Rect,
    pub r_rail: Rect,
    pub r_train_stop: Rect,
    pub r_rocket_silo: Rect,
    pub r_beacon: Rect,
    pub r_electric_furnace: Rect,
    pub r_pump_jack: Rect,

    // FORGE avatar (24×24, 2 expression frames: happy, blink)
    pub r_forge_avatar: [Rect; 2],

    // Items (8×8)
    pub r_item_iron_ore: Rect,
    pub r_item_copper_ore: Rect,
    pub r_item_coal: Rect,
    pub r_item_stone: Rect,
    pub r_item_iron_plate: Rect,
    pub r_item_copper_plate: Rect,
    pub r_item_gear: Rect,
    pub r_item_wire: Rect,
    pub r_item_green_circuit: Rect,
    pub r_item_science_red: Rect,
    pub r_item_steel_plate: Rect,
    pub r_item_stone_brick: Rect,
    pub r_item_pipe: Rect,
    pub r_item_red_circuit: Rect,
    pub r_item_blue_circuit: Rect,
    pub r_item_science_green: Rect,
    pub r_item_science_blue: Rect,
    pub r_item_science_purple: Rect,
    pub r_item_sulfur: Rect,
    pub r_item_plastic: Rect,
    pub r_item_battery: Rect,
    pub r_item_ammo: Rect,
    pub r_item_grenade: Rect,
    pub r_item_engine: Rect,
    pub r_item_rocket_part: Rect,
    pub r_item_rocket_fuel: Rect,
    pub r_item_inserter: Rect,
    pub r_item_iron_stick: Rect,
    pub r_item_speed_module: Rect,
    pub r_item_science_yellow: Rect,
    pub r_item_uranium_ore: Rect,
    pub r_item_rail: Rect,
    pub r_item_concrete: Rect,
    pub r_item_solar_panel: Rect,
    pub r_item_accumulator: Rect,
    pub r_item_robot_frame: Rect,
    pub r_item_uranium_235: Rect,
    pub r_item_uranium_238: Rect,
    pub r_item_nuclear_fuel: Rect,
    pub r_item_low_density: Rect,

    // Enemies & special (variable sizes, 2-frame walk cycle)
    pub r_enemy_small_biter: [Rect; 2],
    pub r_enemy_big_biter: [Rect; 2],
    pub r_enemy_spitter: [Rect; 2],
    pub r_crashed_ship: Rect,
}

impl SpriteAtlas {
    /// Generates all sprites and packs them into a single 512×512 texture atlas.
    ///
    /// All rendering uses `atlas.tex` with source `Rect`s — since every draw
    /// references the same GPU texture, macroquad auto-batches all world rendering
    /// into ~1 draw call. Zero texture switches.
    pub fn generate() -> Self {
        let mut atlas_img = Image::gen_image_color(512, 512, Color::new(0.0, 0.0, 0.0, 0.0));

        // Pack white pixel at (0,0) for solid rect rendering via batcher.
        atlas_img.set_pixel(0, 0, WHITE);
        atlas_img.set_pixel(1, 0, WHITE);
        atlas_img.set_pixel(0, 1, WHITE);
        atlas_img.set_pixel(1, 1, WHITE);

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

        /// Blit a sprite and return its source Rect.
        fn pack(atlas: &mut Image, sprite: &Image, ax: u32, ay: u32) -> Rect {
            blit(atlas, sprite, ax, ay);
            Rect::new(ax as f32, ay as f32, sprite.width() as f32, sprite.height() as f32)
        }

        // Atlas layout (512×512, 1px padding between rows):
        //   Row 0 (y=0):   ground tiles, 16×16, 6 sprites
        //   Row 1 (y=17):  ore deposits, 16×16, 10 sprites
        //   Row 2 (y=34):  machines/structures, 16×16, 12 sprites
        //   Row 3 (y=51):  belts straight, 16×16, 6 sprites (3 colors × 2 frames)
        //   Row 4 (y=68):  belt corners left, 16×16, 6 sprites
        //   Row 5 (y=85):  belt corners right, 16×16, 6 sprites
        //   Row 6 (y=102): items, 8×8, 10 sprites
        //   Row 7 (y=111): enemy 12×12 + crashed ship 80×48

        let s = 17u32; // stride for 16×16 sprites (16 + 1px padding)

        // Row 0: Ground (start at x=4 to avoid white pixel area)
        let r_ground_grass     = pack(&mut atlas_img, &make_ground_image(10, 11, 12), 4, 0);
        let r_ground_grass_alt = pack(&mut atlas_img, &make_ground_image(10, 12, 11), 4 + s, 0);
        let r_ground_desert    = pack(&mut atlas_img, &make_ground_image(16, 15, 14), 4 + s*2, 0);
        let r_ground_forest    = pack(&mut atlas_img, &make_forest_image(), 4 + s*3, 0);
        let r_ground_water     = pack(&mut atlas_img, &make_water_image(25, 27), 4 + s*4, 0);
        let r_ground_water_alt = pack(&mut atlas_img, &make_water_image(26, 28), 4 + s*5, 0);

        // Row 1: Ore deposits
        let y1 = 17u32;
        let r_ore_iron    = pack(&mut atlas_img, &make_ore_sprite(6, 7), 0, y1);
        let r_ore_copper  = pack(&mut atlas_img, &make_ore_sprite(7, 28), s, y1);
        let r_ore_coal    = pack(&mut atlas_img, &make_ore_sprite(27, 2), s*2, y1);
        let r_ore_stone   = pack(&mut atlas_img, &make_ore_sprite(25, 26), s*3, y1);
        let r_ore_uranium = pack(&mut atlas_img, &make_ore_sprite(22, 9), s*4, y1);
        let r_ore_tin     = pack(&mut atlas_img, &make_ore_sprite(4, 5), s*5, y1);
        let r_ore_gold    = pack(&mut atlas_img, &make_ore_sprite(30, 16), s*6, y1);
        let r_ore_sulfur  = pack(&mut atlas_img, &make_ore_sprite(16, 31), s*7, y1);
        let r_ore_crystal = pack(&mut atlas_img, &make_ore_sprite(19, 5), s*8, y1);
        let r_ore_oil     = pack(&mut atlas_img, &make_oil_sprite(), s*9, y1);

        // Row 2: Machines & structures (frame 0)
        let y2 = 34u32;
        let r_miner_0         = pack(&mut atlas_img, &make_miner_sprite(0), 0, y2);
        let r_stone_furnace_0 = pack(&mut atlas_img, &make_stone_furnace_sprite(0), s, y2);
        let r_steel_furnace_0 = pack(&mut atlas_img, &make_steel_furnace_sprite(0), s*2, y2);
        let r_assembler_0     = pack(&mut atlas_img, &make_assembler_sprite(0), s*3, y2);
        let r_lab_0           = pack(&mut atlas_img, &make_lab_sprite(0), s*4, y2);
        let r_boiler        = pack(&mut atlas_img, &make_boiler_sprite(), s*5, y2);
        let r_steam_engine  = pack(&mut atlas_img, &make_steam_engine_sprite(), s*6, y2);
        let r_solar_panel   = pack(&mut atlas_img, &make_solar_panel_sprite(), s*7, y2);
        let r_chest         = pack(&mut atlas_img, &make_chest_sprite(), s*8, y2);
        let r_gun_turret    = pack(&mut atlas_img, &make_gun_turret_sprite(), s*9, y2);
        let r_wall          = pack(&mut atlas_img, &make_wall_sprite(), s*10, y2);
        let r_inserter      = pack(&mut atlas_img, &make_inserter_sprite(), s*11, y2);

        // Row 3: Belts straight (3 colors × 2 frames)
        let y3 = 51u32;
        let r_belt_yellow = [
            pack(&mut atlas_img, &make_belt_sprite(17, 16, 0), 0, y3),
            pack(&mut atlas_img, &make_belt_sprite(17, 16, 1), s, y3),
        ];
        let r_belt_red = [
            pack(&mut atlas_img, &make_belt_sprite(10, 11, 0), s*2, y3),
            pack(&mut atlas_img, &make_belt_sprite(10, 11, 1), s*3, y3),
        ];
        let r_belt_blue = [
            pack(&mut atlas_img, &make_belt_sprite(12, 13, 0), s*4, y3),
            pack(&mut atlas_img, &make_belt_sprite(12, 13, 1), s*5, y3),
        ];

        // Row 4: Belt corners left
        let y4 = 68u32;
        let r_belt_corner_left_yellow = [
            pack(&mut atlas_img, &make_belt_corner_sprite(17, 16, 0, false), 0, y4),
            pack(&mut atlas_img, &make_belt_corner_sprite(17, 16, 1, false), s, y4),
        ];
        let r_belt_corner_left_red = [
            pack(&mut atlas_img, &make_belt_corner_sprite(10, 11, 0, false), s*2, y4),
            pack(&mut atlas_img, &make_belt_corner_sprite(10, 11, 1, false), s*3, y4),
        ];
        let r_belt_corner_left_blue = [
            pack(&mut atlas_img, &make_belt_corner_sprite(12, 13, 0, false), s*4, y4),
            pack(&mut atlas_img, &make_belt_corner_sprite(12, 13, 1, false), s*5, y4),
        ];

        // Row 5: Belt corners right
        let y5 = 85u32;
        let r_belt_corner_right_yellow = [
            pack(&mut atlas_img, &make_belt_corner_sprite(17, 16, 0, true), 0, y5),
            pack(&mut atlas_img, &make_belt_corner_sprite(17, 16, 1, true), s, y5),
        ];
        let r_belt_corner_right_red = [
            pack(&mut atlas_img, &make_belt_corner_sprite(10, 11, 0, true), s*2, y5),
            pack(&mut atlas_img, &make_belt_corner_sprite(10, 11, 1, true), s*3, y5),
        ];
        let r_belt_corner_right_blue = [
            pack(&mut atlas_img, &make_belt_corner_sprite(12, 13, 0, true), s*4, y5),
            pack(&mut atlas_img, &make_belt_corner_sprite(12, 13, 1, true), s*5, y5),
        ];

        // Row 6: Items (8×8, stride=9)
        let y6 = 102u32;
        let si = 9u32;
        let r_item_iron_ore     = pack(&mut atlas_img, &make_item_sprite(14, 16), 0, y6);
        let r_item_copper_ore   = pack(&mut atlas_img, &make_item_sprite(50, 52), si, y6);
        let r_item_coal         = pack(&mut atlas_img, &make_coal_item_sprite(), si*2, y6);
        let r_item_stone        = pack(&mut atlas_img, &make_stone_item_sprite(), si*3, y6);
        let r_item_iron_plate   = pack(&mut atlas_img, &make_plate_item_sprite(3, 4), si*4, y6);
        let r_item_copper_plate = pack(&mut atlas_img, &make_plate_item_sprite(28, 15), si*5, y6);
        let r_item_gear         = pack(&mut atlas_img, &make_gear_item_sprite(), si*6, y6);
        let r_item_wire         = pack(&mut atlas_img, &make_wire_item_sprite(), si*7, y6);
        let r_item_green_circuit = pack(&mut atlas_img, &make_circuit_item_sprite(), si*8, y6);
        let r_item_science_red  = pack(&mut atlas_img, &make_flask_item_sprite(10, 11), si*9, y6);
        // Extra items continued on same row (10+)
        let r_item_steel_plate  = pack(&mut atlas_img, &make_plate_item_sprite(5, 6), si*10, y6);    // silver
        let r_item_stone_brick  = pack(&mut atlas_img, &make_item_sprite(47, 48), si*11, y6);        // tan
        let r_item_pipe         = pack(&mut atlas_img, &make_item_sprite(3, 4), si*12, y6);           // gray
        let r_item_red_circuit  = pack(&mut atlas_img, &make_circuit_item_sprite_colored(22, 23), si*13, y6);
        let r_item_blue_circuit = pack(&mut atlas_img, &make_circuit_item_sprite_colored(25, 27), si*14, y6);
        let r_item_science_green = pack(&mut atlas_img, &make_flask_item_sprite(9, 10), si*15, y6);  // green
        let r_item_science_blue  = pack(&mut atlas_img, &make_flask_item_sprite(25, 27), si*16, y6); // blue
        let r_item_science_purple = pack(&mut atlas_img, &make_flask_item_sprite(29, 31), si*17, y6);// purple
        let r_item_sulfur       = pack(&mut atlas_img, &make_item_sprite(16, 31), si*18, y6);        // yellow
        let r_item_plastic      = pack(&mut atlas_img, &make_item_sprite(5, 7), si*19, y6);          // white
        // Row 6b: more items
        let y6b = y6 + si;
        let r_item_battery      = pack(&mut atlas_img, &make_item_sprite(9, 10), 0, y6b);            // green cell
        let r_item_ammo         = pack(&mut atlas_img, &make_item_sprite(16, 17), si, y6b);          // brass
        let r_item_grenade      = pack(&mut atlas_img, &make_item_sprite(9, 11), si*2, y6b);         // green sphere
        let r_item_engine       = pack(&mut atlas_img, &make_item_sprite(3, 53), si*3, y6b);         // gray mechanical
        let r_item_rocket_part  = pack(&mut atlas_img, &make_item_sprite(7, 22), si*4, y6b);         // white+red
        let r_item_rocket_fuel  = pack(&mut atlas_img, &make_item_sprite(17, 19), si*5, y6b);        // orange
        let r_item_inserter     = pack(&mut atlas_img, &make_item_sprite(3, 17), si*6, y6b);         // gray+orange
        let r_item_iron_stick   = pack(&mut atlas_img, &make_item_sprite(3, 5), si*7, y6b);          // thin bar
        let r_item_speed_module = pack(&mut atlas_img, &make_item_sprite(25, 28), si*8, y6b);        // blue module
        let r_item_science_yellow = pack(&mut atlas_img, &make_flask_item_sprite(16, 17), si*9, y6b);// yellow
        let r_item_uranium_ore  = pack(&mut atlas_img, &make_item_sprite(9, 62), si*10, y6b);        // green glow
        let r_item_rail         = pack(&mut atlas_img, &make_item_sprite(14, 4), si*11, y6b);         // brown+gray
        let r_item_concrete     = pack(&mut atlas_img, &make_item_sprite(3, 2), si*12, y6b);          // dark gray
        let r_item_solar_panel  = pack(&mut atlas_img, &make_item_sprite(25, 8), si*13, y6b);         // blue+white
        let r_item_accumulator  = pack(&mut atlas_img, &make_item_sprite(25, 6), si*14, y6b);         // blue+pale
        let r_item_robot_frame  = pack(&mut atlas_img, &make_item_sprite(4, 29), si*15, y6b);         // gray+purple
        let r_item_uranium_235  = pack(&mut atlas_img, &make_item_sprite(62, 9), si*16, y6b);         // bright green
        let r_item_uranium_238  = pack(&mut atlas_img, &make_item_sprite(9, 2), si*17, y6b);          // dull green
        let r_item_nuclear_fuel = pack(&mut atlas_img, &make_item_sprite(62, 61), si*18, y6b);        // hot green
        let r_item_low_density  = pack(&mut atlas_img, &make_item_sprite(5, 7), si*19, y6b);          // silver-white

        // Row 7: Enemies (2 walk frames each) + crashed ship
        let y7 = 120u32;
        // Small biter (red-brown)
        let r_enemy_0           = pack(&mut atlas_img, &make_enemy_sprite(23, 24, 0), 0, y7);
        let r_enemy_1           = pack(&mut atlas_img, &make_enemy_sprite(23, 24, 1), 14, y7);
        // Big biter (dark armored)
        let r_big_biter_0       = pack(&mut atlas_img, &make_big_biter_sprite(1, 2, 0), 28, y7);
        let r_big_biter_1       = pack(&mut atlas_img, &make_big_biter_sprite(1, 2, 1), 44, y7);
        // Spitter (green, ranged)
        let r_spitter_0         = pack(&mut atlas_img, &make_spitter_sprite(9, 10, 0), 60, y7);
        let r_spitter_1         = pack(&mut atlas_img, &make_spitter_sprite(9, 10, 1), 76, y7);
        let r_crashed_ship      = pack(&mut atlas_img, &make_crashed_ship_sprite(), 92, y7);

        // Row 8-9: Additional buildings (16×16)
        let y8 = 170u32;
        let r_underground_belt = pack(&mut atlas_img, &make_underground_belt_sprite(), 0, y8);
        let r_splitter         = pack(&mut atlas_img, &make_splitter_sprite(), s, y8);
        let r_chemical_plant   = pack(&mut atlas_img, &make_chemical_plant_sprite(), s*2, y8);
        let r_water_pump       = pack(&mut atlas_img, &make_water_pump_sprite(), s*3, y8);
        let r_oil_refinery     = pack(&mut atlas_img, &make_oil_refinery_sprite(), s*4, y8);
        let r_nuclear_reactor  = pack(&mut atlas_img, &make_nuclear_reactor_sprite(), s*5, y8);
        let r_accumulator      = pack(&mut atlas_img, &make_accumulator_sprite(), s*6, y8);
        let r_radar            = pack(&mut atlas_img, &make_radar_sprite(), s*7, y8);
        let r_pipe             = pack(&mut atlas_img, &make_pipe_sprite(), s*8, y8);
        let r_roboport         = pack(&mut atlas_img, &make_roboport_sprite(), s*9, y8);

        let y9 = 187u32;
        let r_rail             = pack(&mut atlas_img, &make_rail_sprite(), 0, y9);
        let r_train_stop       = pack(&mut atlas_img, &make_train_stop_sprite(), s, y9);
        let r_rocket_silo      = pack(&mut atlas_img, &make_rocket_silo_sprite(), s*2, y9);
        let r_beacon           = pack(&mut atlas_img, &make_beacon_sprite(), s*3, y9);
        let r_electric_furnace = pack(&mut atlas_img, &make_electric_furnace_sprite(), s*4, y9);
        let r_pump_jack        = pack(&mut atlas_img, &make_pump_jack_sprite(), s*5, y9);
        let r_laser_turret     = pack(&mut atlas_img, &make_laser_turret_sprite(), s*6, y9);

        // Row 10: Machine animation frame 1 (active state variants)
        let y10 = 204u32;
        let r_miner_1         = pack(&mut atlas_img, &make_miner_sprite(1), 0, y10);
        let r_stone_furnace_1 = pack(&mut atlas_img, &make_stone_furnace_sprite(1), s, y10);
        let r_steel_furnace_1 = pack(&mut atlas_img, &make_steel_furnace_sprite(1), s*2, y10);
        let r_assembler_1     = pack(&mut atlas_img, &make_assembler_sprite(1), s*3, y10);
        let r_lab_1           = pack(&mut atlas_img, &make_lab_sprite(1), s*4, y10);

        // Row 11: FORGE avatar (24×24, 2 expressions)
        let y11 = 221u32;
        let r_forge_0 = pack(&mut atlas_img, &make_forge_avatar_sprite(0), 0, y11);
        let r_forge_1 = pack(&mut atlas_img, &make_forge_avatar_sprite(1), 25, y11);

        // Combine into 2-frame arrays.
        let r_miner = [r_miner_0, r_miner_1];
        let r_stone_furnace = [r_stone_furnace_0, r_stone_furnace_1];
        let r_steel_furnace = [r_steel_furnace_0, r_steel_furnace_1];
        let r_assembler = [r_assembler_0, r_assembler_1];
        let r_lab = [r_lab_0, r_lab_1];
        let r_enemy_small_biter = [r_enemy_0, r_enemy_1];
        let r_enemy_big_biter = [r_big_biter_0, r_big_biter_1];
        let r_enemy_spitter = [r_spitter_0, r_spitter_1];
        let r_forge_avatar = [r_forge_0, r_forge_1];

        // Export atlas as PNG for inspection / art replacement.
        atlas_img.export_png("assets/spritesheet.png");

        // Upload to GPU as single texture.
        let atlas_tex = Texture2D::from_image(&atlas_img);
        atlas_tex.set_filter(FilterMode::Nearest);

        Self {
            tex: atlas_tex,
            tex_size: Vec2::new(512.0, 512.0),

            r_ground_grass, r_ground_grass_alt, r_ground_desert,
            r_ground_forest, r_ground_water, r_ground_water_alt,

            r_ore_iron, r_ore_copper, r_ore_coal, r_ore_stone,
            r_ore_uranium, r_ore_tin, r_ore_gold, r_ore_sulfur,
            r_ore_crystal, r_ore_oil,

            r_belt_yellow, r_belt_red, r_belt_blue,
            r_belt_corner_left_yellow, r_belt_corner_left_red, r_belt_corner_left_blue,
            r_belt_corner_right_yellow, r_belt_corner_right_red, r_belt_corner_right_blue,

            r_miner, r_stone_furnace, r_steel_furnace, r_assembler,
            r_lab, r_boiler, r_steam_engine, r_solar_panel, r_chest,
            r_gun_turret, r_laser_turret, r_wall, r_inserter,
            r_underground_belt, r_splitter, r_chemical_plant, r_water_pump,
            r_oil_refinery, r_nuclear_reactor, r_accumulator, r_radar,
            r_pipe, r_roboport, r_rail, r_train_stop, r_rocket_silo,
            r_beacon, r_electric_furnace, r_pump_jack,

            r_item_iron_ore, r_item_copper_ore, r_item_coal, r_item_stone,
            r_item_iron_plate, r_item_copper_plate, r_item_gear, r_item_wire,
            r_item_green_circuit, r_item_science_red,
            r_item_steel_plate, r_item_stone_brick, r_item_pipe,
            r_item_red_circuit, r_item_blue_circuit,
            r_item_science_green, r_item_science_blue, r_item_science_purple,
            r_item_sulfur, r_item_plastic, r_item_battery,
            r_item_ammo, r_item_grenade, r_item_engine,
            r_item_rocket_part, r_item_rocket_fuel,
            r_item_inserter, r_item_iron_stick, r_item_speed_module,
            r_item_science_yellow,
            r_item_uranium_ore, r_item_rail, r_item_concrete,
            r_item_solar_panel, r_item_accumulator, r_item_robot_frame,
            r_item_uranium_235, r_item_uranium_238, r_item_nuclear_fuel,
            r_item_low_density,

            r_enemy_small_biter, r_enemy_big_biter, r_enemy_spitter, r_crashed_ship,
            r_forge_avatar,
        }
    }
}

// ===========================================================================
// Internal sprite generation helpers
// ===========================================================================

/// Convert palette index to macroquad Color.
fn pal_color(idx: usize) -> Color {
    let (r, g, b, a) = PALETTE[idx];
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0)
}

/// Check if a Color matches a palette index.
fn is_pal(c: Color, idx: usize) -> bool {
    let p = pal_color(idx);
    (c.r - p.r).abs() < 0.01 && (c.g - p.g).abs() < 0.01 && (c.b - p.b).abs() < 0.01
}

/// Creates an [`Image`] from a 2D grid of palette indices at the given size.
fn make_image_rect(pixels: &[&[u8]], width: u16, height: u16) -> Image {
    let mut image = Image::gen_image_color(width, height, Color::new(0.0, 0.0, 0.0, 0.0));
    for (y, row) in pixels.iter().enumerate() {
        for (x, &idx) in row.iter().enumerate() {
            if idx == 0 || x >= width as usize || y >= height as usize {
                continue;
            }
            let (r, g, b, a) = PALETTE[idx as usize];
            image.set_pixel(
                x as u32,
                y as u32,
                Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0),
            );
        }
    }
    image
}

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

/// Corner belt sprite matching straight belt design. Fills the entire tile with
/// the same edge pattern (outline→highlight→inner hl→surface→inner sh→shadow→outline)
/// plus a rounded cutout in the inner corner.
/// `mirror=false`: left-turn (W→N). `mirror=true`: right-turn (E→N).
fn make_belt_corner_sprite(_base_color: u8, _arrow_color: u8, _frame: u8, mirror: bool) -> Image {
    let mut pixels = [[0u8; 16]; 16];

    for y in 0..16u8 {
        for x in 0..16u8 {
            // Work in canonical left-turn space; mirror flips x.
            let sx = if mirror { 15 - x } else { x };

            // Cutout in bottom-right: rounded inner corner.
            let cdx = 15.5 - sx as f32;
            let cdy = 15.5 - y as f32;
            let cdist = (cdx * cdx + cdy * cdy).sqrt();
            let cut_r = 3.0f32;

            if cdist < cut_r {
                pixels[y as usize][x as usize] = 0; // transparent cutout
                continue;
            }

            // Determine edge layers exactly like the straight belt.
            // Straight belt: x=0→outline(1), x=1→hl(55), x=2→ihl(56),
            //                x=13→ish(54), x=14→sh(53), x=15→outline(1)
            //                surface: x<7→55, x>=7→54
            // Corner: apply same pattern on BOTH axes, pick the outermost.

            // Distance from each edge.
            let d_left = sx;
            let d_top = y;
            let d_right = 15 - sx;
            let d_bottom = 15 - y;
            // Distance from inner corner curve.
            let d_curve = cdist - cut_r;

            // The "depth" into the belt from the nearest boundary.
            let d_min = d_left.min(d_top).min(d_right).min(d_bottom).min(d_curve as u8);

            let pixel = match d_min {
                0 => 1,  // outline
                1 => {
                    // Highlight or shadow depending on which edge is closest.
                    if d_left <= d_right && d_left <= d_bottom && d_left <= (d_curve as u8) {
                        if mirror { 53 } else { 55 } // left edge
                    } else if d_top <= d_bottom && d_top <= d_right && d_top <= (d_curve as u8) {
                        55 // top edge (always lit)
                    } else if d_curve as u8 <= d_right && (d_curve as u8) <= d_bottom {
                        55 // inner curve highlight
                    } else if d_right <= d_bottom {
                        if mirror { 55 } else { 53 } // right edge shadow
                    } else {
                        53 // bottom edge shadow
                    }
                }
                2 => {
                    if d_left <= d_right && d_left <= d_bottom && d_left <= (d_curve as u8) {
                        if mirror { 54 } else { 56 }
                    } else if d_top <= d_bottom && d_top <= d_right && d_top <= (d_curve as u8) {
                        56
                    } else if d_curve as u8 <= d_right && (d_curve as u8) <= d_bottom {
                        56
                    } else if d_right <= d_bottom {
                        if mirror { 56 } else { 54 }
                    } else {
                        54
                    }
                }
                _ => {
                    // Belt surface — same gradient as straight belt.
                    if sx < 8 { 55 } else { 54 }
                }
            };

            pixels[y as usize][x as usize] = pixel;
        }
    }

    let rows: Vec<&[u8]> = pixels.iter().map(|r| r.as_slice()).collect();
    make_image(&rows, 16)
}

// ===========================================================================
// Hand-crafted 16×16 machine sprites
// Light source: top-left. Outlines use selective outlining (selout).
// ===========================================================================

/// Creates the miner sprite — orange body with pickaxe, gear detail, rocky base.
/// Frame 1: piston lowered (mining action).
fn make_miner_sprite(frame: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = if frame == 0 { &[
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
    ]} else { &[
        // Frame 1: piston down — body shifts down 1px, gear rotated.
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 1,20,20,19,19,19,18, 1, 0, 0, 0],
        &[0, 0, 0, 1,20,20, 8,20,19,19, 8,18,17, 1, 0, 0],
        &[0, 0, 1,20,20, 8,56,55,19,56,55,18,17,17, 1, 0],
        &[0, 1,20,20,19,55, 5, 4,55, 5, 4,54,17,17, 2, 1],
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
    ]};
    make_image(pixels, 16)
}

/// Creates the stone furnace sprite — warm brick oven with amber fire glow.
/// Frame 1: fire flickers (swap fire colors).
fn make_stone_furnace_sprite(frame: u8) -> Image {
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
    let mut img = make_image(pixels, 16);
    if frame == 1 {
        // Flicker fire: swap palette 61↔39 and 23↔40 in the fire region (rows 4-7, cols 4-10).
        for y in 4..8u32 {
            for x in 4..11u32 {
                let c = img.get_pixel(x, y);
                if is_pal(c, 61) { img.set_pixel(x, y, pal_color(39)); }
                else if is_pal(c, 39) { img.set_pixel(x, y, pal_color(61)); }
                else if is_pal(c, 23) { img.set_pixel(x, y, pal_color(40)); }
                else if is_pal(c, 40) { img.set_pixel(x, y, pal_color(23)); }
            }
        }
    }
    img
}

/// Steel furnace — polished metal body (53-56) with bright fire (61, 39-40).
/// Frame 1: fire flickers.
fn make_steel_furnace_sprite(frame: u8) -> Image {
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
    let mut img = make_image(pixels, 16);
    if frame == 1 {
        for y in 4..8u32 {
            for x in 4..11u32 {
                let c = img.get_pixel(x, y);
                if is_pal(c, 61) { img.set_pixel(x, y, pal_color(40)); }
                else if is_pal(c, 40) { img.set_pixel(x, y, pal_color(61)); }
                else if is_pal(c, 23) { img.set_pixel(x, y, pal_color(39)); }
            }
        }
    }
    img
}

/// Creates the assembler sprite — periwinkle-blue mechanical box with gear detail.
/// Frame 1: gear rotated (swaps 7↔8 in center to simulate rotation).
fn make_assembler_sprite(frame: u8) -> Image {
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
    let mut img = make_image(pixels, 16);
    if frame == 1 {
        // Swap 7↔8 in center gear region to simulate rotation.
        for y in 6..10u32 {
            for x in 6..10u32 {
                let c = img.get_pixel(x, y);
                if is_pal(c, 7) { img.set_pixel(x, y, pal_color(8)); }
                else if is_pal(c, 8) { img.set_pixel(x, y, pal_color(7)); }
            }
        }
    }
    img
}

/// Creates the lab sprite — purple body with flask/beaker detail.
/// Frame 1: bubbles shift (swap indicator positions).
fn make_lab_sprite(frame: u8) -> Image {
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
    let mut img = make_image(pixels, 16);
    if frame == 1 {
        // Swap flask colors 58↔59 to simulate bubbling.
        for y in 4..9u32 {
            for x in 4..9u32 {
                let c = img.get_pixel(x, y);
                if is_pal(c, 58) { img.set_pixel(x, y, pal_color(59)); }
                else if is_pal(c, 59) { img.set_pixel(x, y, pal_color(58)); }
            }
        }
    }
    img
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

// --- Item sprites (8×8) — detailed, fill the full space ---

/// Ore chunk — rough angular rock with colored facets and highlight.
fn make_item_sprite(dark: u8, light: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 1, 1, 0, 0, 0],
        &[0, 0, 1,light, 8,light, 1, 0],
        &[0, 1,light,dark,light,dark, 1, 0],
        &[1,light,dark,light,dark,light,dark, 1],
        &[1,dark,light,dark,light,dark, 1, 0],
        &[0, 1,dark,dark,dark, 1, 0, 0],
        &[0, 0, 1, 1, 1, 1, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Coal chunk — dark jagged irregular shape (distinct from round ores).
fn make_coal_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 1, 0, 0, 0, 0],
        &[0, 0, 1, 1, 1, 0, 0, 0],
        &[0, 1, 1, 2, 1, 1, 0, 0],
        &[1, 1, 2, 3, 2, 1, 1, 0],
        &[0, 1, 1, 2, 1, 1, 0, 0],
        &[0, 0, 1, 1, 1, 0, 0, 0],
        &[0, 0, 0, 1, 1, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Stone piece — flat layered sedimentary shape (visually distinct from ores).
fn make_stone_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 0, 0],
        &[0,45,46,47,47,46, 0, 0],
        &[45,46,47,48,47,46,45, 0],
        &[45,45,46,47,46,45,45, 0],
        &[0,45,46,47,47,46,45, 0],
        &[0,45,45,46,46,45, 0, 0],
        &[0, 0,45,45,45, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Metal plate — flat shiny rectangle with beveled edges and specular corner.
fn make_plate_item_sprite(dark: u8, light: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 1, 1, 1, 1, 1, 1, 0],
        &[1, 8,light,light,light,light,dark, 1],
        &[1,light,light,light,light,dark,dark, 1],
        &[1,light,light,dark,dark,dark,dark, 1],
        &[1,light,dark,dark,dark,dark,dark, 1],
        &[1,dark,dark,dark,dark,dark, 1, 1],
        &[0, 1, 1, 1, 1, 1, 1, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Gear — cogwheel with visible teeth around a central hole.
fn make_gear_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 55, 0,55,55, 0,55, 0],
        &[55,56,55,55,55,55,54, 55],
        &[0, 55,55, 1, 1,55,54, 0],
        &[55,55, 1, 0, 0, 1,54,55],
        &[55,54, 1, 0, 0, 1,53,55],
        &[0, 54,53, 1, 1,53,53, 0],
        &[55,53,54,53,53,54,53,55],
        &[0, 55, 0,55,55, 0,55, 0],
    ];
    make_image(pixels, 8)
}

/// Wire — copper coil with multiple visible loops and shine.
fn make_wire_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0,51, 0, 0, 0, 0, 0],
        &[0,51,52,51, 0, 0, 0, 0],
        &[0, 0,50,51,50, 0, 0, 0],
        &[0, 0, 0,49,51,49, 0, 0],
        &[0, 0,49,51,52,51, 0, 0],
        &[0,49,51,49, 0,49,51, 0],
        &[49,51,49, 0, 0, 0,49,51],
        &[0,49, 0, 0, 0, 0, 0,49],
    ];
    make_image(pixels, 8)
}

/// Circuit board — green PCB with visible traces, components, and solder points.
fn make_circuit_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[57,57,57,57,57,57,57,57],
        &[57, 8,58,58,58,58, 8,57],
        &[57,59, 1,58,58, 1,59,57],
        &[57,58,58,60,59,58,58,57],
        &[57,58,58,59,60,58,58,57],
        &[57,59, 1,58,58, 1,59,57],
        &[57, 8,58,58,58,58, 8,57],
        &[57,57,57,57,57,57,57,57],
    ];
    make_image(pixels, 8)
}

/// Colored circuit board — same shape, different colors (for red/blue circuits).
fn make_circuit_item_sprite_colored(dark: u8, light: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[dark,dark,dark,dark,dark,dark,dark,dark],
        &[dark, 8,light,light,light,light, 8,dark],
        &[dark,light, 1,light,light, 1,light,dark],
        &[dark,light,light,dark,light,light,light,dark],
        &[dark,light,light,light,dark,light,light,dark],
        &[dark,light, 1,light,light, 1,light,dark],
        &[dark, 8,light,light,light,light, 8,dark],
        &[dark,dark,dark,dark,dark,dark,dark,dark],
    ];
    make_image(pixels, 8)
}

/// Science flask — rounded glass bottle with colored liquid and cork top.
fn make_flask_item_sprite(dark: u8, light: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0,16,16, 0, 0, 0],
        &[0, 0, 7, 7, 7, 7, 0, 0],
        &[0, 0, 1,light, 1, 0, 0, 0],
        &[0, 1,light, 8,light, 1, 0, 0],
        &[1,dark,light,light,light,dark, 1, 0],
        &[1,dark,dark,light,dark,dark, 1, 0],
        &[0, 1,dark,dark,dark, 1, 0, 0],
        &[0, 0, 1, 1, 1, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

// --- Enemy sprites (12×12) ---

/// Alien guardian — 3-segment insectoid: head with mandibles, thorax, abdomen.
/// 6 jointed legs, glowing eyes, jagged carapace outline.
/// Professional pixel art anatomy: distinct body segments, sharp silhouette.
fn make_enemy_sprite(body: u8, shadow: u8, frame: u8) -> Image {
    // Frame 0: legs extended left. Frame 1: legs extended right (walk cycle).
    let (l0, l1) = if frame == 0 { (shadow, 0u8) } else { (0u8, shadow) };
    let pixels: Vec<Vec<u8>> = vec![
        //     mandibles    head     mandibles
        vec![shadow, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,shadow],
        vec![0,shadow, 0, 0,shadow,shadow,shadow,shadow, 0, 0,shadow, 0],
        vec![0, 0, 0,shadow,body,63, 8,63,body,shadow, 0, 0],
        vec![0, 0,shadow,body,body,shadow,shadow,body,body,shadow, 0, 0],
        // legs alternate per frame
        vec![l0, l1,shadow,body,body,body,body,body,body,shadow, l1, l0],
        vec![l1,shadow,body,body,shadow,body,body,shadow,body,body,shadow, l1],
        vec![l0, l1,shadow,body,body,body,body,body,body,shadow, l1, l0],
        // abdomen
        vec![l1,shadow, l0,shadow,body,body,body,body,shadow, l0,shadow, l1],
        vec![l0, l1, 0,shadow,shadow,body,body,shadow,shadow, 0, l1, l0],
        vec![0, 0, 0, 0,shadow,shadow,shadow,shadow, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0,shadow,shadow, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    let refs: Vec<&[u8]> = pixels.iter().map(|r| r.as_slice()).collect();
    make_image(&refs, 12)
}

/// Big biter — heavier armored body with thicker legs and a spiked carapace.
/// Uses dark shadow colors for armored feel.
fn make_big_biter_sprite(body: u8, shadow: u8, frame: u8) -> Image {
    let (l0, l1) = if frame == 0 { (shadow, 0u8) } else { (0u8, shadow) };
    let armor = 3u8; // mid gray for armor plates
    let pixels: Vec<Vec<u8>> = vec![
        // mandibles (wider)
        vec![shadow, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,shadow],
        vec![body,shadow, 0,shadow,shadow,shadow,shadow,shadow,shadow, 0,shadow,body],
        // head with armored plates + eyes
        vec![0,shadow,shadow,body,armor,63, 8,63,armor,body,shadow, 0],
        vec![0,shadow,body,armor,body,shadow,shadow,body,armor,body,shadow, 0],
        // armored thorax (thicker)
        vec![l0,shadow,armor,body,armor,body,body,armor,body,armor,shadow, l0],
        vec![l1,body,armor,armor,shadow,armor,armor,shadow,armor,armor,body, l1],
        vec![l0,shadow,armor,body,armor,body,body,armor,body,armor,shadow, l0],
        // abdomen with spikes
        vec![l1,shadow,body,shadow,body,body,body,body,shadow,body,shadow, l1],
        vec![l0,body, 0,shadow,armor,body,body,armor,shadow, 0,body, l0],
        vec![0,shadow, 0, 0,shadow,shadow,shadow,shadow, 0, 0,shadow, 0],
        vec![0, 0, 0, 0,shadow,body,body,shadow, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0,shadow,shadow, 0, 0, 0, 0, 0],
    ];
    let refs: Vec<&[u8]> = pixels.iter().map(|r| r.as_slice()).collect();
    make_image(&refs, 12)
}

/// Spitter — slender body with bulbous acid sac and thin legs.
/// Green-tinted with glowing projectile organ on the abdomen.
fn make_spitter_sprite(body: u8, shadow: u8, frame: u8) -> Image {
    let (l0, l1) = if frame == 0 { (shadow, 0u8) } else { (0u8, shadow) };
    let glow = 62u8; // bright green for acid sac
    let pixels: Vec<Vec<u8>> = vec![
        // thin mandibles
        vec![0, 0, 0, 0,shadow, 0, 0,shadow, 0, 0, 0, 0],
        vec![0, 0, 0,shadow,body,shadow,shadow,body,shadow, 0, 0, 0],
        // small head with glowing eyes
        vec![0, 0,shadow,body,glow, 8,glow, 8,body,shadow, 0, 0],
        vec![0, 0,shadow,body,body,shadow,shadow,body,body,shadow, 0, 0],
        // thin thorax
        vec![l0, 0,shadow,body,body,body,body,body,body,shadow, 0, l0],
        vec![0,l1,shadow,body,shadow,body,body,shadow,body,shadow,l1, 0],
        // acid sac (glowing bulge)
        vec![l0, 0,shadow,body,glow,glow,glow,glow,body,shadow, 0, l0],
        vec![0,l1, 0,shadow,glow, 8, 8,glow,shadow, 0,l1, 0],
        vec![l0, 0, 0,shadow,glow,glow,glow,glow,shadow, 0, 0, l0],
        // tapered tail
        vec![0, 0, 0, 0,shadow,body,body,shadow, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0,shadow,shadow, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    let refs: Vec<&[u8]> = pixels.iter().map(|r| r.as_slice()).collect();
    make_image(&refs, 12)
}

/// 80x48 pixel art crashed colony ship "Horizon's Promise".
/// Hull is dark blue-gray (1-4), damage exposes warm brown internals (13-16),
/// cockpit glows blue-purple (29-31), scorch marks in dark tones.
fn make_crashed_ship_sprite() -> Image {
    // Palette reference:
    // 1=deep shadow, 2=dark shadow, 3=mid gray, 4=light gray
    // 5=silver, 6=pale, 7=near-white, 8=specular
    // 13=dark brown, 14=mid brown, 15=warm brown, 16=tan
    // 17=dark coral, 18=coral, 19=amber, 20=peach
    // 25=deep blue, 26=mid blue, 27=periwinkle, 28=sky blue
    // 29=deep purple, 30=mid purple, 31=lilac
    // 37=dark gold, 38=gold, 39=yellow
    let _=0u8; let H=2u8; let D=1u8; let M=3u8; let L=4u8; let S=5u8;
    let B=14u8; let W=15u8; let G=27u8; let P=30u8; let K=29u8;
    let F=18u8; let Y=38u8; let R=17u8;
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        // Row 0-3: antenna + top debris
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,G,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,G,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,H,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 4-5: broken top wing stub
        &[0,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,H,H,H,D,D,D,D,H,H,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,H,H,D,D,H,H,H,M,M,H,H,H,H,M,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 6-7: top hull surface
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,D,H,H,M,M,M,M,H,H,M,L,M,M,H,H,M,M,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,D,D,H,H,M,M,L,L,M,M,M,M,L,L,S,L,M,H,M,L,M,M,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 8-9: engine block (rear) + hull top with damage
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,D,D,D,D,D,D,D,D,H,H,H,H,H,H,M,M,M,M,M,M,H,H,M,M,M,L,L,S,S,L,L,M,M,B,W,B,M,M,L,L,S,L,L,M,M,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,D,D,H,H,H,H,H,H,M,M,M,M,M,M,M,M,L,L,L,L,M,M,M,L,L,L,S,S,S,S,S,L,M,B,W,B,W,B,M,L,S,S,L,L,M,M,M,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 10-11: engine + main fuselage with exposed internals
        &[0,0,0,0,0,0,0,0,0,0,D,D,H,R,F,D,H,H,M,M,M,L,L,L,L,L,M,M,L,S,S,L,L,M,L,L,S,S,L,L,L,L,L,L,B,W,F,R,F,W,B,L,L,S,S,L,M,M,M,M,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,D,D,H,H,F,R,D,H,M,M,L,L,L,S,S,L,L,L,L,L,S,S,S,L,L,L,S,S,L,L,M,M,M,L,B,W,Y,F,R,Y,W,B,L,L,S,L,L,M,M,M,M,M,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 12-13: mid fuselage with cockpit area
        &[0,0,0,0,0,0,0,0,0,D,H,H,M,D,D,H,M,M,L,L,S,S,S,L,L,L,L,L,S,S,L,L,L,L,L,L,S,L,M,M,M,M,L,B,B,B,B,B,B,B,L,L,L,L,L,M,M,M,M,M,M,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,D,D,H,M,M,H,H,M,M,L,L,S,S,L,L,L,L,L,M,L,L,S,L,L,M,M,M,L,L,M,M,M,M,L,L,M,M,L,L,M,M,L,L,L,M,M,M,M,H,K,P,K,M,M,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 14-15: cockpit glow area
        &[0,0,0,0,0,0,0,0,D,H,M,M,M,M,M,M,L,L,S,S,L,L,M,M,M,M,M,M,L,L,M,M,M,M,M,L,M,M,H,H,M,L,L,M,M,M,M,M,M,M,L,M,M,M,H,K,K,P,G,P,K,M,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,D,D,H,M,M,M,L,L,M,L,S,S,L,L,M,M,M,H,M,M,M,L,M,M,H,H,M,M,M,M,H,H,H,M,L,L,M,M,H,H,M,M,L,L,M,H,H,K,P,G,G,G,G,P,K,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 16-17: main hull center
        &[0,0,0,0,0,0,0,D,H,M,M,M,L,L,S,L,L,S,L,L,M,M,H,H,H,M,M,M,M,M,H,H,H,H,M,M,H,H,D,H,M,M,L,M,H,H,H,H,M,L,M,M,H,K,P,G,G,28,G,G,P,K,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,D,H,M,M,L,L,S,S,L,L,L,L,M,M,H,H,D,H,H,M,M,M,H,H,D,D,H,H,H,H,D,D,H,M,L,L,M,H,D,D,H,M,L,M,M,H,K,P,G,28,28,28,G,P,K,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 18-19: hull plates + rivet lines
        &[0,0,0,0,0,0,0,D,H,M,L,L,S,S,L,L,M,M,M,M,H,H,D,D,D,H,M,M,H,H,D,D,D,D,H,H,D,D,D,H,M,L,L,M,H,D,D,H,M,L,M,M,H,K,P,G,G,28,G,G,P,K,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,D,H,M,L,S,S,L,L,M,M,M,M,H,H,D,D,D,H,H,M,M,H,D,D,D,D,D,D,D,D,D,H,M,M,L,L,M,H,H,D,H,M,L,M,M,H,K,P,G,G,G,G,P,K,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 20-21: hull bottom + damage hole
        &[0,0,0,0,0,0,0,D,H,M,L,L,L,M,M,M,M,H,H,H,D,D,D,H,H,M,M,H,H,D,D,D,D,D,D,D,D,H,M,M,L,L,M,M,H,H,H,M,M,L,M,M,M,H,K,P,P,P,K,K,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,D,H,M,M,L,L,M,M,H,H,H,D,D,D,D,H,H,M,M,M,H,D,D,B,W,B,D,D,D,H,M,M,L,L,L,M,M,H,H,M,M,L,L,M,M,M,M,H,H,K,K,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 22-23: lower fuselage
        &[0,0,0,0,0,0,0,D,H,M,M,M,L,L,M,H,H,D,D,D,D,H,H,M,M,M,H,D,D,B,W,Y,W,B,D,H,M,M,L,L,L,M,M,H,H,M,M,L,L,M,M,M,M,M,H,H,D,D,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,D,H,H,M,M,M,L,M,H,D,D,D,D,H,H,M,M,M,H,D,D,B,W,F,R,F,W,B,M,M,L,L,L,M,M,H,H,M,M,L,L,M,M,M,H,H,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 24-25: hull shadow + lower wing stub
        &[0,0,0,0,0,0,0,0,D,H,H,M,M,M,M,H,D,D,D,H,H,M,M,M,H,H,D,D,D,B,W,B,D,D,H,M,L,L,L,M,M,H,H,D,M,L,L,M,M,M,H,H,D,D,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,D,D,H,H,M,M,M,H,D,D,H,H,M,M,M,H,H,D,D,D,D,D,D,D,D,H,M,M,L,L,M,M,H,H,D,D,M,L,M,M,H,H,D,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 26-27: bottom wing + hull underside
        &[0,0,0,0,0,0,0,0,0,D,D,H,H,M,M,H,D,H,H,M,M,M,H,H,D,D,D,D,D,D,D,D,H,M,M,L,L,M,M,H,H,D,D,D,M,M,M,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,D,D,H,H,M,H,H,H,M,M,H,H,H,D,D,0,0,0,0,0,0,H,M,M,L,L,M,M,H,H,D,D,0,D,M,M,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 28-29: lower hull taper + broken bits
        &[0,0,0,0,0,0,0,0,0,0,0,D,D,H,H,H,H,M,M,H,H,D,D,0,0,0,0,0,0,0,0,H,M,L,L,M,M,H,D,D,D,0,0,D,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,D,D,H,H,M,M,H,H,D,D,0,0,0,0,0,0,0,0,H,M,M,L,M,M,H,D,D,0,0,0,0,0,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 30-31: scattered debris below
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,D,D,H,H,H,D,D,0,0,0,0,0,0,0,0,0,H,M,M,L,M,M,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,D,D,0,0,0,0,0,0,0,0,0,0,D,D,H,D,D,0,0,0,0,0,0,0,0,0,H,M,M,M,M,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 32-35: ground debris / scorch marks
        &[0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,D,D,D,0,0,0,0,D,D,0,0,0,H,M,M,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,D,0,0,0,0,D,0,0,0,0,0,0,0,0,D,0,0,0,0,D,H,D,0,0,0,0,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,D,0,0,0,0,0,D,D,0,0,0,0,0,0,0,D,D,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,D,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,H,D,0,0,0,0,0,0,D,0,0,0,0,0,0,0,0,D,H,D,0,0,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 36-39: more scattered debris
        &[0,0,0,0,0,0,0,0,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,D,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 40-47: empty padding
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    ];
    make_image_rect(pixels, 80, 48)
}

// ===========================================================================
// Additional building sprites
// ===========================================================================

/// Underground belt — dark hole with belt edges visible.
fn make_underground_belt_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
        &[0, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 1, 0],
        &[1, 3, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 3, 1],
        &[1, 3, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 3, 1],
        &[1, 3, 1, 1,17,17,17,17,17,17,17,17, 1, 1, 3, 1],
        &[1, 3, 1, 1,17,16,16,16,16,16,16,17, 1, 1, 3, 1],
        &[1, 3, 1, 1,17,16, 1, 1, 1, 1,16,17, 1, 1, 3, 1],
        &[1, 3, 1, 1,17,16, 1, 2, 2, 1,16,17, 1, 1, 3, 1],
        &[1, 3, 1, 1,17,16, 1, 2, 2, 1,16,17, 1, 1, 3, 1],
        &[1, 3, 1, 1,17,16, 1, 1, 1, 1,16,17, 1, 1, 3, 1],
        &[1, 3, 1, 1,17,16,16,16,16,16,16,17, 1, 1, 3, 1],
        &[1, 3, 1, 1,17,17,17,17,17,17,17,17, 1, 1, 3, 1],
        &[1, 3, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 3, 1],
        &[1, 3, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 3, 1],
        &[0, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 1, 0],
        &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Splitter — wide belt mechanism with divider in center.
fn make_splitter_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        &[1, 3, 3, 3, 3, 3, 3, 4, 4, 3, 3, 3, 3, 3, 3, 1],
        &[1, 3,17,17,17,17, 3, 4, 4, 3,17,17,17,17, 3, 1],
        &[1, 3,17,16,16,17, 3, 5, 5, 3,17,16,16,17, 3, 1],
        &[1, 3,17,16,16,17, 3, 4, 4, 3,17,16,16,17, 3, 1],
        &[1, 3,17,16,16,17, 3, 4, 4, 3,17,16,16,17, 3, 1],
        &[1, 3,17,16,16,17, 3, 5, 5, 3,17,16,16,17, 3, 1],
        &[1, 4, 4, 4, 4, 4, 4, 6, 6, 4, 4, 4, 4, 4, 4, 1],
        &[1, 4, 4, 4, 4, 4, 4, 6, 6, 4, 4, 4, 4, 4, 4, 1],
        &[1, 3,17,16,16,17, 3, 5, 5, 3,17,16,16,17, 3, 1],
        &[1, 3,17,16,16,17, 3, 4, 4, 3,17,16,16,17, 3, 1],
        &[1, 3,17,16,16,17, 3, 4, 4, 3,17,16,16,17, 3, 1],
        &[1, 3,17,16,16,17, 3, 5, 5, 3,17,16,16,17, 3, 1],
        &[1, 3,17,17,17,17, 3, 4, 4, 3,17,17,17,17, 3, 1],
        &[1, 3, 3, 3, 3, 3, 3, 4, 4, 3, 3, 3, 3, 3, 3, 1],
        &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    ];
    make_image(pixels, 16)
}

/// Chemical plant — green industrial structure with pipes and reaction chamber.
fn make_chemical_plant_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
        &[0, 1, 9, 9, 9, 9,10,10,10,10, 9, 9, 9, 9, 1, 0],
        &[1, 9,10,10, 3, 3, 3,12,12, 3, 3, 3,10,10, 9, 1],
        &[1, 9,10, 3, 3, 9, 9,12,12, 9, 9, 3, 3,10, 9, 1],
        &[1, 9, 3, 3, 9,10,10,11,11,10,10, 9, 3, 3, 9, 1],
        &[1,10, 3, 9,10,11,11,12,12,11,11,10, 9, 3,10, 1],
        &[1,10, 3, 9,10,11,12,12,12,12,11,10, 9, 3,10, 1],
        &[1,10, 3, 9,10,11,12,10,10,12,11,10, 9, 3,10, 1],
        &[1,10, 3, 9,10,11,12,10,10,12,11,10, 9, 3,10, 1],
        &[1,10, 3, 9,10,11,12,12,12,12,11,10, 9, 3,10, 1],
        &[1,10, 3, 9,10,11,11,12,12,11,11,10, 9, 3,10, 1],
        &[1, 9, 3, 3, 9,10,10,11,11,10,10, 9, 3, 3, 9, 1],
        &[1, 9,10, 3, 3, 9, 9, 3, 3, 9, 9, 3, 3,10, 9, 1],
        &[1, 9,10,10, 3, 3, 3, 3, 3, 3, 3, 3,10,10, 9, 1],
        &[0, 1, 9, 9, 9, 9,10,10,10,10, 9, 9, 9, 9, 1, 0],
        &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Water pump — blue machine that extracts water from water tiles.
fn make_water_pump_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0],
        &[0, 0, 1,25,25,26,26,26,26,26,26,25,25, 1, 0, 0],
        &[0, 1,25,26,26,27,27,28,28,27,27,26,26,25, 1, 0],
        &[1,25,26,27, 3, 3, 4, 4, 4, 4, 3, 3,27,26,25, 1],
        &[1,25,26, 3, 3,25,25,26,26,25,25, 3, 3,26,25, 1],
        &[1,25,26, 3,25,27,27,28,28,27,27,25, 3,26,25, 1],
        &[1,26,27, 4,25,27,28,28,28,28,27,25, 4,27,26, 1],
        &[1,26,27, 4,26,28,28, 5, 5,28,28,26, 4,27,26, 1],
        &[1,26,27, 4,26,28,28, 5, 5,28,28,26, 4,27,26, 1],
        &[1,26,27, 4,25,27,28,28,28,28,27,25, 4,27,26, 1],
        &[1,25,26, 3,25,27,27,28,28,27,27,25, 3,26,25, 1],
        &[1,25,26, 3, 3,25,25,26,26,25,25, 3, 3,26,25, 1],
        &[1,25,26,27, 3, 3, 4, 4, 4, 4, 3, 3,27,26,25, 1],
        &[0, 1,25,26,26,27,27,28,28,27,27,26,26,25, 1, 0],
        &[0, 0, 1,25,25,26,26,26,26,26,26,25,25, 1, 0, 0],
        &[0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Oil refinery — large industrial block with distillation towers.
fn make_oil_refinery_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        &[1, 2, 2, 3, 3, 3, 4, 4, 4, 4, 3, 3, 3, 2, 2, 1],
        &[1, 2, 3,14,14, 3, 4, 5, 5, 4, 3,14,14, 3, 2, 1],
        &[1, 3,14,15,15,14, 3, 4, 4, 3,14,15,15,14, 3, 1],
        &[1, 3,14,15,16,15, 3, 3, 3, 3,14,15,16,15, 3, 1],
        &[1, 3,14,15,16,15,14, 3, 3,14,14,15,16,15, 3, 1],
        &[1, 3,14,15,16,15,14, 2, 2,14,14,15,16,15, 3, 1],
        &[1, 3, 3,14,15,14, 3, 2, 2, 3,14,15,14, 3, 3, 1],
        &[1, 3, 3,14,15,14, 3, 2, 2, 3,14,15,14, 3, 3, 1],
        &[1, 3,14,15,16,15,14, 2, 2,14,14,15,16,15, 3, 1],
        &[1, 3,14,15,16,15,14, 3, 3,14,14,15,16,15, 3, 1],
        &[1, 3,14,15,16,15, 3, 3, 3, 3,14,15,16,15, 3, 1],
        &[1, 3,14,15,15,14, 3, 4, 4, 3,14,15,15,14, 3, 1],
        &[1, 2, 3,14,14, 3, 4, 5, 5, 4, 3,14,14, 3, 2, 1],
        &[1, 2, 2, 3, 3, 3, 4, 4, 4, 4, 3, 3, 3, 2, 2, 1],
        &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    ];
    make_image(pixels, 16)
}

/// Nuclear reactor — heavy containment structure with glowing core.
fn make_nuclear_reactor_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        &[1, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 1],
        &[1, 3, 4, 4, 4, 4, 3, 3, 3, 3, 4, 4, 4, 4, 3, 1],
        &[1, 3, 4, 3, 3, 3, 9, 9, 9, 9, 3, 3, 3, 4, 3, 1],
        &[1, 3, 4, 3, 9, 9,10,10,10,10, 9, 9, 3, 4, 3, 1],
        &[1, 4, 4, 3, 9,10,11,61,61,11,10, 9, 3, 4, 4, 1],
        &[1, 4, 3, 9,10,11,61,62,62,61,11,10, 9, 3, 4, 1],
        &[1, 4, 3, 9,10,61,62,62,62,62,61,10, 9, 3, 4, 1],
        &[1, 4, 3, 9,10,61,62,62,62,62,61,10, 9, 3, 4, 1],
        &[1, 4, 3, 9,10,11,61,62,62,61,11,10, 9, 3, 4, 1],
        &[1, 4, 4, 3, 9,10,11,61,61,11,10, 9, 3, 4, 4, 1],
        &[1, 3, 4, 3, 9, 9,10,10,10,10, 9, 9, 3, 4, 3, 1],
        &[1, 3, 4, 3, 3, 3, 9, 9, 9, 9, 3, 3, 3, 4, 3, 1],
        &[1, 3, 4, 4, 4, 4, 3, 3, 3, 3, 4, 4, 4, 4, 3, 1],
        &[1, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 1],
        &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    ];
    make_image(pixels, 16)
}

/// Accumulator — energy storage cell with charge level indicator.
fn make_accumulator_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
        &[0, 1, 3, 3, 3, 3, 4, 4, 4, 4, 3, 3, 3, 3, 1, 0],
        &[1, 3, 4, 4,25,25,25,26,26,25,25,25, 4, 4, 3, 1],
        &[1, 3, 4,25,26,26,26,27,27,26,26,26,25, 4, 3, 1],
        &[1, 3,25,26,27,27,27,28,28,27,27,27,26,25, 3, 1],
        &[1, 3,25,26,27,28,28,28,28,28,28,27,26,25, 3, 1],
        &[1, 4,25,26,27,28, 5, 5, 5, 5,28,27,26,25, 4, 1],
        &[1, 4,25,27,28,28, 5, 6, 6, 5,28,28,27,25, 4, 1],
        &[1, 4,25,27,28,28, 5, 6, 6, 5,28,28,27,25, 4, 1],
        &[1, 4,25,26,27,28, 5, 5, 5, 5,28,27,26,25, 4, 1],
        &[1, 3,25,26,27,28,28,28,28,28,28,27,26,25, 3, 1],
        &[1, 3,25,26,27,27,27,28,28,27,27,27,26,25, 3, 1],
        &[1, 3, 4,25,26,26,26,27,27,26,26,26,25, 4, 3, 1],
        &[1, 3, 4, 4,25,25,25,26,26,25,25,25, 4, 4, 3, 1],
        &[0, 1, 3, 3, 3, 3, 4, 4, 4, 4, 3, 3, 3, 3, 1, 0],
        &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Radar — rotating dish antenna on a base.
fn make_radar_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 1, 1, 5, 5, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 5, 6, 6, 6, 6, 5, 1, 0, 0, 0, 0],
        &[0, 0, 0, 1, 5, 6, 7, 7, 7, 7, 6, 5, 1, 0, 0, 0],
        &[0, 0, 1, 5, 6, 7, 7, 8, 8, 7, 7, 6, 5, 1, 0, 0],
        &[0, 0, 1, 5, 6, 7, 8, 8, 8, 8, 7, 6, 5, 1, 0, 0],
        &[0, 0, 0, 1, 5, 6, 7, 7, 7, 7, 6, 5, 1, 0, 0, 0],
        &[0, 0, 0, 0, 1, 5, 4, 4, 4, 4, 5, 1, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 1, 3, 4, 4, 3, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 1, 3, 3, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 1, 3, 3, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 1, 1, 1, 1, 3, 3, 1, 1, 1, 1, 0, 0, 0],
        &[0, 0, 1, 3, 3, 3, 3, 4, 4, 3, 3, 3, 3, 1, 0, 0],
        &[0, 1, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 1, 0],
        &[0, 1, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 3, 1, 0],
        &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Pipe — gray conduit for fluid transport.
fn make_pipe_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 4, 4, 5, 5, 4, 4, 1, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 4, 3, 4, 4, 3, 4, 1, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 3, 3, 3, 3, 3, 3, 1, 0, 0, 0, 0],
        &[1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 1, 1, 1, 1, 1],
        &[1, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 1],
        &[1, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 1],
        &[1, 5, 4, 3, 3, 3, 3, 4, 4, 3, 3, 3, 3, 4, 5, 1],
        &[1, 5, 4, 3, 3, 3, 3, 4, 4, 3, 3, 3, 3, 4, 5, 1],
        &[1, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 1],
        &[1, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 1],
        &[1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 1, 1, 1, 1, 1],
        &[0, 0, 0, 0, 1, 3, 3, 3, 3, 3, 3, 1, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 4, 3, 4, 4, 3, 4, 1, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 4, 4, 5, 5, 4, 4, 1, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Roboport — logistics hub with antenna and landing pad.
fn make_roboport_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 1,29,29, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 1,29,30,30,29, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 1, 3, 3, 1, 0, 0, 0, 0, 0, 0],
        &[0, 1, 1, 1, 1, 1, 1, 3, 3, 1, 1, 1, 1, 1, 1, 0],
        &[1, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 1],
        &[1, 3, 4, 4, 4, 4, 5, 5, 5, 5, 4, 4, 4, 4, 3, 1],
        &[1, 3, 4, 5, 5,29,29,30,30,29,29, 5, 5, 4, 3, 1],
        &[1, 3, 4, 5, 5,29,30,30,30,30,29, 5, 5, 4, 3, 1],
        &[1, 3, 4, 4, 4, 4, 5, 5, 5, 5, 4, 4, 4, 4, 3, 1],
        &[1, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 1],
        &[0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
        &[0, 0, 1,17,17, 1, 0, 0, 0, 0, 1,17,17, 1, 0, 0],
        &[0, 0, 1,17,17, 1, 0, 0, 0, 0, 1,17,17, 1, 0, 0],
        &[0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Rail — track segment with ties and rails.
fn make_rail_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0],
        &[0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0],
        &[0, 0, 1,14,14,14,14,14,14,14,14,14,14, 1, 0, 0],
        &[0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0],
        &[0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0],
        &[0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0],
        &[0, 0, 1,14,14,14,14,14,14,14,14,14,14, 1, 0, 0],
        &[0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0],
        &[0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0],
        &[0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0],
        &[0, 0, 1,14,14,14,14,14,14,14,14,14,14, 1, 0, 0],
        &[0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0],
        &[0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0],
        &[0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0],
        &[0, 0, 1,14,14,14,14,14,14,14,14,14,14, 1, 0, 0],
        &[0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Train stop — platform marker with signal lamp.
fn make_train_stop_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0],
        &[0, 0, 1,22,22,23,23,23,23,23,23,22,22, 1, 0, 0],
        &[0, 1,22,23,23,24,24,24,24,24,24,23,23,22, 1, 0],
        &[1,22,23, 3, 3, 3, 3, 4, 4, 3, 3, 3, 3,23,22, 1],
        &[1,22,23, 3, 4, 4, 4, 4, 4, 4, 4, 4, 3,23,22, 1],
        &[1,23,24, 3, 4, 5,61,61,61,61, 5, 4, 3,24,23, 1],
        &[1,23,24, 3, 4,61,62,62,62,62,61, 4, 3,24,23, 1],
        &[1,23,24, 4, 4,61,62,62,62,62,61, 4, 4,24,23, 1],
        &[1,23,24, 4, 4,61,62,62,62,62,61, 4, 4,24,23, 1],
        &[1,23,24, 3, 4,61,62,62,62,62,61, 4, 3,24,23, 1],
        &[1,23,24, 3, 4, 5,61,61,61,61, 5, 4, 3,24,23, 1],
        &[1,22,23, 3, 4, 4, 4, 4, 4, 4, 4, 4, 3,23,22, 1],
        &[1,22,23, 3, 3, 3, 3, 4, 4, 3, 3, 3, 3,23,22, 1],
        &[0, 1,22,23,23,24,24,24,24,24,24,23,23,22, 1, 0],
        &[0, 0, 1,22,22,23,23,23,23,23,23,22,22, 1, 0, 0],
        &[0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Rocket silo — massive launch facility with circular hatch.
fn make_rocket_silo_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        &[1, 2, 2, 2, 3, 3, 3, 4, 4, 3, 3, 3, 2, 2, 2, 1],
        &[1, 2, 3, 3, 3, 4, 4, 4, 4, 4, 4, 3, 3, 3, 2, 1],
        &[1, 2, 3, 4, 4, 4, 3, 3, 3, 3, 4, 4, 4, 3, 2, 1],
        &[1, 3, 3, 4, 3, 3, 2, 2, 2, 2, 3, 3, 4, 3, 3, 1],
        &[1, 3, 4, 4, 3, 2, 1, 1, 1, 1, 2, 3, 4, 4, 3, 1],
        &[1, 3, 4, 3, 2, 1,22,22,22,22, 1, 2, 3, 4, 3, 1],
        &[1, 4, 4, 3, 2, 1,22,23,23,22, 1, 2, 3, 4, 4, 1],
        &[1, 4, 4, 3, 2, 1,22,23,23,22, 1, 2, 3, 4, 4, 1],
        &[1, 3, 4, 3, 2, 1,22,22,22,22, 1, 2, 3, 4, 3, 1],
        &[1, 3, 4, 4, 3, 2, 1, 1, 1, 1, 2, 3, 4, 4, 3, 1],
        &[1, 3, 3, 4, 3, 3, 2, 2, 2, 2, 3, 3, 4, 3, 3, 1],
        &[1, 2, 3, 4, 4, 4, 3, 3, 3, 3, 4, 4, 4, 3, 2, 1],
        &[1, 2, 3, 3, 3, 4, 4, 4, 4, 4, 4, 3, 3, 3, 2, 1],
        &[1, 2, 2, 2, 3, 3, 3, 4, 4, 3, 3, 3, 2, 2, 2, 1],
        &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    ];
    make_image(pixels, 16)
}

/// Beacon — module broadcaster with purple energy field.
fn make_beacon_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 1, 1, 3, 3, 4, 4, 3, 3, 1, 1, 0, 0, 0],
        &[0, 0, 1, 3, 3, 4,29,29,29,29, 4, 3, 3, 1, 0, 0],
        &[0, 1, 3, 4,29,29,30,30,30,30,29,29, 4, 3, 1, 0],
        &[0, 1, 3,29,30,30,31,31,31,31,30,30,29, 3, 1, 0],
        &[1, 3, 4,29,30,31,32,32,32,32,31,30,29, 4, 3, 1],
        &[1, 3,29,30,31,32,32, 5, 5,32,32,31,30,29, 3, 1],
        &[1, 4,29,30,31,32, 5, 6, 6, 5,32,31,30,29, 4, 1],
        &[1, 4,29,30,31,32, 5, 6, 6, 5,32,31,30,29, 4, 1],
        &[1, 3,29,30,31,32,32, 5, 5,32,32,31,30,29, 3, 1],
        &[1, 3, 4,29,30,31,32,32,32,32,31,30,29, 4, 3, 1],
        &[0, 1, 3,29,30,30,31,31,31,31,30,30,29, 3, 1, 0],
        &[0, 1, 3, 4,29,29,30,30,30,30,29,29, 4, 3, 1, 0],
        &[0, 0, 1, 3, 3, 4,29,29,29,29, 4, 3, 3, 1, 0, 0],
        &[0, 0, 0, 1, 1, 3, 3, 4, 4, 3, 3, 1, 1, 0, 0, 0],
        &[0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Electric furnace — advanced smelter with blue energy coils.
fn make_electric_furnace_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
        &[0, 1, 3, 3, 3, 4, 4, 4, 4, 4, 4, 3, 3, 3, 1, 0],
        &[1, 3, 4, 4,25,25,26,26,26,26,25,25, 4, 4, 3, 1],
        &[1, 3, 4,25,26,27,27,27,27,27,27,26,25, 4, 3, 1],
        &[1, 3,25,26,27, 3, 3, 3, 3, 3, 3,27,26,25, 3, 1],
        &[1, 4,25,27, 3, 3,22,22,22,22, 3, 3,27,25, 4, 1],
        &[1, 4,26,27, 3,22,23,24,24,23,22, 3,27,26, 4, 1],
        &[1, 4,26,27, 3,22,24,24,24,24,22, 3,27,26, 4, 1],
        &[1, 4,26,27, 3,22,24,24,24,24,22, 3,27,26, 4, 1],
        &[1, 4,26,27, 3,22,23,24,24,23,22, 3,27,26, 4, 1],
        &[1, 4,25,27, 3, 3,22,22,22,22, 3, 3,27,25, 4, 1],
        &[1, 3,25,26,27, 3, 3, 3, 3, 3, 3,27,26,25, 3, 1],
        &[1, 3, 4,25,26,27,27,27,27,27,27,26,25, 4, 3, 1],
        &[1, 3, 4, 4,25,25,26,26,26,26,25,25, 4, 4, 3, 1],
        &[0, 1, 3, 3, 3, 4, 4, 4, 4, 4, 4, 3, 3, 3, 1, 0],
        &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Pump jack — oil extractor with rotating arm.
fn make_pump_jack_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 1, 1, 3, 3, 4, 4, 3, 3, 1, 1, 0, 0, 0],
        &[0, 0, 1, 3, 3, 4,14,14,14,14, 4, 3, 3, 1, 0, 0],
        &[0, 1, 3, 4,14,15,15,16,16,15,15,14, 4, 3, 1, 0],
        &[0, 1, 3,14,15,16, 3, 3, 3, 3,16,15,14, 3, 1, 0],
        &[1, 3, 4,14,15, 3, 3, 4, 4, 3, 3,15,14, 4, 3, 1],
        &[1, 3, 4,14, 3, 3, 4, 4, 4, 4, 3, 3,14, 4, 3, 1],
        &[1, 3, 4, 3, 3, 4, 4, 1, 1, 4, 4, 3, 3, 4, 3, 1],
        &[1, 3, 4, 3, 3, 4, 4, 1, 1, 4, 4, 3, 3, 4, 3, 1],
        &[1, 3, 4,14, 3, 3, 4, 4, 4, 4, 3, 3,14, 4, 3, 1],
        &[1, 3, 4,14,15, 3, 3, 4, 4, 3, 3,15,14, 4, 3, 1],
        &[0, 1, 3,14,15,16, 3, 3, 3, 3,16,15,14, 3, 1, 0],
        &[0, 1, 3, 4,14,15,15,16,16,15,15,14, 4, 3, 1, 0],
        &[0, 0, 1, 3, 3, 4,14,14,14,14, 4, 3, 3, 1, 0, 0],
        &[0, 0, 0, 1, 1, 3, 3, 4, 4, 3, 3, 1, 1, 0, 0, 0],
        &[0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Laser turret — sleek blue energy weapon.
fn make_laser_turret_sprite() -> Image {
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 1, 1,25,25,26,26,25,25, 1, 1, 0, 0, 0],
        &[0, 0, 1, 3,25,26,27,28,28,27,26,25, 3, 1, 0, 0],
        &[0, 1, 3, 3, 3,25,26,27,27,26,25, 3, 3, 3, 1, 0],
        &[0, 1, 3, 3, 3, 3,25,26,26,25, 3, 3, 3, 3, 1, 0],
        &[1, 3, 3, 3, 3, 3, 3, 4, 4, 3, 3, 3, 3, 3, 3, 1],
        &[1, 3, 4, 4, 3, 3, 4, 4, 4, 4, 3, 3, 4, 4, 3, 1],
        &[1, 3, 4, 4, 4, 4, 4, 5, 5, 4, 4, 4, 4, 4, 3, 1],
        &[1, 3, 4, 4, 4, 4, 4, 5, 5, 4, 4, 4, 4, 4, 3, 1],
        &[1, 3, 4, 4, 3, 3, 4, 4, 4, 4, 3, 3, 4, 4, 3, 1],
        &[1, 3, 3, 3, 3, 3, 3, 4, 4, 3, 3, 3, 3, 3, 3, 1],
        &[0, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 1, 0],
        &[0, 1, 3, 3, 3, 4, 4, 4, 4, 4, 4, 3, 3, 3, 1, 0],
        &[0, 0, 1, 3, 4, 4, 4, 4, 4, 4, 4, 4, 3, 1, 0, 0],
        &[0, 0, 0, 1, 1, 3, 3, 3, 3, 3, 3, 1, 1, 0, 0, 0],
        &[0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// FORGE avatar — cute round robot AI face (24×24).
/// Purple body, big white eyes with pupils, antenna with glowing tip, smile.
/// Frame 0: happy (open eyes). Frame 1: blink (^_^ face).
fn make_forge_avatar_sprite(frame: u8) -> Image {
    #[rustfmt::skip]
    let base: Vec<Vec<u8>> = vec![
        // Row 0-2: antenna
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        // Row 3-4: top of head
        vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 1,31,31,32,32,32,32,32,32,32,31,31, 1, 0, 0, 0, 0, 0, 0],
        // Row 5-6: forehead
        vec![0, 0, 0, 0, 1,31,32,32,32,32,32,32,32,32,32,32,32,31, 1, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 1,31,32,32,32,32,32,32,32,32,32,32,32,32,32,31, 1, 0, 0, 0, 0],
        // Row 7-10: eyes region
        vec![0, 0, 0, 1,30,32,32, 7, 7, 8, 7,32,32, 7, 7, 8, 7,32,30, 1, 0, 0, 0, 0],
        vec![0, 0, 0, 1,30,32, 7, 8, 8, 8, 8, 7,32, 8, 8, 8, 8, 7,30, 1, 0, 0, 0, 0],
        vec![0, 0, 0, 1,30,32, 7, 8,29, 1, 8, 7,32, 8,29, 1, 8, 7,30, 1, 0, 0, 0, 0],
        vec![0, 0, 0, 1,30,32, 7, 8, 1, 1, 8, 7,32, 8, 1, 1, 8, 7,30, 1, 0, 0, 0, 0],
        // Row 11-12: cheeks + blush
        vec![0, 0, 0, 1,30,32,32, 7, 7, 7, 7,32,32, 7, 7, 7, 7,32,30, 1, 0, 0, 0, 0],
        vec![0, 0, 0, 1,30,32,23,32,32,32,32,32,32,32,32,32,32,23,30, 1, 0, 0, 0, 0],
        // Row 13-14: mouth (happy smile)
        vec![0, 0, 0, 1,30,32,32,32,32, 1, 1, 1, 1, 1, 1,32,32,32,30, 1, 0, 0, 0, 0],
        vec![0, 0, 0, 1,30,32,32,32,32,32, 1,30,30, 1,32,32,32,32,30, 1, 0, 0, 0, 0],
        // Row 15-17: chin + body top
        vec![0, 0, 0, 0, 1,31,32,32,32,32,32,32,32,32,32,32,32,31, 1, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 1,31,31,31,30,30,30,30,30,31,31,31, 1, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 1,30,30,29,29,29,29,29,30,30, 1, 0, 0, 0, 0, 0, 0, 0],
        // Row 18-19: body middle with chest light
        vec![0, 0, 0, 0, 0, 1,30,29,29,29, 5, 6, 5,29,29,29,30, 1, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 1,30,29,29, 5, 6, 8, 6, 5,29,29,30, 1, 0, 0, 0, 0, 0, 0],
        // Row 20-21: arms
        vec![0, 0, 0, 0, 1,30,29, 1,29,29, 5, 6, 5,29,29, 1,29,30, 1, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 1,30, 1, 0, 0, 1,29,29,29,29,29, 1, 0, 0, 1,30, 1, 0, 0, 0, 0],
        // Row 22-23: feet
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 1,29,29,29,29, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let refs: Vec<&[u8]> = base.iter().map(|r| r.as_slice()).collect();
    let mut img = make_image_rect(&refs, 24, 24);

    if frame == 1 {
        // Blink frame: replace eye region with closed ^_^ lines.
        for y in 8..11u32 {
            for x in 7..11u32 { img.set_pixel(x, y, pal_color(32)); }
            for x in 13..17u32 { img.set_pixel(x, y, pal_color(32)); }
        }
        // Draw ^_^ (left eye)
        img.set_pixel(7, 9, pal_color(1));
        img.set_pixel(8, 8, pal_color(1));
        img.set_pixel(9, 9, pal_color(1));
        // Draw ^_^ (right eye)
        img.set_pixel(13, 9, pal_color(1));
        img.set_pixel(14, 8, pal_color(1));
        img.set_pixel(15, 9, pal_color(1));
    }

    img
}
