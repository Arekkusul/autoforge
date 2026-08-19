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

mod belts;
mod buildings_advanced;
mod buildings_core;
mod creatures;
mod helpers;
mod items;
mod power_defense;
mod terrain;

use macroquad::prelude::*;

use belts::*;
use buildings_advanced::*;
use buildings_core::*;
use creatures::*;
use items::*;
use power_defense::*;
use terrain::*;

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

    // Robot worker (10×10, 2 flight animation frames)
    pub r_robot: [Rect; 2],
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
            Rect::new(
                ax as f32,
                ay as f32,
                sprite.width() as f32,
                sprite.height() as f32,
            )
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
        let r_ground_grass = pack(&mut atlas_img, &make_ground_image(10, 11, 12), 4, 0);
        let r_ground_grass_alt = pack(&mut atlas_img, &make_ground_image(10, 12, 11), 4 + s, 0);
        let r_ground_desert = pack(&mut atlas_img, &make_ground_image(16, 15, 14), 4 + s * 2, 0);
        let r_ground_forest = pack(&mut atlas_img, &make_forest_image(), 4 + s * 3, 0);
        let r_ground_water = pack(&mut atlas_img, &make_water_image(25, 27), 4 + s * 4, 0);
        let r_ground_water_alt = pack(&mut atlas_img, &make_water_image(26, 28), 4 + s * 5, 0);

        // Row 1: Ore deposits
        let y1 = 17u32;
        let r_ore_iron = pack(&mut atlas_img, &make_ore_sprite(6, 7), 0, y1);
        let r_ore_copper = pack(&mut atlas_img, &make_ore_sprite(7, 28), s, y1);
        let r_ore_coal = pack(&mut atlas_img, &make_ore_sprite(27, 2), s * 2, y1);
        let r_ore_stone = pack(&mut atlas_img, &make_ore_sprite(25, 26), s * 3, y1);
        let r_ore_uranium = pack(&mut atlas_img, &make_ore_sprite(22, 9), s * 4, y1);
        let r_ore_tin = pack(&mut atlas_img, &make_ore_sprite(4, 5), s * 5, y1);
        let r_ore_gold = pack(&mut atlas_img, &make_ore_sprite(30, 16), s * 6, y1);
        let r_ore_sulfur = pack(&mut atlas_img, &make_ore_sprite(16, 31), s * 7, y1);
        let r_ore_crystal = pack(&mut atlas_img, &make_ore_sprite(19, 5), s * 8, y1);
        let r_ore_oil = pack(&mut atlas_img, &make_oil_sprite(), s * 9, y1);

        // Row 2: Machines & structures (frame 0)
        let y2 = 34u32;
        let r_miner_0 = pack(&mut atlas_img, &make_miner_sprite(0), 0, y2);
        let r_stone_furnace_0 = pack(&mut atlas_img, &make_stone_furnace_sprite(0), s, y2);
        let r_steel_furnace_0 = pack(&mut atlas_img, &make_steel_furnace_sprite(0), s * 2, y2);
        let r_assembler_0 = pack(&mut atlas_img, &make_assembler_sprite(0), s * 3, y2);
        let r_lab_0 = pack(&mut atlas_img, &make_lab_sprite(0), s * 4, y2);
        let r_boiler = pack(&mut atlas_img, &make_boiler_sprite(), s * 5, y2);
        let r_steam_engine = pack(&mut atlas_img, &make_steam_engine_sprite(), s * 6, y2);
        let r_solar_panel = pack(&mut atlas_img, &make_solar_panel_sprite(), s * 7, y2);
        let r_chest = pack(&mut atlas_img, &make_chest_sprite(), s * 8, y2);
        let r_gun_turret = pack(&mut atlas_img, &make_gun_turret_sprite(), s * 9, y2);
        let r_wall = pack(&mut atlas_img, &make_wall_sprite(), s * 10, y2);
        let r_inserter = pack(&mut atlas_img, &make_inserter_sprite(), s * 11, y2);

        // Row 3: Belts straight (3 colors × 2 frames)
        let y3 = 51u32;
        let r_belt_yellow = [
            pack(&mut atlas_img, &make_belt_sprite(17, 16, 0), 0, y3),
            pack(&mut atlas_img, &make_belt_sprite(17, 16, 1), s, y3),
        ];
        let r_belt_red = [
            pack(&mut atlas_img, &make_belt_sprite(10, 11, 0), s * 2, y3),
            pack(&mut atlas_img, &make_belt_sprite(10, 11, 1), s * 3, y3),
        ];
        let r_belt_blue = [
            pack(&mut atlas_img, &make_belt_sprite(12, 13, 0), s * 4, y3),
            pack(&mut atlas_img, &make_belt_sprite(12, 13, 1), s * 5, y3),
        ];

        // Row 4: Belt corners left
        let y4 = 68u32;
        let r_belt_corner_left_yellow = [
            pack(
                &mut atlas_img,
                &make_belt_corner_sprite(17, 16, 0, false),
                0,
                y4,
            ),
            pack(
                &mut atlas_img,
                &make_belt_corner_sprite(17, 16, 1, false),
                s,
                y4,
            ),
        ];
        let r_belt_corner_left_red = [
            pack(
                &mut atlas_img,
                &make_belt_corner_sprite(10, 11, 0, false),
                s * 2,
                y4,
            ),
            pack(
                &mut atlas_img,
                &make_belt_corner_sprite(10, 11, 1, false),
                s * 3,
                y4,
            ),
        ];
        let r_belt_corner_left_blue = [
            pack(
                &mut atlas_img,
                &make_belt_corner_sprite(12, 13, 0, false),
                s * 4,
                y4,
            ),
            pack(
                &mut atlas_img,
                &make_belt_corner_sprite(12, 13, 1, false),
                s * 5,
                y4,
            ),
        ];

        // Row 5: Belt corners right
        let y5 = 85u32;
        let r_belt_corner_right_yellow = [
            pack(
                &mut atlas_img,
                &make_belt_corner_sprite(17, 16, 0, true),
                0,
                y5,
            ),
            pack(
                &mut atlas_img,
                &make_belt_corner_sprite(17, 16, 1, true),
                s,
                y5,
            ),
        ];
        let r_belt_corner_right_red = [
            pack(
                &mut atlas_img,
                &make_belt_corner_sprite(10, 11, 0, true),
                s * 2,
                y5,
            ),
            pack(
                &mut atlas_img,
                &make_belt_corner_sprite(10, 11, 1, true),
                s * 3,
                y5,
            ),
        ];
        let r_belt_corner_right_blue = [
            pack(
                &mut atlas_img,
                &make_belt_corner_sprite(12, 13, 0, true),
                s * 4,
                y5,
            ),
            pack(
                &mut atlas_img,
                &make_belt_corner_sprite(12, 13, 1, true),
                s * 5,
                y5,
            ),
        ];

        // Row 6: Items (8×8, stride=9)
        let y6 = 102u32;
        let si = 9u32;
        let r_item_iron_ore = pack(&mut atlas_img, &make_item_sprite(14, 16), 0, y6);
        let r_item_copper_ore = pack(&mut atlas_img, &make_item_sprite(50, 52), si, y6);
        let r_item_coal = pack(&mut atlas_img, &make_coal_item_sprite(), si * 2, y6);
        let r_item_stone = pack(&mut atlas_img, &make_stone_item_sprite(), si * 3, y6);
        let r_item_iron_plate = pack(&mut atlas_img, &make_plate_item_sprite(3, 4), si * 4, y6);
        let r_item_copper_plate = pack(&mut atlas_img, &make_plate_item_sprite(28, 15), si * 5, y6);
        let r_item_gear = pack(&mut atlas_img, &make_gear_item_sprite(), si * 6, y6);
        let r_item_wire = pack(&mut atlas_img, &make_wire_item_sprite(), si * 7, y6);
        let r_item_green_circuit = pack(&mut atlas_img, &make_circuit_item_sprite(), si * 8, y6);
        let r_item_science_red = pack(&mut atlas_img, &make_flask_item_sprite(10, 11), si * 9, y6);
        // Extra items continued on same row (10+)
        let r_item_steel_plate = pack(&mut atlas_img, &make_plate_item_sprite(5, 6), si * 10, y6); // silver
        let r_item_stone_brick = pack(&mut atlas_img, &make_item_sprite(47, 48), si * 11, y6); // tan
        let r_item_pipe = pack(&mut atlas_img, &make_item_sprite(3, 4), si * 12, y6); // gray
        let r_item_red_circuit = pack(
            &mut atlas_img,
            &make_circuit_item_sprite_colored(22, 23),
            si * 13,
            y6,
        );
        let r_item_blue_circuit = pack(
            &mut atlas_img,
            &make_circuit_item_sprite_colored(25, 27),
            si * 14,
            y6,
        );
        let r_item_science_green =
            pack(&mut atlas_img, &make_flask_item_sprite(9, 10), si * 15, y6); // green
        let r_item_science_blue =
            pack(&mut atlas_img, &make_flask_item_sprite(25, 27), si * 16, y6); // blue
        let r_item_science_purple =
            pack(&mut atlas_img, &make_flask_item_sprite(29, 31), si * 17, y6); // purple
        let r_item_sulfur = pack(&mut atlas_img, &make_item_sprite(16, 31), si * 18, y6); // yellow
        let r_item_plastic = pack(&mut atlas_img, &make_item_sprite(5, 7), si * 19, y6); // white
                                                                                         // Row 6b: more items
        let y6b = y6 + si;
        let r_item_battery = pack(&mut atlas_img, &make_battery_item_sprite(), 0, y6b);
        let r_item_ammo = pack(&mut atlas_img, &make_ammo_item_sprite(), si, y6b);
        let r_item_grenade = pack(&mut atlas_img, &make_grenade_item_sprite(), si * 2, y6b);
        let r_item_engine = pack(&mut atlas_img, &make_engine_item_sprite(), si * 3, y6b);
        let r_item_rocket_part = pack(&mut atlas_img, &make_rocket_part_item_sprite(), si * 4, y6b);
        let r_item_rocket_fuel = pack(&mut atlas_img, &make_item_sprite(17, 19), si * 5, y6b); // orange
        let r_item_inserter = pack(&mut atlas_img, &make_item_sprite(3, 17), si * 6, y6b); // gray+orange
        let r_item_iron_stick = pack(&mut atlas_img, &make_item_sprite(3, 5), si * 7, y6b); // thin bar
        let r_item_speed_module = pack(&mut atlas_img, &make_item_sprite(25, 28), si * 8, y6b); // blue module
        let r_item_science_yellow =
            pack(&mut atlas_img, &make_flask_item_sprite(16, 17), si * 9, y6b); // yellow
        let r_item_uranium_ore = pack(&mut atlas_img, &make_item_sprite(9, 62), si * 10, y6b); // green glow
        let r_item_rail = pack(&mut atlas_img, &make_item_sprite(14, 4), si * 11, y6b); // brown+gray
        let r_item_concrete = pack(&mut atlas_img, &make_item_sprite(3, 2), si * 12, y6b); // dark gray
        let r_item_solar_panel = pack(&mut atlas_img, &make_item_sprite(25, 8), si * 13, y6b); // blue+white
        let r_item_accumulator = pack(&mut atlas_img, &make_item_sprite(25, 6), si * 14, y6b); // blue+pale
        let r_item_robot_frame = pack(&mut atlas_img, &make_item_sprite(4, 29), si * 15, y6b); // gray+purple
        let r_item_uranium_235 = pack(&mut atlas_img, &make_item_sprite(62, 9), si * 16, y6b); // bright green
        let r_item_uranium_238 = pack(&mut atlas_img, &make_item_sprite(9, 2), si * 17, y6b); // dull green
        let r_item_nuclear_fuel = pack(&mut atlas_img, &make_item_sprite(62, 61), si * 18, y6b); // hot green
        let r_item_low_density = pack(&mut atlas_img, &make_item_sprite(5, 7), si * 19, y6b); // silver-white

        // Row 7: Enemies (2 walk frames each) + crashed ship
        let y7 = 120u32;
        // Small biter (red-brown)
        let r_enemy_0 = pack(&mut atlas_img, &make_enemy_sprite(23, 24, 0), 0, y7);
        let r_enemy_1 = pack(&mut atlas_img, &make_enemy_sprite(23, 24, 1), 14, y7);
        // Big biter (dark armored)
        let r_big_biter_0 = pack(&mut atlas_img, &make_big_biter_sprite(1, 2, 0), 28, y7);
        let r_big_biter_1 = pack(&mut atlas_img, &make_big_biter_sprite(1, 2, 1), 44, y7);
        // Spitter (green, ranged)
        let r_spitter_0 = pack(&mut atlas_img, &make_spitter_sprite(9, 10, 0), 60, y7);
        let r_spitter_1 = pack(&mut atlas_img, &make_spitter_sprite(9, 10, 1), 76, y7);
        let r_crashed_ship = pack(&mut atlas_img, &make_crashed_ship_sprite(), 92, y7);

        // Row 8-9: Additional buildings (16×16)
        let y8 = 170u32;
        let r_underground_belt = pack(&mut atlas_img, &make_underground_belt_sprite(), 0, y8);
        let r_splitter = pack(&mut atlas_img, &make_splitter_sprite(), s, y8);
        let r_chemical_plant = pack(&mut atlas_img, &make_chemical_plant_sprite(), s * 2, y8);
        let r_water_pump = pack(&mut atlas_img, &make_water_pump_sprite(), s * 3, y8);
        let r_oil_refinery = pack(&mut atlas_img, &make_oil_refinery_sprite(), s * 4, y8);
        let r_nuclear_reactor = pack(&mut atlas_img, &make_nuclear_reactor_sprite(), s * 5, y8);
        let r_accumulator = pack(&mut atlas_img, &make_accumulator_sprite(), s * 6, y8);
        let r_radar = pack(&mut atlas_img, &make_radar_sprite(), s * 7, y8);
        let r_pipe = pack(&mut atlas_img, &make_pipe_sprite(), s * 8, y8);
        let r_roboport = pack(&mut atlas_img, &make_roboport_sprite(), s * 9, y8);

        let y9 = 187u32;
        let r_rail = pack(&mut atlas_img, &make_rail_sprite(), 0, y9);
        let r_train_stop = pack(&mut atlas_img, &make_train_stop_sprite(), s, y9);
        let r_rocket_silo = pack(&mut atlas_img, &make_rocket_silo_sprite(), s * 2, y9);
        let r_beacon = pack(&mut atlas_img, &make_beacon_sprite(), s * 3, y9);
        let r_electric_furnace = pack(&mut atlas_img, &make_electric_furnace_sprite(), s * 4, y9);
        let r_pump_jack = pack(&mut atlas_img, &make_pump_jack_sprite(), s * 5, y9);
        let r_laser_turret = pack(&mut atlas_img, &make_laser_turret_sprite(), s * 6, y9);

        // Row 10: Machine animation frame 1 (active state variants)
        let y10 = 204u32;
        let r_miner_1 = pack(&mut atlas_img, &make_miner_sprite(1), 0, y10);
        let r_stone_furnace_1 = pack(&mut atlas_img, &make_stone_furnace_sprite(1), s, y10);
        let r_steel_furnace_1 = pack(&mut atlas_img, &make_steel_furnace_sprite(1), s * 2, y10);
        let r_assembler_1 = pack(&mut atlas_img, &make_assembler_sprite(1), s * 3, y10);
        let r_lab_1 = pack(&mut atlas_img, &make_lab_sprite(1), s * 4, y10);

        // Row 11: FORGE avatar (24×24, 2 expressions) + Robot worker (10×10, 2 frames)
        let y11 = 221u32;
        let r_forge_0 = pack(&mut atlas_img, &make_forge_avatar_sprite(0), 0, y11);
        let r_forge_1 = pack(&mut atlas_img, &make_forge_avatar_sprite(1), 25, y11);
        let r_robot_0 = pack(&mut atlas_img, &make_robot_sprite(0), 50, y11);
        let r_robot_1 = pack(&mut atlas_img, &make_robot_sprite(1), 61, y11);

        // Combine into 2-frame arrays.
        let r_miner = [r_miner_0, r_miner_1];
        let r_stone_furnace = [r_stone_furnace_0, r_stone_furnace_1];
        let r_steel_furnace = [r_steel_furnace_0, r_steel_furnace_1];
        let r_assembler = [r_assembler_0, r_assembler_1];
        let r_lab = [r_lab_0, r_lab_1];
        let r_enemy_small_biter = [r_enemy_0, r_enemy_1];
        let r_robot = [r_robot_0, r_robot_1];
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

            r_ground_grass,
            r_ground_grass_alt,
            r_ground_desert,
            r_ground_forest,
            r_ground_water,
            r_ground_water_alt,

            r_ore_iron,
            r_ore_copper,
            r_ore_coal,
            r_ore_stone,
            r_ore_uranium,
            r_ore_tin,
            r_ore_gold,
            r_ore_sulfur,
            r_ore_crystal,
            r_ore_oil,

            r_belt_yellow,
            r_belt_red,
            r_belt_blue,
            r_belt_corner_left_yellow,
            r_belt_corner_left_red,
            r_belt_corner_left_blue,
            r_belt_corner_right_yellow,
            r_belt_corner_right_red,
            r_belt_corner_right_blue,

            r_miner,
            r_stone_furnace,
            r_steel_furnace,
            r_assembler,
            r_lab,
            r_boiler,
            r_steam_engine,
            r_solar_panel,
            r_chest,
            r_gun_turret,
            r_laser_turret,
            r_wall,
            r_inserter,
            r_underground_belt,
            r_splitter,
            r_chemical_plant,
            r_water_pump,
            r_oil_refinery,
            r_nuclear_reactor,
            r_accumulator,
            r_radar,
            r_pipe,
            r_roboport,
            r_rail,
            r_train_stop,
            r_rocket_silo,
            r_beacon,
            r_electric_furnace,
            r_pump_jack,

            r_item_iron_ore,
            r_item_copper_ore,
            r_item_coal,
            r_item_stone,
            r_item_iron_plate,
            r_item_copper_plate,
            r_item_gear,
            r_item_wire,
            r_item_green_circuit,
            r_item_science_red,
            r_item_steel_plate,
            r_item_stone_brick,
            r_item_pipe,
            r_item_red_circuit,
            r_item_blue_circuit,
            r_item_science_green,
            r_item_science_blue,
            r_item_science_purple,
            r_item_sulfur,
            r_item_plastic,
            r_item_battery,
            r_item_ammo,
            r_item_grenade,
            r_item_engine,
            r_item_rocket_part,
            r_item_rocket_fuel,
            r_item_inserter,
            r_item_iron_stick,
            r_item_speed_module,
            r_item_science_yellow,
            r_item_uranium_ore,
            r_item_rail,
            r_item_concrete,
            r_item_solar_panel,
            r_item_accumulator,
            r_item_robot_frame,
            r_item_uranium_235,
            r_item_uranium_238,
            r_item_nuclear_fuel,
            r_item_low_density,

            r_enemy_small_biter,
            r_enemy_big_biter,
            r_enemy_spitter,
            r_crashed_ship,
            r_robot,
            r_forge_avatar,
        }
    }
}
