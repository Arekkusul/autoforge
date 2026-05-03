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

            let target = if counter % 2 == 0 {
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
