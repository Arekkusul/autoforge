//! World and entity rendering.
//!
//! All drawing is split into two passes:
//! 1. **World-space** (under camera transform): terrain, buildings, items, enemies, ghost preview
//! 2. **Screen-space** (UI overlay): handled by [`crate::ui`]
//!
//! Performance: only tiles within the camera's visible bounds are drawn (frustum culling).

use macroquad::prelude::*;

use crate::building::Buildings;
use crate::camera::GameCamera;
use crate::constants::*;
use crate::enemy::Enemies;
use crate::grid::Grid;
use crate::item::ItemPool;
use crate::sprites::SpriteAtlas;
use crate::types::*;

/// Draws all world-space elements: terrain, ore overlays, buildings, items, and grid lines.
///
/// Call this after setting the camera with [`set_camera`].
pub fn draw_world(
    grid: &Grid,
    buildings: &Buildings,
    items: &ItemPool,
    enemies: &Enemies,
    camera: &GameCamera,
    atlas: &SpriteAtlas,
    tick: u64,
    power_satisfaction: f32,
) {
    let (min_world, max_world) = camera.visible_bounds();

    // Convert to tile range, with 1-tile margin for partially visible tiles.
    let min_tile = Grid::world_to_grid(min_world);
    let max_tile = Grid::world_to_grid(max_world);

    let x_start = (min_tile.x - 1).max(0);
    let x_end = (max_tile.x + 1).min(grid.width - 1);
    let y_start = (min_tile.y - 1).max(0);
    let y_end = (max_tile.y + 1).min(grid.height - 1);

    let _anim_frame = ((tick / BELT_ANIM_SPEED as u64) % 2) as usize;
    let _machine_anim = ((tick / 10) % 2) as usize; // Machines cycle every 10 ticks

    // LOD levels based on zoom + aggressive FPS-based quality reduction.
    let fps = get_fps();
    let lod = if fps < 30 {
        2 // Force lowest detail if severely lagging
    } else if fps < 50 {
        1 // Drop to colored rectangles for ground (huge perf win)
    } else if camera.zoom >= 0.8 {
        0
    } else if camera.zoom >= 0.4 {
        1
    } else {
        2
    };

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
                        draw_rectangle(world.x, world.y, tile_draw_size, tile_draw_size,
                            Color::new(0.35, 0.4, 0.1, alpha));
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
                        if (tick / 10 + (x as u64) + (y as u64)) % 2 == 0 {
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
                        draw_texture_ex(
                            &atlas.tex,
                            world.x,
                            world.y,
                            WHITE,
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

    // --- LOD levels based on zoom ---
    // --- Pass 3: Buildings ---
    for (_bid, building) in buildings.iter() {
        let bpos = building.pos;
        // Frustum cull.
        if bpos.x < x_start || bpos.x > x_end || bpos.y < y_start || bpos.y > y_end {
            continue;
        }
        let world = Grid::grid_to_world(bpos);

        if lod <= 1 {
            // Machine animation: active machines cycle between frames.
            let mf = if building.machine_state.as_ref().map(|ms| ms.progress_ticks > 0).unwrap_or(false) {
                _machine_anim
            } else { 0 };

            // Full sprite rendering from unified atlas.
            // For belts at LOD 0: detect corners for corner sprite.
            // At LOD 1: skip corner detection (3 neighbor lookups per belt is expensive).
            let (src_rect, rotation) = if building.kind.is_belt() && lod == 0 {
                let dir = building.direction;
                let behind = bpos.neighbor(dir.opposite());
                let has_input_behind = grid.get_tile(behind)
                    .and_then(|t| t.building)
                    .and_then(|bid2| buildings.get(bid2))
                    .map(|b2| b2.kind.is_belt() && b2.direction == dir)
                    .unwrap_or(false);

                let left_pos = bpos.neighbor(dir.rotated_ccw());
                let has_input_left = grid.get_tile(left_pos)
                    .and_then(|t| t.building)
                    .and_then(|bid2| buildings.get(bid2))
                    .map(|b2| b2.kind.is_belt() && b2.direction == dir.rotated_cw())
                    .unwrap_or(false);

                let right_pos = bpos.neighbor(dir.rotated_cw());
                let has_input_right = grid.get_tile(right_pos)
                    .and_then(|t| t.building)
                    .and_then(|bid2| buildings.get(bid2))
                    .map(|b2| b2.kind.is_belt() && b2.direction == dir.rotated_ccw())
                    .unwrap_or(false);

                let is_corner = !has_input_behind && (has_input_left || has_input_right);

                if is_corner {
                    let corner_src = if has_input_left {
                        match building.kind {
                            BuildingKind::BeltYellow => atlas.r_belt_corner_left_yellow[_anim_frame],
                            BuildingKind::BeltRed => atlas.r_belt_corner_left_red[_anim_frame],
                            BuildingKind::BeltBlue => atlas.r_belt_corner_left_blue[_anim_frame],
                            _ => unreachable!(),
                        }
                    } else {
                        match building.kind {
                            BuildingKind::BeltYellow => atlas.r_belt_corner_right_yellow[_anim_frame],
                            BuildingKind::BeltRed => atlas.r_belt_corner_right_red[_anim_frame],
                            BuildingKind::BeltBlue => atlas.r_belt_corner_right_blue[_anim_frame],
                            _ => unreachable!(),
                        }
                    };
                    (corner_src, direction_to_rotation(dir))
                } else {
                    (building_src_rect(building.kind, atlas, _anim_frame, mf), direction_to_rotation(dir))
                }
            } else {
                (building_src_rect(building.kind, atlas, _anim_frame, mf), direction_to_rotation(building.direction))
            };

            // Damage tint: red when HP < 50%, flash when < 25%.
            let hp_ratio = if building.max_hp > 0.0 { building.hp / building.max_hp } else { 1.0 };
            let tint = if hp_ratio < 0.25 {
                let flash = (tick as f32 * 0.3).sin() * 0.3 + 0.7;
                Color::new(1.0, flash * 0.3, flash * 0.3, 1.0)
            } else if hp_ratio < 0.5 {
                Color::new(1.0, 0.6, 0.6, 1.0)
            } else if !building.kind.is_belt() {
                // Active machine pulse: subtle brightness when processing.
                if let Some(ref ms) = building.machine_state {
                    if ms.progress_ticks > 0 {
                        let pulse = (tick as f32 * 0.15).sin() * 0.08 + 1.05;
                        Color::new(pulse, pulse, pulse, 1.0)
                    } else {
                        WHITE
                    }
                } else {
                    WHITE
                }
            } else {
                WHITE
            };

            // Tier tinting: higher-tier buildings get subtle color to distinguish.
            let mut tint = match building.kind {
                BuildingKind::InserterLong => Color::new(tint.r, tint.g * 0.9, tint.b * 0.7, 1.0),
                BuildingKind::InserterFast => Color::new(tint.r * 0.7, tint.g * 0.8, tint.b, 1.0),
                BuildingKind::InserterStack => Color::new(tint.r * 0.8, tint.g, tint.b * 0.7, 1.0),
                BuildingKind::AssemblerT2 => Color::new(tint.r * 0.8, tint.g * 0.85, tint.b, 1.0),
                BuildingKind::AssemblerT3 => Color::new(tint.r * 0.7, tint.g * 0.75, tint.b, 1.0),
                _ => tint,
            };

            // Brownout tint: dim electric machines when power is low.
            if power_satisfaction < 0.5 && !building.kind.needs_fuel() && !building.kind.is_belt()
                && !building.kind.is_inserter() && building.kind != BuildingKind::Miner
            {
                let dim = 0.5 + power_satisfaction;
                tint.r *= dim;
                tint.g *= dim;
                tint.b *= dim * 1.1; // slight blue tinge
            }

            // Inserter arm swing: add rotation offset based on progress.
            let final_rotation = if building.kind.is_inserter() {
                if let Some(ref ms) = building.machine_state {
                    if ms.total_ticks > 0 && ms.progress_ticks > 0 {
                        let progress = 1.0 - (ms.progress_ticks as f32 / ms.total_ticks as f32);
                        // Swing from -PI/3 (behind) through 0 (neutral) to +PI/3 (front).
                        let swing = (progress * 2.0 - 1.0) * std::f32::consts::FRAC_PI_3;
                        rotation + swing
                    } else {
                        rotation
                    }
                } else {
                    rotation
                }
            } else {
                rotation
            };

            draw_texture_ex(
                &atlas.tex,
                world.x,
                world.y,
                tint,
                DrawTextureParams {
                    source: Some(src_rect),
                    dest_size: Some(Vec2::splat(TILE_SIZE)),
                    rotation: final_rotation,
                    pivot: Some(Vec2::new(world.x + TILE_SIZE * 0.5, world.y + TILE_SIZE * 0.5)),
                    ..Default::default()
                },
            );

            // Progress bar, direction arrow, item badges.
            // (Smoke particles removed — use GPU particle buffers in production.)
            if lod == 0 {
                // Active machine processing state.
                if let Some(ref ms) = building.machine_state {
                    if ms.progress_ticks > 0 && ms.total_ticks > 0 {
                        let progress = 1.0 - (ms.progress_ticks as f32 / ms.total_ticks as f32);
                        // Progress bar.
                        draw_rectangle(
                            world.x + 3.0,
                            world.y + TILE_SIZE - 7.0,
                            (TILE_SIZE - 6.0) * progress,
                            4.0,
                            Color::new(0.3, 1.0, 0.3, 0.9),
                        );
                        draw_rectangle_lines(
                            world.x + 3.0,
                            world.y + TILE_SIZE - 7.0,
                            TILE_SIZE - 6.0,
                            4.0,
                            1.0,
                            Color::new(0.2, 0.2, 0.2, 0.5),
                        );
                    }
                    // Item count badge (top-right corner) showing buffer contents.
                    let total_items = ms.input_buffer.len() + ms.output_buffer.len();
                    if total_items > 0 {
                        let badge_x = world.x + TILE_SIZE - 14.0;
                        let badge_y = world.y + 2.0;
                        draw_rectangle(badge_x, badge_y, 12.0, 12.0, Color::new(0.1, 0.1, 0.2, 0.8));
                        draw_text(
                            &format!("{}", total_items),
                            badge_x + 2.0,
                            badge_y + 10.0,
                            11.0,
                            Color::new(1.0, 0.9, 0.3, 1.0),
                        );
                    }

                    // "NO FUEL" warning for stalled fuel-based machines.
                    if building.kind.needs_fuel()
                        && ms.progress_ticks > 0
                        && ms.fuel_ticks == 0
                        && !ms.input_buffer.iter().any(|&r| r == Resource::Coal)
                    {
                        let warn_x = world.x + 2.0;
                        let warn_y = world.y + 2.0;
                        draw_rectangle(warn_x, warn_y, 42.0, 14.0, Color::new(0.8, 0.1, 0.1, 0.9));
                        draw_text("FUEL!", warn_x + 3.0, warn_y + 11.0, 12.0, WHITE);
                    }

                    // Status indicators.
                    if ms.output_buffer.len() >= MACHINE_BUFFER_CAP && ms.progress_ticks == 0 {
                        // BLOCKED: output full, machine idle — needs belt/inserter to drain output.
                        let warn_x = world.x + 1.0;
                        let warn_y = world.y + TILE_SIZE - 16.0;
                        draw_rectangle(warn_x, warn_y, 52.0, 14.0, Color::new(0.8, 0.2, 0.1, 0.9));
                        draw_text("BLOCKED", warn_x + 2.0, warn_y + 11.0, 10.0, WHITE);
                    } else if ms.output_buffer.len() >= MACHINE_BUFFER_CAP {
                        let warn_x = world.x + 2.0;
                        let warn_y = world.y + TILE_SIZE - 16.0;
                        draw_rectangle(warn_x, warn_y, 38.0, 14.0, Color::new(0.7, 0.5, 0.0, 0.9));
                        draw_text("FULL", warn_x + 3.0, warn_y + 11.0, 12.0, WHITE);
                    } else if ms.progress_ticks == 0 && ms.input_buffer.is_empty() && ms.selected_recipe.is_some() {
                        // EMPTY: has a recipe but no inputs — needs items fed in.
                        let warn_x = world.x + 2.0;
                        let warn_y = world.y + TILE_SIZE - 16.0;
                        draw_rectangle(warn_x, warn_y, 44.0, 14.0, Color::new(0.3, 0.3, 0.6, 0.8));
                        draw_text("EMPTY", warn_x + 3.0, warn_y + 11.0, 11.0, WHITE);
                    }

                    // Low power badge for electric machines experiencing brownout.
                    if power_satisfaction < 0.5 && !building.kind.needs_fuel()
                        && !building.kind.is_belt() && !building.kind.is_inserter()
                        && building.kind != BuildingKind::Miner
                    {
                        let warn_x = world.x + TILE_SIZE - 38.0;
                        let warn_y = world.y + 2.0;
                        let flash = ((tick as f32 * 0.2).sin() * 0.3 + 0.7).max(0.0);
                        draw_rectangle(warn_x, warn_y, 36.0, 12.0, Color::new(0.6, 0.2, 0.6, 0.8 * flash));
                        draw_text("PWR!", warn_x + 2.0, warn_y + 10.0, 10.0, Color::new(1.0, 0.8, 1.0, flash));
                    }
                }

                // Direction arrow (output side) — larger, more visible.
                let center = Vec2::new(world.x + TILE_SIZE * 0.5, world.y + TILE_SIZE * 0.5);
                let (dx, dy) = building.direction.delta();
                let edge = center + Vec2::new(dx as f32, dy as f32) * (TILE_SIZE * 0.42);
                if !building.kind.is_belt() {
                    // Draw a triangle arrow pointing outward.
                    let perp = Vec2::new(-dy as f32, dx as f32) * 4.0;
                    let tip = edge + Vec2::new(dx as f32, dy as f32) * 5.0;
                    let base1 = edge + perp;
                    let base2 = edge - perp;
                    draw_triangle(tip, base1, base2, Color::new(1.0, 1.0, 1.0, 0.8));
                }
            }

            // Underground belt pairing line.
            if lod == 0 && building.kind.is_underground_belt() {
                if let Some(pair_pos) = building.underground_pair {
                    let pair_world = Grid::grid_to_world_center(pair_pos);
                    let my_center = Vec2::new(world.x + TILE_SIZE * 0.5, world.y + TILE_SIZE * 0.5);
                    draw_line(my_center.x, my_center.y, pair_world.x, pair_world.y, 1.5,
                        Color::new(0.4, 0.6, 1.0, 0.3));
                }
            }

            // Health bar for damaged buildings.
            if lod == 0 && building.hp < building.max_hp && building.max_hp > 0.0 {
                let bar_w = TILE_SIZE - 6.0;
                let bar_h = 4.0;
                let bar_x = world.x + 3.0;
                let bar_y = world.y - 6.0;
                let fill = building.hp / building.max_hp;
                let bar_color = if fill > 0.5 {
                    Color::new(0.2, 0.8, 0.2, 0.9)
                } else if fill > 0.25 {
                    Color::new(0.9, 0.7, 0.1, 0.9)
                } else {
                    Color::new(0.9, 0.2, 0.1, 0.9)
                };
                draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.2, 0.2, 0.2, 0.7));
                draw_rectangle(bar_x, bar_y, bar_w * fill, bar_h, bar_color);
            }

            // Nuclear reactor glow when fueled.
            if building.kind == BuildingKind::NuclearReactor {
                if let Some(ref ms) = building.machine_state {
                    if ms.fuel_ticks > 0 {
                        let glow = (tick as f32 * 0.1).sin() * 0.1 + 0.3;
                        draw_circle(
                            world.x + TILE_SIZE * 0.5,
                            world.y + TILE_SIZE * 0.5,
                            TILE_SIZE * 0.8,
                            Color::new(0.2, 0.8, 0.3, glow),
                        );
                    }
                }
            }

            // Recipe label above assemblers (LOD 0 only).
            if lod == 0 && (building.kind == BuildingKind::AssemblerT1
                || building.kind == BuildingKind::AssemblerT2
                || building.kind == BuildingKind::AssemblerT3
                || building.kind == BuildingKind::ChemicalPlant)
            {
                if let Some(ref ms) = building.machine_state {
                    if let Some(rid) = ms.selected_recipe {
                        if rid.0 < crate::recipe::RECIPES.len() {
                            let name = crate::recipe::RECIPES[rid.0].name;
                            // Short name (take first word after "Craft ")
                            let short = name.strip_prefix("Craft ").unwrap_or(name);
                            draw_text(
                                short,
                                world.x,
                                world.y - 8.0,
                                10.0,
                                Color::new(0.8, 0.8, 0.9, 0.7),
                            );
                        }
                    }
                }
            }

            // (Belt arrows removed — direction is clear from the sprite texture itself.)
        } else {
            // LOD 2: Simple colored rectangle with activity indicator.
            let mut color = building_lod_color(building.kind);
            // Brighten active machines slightly.
            if let Some(ref ms) = building.machine_state {
                if ms.progress_ticks > 0 {
                    color.r = (color.r + 0.15).min(1.0);
                    color.g = (color.g + 0.15).min(1.0);
                    color.b = (color.b + 0.15).min(1.0);
                }
            }
            draw_rectangle(world.x + 1.0, world.y + 1.0, TILE_SIZE - 2.0, TILE_SIZE - 2.0, color);
        }
    }

    // --- Pass 4: Items on belts (visible at LOD 0 and LOD 1) ---
    if lod <= 1 {
        for (_id, item) in items.iter() {
            let item_pos = item.pos;
            // Frustum cull items.
            if item_pos.x < x_start || item_pos.x > x_end
                || item_pos.y < y_start || item_pos.y > y_end
            {
                continue;
            }

            let base = Grid::grid_to_world_center(item_pos);

            // Get belt direction for smooth interpolation.
            let offset = if let Some(tile) = grid.get_tile(item_pos) {
                if let Some(bid) = tile.building {
                    if let Some(b) = buildings.get(bid) {
                        if b.kind.is_belt() {
                            let (dx, dy) = b.direction.delta();
                            Vec2::new(dx as f32, dy as f32) * TILE_SIZE * item.progress
                        } else {
                            Vec2::ZERO
                        }
                    } else {
                        Vec2::ZERO
                    }
                } else {
                    Vec2::ZERO
                }
            } else {
                Vec2::ZERO
            };

            let draw_pos = base + offset;
            // Items are 65% of tile size — big and visible.
            let item_size = TILE_SIZE * 0.65;

            if lod == 0 {
                // Full sprite rendering from atlas.
                let item_src = item_src_rect(item.resource, atlas);
                draw_texture_ex(
                    &atlas.tex,
                    draw_pos.x - item_size * 0.5,
                    draw_pos.y - item_size * 0.5,
                    WHITE,
                    DrawTextureParams {
                        source: Some(item_src),
                        dest_size: Some(Vec2::splat(item_size)),
                        ..Default::default()
                    },
                );
            } else {
                // LOD 1: Colored dot (still visible when zoomed out).
                let color = item_lod_color(item.resource);
                draw_circle(draw_pos.x, draw_pos.y, TILE_SIZE * 0.2, color);
            }
        }
    }

    // --- Pass 5: Enemies (frustum culled) ---
    let (min_w, max_w) = camera.visible_bounds();
    for enemy in &enemies.list {
        if !enemy.alive {
            continue;
        }
        // Frustum cull enemies.
        if enemy.x < min_w.x || enemy.x > max_w.x || enemy.y < min_w.y || enemy.y > max_w.y {
            continue;
        }

        // Size varies by enemy type.
        let size = match enemy.kind {
            crate::enemy::EnemyKind::BigBiter | crate::enemy::EnemyKind::BehemothBiter => TILE_SIZE * 1.1,
            crate::enemy::EnemyKind::BigSpitter | crate::enemy::EnemyKind::BehemothSpitter => TILE_SIZE * 1.0,
            crate::enemy::EnemyKind::MediumBiter | crate::enemy::EnemyKind::MediumSpitter => TILE_SIZE * 0.85,
            _ => TILE_SIZE * 0.7,
        };
        let ex = enemy.x - size * 0.5;
        let ey = enemy.y - size * 0.5;

        // Color tint by type.
        let tint = match enemy.kind {
            crate::enemy::EnemyKind::SmallBiter => WHITE,
            crate::enemy::EnemyKind::MediumBiter => Color::new(1.0, 0.8, 0.6, 1.0),
            crate::enemy::EnemyKind::BigBiter => Color::new(1.0, 0.5, 0.3, 1.0),
            crate::enemy::EnemyKind::BehemothBiter => Color::new(0.8, 0.2, 0.8, 1.0),
            crate::enemy::EnemyKind::SmallSpitter => Color::new(0.7, 1.0, 0.7, 1.0),
            crate::enemy::EnemyKind::MediumSpitter => Color::new(0.5, 1.0, 0.5, 1.0),
            crate::enemy::EnemyKind::BigSpitter => Color::new(0.3, 0.9, 0.3, 1.0),
            crate::enemy::EnemyKind::BehemothSpitter => Color::new(0.2, 0.7, 1.0, 1.0),
        };

        if lod <= 1 {
            // Rotate sprite to face movement direction.
            // Sprite is drawn pointing up (North) by default, so offset by PI/2.
            let rotation = enemy.facing + std::f32::consts::FRAC_PI_2;
            draw_texture_ex(
                &atlas.tex,
                ex,
                ey,
                tint,
                DrawTextureParams {
                    source: Some(atlas.r_enemy_small_biter[_anim_frame]),
                    dest_size: Some(Vec2::splat(size)),
                    rotation,
                    pivot: Some(Vec2::new(enemy.x, enemy.y)),
                    ..Default::default()
                },
            );
        } else {
            // LOD 2: red dot.
            draw_circle(enemy.x, enemy.y, TILE_SIZE * 0.3, Color::new(0.9, 0.1, 0.1, 0.9));
        }

        // Health bar (only at LOD 0).
        if lod == 0 {
            let max_hp = enemy.kind.max_hp();
            if enemy.hp < max_hp {
                let bar_w = size;
                let bar_h = 3.0;
                let bar_x = ex;
                let bar_y = ey - 5.0;
                let fill = enemy.hp / max_hp;
                draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.3, 0.0, 0.0, 0.8));
                draw_rectangle(bar_x, bar_y, bar_w * fill, bar_h, Color::new(0.9, 0.1, 0.1, 0.9));
            }
        }
    }

    // --- Pass 7: Grid lines (very faint, only at high zoom) ---
    if camera.zoom > 1.5 {
        let grid_color = Color::new(1.0, 1.0, 1.0, 0.05);
        for x in x_start..=x_end {
            let wx = x as f32 * TILE_SIZE;
            draw_line(
                wx,
                y_start as f32 * TILE_SIZE,
                wx,
                (y_end + 1) as f32 * TILE_SIZE,
                0.5,
                grid_color,
            );
        }
        for y in y_start..=y_end {
            let wy = y as f32 * TILE_SIZE;
            draw_line(
                x_start as f32 * TILE_SIZE,
                wy,
                (x_end + 1) as f32 * TILE_SIZE,
                wy,
                0.5,
                grid_color,
            );
        }
    }

    // --- Crashed ship / base at map center ---
    {
        let ship_cx = grid.width as f32 * TILE_SIZE * 0.5;
        let ship_cy = grid.height as f32 * TILE_SIZE * 0.5;
        let ship_w = TILE_SIZE * 5.0;
        let ship_h = TILE_SIZE * 3.0;
        let ship_x = ship_cx - ship_w * 0.5;
        let ship_y = ship_cy - ship_h * 0.5;

        if ship_x >= min_world.x - ship_w && ship_x <= max_world.x + ship_w
            && ship_y >= min_world.y - ship_h && ship_y <= max_world.y + ship_h
        {
            // Pixel art crashed ship sprite (80x48 scaled to 5x3 tiles).
            draw_texture_ex(
                &atlas.tex,
                ship_x,
                ship_y,
                WHITE,
                DrawTextureParams {
                    source: Some(atlas.r_crashed_ship),
                    dest_size: Some(Vec2::new(ship_w, ship_h)),
                    ..Default::default()
                },
            );

            // Label.
            if lod == 0 {
                draw_text("FORGE BASE", ship_cx - 40.0, ship_y - 8.0, 16.0,
                    Color::new(0.7, 0.6, 0.9, 0.8));
                draw_text("Horizon's Promise", ship_cx - 50.0, ship_y + 6.0, 12.0,
                    Color::new(0.4, 0.4, 0.5, 0.5));
                draw_text("[ click for lore ]", ship_cx - 48.0, ship_y + ship_h + 18.0, 11.0,
                    Color::new(0.5, 0.5, 0.6, 0.4));
            }

            // Robot docking area — small idle robots parked near the ship.
            let dock_x = ship_x + ship_w + 8.0;
            let dock_y = ship_cy - 10.0;
            for i in 0..4u32 {
                let rx = dock_x + (i % 2) as f32 * 12.0;
                let ry = dock_y + (i / 2) as f32 * 12.0;
                draw_circle(rx, ry, 3.0, Color::new(0.3, 0.45, 0.7, 0.6));
                draw_circle(rx, ry, 1.5, Color::new(0.5, 0.65, 0.9, 0.5));
            }
        }
    }

    // Ambient particles removed — they added 12 draw calls per frame with no gameplay value.
    // Industry standard: cosmetic particles should be GPU instanced or in a particle buffer,
    // never individual draw calls.
}

/// Draws a darkness overlay for the night cycle with light pools around buildings.
///
/// Call after `draw_world` while still in camera space.
pub fn draw_night_overlay(darkness: f32, buildings: &Buildings, camera: &GameCamera) {
    if darkness < 0.01 {
        return;
    }
    // Global darkness.
    draw_rectangle(
        -100000.0,
        -100000.0,
        200000.0,
        200000.0,
        Color::new(0.0, 0.0, 0.05, darkness),
    );

    // Light pools around active buildings (additive circles that punch through darkness).
    if darkness > 0.05 {
        let (min_vis, max_vis) = camera.visible_bounds();
        let light_alpha = (darkness * 0.6).min(0.35); // brighter as night gets darker

        for (_bid, building) in buildings.iter() {
            let bpos = building.pos;
            let wx = bpos.x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
            let wy = bpos.y as f32 * TILE_SIZE + TILE_SIZE * 0.5;

            // Frustum cull lights.
            if wx < min_vis.x - TILE_SIZE * 4.0 || wx > max_vis.x + TILE_SIZE * 4.0
                || wy < min_vis.y - TILE_SIZE * 4.0 || wy > max_vis.y + TILE_SIZE * 4.0
            {
                continue;
            }

            // Determine light color and radius based on building type.
            let (color, radius) = match building.kind {
                // Furnaces: warm orange fire glow.
                BuildingKind::StoneFurnace | BuildingKind::SteelFurnace => {
                    let active = building.machine_state.as_ref().map(|ms| ms.progress_ticks > 0).unwrap_or(false);
                    if active {
                        (Color::new(1.0, 0.7, 0.3, light_alpha), TILE_SIZE * 3.0)
                    } else { continue; }
                }
                // Labs: blue-purple glow.
                BuildingKind::Lab => {
                    let active = building.machine_state.as_ref().map(|ms| ms.progress_ticks > 0).unwrap_or(false);
                    if active {
                        (Color::new(0.5, 0.4, 1.0, light_alpha), TILE_SIZE * 2.5)
                    } else { continue; }
                }
                // Steam engines, solar: white glow (always on if placed).
                BuildingKind::SteamEngine => {
                    (Color::new(0.9, 0.9, 1.0, light_alpha * 0.7), TILE_SIZE * 2.0)
                }
                // Laser turrets: blue glow.
                BuildingKind::LaserTurret => {
                    (Color::new(0.3, 0.5, 1.0, light_alpha * 0.6), TILE_SIZE * 2.5)
                }
                // Nuclear reactor: green glow.
                BuildingKind::NuclearReactor => {
                    (Color::new(0.3, 1.0, 0.5, light_alpha), TILE_SIZE * 5.0)
                }
                // Assemblers when active: dim blue.
                BuildingKind::AssemblerT1 | BuildingKind::AssemblerT2 | BuildingKind::AssemblerT3 => {
                    let active = building.machine_state.as_ref().map(|ms| ms.progress_ticks > 0).unwrap_or(false);
                    if active {
                        (Color::new(0.6, 0.6, 0.9, light_alpha * 0.4), TILE_SIZE * 1.5)
                    } else { continue; }
                }
                _ => continue,
            };

            // Draw concentric circles for soft light falloff.
            draw_circle(wx, wy, radius, Color::new(color.r, color.g, color.b, color.a * 0.3));
            draw_circle(wx, wy, radius * 0.6, Color::new(color.r, color.g, color.b, color.a * 0.5));
            draw_circle(wx, wy, radius * 0.3, Color::new(color.r, color.g, color.b, color.a * 0.7));
        }
    }
}

/// Draws the ghost preview of the building the player is about to place.
///
/// Shows a semi-transparent building sprite at the cursor position:
/// green tint if placement is valid, red if invalid.
pub fn draw_ghost_preview(
    grid: &Grid,
    camera: &GameCamera,
    atlas: &SpriteAtlas,
    selected: Option<BuildingKind>,
    direction: Direction,
) {
    let mouse_screen = Vec2::new(mouse_position().0, mouse_position().1);
    let mouse_world = camera.screen_to_world(mouse_screen);
    let grid_pos = Grid::world_to_grid(mouse_world);
    let world = Grid::grid_to_world(grid_pos);

    if !grid.in_bounds(grid_pos) {
        return;
    }

    if let Some(kind) = selected {
        let can_place = grid
            .get_tile(grid_pos)
            .map(|t| t.building.is_none() && t.terrain.is_buildable())
            .unwrap_or(false);

        let tint = if can_place {
            Color::new(0.3, 1.0, 0.3, 0.5)
        } else {
            Color::new(1.0, 0.3, 0.3, 0.5)
        };

        let src_rect = building_src_rect(kind, atlas, 0, 0);
        let rotation = direction_to_rotation(direction);

        draw_texture_ex(
            &atlas.tex,
            world.x,
            world.y,
            tint,
            DrawTextureParams {
                source: Some(src_rect),
                dest_size: Some(Vec2::splat(TILE_SIZE)),
                rotation,
                pivot: Some(Vec2::new(
                    world.x + TILE_SIZE * 0.5,
                    world.y + TILE_SIZE * 0.5,
                )),
                ..Default::default()
            },
        );

        // Inserter direction labels: show pickup ← and delivery → sides.
        if kind.is_inserter() {
            let (dx, dy) = direction.delta();
            let cx = world.x + TILE_SIZE * 0.5;
            let cy = world.y + TILE_SIZE * 0.5;
            // Delivery side (front): green "OUT" label.
            let fx = cx + dx as f32 * TILE_SIZE * 0.7;
            let fy = cy + dy as f32 * TILE_SIZE * 0.7;
            draw_text("OUT", fx - 10.0, fy + 4.0, 11.0, Color::new(0.3, 1.0, 0.3, 0.7));
            // Pickup side (behind): yellow "IN" label.
            let bx = cx - dx as f32 * TILE_SIZE * 0.7;
            let by = cy - dy as f32 * TILE_SIZE * 0.7;
            draw_text("IN", bx - 6.0, by + 4.0, 11.0, Color::new(1.0, 0.9, 0.3, 0.7));
        }

        // Range indicators for turrets and roboports.
        let center_x = world.x + TILE_SIZE * 0.5;
        let center_y = world.y + TILE_SIZE * 0.5;
        if kind == BuildingKind::GunTurret || kind == BuildingKind::LaserTurret {
            draw_circle_lines(center_x, center_y, TILE_SIZE * 6.0, 1.0, Color::new(1.0, 0.3, 0.3, 0.25));
        } else if kind == BuildingKind::Roboport {
            draw_circle_lines(center_x, center_y, TILE_SIZE * 10.0, 1.0, Color::new(0.3, 0.5, 1.0, 0.2));
        }
    } else {
        // No building selected — just show cursor highlight.
        draw_rectangle_lines(world.x, world.y, TILE_SIZE, TILE_SIZE, 1.0, YELLOW);
    }
}

/// Returns the rotation angle (radians) for a direction.
///
/// Sprites are drawn facing North (up) by default.
fn direction_to_rotation(dir: Direction) -> f32 {
    match dir {
        Direction::North => 0.0,
        Direction::East => std::f32::consts::FRAC_PI_2,
        Direction::South => std::f32::consts::PI,
        Direction::West => -std::f32::consts::FRAC_PI_2,
    }
}

/// Returns the atlas source rect for a building kind.
/// `anim_frame` cycles belt animation; `machine_frame` cycles machine animation (0=idle, 1=active).
fn building_src_rect(kind: BuildingKind, atlas: &SpriteAtlas, anim_frame: usize, machine_frame: usize) -> Rect {
    let mf = machine_frame.min(1);
    match kind {
        BuildingKind::BeltYellow => atlas.r_belt_yellow[anim_frame],
        BuildingKind::BeltRed => atlas.r_belt_red[anim_frame],
        BuildingKind::BeltBlue => atlas.r_belt_blue[anim_frame],
        BuildingKind::Miner => atlas.r_miner[mf],
        BuildingKind::PumpJack => atlas.r_pump_jack,
        BuildingKind::StoneFurnace => atlas.r_stone_furnace[mf],
        BuildingKind::SteelFurnace => atlas.r_steel_furnace[mf],
        BuildingKind::ElectricFurnace => atlas.r_electric_furnace,
        BuildingKind::AssemblerT1 | BuildingKind::AssemblerT2 | BuildingKind::AssemblerT3 => {
            atlas.r_assembler[mf]
        }
        BuildingKind::ChemicalPlant => atlas.r_chemical_plant,
        BuildingKind::OilRefinery => atlas.r_oil_refinery,
        BuildingKind::Lab => atlas.r_lab[mf],
        BuildingKind::Boiler => atlas.r_boiler,
        BuildingKind::SteamEngine => atlas.r_steam_engine,
        BuildingKind::SolarPanel => atlas.r_solar_panel,
        BuildingKind::NuclearReactor => atlas.r_nuclear_reactor,
        BuildingKind::Accumulator => atlas.r_accumulator,
        BuildingKind::StorageChest => atlas.r_chest,
        BuildingKind::GunTurret => atlas.r_gun_turret,
        BuildingKind::LaserTurret => atlas.r_laser_turret,
        BuildingKind::Wall | BuildingKind::Gate => atlas.r_wall,
        BuildingKind::InserterRegular
        | BuildingKind::InserterLong
        | BuildingKind::InserterFast
        | BuildingKind::InserterStack => atlas.r_inserter,
        BuildingKind::UndergroundBeltYellow
        | BuildingKind::UndergroundBeltRed
        | BuildingKind::UndergroundBeltBlue => atlas.r_underground_belt,
        BuildingKind::Splitter => atlas.r_splitter,
        BuildingKind::WaterPump => atlas.r_water_pump,
        BuildingKind::Centrifuge => atlas.r_chemical_plant,
        BuildingKind::RocketSilo => atlas.r_rocket_silo,
        BuildingKind::Radar => atlas.r_radar,
        BuildingKind::PipeSegment | BuildingKind::UndergroundPipe | BuildingKind::StorageTank => atlas.r_pipe,
        BuildingKind::RailStraight | BuildingKind::RailCurved => atlas.r_rail,
        BuildingKind::TrainStop | BuildingKind::RailSignal => atlas.r_train_stop,
        BuildingKind::Roboport => atlas.r_roboport,
        BuildingKind::Beacon => atlas.r_beacon,
    }
}

/// Returns the atlas source rect for an item resource.
fn item_src_rect(resource: Resource, atlas: &SpriteAtlas) -> Rect {
    match resource {
        Resource::IronOre => atlas.r_item_iron_ore,
        Resource::CopperOre => atlas.r_item_copper_ore,
        Resource::Coal => atlas.r_item_coal,
        Resource::Stone => atlas.r_item_stone,
        Resource::IronPlate => atlas.r_item_iron_plate,
        Resource::CopperPlate => atlas.r_item_copper_plate,
        Resource::SteelPlate => atlas.r_item_steel_plate,
        Resource::StoneBrick => atlas.r_item_stone_brick,
        Resource::Gear => atlas.r_item_gear,
        Resource::Wire => atlas.r_item_wire,
        Resource::Pipe => atlas.r_item_pipe,
        Resource::IronStick => atlas.r_item_iron_stick,
        Resource::GreenCircuit => atlas.r_item_green_circuit,
        Resource::RedCircuit => atlas.r_item_red_circuit,
        Resource::BlueCircuit => atlas.r_item_blue_circuit,
        Resource::ScienceRed => atlas.r_item_science_red,
        Resource::ScienceGreen => atlas.r_item_science_green,
        Resource::ScienceBlue => atlas.r_item_science_blue,
        Resource::SciencePurple => atlas.r_item_science_purple,
        Resource::ScienceYellow => atlas.r_item_science_yellow,
        Resource::Sulfur => atlas.r_item_sulfur,
        Resource::Plastic => atlas.r_item_plastic,
        Resource::Battery => atlas.r_item_battery,
        Resource::BasicAmmo | Resource::PiercingAmmo => atlas.r_item_ammo,
        Resource::Grenade => atlas.r_item_grenade,
        Resource::EngineUnit | Resource::ElectricEngine => atlas.r_item_engine,
        Resource::RocketPart => atlas.r_item_rocket_part,
        Resource::RocketFuel => atlas.r_item_rocket_fuel,
        Resource::Inserter => atlas.r_item_inserter,
        Resource::SpeedModule | Resource::EfficiencyModule | Resource::ProductivityModule => atlas.r_item_speed_module,
        Resource::UraniumOre => atlas.r_item_uranium_ore,
        Resource::Rail => atlas.r_item_rail,
        Resource::Concrete => atlas.r_item_concrete,
        Resource::SolarPanelItem => atlas.r_item_solar_panel,
        Resource::AccumulatorItem => atlas.r_item_accumulator,
        Resource::FlyingRobotFrame => atlas.r_item_robot_frame,
        Resource::Uranium235 => atlas.r_item_uranium_235,
        Resource::Uranium238 => atlas.r_item_uranium_238,
        Resource::NuclearFuelCell => atlas.r_item_nuclear_fuel,
        Resource::LowDensityStructure => atlas.r_item_low_density,
        _ => atlas.r_item_iron_ore, // SpaceScience only
    }
}

/// Returns a simple color for LOD 2 building rendering (colored rectangle).
/// Returns a color for items at LOD 1 (colored dots when zoomed out).
fn item_lod_color(resource: Resource) -> Color {
    match resource {
        Resource::IronOre | Resource::IronPlate => Color::new(0.55, 0.55, 0.6, 1.0),
        Resource::CopperOre | Resource::CopperPlate => Color::new(0.85, 0.55, 0.2, 1.0),
        Resource::Coal => Color::new(0.2, 0.2, 0.2, 1.0),
        Resource::Stone | Resource::StoneBrick => Color::new(0.6, 0.6, 0.55, 1.0),
        Resource::Gear => Color::new(0.5, 0.5, 0.55, 1.0),
        Resource::Wire => Color::new(0.8, 0.5, 0.15, 1.0),
        Resource::GreenCircuit => Color::new(0.2, 0.8, 0.3, 1.0),
        Resource::RedCircuit => Color::new(0.8, 0.2, 0.2, 1.0),
        Resource::ScienceRed => Color::new(1.0, 0.2, 0.2, 1.0),
        Resource::ScienceGreen => Color::new(0.2, 1.0, 0.2, 1.0),
        _ => Color::new(0.7, 0.7, 0.3, 1.0),
    }
}

fn building_lod_color(kind: BuildingKind) -> Color {
    match kind {
        BuildingKind::BeltYellow => Color::new(0.45, 0.4, 0.15, 1.0),
        BuildingKind::BeltRed => Color::new(0.55, 0.2, 0.15, 1.0),
        BuildingKind::BeltBlue => Color::new(0.15, 0.2, 0.55, 1.0),
        BuildingKind::Miner | BuildingKind::PumpJack => Color::new(0.6, 0.4, 0.1, 1.0),
        BuildingKind::StoneFurnace | BuildingKind::SteelFurnace | BuildingKind::ElectricFurnace => {
            Color::new(0.55, 0.35, 0.15, 1.0) // warm orange-brown (distinct from red belts)
        }
        BuildingKind::AssemblerT1 | BuildingKind::AssemblerT2 | BuildingKind::AssemblerT3 => {
            Color::new(0.2, 0.3, 0.55, 1.0) // darker blue (distinct from solar)
        }
        BuildingKind::Lab => Color::new(0.45, 0.2, 0.55, 1.0),
        BuildingKind::GunTurret => Color::new(0.45, 0.45, 0.4, 1.0),  // warm gray
        BuildingKind::LaserTurret => Color::new(0.3, 0.4, 0.7, 1.0),  // blue-gray (distinct from gun)
        BuildingKind::Wall | BuildingKind::Gate => Color::new(0.35, 0.35, 0.3, 1.0),
        BuildingKind::Boiler | BuildingKind::SteamEngine => Color::new(0.15, 0.4, 0.35, 1.0),
        BuildingKind::SolarPanel => Color::new(0.1, 0.15, 0.45, 1.0), // deep blue (distinct from assembler)
        BuildingKind::NuclearReactor => Color::new(0.2, 0.5, 0.3, 1.0), // green glow
        BuildingKind::StorageChest => Color::new(0.5, 0.4, 0.2, 1.0),   // wooden brown
        _ => Color::new(0.3, 0.3, 0.3, 1.0),
    }
}
