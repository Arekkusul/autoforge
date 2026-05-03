//! Combat system: turrets targeting and shooting enemies.
//!
//! Gun turrets consume ammo from their input buffer. Laser turrets use power
//! (not yet implemented — for now they work without ammo).

use crate::building::Buildings;
use crate::constants::*;
use crate::enemy::Enemies;
use crate::grid::Grid;
use crate::types::*;

/// Turret attack range in world pixels.
const GUN_TURRET_RANGE: f32 = TILE_SIZE * 6.0;
/// Damage per gun turret shot.
const GUN_TURRET_DAMAGE: f32 = 10.0;
/// Ticks between gun turret shots.
const GUN_TURRET_COOLDOWN: u32 = 10;

/// Ticks all turrets: find closest enemy in range, shoot, consume ammo.
pub fn tick_combat(
    _grid: &Grid,
    buildings: &mut Buildings,
    enemies: &mut Enemies,
    enemies_killed: &mut u64,
) {
    let ids = buildings.alive_ids();

    for bid in ids {
        let building = match buildings.get(bid) {
            Some(b) => b,
            None => continue,
        };

        if building.kind != BuildingKind::GunTurret && building.kind != BuildingKind::LaserTurret {
            continue;
        }

        let ms = match &building.machine_state {
            Some(ms) => ms,
            None => continue,
        };

        // Cooldown.
        if ms.progress_ticks > 0 {
            let building = buildings.get_mut(bid).unwrap();
            let ms = building.machine_state.as_mut().unwrap();
            ms.progress_ticks -= 1;
            continue;
        }

        // Gun turrets need ammo.
        if building.kind == BuildingKind::GunTurret {
            let has_ammo = ms.input_buffer.iter().any(|&r| {
                r == Resource::BasicAmmo || r == Resource::PiercingAmmo
            });
            if !has_ammo {
                continue;
            }
        }

        let turret_world = Grid::grid_to_world_center(building.pos);
        let range = GUN_TURRET_RANGE;

        // Find closest alive enemy in range.
        let mut closest_idx = None;
        let mut closest_dist = f32::MAX;

        for (i, enemy) in enemies.list.iter().enumerate() {
            if !enemy.alive {
                continue;
            }
            let dx = enemy.x - turret_world.x;
            let dy = enemy.y - turret_world.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < range && dist < closest_dist {
                closest_dist = dist;
                closest_idx = Some(i);
            }
        }

        if let Some(idx) = closest_idx {
            // Determine damage (piercing ammo does more).
            let building = buildings.get_mut(bid).unwrap();
            let ms = building.machine_state.as_mut().unwrap();

            let mut damage = GUN_TURRET_DAMAGE;

            if building.kind == BuildingKind::GunTurret {
                // Consume ammo — prefer piercing.
                if let Some(pos) = ms.input_buffer.iter().position(|&r| r == Resource::PiercingAmmo) {
                    ms.input_buffer.remove(pos);
                    damage = GUN_TURRET_DAMAGE * 2.0;
                } else if let Some(pos) = ms.input_buffer.iter().position(|&r| r == Resource::BasicAmmo) {
                    ms.input_buffer.remove(pos);
                }
            }

            ms.progress_ticks = GUN_TURRET_COOLDOWN;

            // Apply damage to enemy.
            let enemy = &mut enemies.list[idx];
            enemy.hp -= damage;
            if enemy.hp <= 0.0 {
                enemy.alive = false;
                *enemies_killed += 1;
            }
        }
    }
}
