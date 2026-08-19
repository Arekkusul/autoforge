//! Pass 1+2: ground terrain sprites/rects and ore deposit overlays.

use macroquad::prelude::*;

use crate::constants::*;
use crate::grid::Grid;
use crate::sprites::SpriteAtlas;
use crate::types::*;

/// Draws ground terrain (Pass 1) and ore deposit overlays (Pass 2), frustum-culled
/// to the given tile range.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_terrain_pass(
    grid: &Grid,
    atlas: &SpriteAtlas,
    x_start: i32,
    x_end: i32,
    y_start: i32,
    y_end: i32,
    lod: i32,
    tick: u64,
) {
    // --- Pass 1: Ground terrain ---
    // Performance strategy:
    // - LOD 0: Full sprite textures (standard quality)
    // - LOD 1: Colored rectangles (no texture switch, massively faster)
    // - LOD 2: Colored rects, skip every other tile (4x fewer draws)
    let step = if lod >= 2 { 2 } else { 1 };
    let tile_draw_size = if lod >= 2 { TILE_SIZE * 2.0 } else { TILE_SIZE };

    for y in (y_start..=y_end).step_by(step as usize) {
        for x in (x_start..=x_end).step_by(step as usize) {
            let pos = GridPos::new(x, y);
            let world = Grid::grid_to_world(pos);

            if let Some(tile) = grid.get_tile(pos) {
                // At LOD 1+, use fast colored rectangles instead of textured sprites.
                // This eliminates texture switches — macroquad batches all rects in one draw call.
                if lod >= 1 {
                    let color = match tile.terrain {
                        Terrain::Grass => Color::new(0.2, 0.32, 0.2, 1.0),
                        Terrain::Desert => Color::new(0.4, 0.35, 0.2, 1.0),
                        Terrain::Forest => Color::new(0.1, 0.18, 0.1, 1.0),
                        Terrain::Water => Color::new(0.15, 0.2, 0.4, 1.0),
                        Terrain::Cliff => Color::new(0.3, 0.3, 0.28, 1.0),
                    };
                    draw_rectangle(world.x, world.y, tile_draw_size, tile_draw_size, color);

                    // Pollution overlay (simplified).
                    if tile.pollution > 0.1 {
                        let alpha = (tile.pollution * 0.08).min(0.5);
                        draw_rectangle(
                            world.x,
                            world.y,
                            tile_draw_size,
                            tile_draw_size,
                            Color::new(0.35, 0.4, 0.1, alpha),
                        );
                    }
                    continue;
                }

                // Draw ground from the unified atlas (1 texture = all batched).
                let source_rect = match tile.terrain {
                    Terrain::Grass => {
                        if (x + y) % 3 == 0 {
                            atlas.r_ground_grass_alt
                        } else {
                            atlas.r_ground_grass
                        }
                    }
                    Terrain::Desert => atlas.r_ground_desert,
                    Terrain::Forest => atlas.r_ground_forest,
                    Terrain::Water => {
                        if (tick / 10 + (x as u64) + (y as u64)).is_multiple_of(2) {
                            atlas.r_ground_water
                        } else {
                            atlas.r_ground_water_alt
                        }
                    }
                    Terrain::Cliff => atlas.r_ground_grass,
                };

                // All ground draws use atlas.tex with source rects.
                // macroquad batches ALL of these into 1 GPU draw call.
                draw_texture_ex(
                    &atlas.tex,
                    world.x,
                    world.y,
                    WHITE,
                    DrawTextureParams {
                        source: Some(source_rect),
                        dest_size: Some(Vec2::splat(tile_draw_size)),
                        ..Default::default()
                    },
                );

                // --- Pollution haze overlay (green-brown smog) ---
                if tile.pollution > 0.05 {
                    let alpha = (tile.pollution * 0.08).min(0.7);
                    // Use tile_draw_size for LOD 2 compatibility.
                    draw_rectangle(
                        world.x,
                        world.y,
                        tile_draw_size,
                        tile_draw_size,
                        Color::new(0.35, 0.4, 0.1, alpha),
                    );
                }
            }
        }
    }

    // --- Pass 2: Ore deposit overlays (separate pass so 2×2 rocks aren't covered by ground) ---
    for y in y_start..=y_end {
        for x in x_start..=x_end {
            let pos = GridPos::new(x, y);
            if let Some(tile) = grid.get_tile(pos) {
                if tile.ore_origin {
                    if let Some(deposit) = tile.deposit {
                        let ore_src = match deposit {
                            OreDeposit::Iron => atlas.r_ore_iron,
                            OreDeposit::Copper => atlas.r_ore_copper,
                            OreDeposit::Coal => atlas.r_ore_coal,
                            OreDeposit::Stone => atlas.r_ore_stone,
                            OreDeposit::Uranium => atlas.r_ore_uranium,
                            OreDeposit::Tin => atlas.r_ore_tin,
                            OreDeposit::Gold => atlas.r_ore_gold,
                            OreDeposit::Sulfur => atlas.r_ore_sulfur,
                            OreDeposit::Crystal => atlas.r_ore_crystal,
                            OreDeposit::Oil => atlas.r_ore_oil,
                        };
                        let world = Grid::grid_to_world(pos);
                        // Fade ore sprite as deposit depletes (dims below 2000, fades below 500).
                        let ore_tint = if tile.ore_amount == u32::MAX {
                            WHITE
                        } else if tile.ore_amount < 500 {
                            let fade = tile.ore_amount as f32 / 500.0;
                            Color::new(1.0, 0.6 + fade * 0.4, 0.6 + fade * 0.4, 0.5 + fade * 0.5)
                        } else if tile.ore_amount < 2000 {
                            let dim = tile.ore_amount as f32 / 2000.0;
                            Color::new(1.0, 0.8 + dim * 0.2, 0.8 + dim * 0.2, 1.0)
                        } else {
                            WHITE
                        };
                        draw_texture_ex(
                            &atlas.tex,
                            world.x,
                            world.y,
                            ore_tint,
                            DrawTextureParams {
                                source: Some(ore_src),
                                dest_size: Some(Vec2::splat(TILE_SIZE * 2.0)),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }
    }
}
