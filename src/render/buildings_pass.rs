//! Pass 3: buildings — sprites, tinting, status badges, and LOD-2 fallback rects.

use macroquad::prelude::*;

use crate::building::Buildings;
use crate::constants::*;
use crate::grid::Grid;
use crate::sprites::SpriteAtlas;
use crate::types::*;

use super::sprite_lookup::{building_lod_color, building_src_rect, direction_to_rotation};

/// Draws all buildings in view (Pass 3): full sprites at LOD 0-1, colored rects at LOD 2.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_buildings_pass(
    grid: &Grid,
    buildings: &Buildings,
    atlas: &SpriteAtlas,
    x_start: i32,
    x_end: i32,
    y_start: i32,
    y_end: i32,
    lod: i32,
    _anim_frame: usize,
    _machine_anim: usize,
    tick: u64,
    power_satisfaction: f32,
) {
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
            let mf = if building
                .machine_state
                .as_ref()
                .map(|ms| ms.progress_ticks > 0)
                .unwrap_or(false)
            {
                _machine_anim
            } else {
                0
            };

            // Full sprite rendering from unified atlas.
            // For belts at LOD 0: detect corners for corner sprite.
            // At LOD 1: skip corner detection (3 neighbor lookups per belt is expensive).
            let (src_rect, rotation) = if building.kind.is_belt() && lod == 0 {
                let dir = building.direction;
                let behind = bpos.neighbor(dir.opposite());
                let has_input_behind = grid
                    .get_tile(behind)
                    .and_then(|t| t.building)
                    .and_then(|bid2| buildings.get(bid2))
                    .map(|b2| b2.kind.is_belt() && b2.direction == dir)
                    .unwrap_or(false);

                let left_pos = bpos.neighbor(dir.rotated_ccw());
                let has_input_left = grid
                    .get_tile(left_pos)
                    .and_then(|t| t.building)
                    .and_then(|bid2| buildings.get(bid2))
                    .map(|b2| b2.kind.is_belt() && b2.direction == dir.rotated_cw())
                    .unwrap_or(false);

                let right_pos = bpos.neighbor(dir.rotated_cw());
                let has_input_right = grid
                    .get_tile(right_pos)
                    .and_then(|t| t.building)
                    .and_then(|bid2| buildings.get(bid2))
                    .map(|b2| b2.kind.is_belt() && b2.direction == dir.rotated_ccw())
                    .unwrap_or(false);

                let is_corner = !has_input_behind && (has_input_left || has_input_right);

                if is_corner {
                    let corner_src = if has_input_left {
                        match building.kind {
                            BuildingKind::BeltYellow => {
                                atlas.r_belt_corner_left_yellow[_anim_frame]
                            }
                            BuildingKind::BeltRed => atlas.r_belt_corner_left_red[_anim_frame],
                            BuildingKind::BeltBlue => atlas.r_belt_corner_left_blue[_anim_frame],
                            _ => unreachable!(),
                        }
                    } else {
                        match building.kind {
                            BuildingKind::BeltYellow => {
                                atlas.r_belt_corner_right_yellow[_anim_frame]
                            }
                            BuildingKind::BeltRed => atlas.r_belt_corner_right_red[_anim_frame],
                            BuildingKind::BeltBlue => atlas.r_belt_corner_right_blue[_anim_frame],
                            _ => unreachable!(),
                        }
                    };
                    (corner_src, direction_to_rotation(dir))
                } else {
                    (
                        building_src_rect(building.kind, atlas, _anim_frame, mf),
                        direction_to_rotation(dir),
                    )
                }
            } else {
                (
                    building_src_rect(building.kind, atlas, _anim_frame, mf),
                    direction_to_rotation(building.direction),
                )
            };

            // Damage tint: red when HP < 50%, flash when < 25%.
            let hp_ratio = if building.max_hp > 0.0 {
                building.hp / building.max_hp
            } else {
                1.0
            };
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
            if power_satisfaction < 0.5
                && !building.kind.needs_fuel()
                && !building.kind.is_belt()
                && !building.kind.is_inserter()
                && building.kind != BuildingKind::Miner
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
                    pivot: Some(Vec2::new(
                        world.x + TILE_SIZE * 0.5,
                        world.y + TILE_SIZE * 0.5,
                    )),
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
                    // Smoke puffs for active furnaces/boilers.
                    if ms.progress_ticks > 0
                        && (building.kind == BuildingKind::StoneFurnace
                            || building.kind == BuildingKind::SteelFurnace
                            || building.kind == BuildingKind::Boiler)
                    {
                        let smoke_phase = (tick as f32 * 0.1 + world.x * 0.3) % 3.0;
                        for i in 0..3u32 {
                            let phase = (smoke_phase + i as f32) % 3.0;
                            let sx = world.x + TILE_SIZE * 0.5 + (phase * 2.0).sin() * 3.0;
                            let sy = world.y - phase * 6.0;
                            let alpha = (1.0 - phase / 3.0) * 0.3;
                            let size = 2.0 + phase * 1.5;
                            draw_circle(sx, sy, size, Color::new(0.4, 0.4, 0.45, alpha));
                        }
                    }

                    // Electric furnace active glow (blue energy ring).
                    if ms.progress_ticks > 0 && building.kind == BuildingKind::ElectricFurnace {
                        let glow_phase = (tick as f32 * 0.15).sin() * 0.15 + 0.35;
                        let cx = world.x + TILE_SIZE * 0.5;
                        let cy = world.y + TILE_SIZE * 0.5;
                        draw_circle(
                            cx,
                            cy,
                            TILE_SIZE * 0.45,
                            Color::new(0.3, 0.5, 1.0, glow_phase * 0.3),
                        );
                        draw_circle(
                            cx,
                            cy,
                            TILE_SIZE * 0.3,
                            Color::new(0.4, 0.6, 1.0, glow_phase * 0.2),
                        );
                    }

                    // Item count badge (top-right corner) showing buffer contents.
                    let total_items = ms.input_buffer.len() + ms.output_buffer.len();
                    if total_items > 0 {
                        let badge_x = world.x + TILE_SIZE - 14.0;
                        let badge_y = world.y + 2.0;
                        draw_rectangle(
                            badge_x,
                            badge_y,
                            12.0,
                            12.0,
                            Color::new(0.1, 0.1, 0.2, 0.8),
                        );
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
                        && !ms.input_buffer.contains(&Resource::Coal)
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
                    } else if ms.progress_ticks == 0
                        && ms.input_buffer.is_empty()
                        && ms.selected_recipe.is_some()
                    {
                        // EMPTY: has a recipe but no inputs — needs items fed in.
                        let warn_x = world.x + 2.0;
                        let warn_y = world.y + TILE_SIZE - 16.0;
                        draw_rectangle(warn_x, warn_y, 44.0, 14.0, Color::new(0.3, 0.3, 0.6, 0.8));
                        draw_text("EMPTY", warn_x + 3.0, warn_y + 11.0, 11.0, WHITE);
                    }

                    // Low power badge for electric machines experiencing brownout.
                    if power_satisfaction < 0.5
                        && !building.kind.needs_fuel()
                        && !building.kind.is_belt()
                        && !building.kind.is_inserter()
                        && building.kind != BuildingKind::Miner
                    {
                        let warn_x = world.x + TILE_SIZE - 38.0;
                        let warn_y = world.y + 2.0;
                        let flash = ((tick as f32 * 0.2).sin() * 0.3 + 0.7).max(0.0);
                        draw_rectangle(
                            warn_x,
                            warn_y,
                            36.0,
                            12.0,
                            Color::new(0.6, 0.2, 0.6, 0.8 * flash),
                        );
                        draw_text(
                            "PWR!",
                            warn_x + 2.0,
                            warn_y + 10.0,
                            10.0,
                            Color::new(1.0, 0.8, 1.0, flash),
                        );
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
                    draw_line(
                        my_center.x,
                        my_center.y,
                        pair_world.x,
                        pair_world.y,
                        1.5,
                        Color::new(0.4, 0.6, 1.0, 0.3),
                    );
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
            if lod == 0
                && (building.kind == BuildingKind::AssemblerT1
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
            draw_rectangle(
                world.x + 1.0,
                world.y + 1.0,
                TILE_SIZE - 2.0,
                TILE_SIZE - 2.0,
                color,
            );
        }
    }
}
