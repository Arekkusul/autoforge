//! Inserter system: moves items between buildings, belts, and chests.
//!
//! Inserters are the bridge between belts and machines. Each inserter has a
//! source (tile behind it) and target (tile in front). Each tick it tries to
//! pick an item from the source and deliver it to the target.
//!
//! Uses a collect-then-apply pattern to avoid borrow conflicts.

use crate::building::Buildings;
use crate::constants::*;
use crate::grid::Grid;
use crate::item::ItemPool;
use crate::types::*;

/// Describes a transfer an inserter wants to make.
struct Transfer {
    inserter_id: BuildingId,
    source_pos: GridPos,
    target_pos: GridPos,
    source_building: Option<BuildingId>,
    target_building: Option<BuildingId>,
}

/// Ticks all inserters: pick from source, place at target.
///
/// Two-pass approach:
/// 1. Collect all inserter transfer intents (read-only scan).
/// 2. Execute transfers that are valid (mutating).
pub fn tick_inserters(grid: &mut Grid, buildings: &mut Buildings, items: &mut ItemPool) {
    let ids = buildings.alive_ids();

    // --- Pass 0: Decrement cooldowns FIRST (fixes off-by-one) ---
    for bid in &ids {
        let building = match buildings.get_mut(*bid) {
            Some(b) => b,
            None => continue,
        };
        if !building.kind.is_inserter() {
            continue;
        }
        if let Some(ms) = &mut building.machine_state {
            if ms.progress_ticks > 0 {
                ms.progress_ticks -= 1;
            }
        }
    }

    // --- Pass 1: Collect transfer intents ---
    let mut transfers: Vec<Transfer> = Vec::new();

    for bid in &ids {
        let building = match buildings.get(*bid) {
            Some(b) => b,
            None => continue,
        };
        if !building.kind.is_inserter() {
            continue;
        }

        let ms = match &building.machine_state {
            Some(ms) => ms,
            None => continue,
        };

        // Cooldown
        if ms.progress_ticks > 0 {
            continue;
        }

        let pos = building.pos;
        let dir = building.direction;
        let source_pos = pos.neighbor(dir.opposite());
        let target_pos = pos.neighbor(dir);

        let source_building = grid.get_tile(source_pos).and_then(|t| t.building);
        let target_building = grid.get_tile(target_pos).and_then(|t| t.building);

        transfers.push(Transfer {
            inserter_id: *bid,
            source_pos,
            target_pos,
            source_building,
            target_building,
        });
    }

    // --- Pass 2: Execute transfers ---
    for transfer in transfers {
        let inserter = match buildings.get(transfer.inserter_id) {
            Some(b) => b,
            None => continue,
        };
        let ms = match &inserter.machine_state {
            Some(ms) => ms,
            None => continue,
        };
        if ms.progress_ticks > 0 {
            continue;
        }

        let kind = inserter.kind;
        let swing_ticks = match kind {
            BuildingKind::InserterRegular => INSERTER_REGULAR_TICKS,
            BuildingKind::InserterLong => INSERTER_LONG_TICKS,
            BuildingKind::InserterFast => INSERTER_FAST_TICKS,
            BuildingKind::InserterStack => INSERTER_STACK_TICKS,
            _ => continue,
        };

        // Check if inserter is holding an item (in output_buffer).
        let holding = !ms.output_buffer.is_empty();

        if holding {
            // Try to deliver the held item to target.
            let resource = ms.output_buffer[0];
            let delivered = deliver_to_target(
                grid,
                buildings,
                items,
                transfer.target_pos,
                transfer.target_building,
                resource,
                transfer.inserter_id,
            );
            if delivered {
                let ins = buildings.get_mut(transfer.inserter_id).unwrap();
                let ms = ins.machine_state.as_mut().unwrap();
                ms.output_buffer.remove(0);
                ms.progress_ticks = swing_ticks;
            }
        } else {
            // Try to pick an item from source.
            let picked = pick_from_source(
                grid,
                buildings,
                items,
                transfer.source_pos,
                transfer.source_building,
                transfer.inserter_id,
            );
            if let Some(resource) = picked {
                let ins = buildings.get_mut(transfer.inserter_id).unwrap();
                let ms = ins.machine_state.as_mut().unwrap();
                ms.output_buffer.push(resource);
                ms.progress_ticks = swing_ticks;
            }
        }
    }

    // (Cooldowns already decremented in Pass 0 above.)
    // Pass 3 removed — was causing double-decrement.
    for _bid in &ids {
    }
}

/// Tries to pick one item from a source tile (belt, machine output buffer, or ground).
fn pick_from_source(
    grid: &mut Grid,
    buildings: &mut Buildings,
    items: &mut ItemPool,
    pos: GridPos,
    src_bid: Option<BuildingId>,
    _inserter_id: BuildingId,
) -> Option<Resource> {
    // If no building on source tile, try to pick items from the ground.
    if src_bid.is_none() {
        let item_ids: Vec<ItemId> = grid.items_at(pos).to_vec();
        for item_id in item_ids {
            if let Some(item) = items.get(item_id) {
                let resource = item.resource;
                items.despawn(item_id);
                grid.remove_item_from_tile(pos, item_id);
                return Some(resource);
            }
        }
        return None;
    }
    let bid = src_bid?;
    let src = buildings.get(bid)?;

    // Pick from belt.
    if src.kind.is_belt() {
        let item_ids: Vec<ItemId> = grid.items_at(pos).to_vec();
        for item_id in item_ids {
            if let Some(item) = items.get(item_id) {
                if item.progress >= 0.5 {
                    let resource = item.resource;
                    items.despawn(item_id);
                    grid.remove_item_from_tile(pos, item_id);
                    return Some(resource);
                }
            }
        }
        return None;
    }

    // Pick from machine output buffer (or storage chest input buffer).
    if let Some(ms) = &src.machine_state {
        // Storage chests: pick from input_buffer (it serves as general storage).
        if src.kind == BuildingKind::StorageChest {
            if !ms.input_buffer.is_empty() {
                let resource = ms.input_buffer[0];
                let src = buildings.get_mut(bid).unwrap();
                let ms = src.machine_state.as_mut().unwrap();
                ms.input_buffer.remove(0);
                return Some(resource);
            }
        } else if !ms.output_buffer.is_empty() {
            let resource = ms.output_buffer[0];
            let src = buildings.get_mut(bid).unwrap();
            let ms = src.machine_state.as_mut().unwrap();
            ms.output_buffer.remove(0);
            return Some(resource);
        }
    }

    None
}

/// Tries to deliver one item to a target tile (belt or machine input buffer).
fn deliver_to_target(
    grid: &mut Grid,
    buildings: &mut Buildings,
    items: &mut ItemPool,
    pos: GridPos,
    tgt_bid: Option<BuildingId>,
    resource: Resource,
    _inserter_id: BuildingId,
) -> bool {
    let bid = match tgt_bid {
        Some(b) => b,
        None => return false,
    };
    let tgt = match buildings.get(bid) {
        Some(b) => b,
        None => return false,
    };

    // Place onto belt.
    if tgt.kind.is_belt() {
        if grid.items_at(pos).is_empty() {
            let item_id = items.spawn(resource, pos);
            grid.add_item_to_tile(pos, item_id);
            return true;
        }
        return false;
    }

    // Place into machine input buffer.
    if let Some(ms) = &tgt.machine_state {
        // Storage chests have much larger capacity.
        let cap = if tgt.kind == BuildingKind::StorageChest {
            STORAGE_CHEST_STACKS * STACK_SIZE as usize
        } else {
            MACHINE_BUFFER_CAP
        };
        if ms.input_buffer.len() < cap {
            let tgt = buildings.get_mut(bid).unwrap();
            let ms = tgt.machine_state.as_mut().unwrap();
            ms.input_buffer.push(resource);
            return true;
        }
    }

    false
}
