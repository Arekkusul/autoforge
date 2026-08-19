//! Pass 7 (faint tile grid at high zoom) and the crashed-ship landmark at map center.

use macroquad::prelude::*;

use crate::camera::GameCamera;
use crate::constants::*;
use crate::grid::Grid;
use crate::sprites::SpriteAtlas;

/// Draws faint tile grid lines when zoomed in close (Pass 7).
pub(super) fn draw_grid_lines(
    camera: &GameCamera,
    x_start: i32,
    x_end: i32,
    y_start: i32,
    y_end: i32,
) {
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
}

/// Draws the crashed ship landmark and its idle robot dock at the map center.
pub(super) fn draw_crashed_ship(
    grid: &Grid,
    atlas: &SpriteAtlas,
    min_world: Vec2,
    max_world: Vec2,
    lod: i32,
    tick: u64,
) {
    // --- Crashed ship / base at map center ---
    {
        let ship_cx = grid.width as f32 * TILE_SIZE * 0.5;
        let ship_cy = grid.height as f32 * TILE_SIZE * 0.5;
        let ship_w = TILE_SIZE * 5.0;
        let ship_h = TILE_SIZE * 3.0;
        let ship_x = ship_cx - ship_w * 0.5;
        let ship_y = ship_cy - ship_h * 0.5;

        if ship_x >= min_world.x - ship_w
            && ship_x <= max_world.x + ship_w
            && ship_y >= min_world.y - ship_h
            && ship_y <= max_world.y + ship_h
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

            // Nameplate with semi-transparent background panel.
            if lod == 0 {
                let np_w = 120.0;
                let np_h = 28.0;
                let np_x = ship_cx - np_w * 0.5;
                let np_y = ship_y - np_h - 4.0;
                draw_rectangle(np_x, np_y, np_w, np_h, Color::new(0.06, 0.06, 0.1, 0.8));
                draw_rectangle_lines(np_x, np_y, np_w, np_h, 1.0, Color::new(0.3, 0.25, 0.5, 0.6));
                draw_text(
                    "FORGE BASE",
                    np_x + 14.0,
                    np_y + 14.0,
                    16.0,
                    Color::new(0.95, 0.82, 0.35, 0.9),
                );
                draw_text(
                    "Horizon's Promise",
                    np_x + 8.0,
                    np_y + 26.0,
                    11.0,
                    Color::new(0.5, 0.5, 0.6, 0.6),
                );
                // Interaction hint below the ship.
                draw_text(
                    "[ click for lore ]",
                    ship_cx - 48.0,
                    ship_y + ship_h + 18.0,
                    12.0,
                    Color::new(0.6, 0.5, 0.8, 0.5),
                );
            }

            // Robot docking area — idle robots parked near the ship.
            let dock_x = ship_x + ship_w + 8.0;
            let dock_y = ship_cy - 10.0;
            let robot_size = TILE_SIZE * 0.4;
            let robot_frame = ((tick / 4) % 2) as usize;
            for i in 0..4u32 {
                let rx = dock_x + (i % 2) as f32 * (robot_size + 4.0);
                let ry = dock_y + (i / 2) as f32 * (robot_size + 4.0);
                draw_texture_ex(
                    &atlas.tex,
                    rx,
                    ry,
                    Color::new(1.0, 1.0, 1.0, 0.7),
                    DrawTextureParams {
                        source: Some(atlas.r_robot[robot_frame]),
                        dest_size: Some(Vec2::splat(robot_size)),
                        ..Default::default()
                    },
                );
            }
        }
    }
}
