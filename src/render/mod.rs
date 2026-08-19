//! World and entity rendering.
//!
//! All drawing is split into two passes:
//! 1. **World-space** (under camera transform): terrain, buildings, items, enemies, ghost preview
//! 2. **Screen-space** (UI overlay): handled by [`crate::ui`]
//!
//! Performance: only tiles within the camera's visible bounds are drawn (frustum culling).

use macroquad::prelude::get_fps;

use crate::building::Buildings;
use crate::camera::GameCamera;
use crate::constants::BELT_ANIM_SPEED;
use crate::enemy::Enemies;
use crate::grid::Grid;
use crate::item::ItemPool;
use crate::sprites::SpriteAtlas;

mod buildings_pass;
mod entities_pass;
mod overlays;
mod scene_extras;
mod sprite_lookup;
mod terrain_pass;

pub use overlays::{draw_ghost_preview, draw_night_overlay};
pub use sprite_lookup::building_src_rect;

use buildings_pass::draw_buildings_pass;
use entities_pass::{draw_enemies_pass, draw_items_pass};
use scene_extras::{draw_crashed_ship, draw_grid_lines};
use terrain_pass::draw_terrain_pass;

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

    draw_terrain_pass(grid, atlas, x_start, x_end, y_start, y_end, lod, tick);

    draw_buildings_pass(
        grid,
        buildings,
        atlas,
        x_start,
        x_end,
        y_start,
        y_end,
        lod,
        _anim_frame,
        _machine_anim,
        tick,
        power_satisfaction,
    );

    draw_items_pass(
        grid, buildings, items, atlas, x_start, x_end, y_start, y_end, lod,
    );

    draw_enemies_pass(enemies, camera, atlas, lod, _anim_frame);

    draw_grid_lines(camera, x_start, x_end, y_start, y_end);

    draw_crashed_ship(grid, atlas, min_world, max_world, lod, tick);

    // Ambient particles removed — they added 12 draw calls per frame with no gameplay value.
    // Industry standard: cosmetic particles should be GPU instanced or in a particle buffer,
    // never individual draw calls.
}
