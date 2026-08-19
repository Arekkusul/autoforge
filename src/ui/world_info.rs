//! World-position-dependent info: the hovered-tile inspector panel
//! (top-right) and the minimap (bottom-right, above the toolbar).

use macroquad::prelude::*;

use crate::building;
use crate::constants::{MACHINE_BUFFER_CAP, TILE_SIZE};
use crate::game::GameState;
use crate::grid;
use crate::recipe;
use crate::types;

use super::{
    building_description, draw_panel, fmt_num, short_resource_name, TEXT_ACCENT, TEXT_BRIGHT,
    TEXT_DIM,
};

pub(super) fn draw_hovered_tile_info(state: &GameState) {
    // --- Top-right: hovered tile info panel ---
    let mouse_screen = Vec2::new(mouse_position().0, mouse_position().1);
    let mouse_world = state.camera.screen_to_world(mouse_screen);
    let grid_pos = grid::Grid::world_to_grid(mouse_world);

    let panel_w = 300.0;
    let panel_x = screen_width() - panel_w - 10.0;

    if let Some(tile) = state.grid.get_tile(grid_pos) {
        let mut lines: Vec<(String, Color)> = Vec::new();
        lines.push((format!("({}, {})", grid_pos.x, grid_pos.y), TEXT_DIM));
        let terrain_info = if tile.terrain == types::Terrain::Forest {
            format!("{:?} (absorbs pollution)", tile.terrain)
        } else if tile.terrain == types::Terrain::Water {
            format!("{:?} (not buildable)", tile.terrain)
        } else {
            format!("{:?}", tile.terrain)
        };
        lines.push((terrain_info, TEXT_BRIGHT));

        // Show pollution level if significant.
        if tile.pollution > 0.1 {
            let pol_color = if tile.pollution > 1.0 {
                Color::new(0.9, 0.4, 0.2, 0.9)
            } else {
                Color::new(0.7, 0.6, 0.2, 0.8)
            };
            lines.push((format!("Pollution: {:.1}", tile.pollution), pol_color));
        }

        if let Some(deposit) = tile.deposit {
            let amount_str = if tile.ore_amount == u32::MAX {
                "Infinite".to_string()
            } else {
                fmt_num(tile.ore_amount as u64)
            };
            lines.push((
                format!("{} ({})", deposit.display_name(), amount_str),
                Color::new(0.9, 0.7, 0.3, 1.0),
            ));
        }

        if let Some(bid) = tile.building {
            if let Some(b) = state.buildings.get(bid) {
                lines.push((b.kind.display_name().to_string(), TEXT_ACCENT));
                // One-liner description.
                let desc = building_description(b.kind);
                if !desc.is_empty() {
                    lines.push((desc.to_string(), Color::new(0.6, 0.6, 0.7, 0.7)));
                }
                lines.push((
                    format!("Facing: {:?}  (R to rotate)", b.direction),
                    TEXT_DIM,
                ));
                // Miner: show what ore it's extracting.
                if b.kind == types::BuildingKind::Miner || b.kind == types::BuildingKind::PumpJack {
                    if let Some(deposit) = tile.deposit {
                        lines.push((
                            format!("Mining: {}", deposit.display_name()),
                            Color::new(0.9, 0.7, 0.3, 0.9),
                        ));
                    } else {
                        lines.push((
                            "No ore remaining!".to_string(),
                            Color::new(0.9, 0.3, 0.3, 0.9),
                        ));
                    }
                }
                // Show HP if damaged (with auto-repair hint).
                if b.hp < b.max_hp {
                    let hp_pct = (b.hp / b.max_hp * 100.0) as u32;
                    let hp_color = if hp_pct > 50 {
                        Color::new(0.9, 0.8, 0.2, 1.0)
                    } else {
                        Color::new(0.9, 0.3, 0.3, 1.0)
                    };
                    lines.push((
                        format!(
                            "HP: {}% ({:.0}/{:.0}) — auto-repairing",
                            hp_pct, b.hp, b.max_hp
                        ),
                        hp_color,
                    ));
                }
                // Belt speed info.
                if b.kind.is_belt() {
                    let (tier, speed) = match b.kind {
                        types::BuildingKind::BeltYellow => ("Yellow", "1x"),
                        types::BuildingKind::BeltRed => ("Red", "2x"),
                        types::BuildingKind::BeltBlue => ("Blue", "3x"),
                        _ => ("", ""),
                    };
                    if !tier.is_empty() {
                        lines.push((
                            format!("{} belt — {} speed (scroll to upgrade)", tier, speed),
                            Color::new(0.7, 0.7, 0.5, 0.8),
                        ));
                    }
                }
                // Splitter next-output indicator.
                if b.kind == types::BuildingKind::Splitter {
                    if let Some(ref ms) = b.machine_state {
                        let next_dir = if ms.fuel_ticks % 2 == 0 {
                            "straight"
                        } else {
                            "right"
                        };
                        lines.push((
                            format!("Next item goes: {}", next_dir),
                            Color::new(0.6, 0.7, 0.8, 0.8),
                        ));
                    }
                }
                if let Some(ref ms) = b.machine_state {
                    // Show recipe with inputs → outputs clearly.
                    if let Some(rid) = ms.selected_recipe {
                        if rid.0 < recipe::RECIPES.len() {
                            let r = &recipe::RECIPES[rid.0];
                            lines.push((
                                format!("Recipe: {}", r.name),
                                Color::new(0.9, 0.8, 0.4, 1.0),
                            ));
                            // Show what goes IN.
                            let inputs: String = r
                                .inputs
                                .iter()
                                .map(|(res, c)| format!("{}x {}", c, short_resource_name(*res)))
                                .collect::<Vec<_>>()
                                .join(" + ");
                            lines.push((
                                format!("Needs: {}", inputs),
                                Color::new(0.7, 0.8, 0.7, 0.9),
                            ));
                            // Show what comes OUT.
                            let outputs: String = r
                                .outputs
                                .iter()
                                .map(|(res, c)| format!("{}x {}", c, short_resource_name(*res)))
                                .collect::<Vec<_>>()
                                .join(" + ");
                            lines.push((
                                format!("Makes: {}", outputs),
                                Color::new(0.5, 0.9, 0.5, 0.9),
                            ));
                        }
                    } else if b.kind == types::BuildingKind::AssemblerT1
                        || b.kind == types::BuildingKind::AssemblerT2
                        || b.kind == types::BuildingKind::AssemblerT3
                        || b.kind == types::BuildingKind::ChemicalPlant
                    {
                        lines.push((
                            "Click to set recipe!".to_string(),
                            Color::new(0.9, 0.7, 0.3, 1.0),
                        ));
                    }
                    // Progress/status indicator (different for inserters vs machines).
                    if b.kind.is_inserter() {
                        if ms.progress_ticks > 0 {
                            lines.push(("Swinging...".to_string(), Color::new(0.4, 0.8, 1.0, 0.9)));
                        } else {
                            lines.push(("Ready".to_string(), Color::new(0.6, 0.6, 0.4, 0.8)));
                        }
                    } else if ms.progress_ticks > 0 && ms.total_ticks > 0 {
                        let pct = ((ms.total_ticks - ms.progress_ticks) as f32
                            / ms.total_ticks as f32
                            * 100.0) as u32;
                        lines.push((
                            format!("Progress: {}%", pct),
                            Color::new(0.4, 0.8, 1.0, 0.9),
                        ));
                    } else if ms.selected_recipe.is_some() && ms.progress_ticks == 0 {
                        lines.push((
                            "Idle — waiting for inputs".to_string(),
                            Color::new(0.6, 0.6, 0.4, 0.8),
                        ));
                    }
                    // Buffer contents — show item names for chests, counts for machines.
                    if !ms.input_buffer.is_empty() {
                        if b.kind == types::BuildingKind::StorageChest {
                            // Show first few item names.
                            let items: String = ms
                                .input_buffer
                                .iter()
                                .take(4)
                                .map(|r| short_resource_name(*r))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let more = if ms.input_buffer.len() > 4 {
                                format!(" +{}", ms.input_buffer.len() - 4)
                            } else {
                                String::new()
                            };
                            lines.push((format!("Stored: {}{}", items, more), TEXT_DIM));
                        } else {
                            lines.push((
                                format!("Input: {}/{}", ms.input_buffer.len(), MACHINE_BUFFER_CAP),
                                TEXT_DIM,
                            ));
                        }
                    }
                    if !ms.output_buffer.is_empty() {
                        lines.push((
                            format!("Output: {}/{}", ms.output_buffer.len(), MACHINE_BUFFER_CAP),
                            TEXT_DIM,
                        ));
                    }
                    if ms.fuel_ticks > 0 {
                        lines.push((
                            format!("Fuel: {:.1}s", ms.fuel_ticks as f32 / 20.0),
                            TEXT_DIM,
                        ));
                    }
                    // Show installed modules.
                    if !ms.modules.is_empty() {
                        let mod_names: String = ms
                            .modules
                            .iter()
                            .map(|m| short_resource_name(*m))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let slots_left = building::MAX_MODULE_SLOTS - ms.modules.len();
                        let slot_info = if slots_left > 0 {
                            format!(" ({} slot free)", slots_left)
                        } else {
                            String::new()
                        };
                        lines.push((
                            format!("Modules: {}{}", mod_names, slot_info),
                            Color::new(0.6, 0.8, 1.0, 0.9),
                        ));
                    } else if !b.kind.is_belt()
                        && !b.kind.is_inserter()
                        && b.kind != types::BuildingKind::StorageChest
                        && b.kind != types::BuildingKind::Wall
                        && b.kind != types::BuildingKind::Gate
                    {
                        lines.push((
                            "Modules: empty (middle-click to add)".to_string(),
                            Color::new(0.5, 0.5, 0.6, 0.6),
                        ));
                    }
                }
            }
        }

        // Show items on this tile (on belts).
        let items_here = state.grid.items_at(grid_pos);
        if !items_here.is_empty() {
            for &item_id in items_here.iter().take(3) {
                if let Some(item) = state.items.get(item_id) {
                    lines.push((
                        format!("Item: {}", item.resource.display_name()),
                        Color::new(0.9, 0.85, 0.5, 1.0),
                    ));
                }
            }
        }

        // Check for enemies near the cursor.
        let mouse_world = state.camera.screen_to_world(mouse_screen);
        for enemy in &state.enemies.list {
            if !enemy.alive {
                continue;
            }
            let dx = enemy.x - mouse_world.x;
            let dy = enemy.y - mouse_world.y;
            if dx * dx + dy * dy < (TILE_SIZE * TILE_SIZE) {
                let hp_pct = (enemy.hp / enemy.kind.max_hp() * 100.0) as u32;
                lines.push((
                    format!("{:?} — HP: {}%", enemy.kind, hp_pct),
                    Color::new(1.0, 0.4, 0.3, 1.0),
                ));
                lines.push((
                    format!(
                        "Dmg: {:.0}  Spd: {:.1}",
                        enemy.kind.damage(),
                        enemy.kind.speed()
                    ),
                    Color::new(0.8, 0.5, 0.4, 0.8),
                ));
                break; // only show one enemy
            }
        }

        // Check for nearby enemy nests.
        for nest_pos in &state.nests {
            let dist = grid_pos.distance(*nest_pos);
            if dist < 3.0 {
                lines.push((
                    "Enemy Nest nearby!".to_string(),
                    Color::new(0.8, 0.2, 0.2, 0.9),
                ));
                break;
            }
        }

        // Only show panel if there's meaningful info (skip bare grass tiles).
        let has_info = tile.deposit.is_some()
            || tile.building.is_some()
            || !items_here.is_empty()
            || lines.len() > 2;
        let panel_h = 8.0 + lines.len() as f32 * 20.0 + 8.0;
        if has_info || lines.len() > 2 {
            let (tx, mut ty) = draw_panel(panel_x, 8.0, panel_w, panel_h, None, false);
            for (text, color) in &lines {
                draw_text(text, tx, ty + 4.0, 14.0, *color);
                ty += 20.0;
            }
        }
    }
}

pub(super) fn draw_minimap(state: &mut GameState) {
    // --- Minimap (top-right corner, below info panel) ---
    {
        let mm_size = 140.0;
        let mm_x = screen_width() - mm_size - 10.0;
        let mm_y = screen_height() - 210.0 - mm_size; // above toolbar area
        let _panel_bg = Color::new(0.06, 0.06, 0.08, 0.9);

        draw_panel(
            mm_x - 4.0,
            mm_y - 24.0,
            mm_size + 8.0,
            mm_size + 56.0,
            Some("Map"),
            false,
        );

        // Draw a simplified view of the map (each pixel = 4 tiles).
        let tiles_per_pixel = 4;
        let map_pixels = (mm_size as i32).min(state.grid.width / tiles_per_pixel);

        // Center the minimap on the camera position.
        let cam_grid = grid::Grid::world_to_grid(state.camera.target);
        let half_range = map_pixels * tiles_per_pixel / 2;

        for py in 0..map_pixels {
            for px in 0..map_pixels {
                let gx = cam_grid.x - half_range + px * tiles_per_pixel;
                let gy = cam_grid.y - half_range + py * tiles_per_pixel;
                let pos = types::GridPos::new(gx, gy);

                let color = if let Some(tile) = state.grid.get_tile(pos) {
                    if tile.building.is_some() {
                        Color::new(0.5, 0.5, 0.8, 1.0) // buildings = blue dots
                    } else if tile.deposit.is_some() {
                        Color::new(0.7, 0.5, 0.2, 1.0) // ore = orange
                    } else if tile.terrain == types::Terrain::Water {
                        Color::new(0.2, 0.3, 0.6, 1.0) // water = dark blue
                    } else if tile.terrain == types::Terrain::Forest {
                        Color::new(0.1, 0.4, 0.1, 1.0) // forest = green
                    } else if tile.pollution > 0.15 {
                        Color::new(0.4, 0.4, 0.1, 0.8) // pollution = yellow-green
                    } else {
                        Color::new(0.15, 0.15, 0.12, 1.0) // ground = dark
                    }
                } else {
                    Color::new(0.05, 0.05, 0.05, 1.0) // out of bounds
                };

                let screen_px = mm_x + px as f32 * (mm_size / map_pixels as f32);
                let screen_py = mm_y + py as f32 * (mm_size / map_pixels as f32);
                let pixel_size = mm_size / map_pixels as f32;
                draw_rectangle(screen_px, screen_py, pixel_size, pixel_size, color);
            }
        }

        // Draw enemy nests as dark red diamonds on minimap.
        for nest_pos in &state.nests {
            let npx = (nest_pos.x - (cam_grid.x - half_range)) as f32
                / (map_pixels * tiles_per_pixel) as f32
                * mm_size;
            let npy = (nest_pos.y - (cam_grid.y - half_range)) as f32
                / (map_pixels * tiles_per_pixel) as f32
                * mm_size;
            if npx >= 0.0 && npx < mm_size && npy >= 0.0 && npy < mm_size {
                draw_circle(mm_x + npx, mm_y + npy, 3.0, Color::new(0.6, 0.1, 0.1, 0.8));
            }
        }

        // Draw enemies as red dots on minimap.
        for enemy in &state.enemies.list {
            if !enemy.alive {
                continue;
            }
            let eg = grid::Grid::world_to_grid(Vec2::new(enemy.x, enemy.y));
            let rpx = (eg.x - (cam_grid.x - half_range)) as f32
                / (map_pixels * tiles_per_pixel) as f32
                * mm_size;
            let rpy = (eg.y - (cam_grid.y - half_range)) as f32
                / (map_pixels * tiles_per_pixel) as f32
                * mm_size;
            if rpx >= 0.0 && rpx < mm_size && rpy >= 0.0 && rpy < mm_size {
                draw_circle(mm_x + rpx, mm_y + rpy, 2.0, Color::new(1.0, 0.1, 0.1, 0.9));
            }
        }

        // Camera viewport rectangle.
        let (vis_min, vis_max) = state.camera.visible_bounds();
        let vis_min_g = grid::Grid::world_to_grid(vis_min);
        let vis_max_g = grid::Grid::world_to_grid(vis_max);
        let rx = (vis_min_g.x - (cam_grid.x - half_range)) as f32
            / (map_pixels * tiles_per_pixel) as f32
            * mm_size;
        let ry = (vis_min_g.y - (cam_grid.y - half_range)) as f32
            / (map_pixels * tiles_per_pixel) as f32
            * mm_size;
        let rw =
            (vis_max_g.x - vis_min_g.x) as f32 / (map_pixels * tiles_per_pixel) as f32 * mm_size;
        let rh =
            (vis_max_g.y - vis_min_g.y) as f32 / (map_pixels * tiles_per_pixel) as f32 * mm_size;
        draw_rectangle_lines(mm_x + rx, mm_y + ry, rw, rh, 1.0, WHITE);

        // Click minimap to teleport camera.
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx >= mm_x && mx < mm_x + mm_size && my >= mm_y && my < mm_y + mm_size {
                let frac_x = (mx - mm_x) / mm_size;
                let frac_y = (my - mm_y) / mm_size;
                let target_gx = cam_grid.x - half_range
                    + (frac_x * (map_pixels * tiles_per_pixel) as f32) as i32;
                let target_gy = cam_grid.y - half_range
                    + (frac_y * (map_pixels * tiles_per_pixel) as f32) as i32;
                state.camera.target =
                    grid::Grid::grid_to_world_center(types::GridPos::new(target_gx, target_gy));
            }
        }

        // Minimap legend (color key).
        let lg_y = mm_y + mm_size + 4.0;
        let lg_items: &[(&str, Color)] = &[
            ("Bld", Color::new(0.5, 0.5, 0.8, 1.0)),
            ("Ore", Color::new(0.7, 0.5, 0.2, 1.0)),
            ("H2O", Color::new(0.2, 0.3, 0.6, 1.0)),
            ("Pol", Color::new(0.4, 0.4, 0.1, 1.0)),
            ("Nest", Color::new(0.6, 0.1, 0.1, 1.0)),
        ];
        for (i, (label, color)) in lg_items.iter().enumerate() {
            let lx = mm_x + i as f32 * 28.0;
            draw_rectangle(lx, lg_y, 6.0, 6.0, *color);
            draw_text(label, lx + 8.0, lg_y + 7.0, 9.0, TEXT_DIM);
        }
    }
}
