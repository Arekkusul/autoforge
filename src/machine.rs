//! Machine processing system.
//!
//! Handles three jobs each tick:
//! 1. **Process**: Active machines count down their timer. When done, push outputs to buffer.
//! 2. **Start**: Idle machines check input buffers for matching recipes and begin crafting.
//! 3. **Eject**: Machines with outputs push items onto adjacent output belts via inserters.
//!
//! Miners are a special case — they don't consume items, they extract from the tile's deposit.

use crate::building::Buildings;
use crate::constants::*;
use crate::game::GameStats;
use crate::grid::Grid;
use crate::item::ItemPool;
use crate::recipe::{self, RECIPES};
use crate::types::*;

/// Ticks all production machines: miners, smelters, assemblers, etc.
pub fn tick_machines(
    grid: &mut Grid,
    buildings: &mut Buildings,
    items: &mut ItemPool,
    stats: &mut GameStats,
) {
    let ids = buildings.alive_ids();

    for bid in ids {
        let building = match buildings.get(bid) {
            Some(b) => b,
            None => continue,
        };

        // Skip non-production buildings.
        let kind = building.kind;
        if kind.is_belt() || kind.is_inserter() || kind.is_underground_belt() {
            continue;
        }
        if building.machine_state.is_none() {
            continue;
        }

        let pos = building.pos;
        let direction = building.direction;

        // --- Handle miners specially ---
        if kind == BuildingKind::Miner {
            tick_miner(grid, buildings, items, bid, pos, direction, stats);
            continue;
        }

        // --- Generic machine processing ---
        let building = buildings.get_mut(bid).unwrap();
        let ms = building.machine_state.as_mut().unwrap();

        // STEP 1: If currently processing, count down.
        if ms.progress_ticks > 0 {
            // Fuel-based machines (stone/steel furnace) need fuel to operate.
            if kind.needs_fuel() {
                if ms.fuel_ticks == 0 {
                    // Try to burn coal from input buffer.
                    if let Some(coal_idx) = ms.input_buffer.iter().position(|&r| r == Resource::Coal) {
                        ms.input_buffer.remove(coal_idx);
                        ms.fuel_ticks = COAL_FUEL_TICKS;
                    } else {
                        continue; // no fuel, machine stalls
                    }
                }
                ms.fuel_ticks -= 1;
            }

            ms.progress_ticks -= 1;

            if ms.progress_ticks == 0 {
                // Crafting complete — push outputs.
                if let Some(rid) = ms.selected_recipe {
                    let recipe = &RECIPES[rid.0];
                    for &(resource, count) in recipe.outputs {
                        for _ in 0..count {
                            if ms.output_buffer.len() < MACHINE_BUFFER_CAP {
                                ms.output_buffer.push(resource);
                                stats.items_crafted += 1;
                            }
                        }
                    }
                    // Only clear recipe lock for furnaces (they auto-detect input type).
                    // Assemblers and chemical plants KEEP their lock permanently to
                    // prevent recipe collision (e.g., Pipe vs Gear both use IronPlate).
                    if kind.needs_fuel() || kind == BuildingKind::ElectricFurnace {
                        ms.selected_recipe = None;
                    }
                }
            }
            continue;
        }

        // STEP 2: Idle — try to start a new recipe.
        if ms.output_buffer.len() >= MACHINE_BUFFER_CAP {
            continue; // output full, can't start
        }

        if let Some(rid) = recipe::find_matching_recipe(kind, &ms.input_buffer, ms.selected_recipe) {
            let recipe = &RECIPES[rid.0];
            recipe::consume_inputs(&mut ms.input_buffer, recipe);

            // Lock assemblers to the first recipe they craft (prevents wrong-item bugs).
            if ms.selected_recipe.is_none() && (kind == BuildingKind::AssemblerT1
                || kind == BuildingKind::AssemblerT2
                || kind == BuildingKind::AssemblerT3
                || kind == BuildingKind::ChemicalPlant)
            {
                ms.selected_recipe = Some(rid);
            }

            // Apply speed based on machine tier.
            let ticks = match kind {
                BuildingKind::SteelFurnace => recipe.base_ticks * 2 / 3, // 1.5× speed
                BuildingKind::ElectricFurnace => recipe.base_ticks / 2,  // 2× speed
                BuildingKind::AssemblerT2 => recipe.base_ticks * 3 / 4,  // 1.33× speed
                BuildingKind::AssemblerT3 => recipe.base_ticks / 2,      // 2× speed
                _ => recipe.base_ticks,
            };

            ms.progress_ticks = ticks.max(1);
            ms.total_ticks = ticks.max(1);
            ms.selected_recipe = Some(rid);
        }
    }
}

/// Handles miner tick: extract ore from the tile deposit and push to output buffer.
///
/// IMPORTANT: Ejection runs FIRST every tick so the buffer can drain even while
/// mining or when full. Without this, the buffer fills to 8 and never empties.
fn tick_miner(
    grid: &mut Grid,
    buildings: &mut Buildings,
    items: &mut ItemPool,
    bid: BuildingId,
    pos: GridPos,
    direction: Direction,
    stats: &mut GameStats,
) {
    // --- STEP 1: Always try to eject output onto adjacent belt FIRST. ---
    // This ensures the buffer can drain regardless of mining state.
    let has_output = buildings
        .get(bid)
        .and_then(|b| b.machine_state.as_ref())
        .map(|ms| !ms.output_buffer.is_empty())
        .unwrap_or(false);

    if has_output {
        let out_pos = pos.neighbor(direction);
        let can_eject = if let Some(out_tile) = grid.get_tile(out_pos) {
            if let Some(out_bid) = out_tile.building {
                if let Some(out_b) = buildings.get(out_bid) {
                    out_b.kind.is_belt() && grid.items_at(out_pos).is_empty()
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if can_eject {
            let building = buildings.get_mut(bid).unwrap();
            let ms = building.machine_state.as_mut().unwrap();
            let resource = ms.output_buffer.remove(0);
            let item_id = items.spawn(resource, out_pos);
            grid.add_item_to_tile(out_pos, item_id);
        }
    }

    // --- STEP 2: If currently mining, count down. ---
    let building = buildings.get_mut(bid).unwrap();
    let ms = building.machine_state.as_mut().unwrap();

    if ms.progress_ticks > 0 {
        ms.progress_ticks -= 1;

        if ms.progress_ticks == 0 {
            // Mining complete — determine what we mined.
            if let Some(tile) = grid.get_tile(pos) {
                if let Some(deposit) = tile.deposit {
                    if let Some(resource) = deposit.mined_resource() {
                        if ms.output_buffer.len() < MACHINE_BUFFER_CAP {
                            ms.output_buffer.push(resource);
                            stats.items_crafted += 1;
                        }
                    }
                }
            }
            // Deplete ore.
            if let Some(tile) = grid.get_tile_mut(pos) {
                if tile.ore_amount != u32::MAX && tile.ore_amount > 0 {
                    tile.ore_amount -= 1;
                    if tile.ore_amount == 0 {
                        tile.deposit = None;
                        tile.ore_origin = false;
                    }
                }
            }
            ms.selected_recipe = None;
        }
        return;
    }

    // --- STEP 3: Idle — try to start a new mining job. ---
    if ms.output_buffer.len() >= MACHINE_BUFFER_CAP {
        return; // buffer full, wait for ejection to make space
    }

    let can_mine = grid
        .get_tile(pos)
        .and_then(|t| t.deposit)
        .and_then(|d| d.mined_resource())
        .is_some();

    if can_mine {
        let building = buildings.get_mut(bid).unwrap();
        let ms = building.machine_state.as_mut().unwrap();
        ms.progress_ticks = MINER_TICKS;
        ms.total_ticks = MINER_TICKS;
    }
}

/// Ejects items from machine output buffers onto adjacent belts.
///
/// This runs for non-miner machines (miners handle ejection internally).
/// Machines try their facing direction first, then check all 4 neighbors
/// for any belt with space. This makes placement more forgiving.
pub fn tick_machine_output(
    grid: &mut Grid,
    buildings: &mut Buildings,
    items: &mut ItemPool,
) {
    let ids = buildings.alive_ids();

    for bid in ids {
        let building = match buildings.get(bid) {
            Some(b) => b,
            None => continue,
        };

        let kind = building.kind;
        if kind.is_belt() || kind.is_inserter() || kind == BuildingKind::Miner {
            continue;
        }
        if building.machine_state.is_none() {
            continue;
        }

        let pos = building.pos;
        let direction = building.direction;

        let has_output = building
            .machine_state
            .as_ref()
            .map(|ms| !ms.output_buffer.is_empty())
            .unwrap_or(false);

        if !has_output {
            continue;
        }

        // Try facing direction first, then all 4 neighbors for any belt with space.
        let directions = [
            direction,
            direction.rotated_cw(),
            direction.opposite(),
            direction.rotated_cw().rotated_cw().rotated_cw(),
        ];

        let mut eject_pos = None;
        for dir in &directions {
            let check_pos = pos.neighbor(*dir);
            if let Some(tile) = grid.get_tile(check_pos) {
                if let Some(check_bid) = tile.building {
                    if let Some(check_b) = buildings.get(check_bid) {
                        if check_b.kind.is_belt() && grid.items_at(check_pos).is_empty() {
                            eject_pos = Some(check_pos);
                            break;
                        }
                    }
                }
            }
        }

        if let Some(out_pos) = eject_pos {
            let building = buildings.get_mut(bid).unwrap();
            let ms = building.machine_state.as_mut().unwrap();
            let resource = ms.output_buffer.remove(0);
            let item_id = items.spawn(resource, out_pos);
            grid.add_item_to_tile(out_pos, item_id);
        }
    }
}
