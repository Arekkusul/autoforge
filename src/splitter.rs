//! Splitter system: splits items from one belt into two output belts.
//!
//! A splitter is placed on a belt line. Items arriving at the splitter alternate
//! between going left (relative to splitter facing) and right. The splitter's
//! `direction` field determines its facing; items exit to the left and right sides.
//!
//! For this implementation, splitters act as belt tiles that output alternating
//! items to the tile in their facing direction and the tile to their right.

use crate::building::Buildings;
use crate::grid::Grid;
use crate::item::ItemPool;
use crate::types::*;

/// Ticks all splitters: route items arriving at splitter tiles.
///
/// A splitter works by checking for items that have reached it (progress >= 0.99)
/// and redirecting them alternately to two output directions.
pub fn tick_splitters(grid: &mut Grid, buildings: &mut Buildings, items: &mut ItemPool) {
    let ids = buildings.alive_ids();

    for bid in ids {
        let building = match buildings.get(bid) {
            Some(b) => b,
            None => continue,
        };
        if building.kind != BuildingKind::Splitter {
            continue;
        }

        let pos = building.pos;
        let dir = building.direction;

        // Splitter outputs: straight ahead and to the right.
        let out_straight = pos.neighbor(dir);
        let out_right = pos.neighbor(dir.rotated_cw());

        // Check for items on the splitter tile.
        let item_ids: Vec<ItemId> = grid.items_at(pos).to_vec();
        if item_ids.is_empty() {
            continue;
        }

        for item_id in item_ids {
            let item = match items.get(item_id) {
                Some(i) => i,
                None => continue,
            };
            if item.progress < 0.99 {
                continue;
            }

            // Alternate based on a counter stored in the machine state.
            let counter = buildings
                .get(bid)
                .and_then(|b| b.machine_state.as_ref())
                .map(|ms| ms.fuel_ticks) // reuse fuel_ticks as counter
                .unwrap_or(0);

            let target = if counter.is_multiple_of(2) {
                out_straight
            } else {
                out_right
            };

            // Check if target is a belt with space.
            let can_move = if let Some(tile) = grid.get_tile(target) {
                if let Some(tbid) = tile.building {
                    if let Some(tb) = buildings.get(tbid) {
                        tb.kind.is_belt() && grid.items_at(target).is_empty()
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

            if can_move {
                let resource = items.get(item_id).unwrap().resource;
                items.despawn(item_id);
                grid.remove_item_from_tile(pos, item_id);
                let new_id = items.spawn(resource, target);
                grid.add_item_to_tile(target, new_id);

                // Increment counter.
                if let Some(b) = buildings.get_mut(bid) {
                    if let Some(ms) = &mut b.machine_state {
                        ms.fuel_ticks = counter.wrapping_add(1);
                    }
                }
                break; // one item per tick per splitter
            }
        }
    }
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

    fn splitter_building(pos: GridPos, dir: Direction) -> Building {
        Building {
            kind: BuildingKind::Splitter,
            pos,
            direction: dir,
            machine_state: Some(MachineState::new()),
            hp: 1.0,
            max_hp: 1.0,
            underground_pair: None,
        }
    }

    /// Places an item on the splitter tile already ready to be routed (progress >= 0.99).
    fn ready_item(grid: &mut Grid, items: &mut ItemPool, pos: GridPos, res: Resource) -> ItemId {
        let id = items.spawn(res, pos);
        items.get_mut(id).unwrap().progress = 0.99;
        grid.add_item_to_tile(pos, id);
        id
    }

    /// Splitter facing east at (2,2): straight output at (3,2), right output at (2,3).
    fn setup() -> (Grid, Buildings, ItemPool, BuildingId, GridPos, GridPos, GridPos) {
        let mut grid = Grid::new(16, 16);
        let mut buildings = Buildings::new();
        let items = ItemPool::new(64);
        let sp = GridPos::new(2, 2);
        let straight = GridPos::new(3, 2); // east
        let right = GridPos::new(2, 3); // south (east rotated cw)
        let sid = buildings.place(splitter_building(sp, Direction::East), &mut grid).unwrap();
        buildings.place(belt_building(straight, Direction::East), &mut grid).unwrap();
        buildings.place(belt_building(right, Direction::South), &mut grid).unwrap();
        (grid, buildings, items, sid, sp, straight, right)
    }

    /// Removes an item from the world so an output belt tile is free again.
    fn clear(grid: &mut Grid, items: &mut ItemPool, pos: GridPos, id: ItemId) {
        items.despawn(id);
        grid.remove_item_from_tile(pos, id);
    }

    #[test]
    fn splitter_alternates_between_straight_and_right_outputs() {
        let (mut grid, mut buildings, mut items, _sid, sp, straight, right) = setup();

        // 1st item -> straight (counter starts even).
        ready_item(&mut grid, &mut items, sp, Resource::IronOre);
        tick_splitters(&mut grid, &mut buildings, &mut items);
        let first = grid.items_at(straight).to_vec();
        assert_eq!(first.len(), 1, "first item routed straight ahead");
        assert!(grid.items_at(right).is_empty());
        clear(&mut grid, &mut items, straight, first[0]);

        // 2nd item -> right (counter now odd).
        ready_item(&mut grid, &mut items, sp, Resource::CopperOre);
        tick_splitters(&mut grid, &mut buildings, &mut items);
        let second = grid.items_at(right).to_vec();
        assert_eq!(second.len(), 1, "second item routed to the right");
        assert!(grid.items_at(straight).is_empty());
        clear(&mut grid, &mut items, right, second[0]);

        // 3rd item -> straight again (counter even).
        ready_item(&mut grid, &mut items, sp, Resource::Coal);
        tick_splitters(&mut grid, &mut buildings, &mut items);
        assert_eq!(grid.items_at(straight).len(), 1, "third item routed straight again");
        assert!(grid.items_at(right).is_empty());
    }

    #[test]
    fn splitter_preserves_resource_type_and_item_count() {
        let (mut grid, mut buildings, mut items, _sid, sp, straight, _right) = setup();
        ready_item(&mut grid, &mut items, sp, Resource::SteelPlate);

        tick_splitters(&mut grid, &mut buildings, &mut items);

        assert!(grid.items_at(sp).is_empty(), "item left the splitter tile");
        let out = grid.items_at(straight);
        assert_eq!(out.len(), 1);
        assert_eq!(
            items.get(out[0]).unwrap().resource,
            Resource::SteelPlate,
            "resource type is preserved through the split"
        );
        assert_eq!(items.alive_ids().len(), 1, "no duplication or loss (one despawn, one spawn)");
    }

    #[test]
    fn splitter_does_not_route_item_that_has_not_arrived() {
        let (mut grid, mut buildings, mut items, _sid, sp, straight, right) = setup();
        // Item is mid-transit across the splitter tile (progress < 0.99).
        let id = items.spawn(Resource::Gear, sp);
        items.get_mut(id).unwrap().progress = 0.5;
        grid.add_item_to_tile(sp, id);

        tick_splitters(&mut grid, &mut buildings, &mut items);

        assert_eq!(grid.items_at(sp), &[id], "unfinished item stays on the splitter");
        assert!(grid.items_at(straight).is_empty());
        assert!(grid.items_at(right).is_empty());
    }

    #[test]
    fn splitter_holds_item_when_selected_output_is_blocked() {
        let (mut grid, mut buildings, mut items, sid, sp, straight, _right) = setup();
        // Block the straight output (the even-counter target) with a resident item.
        let blocker = items.spawn(Resource::Stone, straight);
        grid.add_item_to_tile(straight, blocker);

        let id = ready_item(&mut grid, &mut items, sp, Resource::IronOre);
        tick_splitters(&mut grid, &mut buildings, &mut items);

        // Item is held on the splitter and the alternation counter does not advance.
        assert_eq!(grid.items_at(sp), &[id], "item waits on the splitter while straight is blocked");
        let counter = buildings
            .get(sid)
            .unwrap()
            .machine_state
            .as_ref()
            .unwrap()
            .fuel_ticks;
        assert_eq!(counter, 0, "counter only advances on a successful hand-off");
    }
}
