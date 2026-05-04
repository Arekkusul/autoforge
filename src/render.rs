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

    // LOD levels based on zoom + aggressive FPS-based quality reduction.
    let fps = get_fps();
    let lod = if fps < 25 {
        2 // Force lowest detail if severely lagging
    } else if fps < 40 {
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
                        let ore_tex = match deposit {
                            OreDeposit::Iron => &atlas.ore_iron,
                            OreDeposit::Copper => &atlas.ore_copper,
                            OreDeposit::Coal => &atlas.ore_coal,
                            OreDeposit::Stone => &atlas.ore_stone,
                            OreDeposit::Uranium => &atlas.ore_uranium,
                            OreDeposit::Tin => &atlas.ore_tin,
                            OreDeposit::Gold => &atlas.ore_gold,
                            OreDeposit::Sulfur => &atlas.ore_sulfur,
                            OreDeposit::Crystal => &atlas.ore_crystal,
                            OreDeposit::Oil => &atlas.ore_oil,
                        };
                        let world = Grid::grid_to_world(pos);
                        draw_texture_ex(
                            ore_tex,
                            world.x,
                            world.y,
                            WHITE,
                            DrawTextureParams {
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
            // Full sprite rendering.
            let tex = building_texture(building.kind, atlas, _anim_frame);
            let rotation = direction_to_rotation(building.direction);

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

            draw_texture_ex(
                tex,
                world.x,
                world.y,
                tint,
                DrawTextureParams {
                    dest_size: Some(Vec2::splat(TILE_SIZE)),
                    rotation,
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
                    if ms.output_buffer.len() >= 8 && ms.progress_ticks == 0 {
                        // BLOCKED: output full, machine idle — needs belt/inserter to drain output.
                        let warn_x = world.x + 1.0;
                        let warn_y = world.y + TILE_SIZE - 16.0;
                        draw_rectangle(warn_x, warn_y, 52.0, 14.0, Color::new(0.8, 0.2, 0.1, 0.9));
                        draw_text("BLOCKED", warn_x + 2.0, warn_y + 11.0, 10.0, WHITE);
                    } else if ms.output_buffer.len() >= 8 {
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

            // Belt direction indicator (LOD 0 only — saves 2 triangle draws per belt at LOD 1).
            if building.kind.is_belt() && lod == 0 {
                let center = Vec2::new(world.x + TILE_SIZE * 0.5, world.y + TILE_SIZE * 0.5);
                let (dx, dy) = building.direction.delta();
                let dir_vec = Vec2::new(dx as f32, dy as f32);

                // Animated chevron — green if items present, dim yellow if empty.
                let anim_offset = (tick as f32 * 0.15) % TILE_SIZE;
                let has_items = !grid.items_at(bpos).is_empty();
                let chevron_color = if has_items {
                    Color::new(0.3, 1.0, 0.4, 0.7) // green = flowing
                } else {
                    Color::new(0.8, 0.75, 0.3, 0.4) // dim yellow = idle
                };

                // Draw 2 chevron arrows along the belt.
                for i in 0..2 {
                    let offset = (anim_offset + i as f32 * TILE_SIZE * 0.5) % TILE_SIZE - TILE_SIZE * 0.5;
                    let pos = center + dir_vec * offset;
                    let perp = Vec2::new(-dy as f32, dx as f32) * (TILE_SIZE * 0.2);
                    let tip = pos + dir_vec * 5.0;
                    let base1 = pos - dir_vec * 3.0 + perp;
                    let base2 = pos - dir_vec * 3.0 - perp;
                    draw_triangle(tip, base1, base2, chevron_color);
                }
            }
        } else {
            // LOD 2: Simple colored rectangle (very fast).
            let color = building_lod_color(building.kind);
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
                // Full sprite rendering (no background circle — saves 1 draw call per item).
                let item_tex = item_texture(item.resource, atlas);
                draw_texture_ex(
                    item_tex,
                    draw_pos.x - item_size * 0.5,
                    draw_pos.y - item_size * 0.5,
                    WHITE,
                    DrawTextureParams {
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
            draw_texture_ex(
                &atlas.enemy_small_biter,
                ex,
                ey,
                tint,
                DrawTextureParams {
                    dest_size: Some(Vec2::splat(size)),
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
            // CRASHED SHIP — tilted, asymmetric damage, debris, exposed internals.
            let hull = Color::new(0.2, 0.2, 0.28, 1.0);
            let hull_hi = Color::new(0.3, 0.3, 0.4, 1.0);
            let hull_dk = Color::new(0.1, 0.1, 0.16, 1.0);
            let damage = Color::new(0.35, 0.2, 0.12, 0.7);
            let exposed = Color::new(0.5, 0.35, 0.15, 0.8);
            let wire_color = Color::new(0.8, 0.5, 0.1, 0.6);
            let glow = (tick as f32 * 0.1).sin() * 0.15 + 0.55;

            // Crash crater / scorched ground beneath.
            draw_ellipse(ship_cx + 10.0, ship_cy + 5.0, ship_w * 0.5, ship_h * 0.4, 0.0,
                Color::new(0.08, 0.08, 0.06, 0.5));

            // Scattered debris pieces around the crash site.
            for i in 0..8u32 {
                let angle = i as f32 * 0.8 + 0.3;
                let dist = 60.0 + (i as f32 * 17.3).sin().abs() * 40.0;
                let dx = ship_cx + angle.cos() * dist;
                let dy = ship_cy + angle.sin() * dist;
                let sz = 3.0 + (i % 3) as f32 * 2.0;
                draw_rectangle(dx, dy, sz, sz, hull_dk);
            }

            // Main hull — tilted ~10 degrees (drawn with offset polygons).
            let tilt = 8.0; // pixels of vertical offset for tilt effect
            // Fuselage (main body — slightly tilted).
            draw_triangle(
                Vec2::new(ship_x + ship_w * 0.85, ship_cy - tilt * 0.3), // nose (tilted up)
                Vec2::new(ship_x + ship_w * 0.12, ship_y + ship_h * 0.15),
                Vec2::new(ship_x + ship_w * 0.12, ship_y + ship_h * 0.85 + tilt),
                hull,
            );
            // Hull top surface (lighter — catches light).
            draw_rectangle(ship_x + ship_w * 0.12, ship_y + ship_h * 0.2,
                ship_w * 0.6, ship_h * 0.25, hull_hi);
            // Hull bottom (darker shadow).
            draw_rectangle(ship_x + ship_w * 0.12, ship_cy + ship_h * 0.1 + tilt * 0.5,
                ship_w * 0.55, ship_h * 0.2, hull_dk);

            // Broken wing stub (top — intact).
            draw_triangle(
                Vec2::new(ship_x + ship_w * 0.35, ship_y - 8.0),
                Vec2::new(ship_x + ship_w * 0.2, ship_y + ship_h * 0.2),
                Vec2::new(ship_x + ship_w * 0.55, ship_y + ship_h * 0.2),
                hull,
            );
            // Broken wing stub (bottom — snapped off, jagged edge).
            draw_triangle(
                Vec2::new(ship_x + ship_w * 0.25, ship_y + ship_h + tilt + 5.0),
                Vec2::new(ship_x + ship_w * 0.2, ship_y + ship_h * 0.75 + tilt),
                Vec2::new(ship_x + ship_w * 0.4, ship_y + ship_h * 0.8 + tilt),
                hull_dk,
            );

            // Damage breach (hole in hull exposing internals — top right area).
            draw_rectangle(ship_x + ship_w * 0.5, ship_y + ship_h * 0.2,
                ship_w * 0.15, ship_h * 0.25, exposed);
            // Dangling wires from breach.
            draw_line(ship_x + ship_w * 0.52, ship_y + ship_h * 0.3,
                ship_x + ship_w * 0.48, ship_y + ship_h * 0.5, 1.5, wire_color);
            draw_line(ship_x + ship_w * 0.58, ship_y + ship_h * 0.25,
                ship_x + ship_w * 0.62, ship_y + ship_h * 0.45, 1.0, wire_color);

            // Cockpit (cracked glass, still faintly glowing blue — FORGE is alive).
            draw_circle(ship_x + ship_w * 0.75, ship_cy - tilt * 0.2, ship_h * 0.16,
                Color::new(0.15, 0.3, glow * 0.7, 0.8));
            draw_circle(ship_x + ship_w * 0.75, ship_cy - tilt * 0.2, ship_h * 0.1,
                Color::new(0.2, 0.4, glow, 0.6));
            // Crack lines across cockpit glass.
            draw_line(ship_x + ship_w * 0.72, ship_cy - tilt * 0.2 - 5.0,
                ship_x + ship_w * 0.78, ship_cy - tilt * 0.2 + 8.0, 1.0,
                Color::new(0.5, 0.5, 0.6, 0.4));

            // Engine block (rear, partially destroyed).
            draw_rectangle(ship_x + ship_w * 0.02, ship_y + ship_h * 0.2,
                ship_w * 0.12, ship_h * 0.6 + tilt, hull_dk);
            // One engine still smoldering (orange glow).
            draw_circle(ship_x + ship_w * 0.04, ship_cy + ship_h * 0.15, 5.0,
                Color::new(0.7, 0.3, 0.05, 0.5));

            // Scorch marks radiating from crash.
            for i in 0..5 {
                let a = i as f32 * 1.2 + 0.5;
                let len = 30.0 + (i as f32 * 7.0);
                draw_line(ship_cx, ship_cy + tilt * 0.5,
                    ship_cx + a.cos() * len, ship_cy + tilt * 0.5 + a.sin() * len,
                    2.0, Color::new(0.15, 0.12, 0.08, 0.3));
            }

            // Antenna (bent, still transmitting).
            draw_line(ship_cx + 15.0, ship_y + ship_h * 0.2,
                ship_cx + 20.0, ship_y - 18.0, 1.5, Color::new(0.35, 0.35, 0.45, 0.7));
            draw_line(ship_cx + 20.0, ship_y - 18.0,
                ship_cx + 12.0, ship_y - 25.0, 1.0, Color::new(0.35, 0.35, 0.45, 0.5));
            let ant_glow = (tick as f32 * 0.2).sin() * 0.4 + 0.6;
            draw_circle(ship_cx + 12.0, ship_y - 25.0, 3.0,
                Color::new(0.3, ant_glow, 0.9, 0.8));

            // Label.
            if lod == 0 {
                draw_text("FORGE BASE", ship_cx - 40.0, ship_y - 30.0, 16.0,
                    Color::new(0.7, 0.6, 0.9, 0.8));
                draw_text("Horizon's Promise", ship_cx - 50.0, ship_y - 16.0, 12.0,
                    Color::new(0.4, 0.4, 0.5, 0.5));
                draw_text("[ click for lore ]", ship_cx - 48.0, ship_y + ship_h + tilt + 18.0, 11.0,
                    Color::new(0.5, 0.5, 0.6, 0.4));
            }

            // Robot docking area — small idle robots parked near the ship.
            let dock_x = ship_x + ship_w + 8.0;
            let dock_y = ship_cy - 10.0;
            for i in 0..4 {
                let rx = dock_x + (i % 2) as f32 * 12.0;
                let ry = dock_y + (i / 2) as f32 * 12.0;
                // Idle robot (small blue dot, dimmer than active robots).
                draw_circle(rx, ry, 3.0, Color::new(0.3, 0.45, 0.7, 0.6));
                draw_circle(rx, ry, 1.5, Color::new(0.5, 0.65, 0.9, 0.5));
            }
        }
    }

    // Ambient particles removed — they added 12 draw calls per frame with no gameplay value.
    // Industry standard: cosmetic particles should be GPU instanced or in a particle buffer,
    // never individual draw calls.
}

/// Draws a darkness overlay for the night cycle.
///
/// Call after `draw_world` while still in camera space.
pub fn draw_night_overlay(darkness: f32) {
    if darkness > 0.01 {
        // Draw a large dark rectangle covering the visible area.
        // Using a very large rect since we're in world space under camera.
        draw_rectangle(
            -100000.0,
            -100000.0,
            200000.0,
            200000.0,
            Color::new(0.0, 0.0, 0.05, darkness),
        );
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

        let tex = building_texture(kind, atlas, 0);
        let rotation = direction_to_rotation(direction);

        draw_texture_ex(
            tex,
            world.x,
            world.y,
            tint,
            DrawTextureParams {
                dest_size: Some(Vec2::splat(TILE_SIZE)),
                rotation,
                pivot: Some(Vec2::new(
                    world.x + TILE_SIZE * 0.5,
                    world.y + TILE_SIZE * 0.5,
                )),
                ..Default::default()
            },
        );
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

/// Selects the appropriate sprite texture for a building kind.
fn building_texture<'a>(kind: BuildingKind, atlas: &'a SpriteAtlas, anim_frame: usize) -> &'a Texture2D {
    match kind {
        BuildingKind::BeltYellow => &atlas.belt_yellow[anim_frame],
        BuildingKind::BeltRed => &atlas.belt_red[anim_frame],
        BuildingKind::BeltBlue => &atlas.belt_blue[anim_frame],
        BuildingKind::Miner | BuildingKind::PumpJack => &atlas.miner,
        BuildingKind::StoneFurnace => &atlas.stone_furnace,
        BuildingKind::SteelFurnace => &atlas.steel_furnace,
        BuildingKind::ElectricFurnace => &atlas.steel_furnace,
        BuildingKind::AssemblerT1 | BuildingKind::AssemblerT2 | BuildingKind::AssemblerT3 => {
            &atlas.assembler
        }
        BuildingKind::Lab => &atlas.lab,
        BuildingKind::Boiler => &atlas.boiler,
        BuildingKind::SteamEngine => &atlas.steam_engine,
        BuildingKind::SolarPanel => &atlas.solar_panel,
        BuildingKind::StorageChest => &atlas.chest,
        BuildingKind::GunTurret | BuildingKind::LaserTurret => &atlas.gun_turret,
        BuildingKind::Wall | BuildingKind::Gate => &atlas.wall,
        BuildingKind::InserterRegular
        | BuildingKind::InserterLong
        | BuildingKind::InserterFast
        | BuildingKind::InserterStack => &atlas.inserter,
        _ => &atlas.chest, // fallback for unimplemented sprites
    }
}

/// Selects the appropriate sprite texture for an item resource.
fn item_texture<'a>(resource: Resource, atlas: &'a SpriteAtlas) -> &'a Texture2D {
    match resource {
        Resource::IronOre => &atlas.item_iron_ore,
        Resource::CopperOre => &atlas.item_copper_ore,
        Resource::Coal => &atlas.item_coal,
        Resource::Stone => &atlas.item_stone,
        Resource::IronPlate => &atlas.item_iron_plate,
        Resource::CopperPlate => &atlas.item_copper_plate,
        Resource::Gear => &atlas.item_gear,
        Resource::Wire => &atlas.item_wire,
        Resource::GreenCircuit => &atlas.item_green_circuit,
        Resource::ScienceRed => &atlas.item_science_red,
        _ => &atlas.item_iron_ore, // fallback for items without sprites yet
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
        BuildingKind::BeltYellow => Color::new(0.4, 0.4, 0.15, 1.0),
        BuildingKind::BeltRed => Color::new(0.5, 0.2, 0.15, 1.0),
        BuildingKind::BeltBlue => Color::new(0.15, 0.2, 0.5, 1.0),
        BuildingKind::Miner | BuildingKind::PumpJack => Color::new(0.6, 0.4, 0.1, 1.0),
        BuildingKind::StoneFurnace | BuildingKind::SteelFurnace | BuildingKind::ElectricFurnace => {
            Color::new(0.5, 0.2, 0.15, 1.0)
        }
        BuildingKind::AssemblerT1 | BuildingKind::AssemblerT2 | BuildingKind::AssemblerT3 => {
            Color::new(0.15, 0.25, 0.5, 1.0)
        }
        BuildingKind::Lab => Color::new(0.4, 0.2, 0.5, 1.0),
        BuildingKind::GunTurret | BuildingKind::LaserTurret => Color::new(0.4, 0.4, 0.4, 1.0),
        BuildingKind::Wall | BuildingKind::Gate => Color::new(0.35, 0.35, 0.3, 1.0),
        BuildingKind::Boiler | BuildingKind::SteamEngine => Color::new(0.15, 0.35, 0.35, 1.0),
        BuildingKind::SolarPanel => Color::new(0.15, 0.2, 0.5, 1.0),
        _ => Color::new(0.3, 0.3, 0.3, 1.0),
    }
}
