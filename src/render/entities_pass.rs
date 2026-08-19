//! Pass 4+5: items in transit on belts, and enemies in the visible frustum.

use macroquad::prelude::*;

use crate::building::Buildings;
use crate::camera::GameCamera;
use crate::constants::*;
use crate::enemy::Enemies;
use crate::grid::Grid;
use crate::item::ItemPool;
use crate::sprites::SpriteAtlas;

use super::sprite_lookup::{item_lod_color, item_src_rect};

/// Draws items in transit on belts (Pass 4), frustum-culled to the given tile range.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_items_pass(
    grid: &Grid,
    buildings: &Buildings,
    items: &ItemPool,
    atlas: &SpriteAtlas,
    x_start: i32,
    x_end: i32,
    y_start: i32,
    y_end: i32,
    lod: i32,
) {
    // --- Pass 4: Items on belts (visible at LOD 0 and LOD 1) ---
    if lod <= 1 {
        for (_id, item) in items.iter() {
            let item_pos = item.pos;
            // Frustum cull items.
            if item_pos.x < x_start
                || item_pos.x > x_end
                || item_pos.y < y_start
                || item_pos.y > y_end
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
}

/// Draws enemies in the camera's visible bounds (Pass 5).
pub(super) fn draw_enemies_pass(
    enemies: &Enemies,
    camera: &GameCamera,
    atlas: &SpriteAtlas,
    lod: i32,
    _anim_frame: usize,
) {
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
            crate::enemy::EnemyKind::BigBiter | crate::enemy::EnemyKind::BehemothBiter => {
                TILE_SIZE * 1.1
            }
            crate::enemy::EnemyKind::BigSpitter | crate::enemy::EnemyKind::BehemothSpitter => {
                TILE_SIZE * 1.0
            }
            crate::enemy::EnemyKind::MediumBiter | crate::enemy::EnemyKind::MediumSpitter => {
                TILE_SIZE * 0.85
            }
            _ => TILE_SIZE * 0.7,
        };
        let ex = enemy.x - size * 0.5;
        let ey = enemy.y - size * 0.5;

        // Color tint by type + attack flash.
        let mut tint = match enemy.kind {
            crate::enemy::EnemyKind::SmallBiter => WHITE,
            crate::enemy::EnemyKind::MediumBiter => Color::new(1.0, 0.8, 0.6, 1.0),
            crate::enemy::EnemyKind::BigBiter => Color::new(1.0, 0.5, 0.3, 1.0),
            crate::enemy::EnemyKind::BehemothBiter => Color::new(0.8, 0.2, 0.8, 1.0),
            crate::enemy::EnemyKind::SmallSpitter => Color::new(0.7, 1.0, 0.7, 1.0),
            crate::enemy::EnemyKind::MediumSpitter => Color::new(0.5, 1.0, 0.5, 1.0),
            crate::enemy::EnemyKind::BigSpitter => Color::new(0.3, 0.9, 0.3, 1.0),
            crate::enemy::EnemyKind::BehemothSpitter => Color::new(0.2, 0.7, 1.0, 1.0),
        };
        // Flash red when attacking.
        if enemy.attack_cooldown > 15 {
            tint.r = (tint.r + 0.4).min(1.0);
            tint.g *= 0.5;
            tint.b *= 0.5;
        }

        if lod <= 1 {
            // Select sprite based on enemy type.
            let sprite = match enemy.kind {
                crate::enemy::EnemyKind::BigBiter | crate::enemy::EnemyKind::BehemothBiter => {
                    atlas.r_enemy_big_biter[_anim_frame]
                }
                crate::enemy::EnemyKind::SmallSpitter
                | crate::enemy::EnemyKind::MediumSpitter
                | crate::enemy::EnemyKind::BigSpitter
                | crate::enemy::EnemyKind::BehemothSpitter => atlas.r_enemy_spitter[_anim_frame],
                _ => atlas.r_enemy_small_biter[_anim_frame],
            };

            // Rotate sprite to face movement direction + body bob for walk animation.
            let rotation = enemy.facing + std::f32::consts::FRAC_PI_2;
            let bob = if _anim_frame == 0 { -0.5 } else { 0.5 }; // subtle vertical bob
            draw_texture_ex(
                &atlas.tex,
                ex,
                ey + bob,
                tint,
                DrawTextureParams {
                    source: Some(sprite),
                    dest_size: Some(Vec2::splat(size)),
                    rotation,
                    pivot: Some(Vec2::new(enemy.x, enemy.y + bob)),
                    ..Default::default()
                },
            );
        } else {
            // LOD 2: red dot.
            draw_circle(
                enemy.x,
                enemy.y,
                TILE_SIZE * 0.3,
                Color::new(0.9, 0.1, 0.1, 0.9),
            );
        }

        // Health bar (LOD 0-1, wider bar for visibility).
        if lod <= 1 {
            let max_hp = enemy.kind.max_hp();
            if enemy.hp < max_hp {
                let bar_w = size * 1.2;
                let bar_h = 4.0;
                let bar_x = ex - size * 0.1;
                let bar_y = ey - 6.0;
                let fill = (enemy.hp / max_hp).max(0.0);
                let fill_color = if fill > 0.5 {
                    Color::new(0.9, 0.5, 0.1, 0.9)
                } else {
                    Color::new(0.9, 0.1, 0.1, 0.9)
                };
                draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.15, 0.0, 0.0, 0.8));
                draw_rectangle(bar_x, bar_y, bar_w * fill, bar_h, fill_color);
            }
        }
    }
}
