//! Standalone overlay draws that run outside the per-frame world pass:
//! the night-cycle darkness overlay and the building placement ghost preview.

use macroquad::prelude::*;

use crate::building::Buildings;
use crate::camera::GameCamera;
use crate::constants::*;
use crate::grid::Grid;
use crate::sprites::SpriteAtlas;
use crate::types::*;

use super::sprite_lookup::{building_src_rect, direction_to_rotation};

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
            if wx < min_vis.x - TILE_SIZE * 4.0
                || wx > max_vis.x + TILE_SIZE * 4.0
                || wy < min_vis.y - TILE_SIZE * 4.0
                || wy > max_vis.y + TILE_SIZE * 4.0
            {
                continue;
            }

            // Determine light color and radius based on building type.
            let (color, radius) = match building.kind {
                // Furnaces: warm orange fire glow.
                BuildingKind::StoneFurnace | BuildingKind::SteelFurnace => {
                    let active = building
                        .machine_state
                        .as_ref()
                        .map(|ms| ms.progress_ticks > 0)
                        .unwrap_or(false);
                    if active {
                        (Color::new(1.0, 0.7, 0.3, light_alpha), TILE_SIZE * 3.0)
                    } else {
                        continue;
                    }
                }
                // Labs: blue-purple glow.
                BuildingKind::Lab => {
                    let active = building
                        .machine_state
                        .as_ref()
                        .map(|ms| ms.progress_ticks > 0)
                        .unwrap_or(false);
                    if active {
                        (Color::new(0.5, 0.4, 1.0, light_alpha), TILE_SIZE * 2.5)
                    } else {
                        continue;
                    }
                }
                // Steam engines, solar: white glow (always on if placed).
                BuildingKind::SteamEngine => (
                    Color::new(0.9, 0.9, 1.0, light_alpha * 0.7),
                    TILE_SIZE * 2.0,
                ),
                // Laser turrets: blue glow.
                BuildingKind::LaserTurret => (
                    Color::new(0.3, 0.5, 1.0, light_alpha * 0.6),
                    TILE_SIZE * 2.5,
                ),
                // Nuclear reactor: green glow.
                BuildingKind::NuclearReactor => {
                    (Color::new(0.3, 1.0, 0.5, light_alpha), TILE_SIZE * 5.0)
                }
                // Assemblers when active: dim blue.
                BuildingKind::AssemblerT1
                | BuildingKind::AssemblerT2
                | BuildingKind::AssemblerT3 => {
                    let active = building
                        .machine_state
                        .as_ref()
                        .map(|ms| ms.progress_ticks > 0)
                        .unwrap_or(false);
                    if active {
                        (
                            Color::new(0.6, 0.6, 0.9, light_alpha * 0.4),
                            TILE_SIZE * 1.5,
                        )
                    } else {
                        continue;
                    }
                }
                _ => continue,
            };

            // Draw concentric circles for soft light falloff.
            draw_circle(
                wx,
                wy,
                radius,
                Color::new(color.r, color.g, color.b, color.a * 0.3),
            );
            draw_circle(
                wx,
                wy,
                radius * 0.6,
                Color::new(color.r, color.g, color.b, color.a * 0.5),
            );
            draw_circle(
                wx,
                wy,
                radius * 0.3,
                Color::new(color.r, color.g, color.b, color.a * 0.7),
            );
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

        // Contextual hint below ghost preview.
        draw_text(
            "R:rotate  Esc:cancel",
            world.x - 12.0,
            world.y + TILE_SIZE + 12.0,
            10.0,
            Color::new(0.7, 0.7, 0.8, 0.5),
        );

        // Inserter direction labels: show pickup ← and delivery → sides.
        if kind.is_inserter() {
            let (dx, dy) = direction.delta();
            let cx = world.x + TILE_SIZE * 0.5;
            let cy = world.y + TILE_SIZE * 0.5;
            // Highlight delivery tile (front) in green.
            let delivery_pos = grid_pos.neighbor(direction);
            let delivery_world = Grid::grid_to_world(delivery_pos);
            draw_rectangle(
                delivery_world.x,
                delivery_world.y,
                TILE_SIZE,
                TILE_SIZE,
                Color::new(0.2, 0.8, 0.2, 0.2),
            );
            draw_rectangle_lines(
                delivery_world.x,
                delivery_world.y,
                TILE_SIZE,
                TILE_SIZE,
                2.0,
                Color::new(0.3, 1.0, 0.3, 0.5),
            );
            // Highlight pickup tile (behind) in yellow.
            let pickup_pos = grid_pos.neighbor(direction.opposite());
            let pickup_world = Grid::grid_to_world(pickup_pos);
            draw_rectangle(
                pickup_world.x,
                pickup_world.y,
                TILE_SIZE,
                TILE_SIZE,
                Color::new(0.8, 0.7, 0.1, 0.15),
            );
            draw_rectangle_lines(
                pickup_world.x,
                pickup_world.y,
                TILE_SIZE,
                TILE_SIZE,
                2.0,
                Color::new(1.0, 0.9, 0.3, 0.5),
            );
            // Delivery side (front): green "OUT" label.
            let fx = cx + dx as f32 * TILE_SIZE * 0.7;
            let fy = cy + dy as f32 * TILE_SIZE * 0.7;
            draw_text(
                "OUT",
                fx - 10.0,
                fy + 4.0,
                11.0,
                Color::new(0.3, 1.0, 0.3, 0.7),
            );
            // Pickup side (behind): yellow "IN" label.
            let bx = cx - dx as f32 * TILE_SIZE * 0.7;
            let by = cy - dy as f32 * TILE_SIZE * 0.7;
            draw_text(
                "IN",
                bx - 6.0,
                by + 4.0,
                11.0,
                Color::new(1.0, 0.9, 0.3, 0.7),
            );
        }

        // Range indicators for turrets and roboports.
        let center_x = world.x + TILE_SIZE * 0.5;
        let center_y = world.y + TILE_SIZE * 0.5;
        if kind == BuildingKind::GunTurret || kind == BuildingKind::LaserTurret {
            draw_circle_lines(
                center_x,
                center_y,
                TILE_SIZE * 6.0,
                1.0,
                Color::new(1.0, 0.3, 0.3, 0.25),
            );
        } else if kind == BuildingKind::Roboport {
            draw_circle_lines(
                center_x,
                center_y,
                TILE_SIZE * 10.0,
                1.0,
                Color::new(0.3, 0.5, 1.0, 0.2),
            );
        }
    } else {
        // No building selected — just show cursor highlight.
        draw_rectangle_lines(world.x, world.y, TILE_SIZE, TILE_SIZE, 1.0, YELLOW);
    }
}
