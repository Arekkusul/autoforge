//! Enemy spawning, movement, and attack AI.
//!
//! Enemies spawn from nests when pollution accumulates. They path toward the
//! highest pollution (the factory) and attack buildings they encounter.

use serde::{Deserialize, Serialize};

use crate::building::Buildings;
use crate::constants::*;
use crate::grid::Grid;
use crate::types::*;

/// Enemy variant determines stats.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum EnemyKind {
    SmallBiter,
    MediumBiter,
    BigBiter,
    BehemothBiter,
    SmallSpitter,
    MediumSpitter,
    BigSpitter,
    BehemothSpitter,
}

impl EnemyKind {
    /// Maximum hit points.
    pub fn max_hp(self) -> f32 {
        match self {
            EnemyKind::SmallBiter => 15.0,
            EnemyKind::MediumBiter => 75.0,
            EnemyKind::BigBiter => 375.0,
            EnemyKind::BehemothBiter => 3000.0,
            EnemyKind::SmallSpitter => 10.0,
            EnemyKind::MediumSpitter => 50.0,
            EnemyKind::BigSpitter => 200.0,
            EnemyKind::BehemothSpitter => 1500.0,
        }
    }

    /// Movement speed in world pixels per tick.
    pub fn speed(self) -> f32 {
        match self {
            EnemyKind::SmallBiter => 1.6,
            EnemyKind::MediumBiter => 1.4,
            EnemyKind::BigBiter => 1.0,
            EnemyKind::BehemothBiter => 0.8,
            EnemyKind::SmallSpitter => 1.8,
            EnemyKind::MediumSpitter => 1.4,
            EnemyKind::BigSpitter => 1.0,
            EnemyKind::BehemothSpitter => 0.7,
        }
    }

    /// Damage dealt to buildings per attack tick.
    pub fn damage(self) -> f32 {
        match self {
            EnemyKind::SmallBiter => 7.0,
            EnemyKind::MediumBiter => 15.0,
            EnemyKind::BigBiter => 30.0,
            EnemyKind::BehemothBiter => 90.0,
            EnemyKind::SmallSpitter => 12.0,
            EnemyKind::MediumSpitter => 20.0,
            EnemyKind::BigSpitter => 35.0,
            EnemyKind::BehemothSpitter => 75.0,
        }
    }

    /// Minimum evolution factor for this enemy to spawn.
    pub fn min_evolution(self) -> f64 {
        match self {
            EnemyKind::SmallBiter => 0.0,
            EnemyKind::MediumBiter => 0.2,
            EnemyKind::BigBiter => 0.5,
            EnemyKind::BehemothBiter => 0.9,
            EnemyKind::SmallSpitter => 0.25,
            EnemyKind::MediumSpitter => 0.4,
            EnemyKind::BigSpitter => 0.6,
            EnemyKind::BehemothSpitter => 0.9,
        }
    }

    /// Whether this is a ranged (spitter) type.
    pub fn is_ranged(self) -> bool {
        matches!(
            self,
            EnemyKind::SmallSpitter
                | EnemyKind::MediumSpitter
                | EnemyKind::BigSpitter
                | EnemyKind::BehemothSpitter
        )
    }

    /// Attack range for spitters (world pixels). Biters attack adjacent only.
    pub fn attack_range(self) -> f32 {
        if self.is_ranged() {
            TILE_SIZE * 4.0
        } else {
            TILE_SIZE * 1.5
        }
    }
}

/// A live enemy entity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Enemy {
    pub kind: EnemyKind,
    /// World-space position (not grid-locked).
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    /// Ticks until next attack (if adjacent to building).
    pub attack_cooldown: u32,
    pub alive: bool,
    /// Facing angle in radians (0 = right/east, rotates clockwise in screen space).
    pub facing: f32,
}

/// Container for all enemies.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Enemies {
    pub list: Vec<Enemy>,
    /// Accumulated pollution absorbed by nests — triggers waves when threshold reached.
    pub wave_accumulator: f32,
    /// Current wave number (increases difficulty).
    pub wave_number: u32,
    /// Whether a wave warning has been shown for the current threshold.
    pub wave_warned: bool,
}

impl Enemies {
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawns a wave of enemies at the given nest positions.
    pub fn spawn_wave(&mut self, nests: &[GridPos], evolution: f64) {
        self.wave_number += 1;
        let count = (3 + self.wave_number * 2).min(60) as usize;

        for i in 0..count {
            if nests.is_empty() {
                break;
            }
            let nest = nests[i % nests.len()];
            let world_x = nest.x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
            let world_y = nest.y as f32 * TILE_SIZE + TILE_SIZE * 0.5;

            // Pick enemy kind based on evolution — harder enemies appear as evolution increases.
            let kind = if evolution >= 0.9 && i % 8 == 0 {
                if i % 2 == 0 { EnemyKind::BehemothBiter } else { EnemyKind::BehemothSpitter }
            } else if evolution >= 0.6 && i % 5 == 0 {
                if i % 2 == 0 { EnemyKind::BigSpitter } else { EnemyKind::BigBiter }
            } else if evolution >= 0.4 && i % 4 == 0 {
                EnemyKind::MediumSpitter
            } else if evolution >= 0.2 && i % 3 == 0 {
                EnemyKind::MediumBiter
            } else if evolution >= 0.25 && i % 4 == 0 {
                EnemyKind::SmallSpitter
            } else {
                EnemyKind::SmallBiter
            };

            self.list.push(Enemy {
                kind,
                x: world_x,
                y: world_y,
                hp: kind.max_hp(),
                attack_cooldown: 0,
                alive: true,
                facing: 0.0,
            });
        }
    }
}

/// Ticks enemy AI: movement toward factory, attacking buildings.
pub fn tick_enemies(
    grid: &mut Grid,
    buildings: &mut Buildings,
    enemies: &mut Enemies,
    nests: &[GridPos],
    evolution: &mut f64,
    total_ticks: u64,
    _enemies_killed: &mut u64,
) {
    // Check if we should spawn a wave.
    let total_pollution = crate::pollution::total_pollution(grid);
    let pollution_threshold = 50.0 + enemies.wave_number as f32 * 100.0;
    let time_threshold = 2400 + enemies.wave_number as u64 * 1200;
    let near_threshold = (total_pollution > pollution_threshold * 0.7
        || total_ticks > time_threshold.saturating_sub(600))
        && !enemies.wave_warned
        && !nests.is_empty();

    // Wave warning 30 seconds before.
    if near_threshold {
        enemies.wave_warned = true;
    }

    let should_spawn = (total_pollution > pollution_threshold || total_ticks > time_threshold)
        && total_ticks % 200 == 0
        && !nests.is_empty();
    if should_spawn {
        enemies.spawn_wave(nests, *evolution);
        enemies.wave_warned = false; // reset for next wave
    }

    // Advance evolution over time.
    *evolution = (*evolution + EVOLUTION_TIME_FACTOR).min(1.0);

    let center_x = grid.width as f32 * TILE_SIZE * 0.5;
    let center_y = grid.height as f32 * TILE_SIZE * 0.5;

    for enemy in &mut enemies.list {
        if !enemy.alive {
            continue;
        }

        // AI: move toward map center. Spitters stop at range.
        let dx = center_x - enemy.x;
        let dy = center_y - enemy.y;
        let dist = (dx * dx + dy * dy).sqrt();

        // Spitters stop at 4-tile range, biters get close.
        let stop_dist = if enemy.kind.is_ranged() {
            TILE_SIZE * 4.0
        } else {
            TILE_SIZE * 1.2
        };

        if dist > stop_dist {
            let speed = enemy.kind.speed();
            enemy.x += (dx / dist) * speed;
            enemy.y += (dy / dist) * speed;
            // Face movement direction.
            enemy.facing = dy.atan2(dx);
        }

        // Check if adjacent to a building — attack it.
        let grid_pos = Grid::world_to_grid(macroquad::prelude::Vec2::new(enemy.x, enemy.y));

        // Find nearest building to attack (range depends on type).
        let attack_range = if enemy.kind.is_ranged() { TILE_SIZE * 4.0 } else { TILE_SIZE * 1.5 };
        let search_radius = (attack_range / TILE_SIZE) as i32 + 1;

        let mut target_bid = None;
        let mut best_dist = f32::MAX;
        for dy in -search_radius..=search_radius {
            for dx in -search_radius..=search_radius {
                let check_pos = GridPos::new(grid_pos.x + dx, grid_pos.y + dy);
                if let Some(tile) = grid.get_tile(check_pos) {
                    if let Some(bid) = tile.building {
                        let bworld = Grid::grid_to_world_center(check_pos);
                        let bdx = bworld.x - enemy.x;
                        let bdy = bworld.y - enemy.y;
                        let dist = (bdx * bdx + bdy * bdy).sqrt();
                        if dist < attack_range && dist < best_dist {
                            best_dist = dist;
                            target_bid = Some(bid);
                        }
                    }
                }
            }
        }

        if let Some(bid) = target_bid {
            if enemy.attack_cooldown == 0 {
                if let Some(building) = buildings.get_mut(bid) {
                    building.hp -= enemy.kind.damage();
                    if building.hp <= 0.0 {
                        buildings.remove(bid, grid);
                    }
                }
                enemy.attack_cooldown = 20; // attack every ~1 second
            }
        }

        if enemy.attack_cooldown > 0 {
            enemy.attack_cooldown -= 1;
        }
    }

    // Remove dead enemies.
    // Count newly dead enemies for loot drops (tracked externally).
    enemies.list.retain(|e| e.alive);
}
