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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::building::{Building, MachineState};

    fn belt_building(pos: GridPos, dir: Direction) -> Building {
        Building {
            kind: BuildingKind::BeltYellow,
            pos,
            direction: dir,
            machine_state: None,
            hp: 1.0,
            max_hp: 1.0,
            underground_pair: None,
        }
    }

    fn machine_building(pos: GridPos, dir: Direction, kind: BuildingKind) -> Building {
        Building {
            kind,
            pos,
            direction: dir,
            machine_state: Some(MachineState::new()),
            hp: 1.0,
            max_hp: 1.0,
            underground_pair: None,
        }
    }

    /// Spawns a belt item at a given progress and registers it on the tile.
    fn place_item_with_progress(
        grid: &mut Grid,
        items: &mut ItemPool,
        pos: GridPos,
        res: Resource,
        progress: f32,
    ) -> ItemId {
        let id = items.spawn(res, pos);
        items.get_mut(id).unwrap().progress = progress;
        grid.add_item_to_tile(pos, id);
        id
    }

    fn world() -> (Grid, Buildings, ItemPool) {
        (Grid::new(16, 16), Buildings::new(), ItemPool::new(64))
    }

    /// Standard layout: belt source (0), inserter facing east (1), chest target (2).
    fn belt_inserter_chest(
        grid: &mut Grid,
        buildings: &mut Buildings,
    ) -> (GridPos, GridPos, BuildingId) {
        let source = GridPos::new(1, 1);
        let inserter_pos = GridPos::new(2, 1);
        let target = GridPos::new(3, 1);
        buildings
            .place(belt_building(source, Direction::East), grid)
            .unwrap();
        let ins = buildings
            .place(
                machine_building(inserter_pos, Direction::East, BuildingKind::InserterRegular),
                grid,
            )
            .unwrap();
        buildings
            .place(machine_building(target, Direction::East, BuildingKind::StorageChest), grid)
            .unwrap();
        (source, target, ins)
    }

    #[test]
    fn inserter_picks_from_belt_into_hand_then_delivers_to_chest() {
        let (mut grid, mut buildings, mut items) = world();
        let (source, target, ins) = belt_inserter_chest(&mut grid, &mut buildings);
        let target_bid = grid.get_tile(target).unwrap().building.unwrap();
        place_item_with_progress(&mut grid, &mut items, source, Resource::IronPlate, 0.9);

        // First tick: pick the belt item into the inserter's hand.
        tick_inserters(&mut grid, &mut buildings, &mut items);
        assert!(grid.items_at(source).is_empty(), "belt item was consumed");
        assert_eq!(items.alive_ids().len(), 0, "picked item leaves the world pool");
        let held = &buildings.get(ins).unwrap().machine_state.as_ref().unwrap().output_buffer;
        assert_eq!(held, &[Resource::IronPlate], "inserter now holds the item");

        // Run enough ticks for the swing cooldown to elapse and the item to be placed.
        for _ in 0..INSERTER_REGULAR_TICKS + 2 {
            tick_inserters(&mut grid, &mut buildings, &mut items);
        }
        let chest = buildings.get(target_bid).unwrap().machine_state.as_ref().unwrap();
        assert_eq!(chest.input_buffer, vec![Resource::IronPlate], "delivered to chest");
        assert!(
            buildings.get(ins).unwrap().machine_state.as_ref().unwrap().output_buffer.is_empty(),
            "hand is empty after delivery"
        );
    }

    #[test]
    fn inserter_ignores_belt_item_below_half_progress() {
        let (mut grid, mut buildings, mut items) = world();
        let (source, _target, ins) = belt_inserter_chest(&mut grid, &mut buildings);
        // Item has only just entered the belt tile (progress < 0.5): not yet reachable.
        let id = place_item_with_progress(&mut grid, &mut items, source, Resource::Coal, 0.4);

        tick_inserters(&mut grid, &mut buildings, &mut items);
        assert_eq!(grid.items_at(source), &[id], "item still on the belt");
        assert!(
            buildings.get(ins).unwrap().machine_state.as_ref().unwrap().output_buffer.is_empty(),
            "inserter picked nothing"
        );
    }

    #[test]
    fn inserter_moves_only_one_item_per_swing() {
        let (mut grid, mut buildings, mut items) = world();
        let (source, _target, ins) = belt_inserter_chest(&mut grid, &mut buildings);
        // Two reachable items sharing the source tile.
        place_item_with_progress(&mut grid, &mut items, source, Resource::IronPlate, 0.9);
        place_item_with_progress(&mut grid, &mut items, source, Resource::CopperPlate, 0.9);

        // Pick (tick 1), then a tick still inside the cooldown window must not grab again.
        tick_inserters(&mut grid, &mut buildings, &mut items);
        tick_inserters(&mut grid, &mut buildings, &mut items);

        assert_eq!(grid.items_at(source).len(), 1, "exactly one item remains on the belt");
        assert_eq!(
            buildings.get(ins).unwrap().machine_state.as_ref().unwrap().output_buffer.len(),
            1,
            "inserter holds exactly one item — the cooldown gates the second pick"
        );
    }

    #[test]
    fn inserter_pulls_from_machine_output_buffer_onto_belt() {
        let (mut grid, mut buildings, mut items) = world();
        let source = GridPos::new(1, 1);
        let inserter_pos = GridPos::new(2, 1);
        let target = GridPos::new(3, 1);
        // Source is a furnace with a finished plate in its output buffer.
        let furnace = buildings
            .place(machine_building(source, Direction::East, BuildingKind::StoneFurnace), &mut grid)
            .unwrap();
        buildings
            .get_mut(furnace)
            .unwrap()
            .machine_state
            .as_mut()
            .unwrap()
            .output_buffer
            .push(Resource::IronPlate);
        let ins = buildings
            .place(
                machine_building(inserter_pos, Direction::East, BuildingKind::InserterRegular),
                &mut grid,
            )
            .unwrap();
        buildings
            .place(belt_building(target, Direction::East), &mut grid)
            .unwrap();

        // Pick from furnace output.
        tick_inserters(&mut grid, &mut buildings, &mut items);
        assert!(
            buildings.get(furnace).unwrap().machine_state.as_ref().unwrap().output_buffer.is_empty(),
            "furnace output buffer drained"
        );
        assert_eq!(
            buildings.get(ins).unwrap().machine_state.as_ref().unwrap().output_buffer,
            vec![Resource::IronPlate]
        );

        // Deliver onto the belt after the swing cooldown.
        for _ in 0..INSERTER_REGULAR_TICKS + 2 {
            tick_inserters(&mut grid, &mut buildings, &mut items);
        }
        let on_belt = grid.items_at(target);
        assert_eq!(on_belt.len(), 1, "one fresh item spawned on the target belt");
        assert_eq!(items.get(on_belt[0]).unwrap().resource, Resource::IronPlate);
    }
}
