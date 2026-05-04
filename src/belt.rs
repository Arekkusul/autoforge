//! Belt item movement system.
//!
//! Items on belts advance their `progress` each tick. When progress reaches 1.0,
//! the item attempts to move to the next tile in the belt's direction. If the
//! next tile is a belt with no item, the transfer succeeds. If blocked, the item
//! waits at progress ~1.0 until space opens.
//!
//! Items are rendered with interpolated positions between ticks for smooth motion.

use crate::building::Buildings;
use crate::constants::*;
use crate::grid::Grid;
use crate::item::ItemPool;
use crate::types::*;

/// Advances all items on belts by one tick.
///
/// Items further downstream are processed first to avoid pile-up artifacts.
/// The belt tier determines movement speed.
pub fn tick_belts(grid: &mut Grid, buildings: &Buildings, items: &mut ItemPool) {
    let ids = items.alive_ids();

    for id in ids {
        let item = match items.get(id) {
            Some(i) => i,
            None => continue,
        };

        let item_pos = item.pos;

        // Check if this item is on a belt tile.
        let tile = match grid.get_tile(item_pos) {
            Some(t) => t,
            None => continue,
        };
        let building_id = match tile.building {
            Some(bid) => bid,
            None => continue,
        };
        let building = match buildings.get(building_id) {
            Some(b) => b,
            None => continue,
        };

        // Determine speed based on belt tier.
        let move_ticks = match building.kind {
            BuildingKind::BeltYellow => BELT_YELLOW_TICKS,
            BuildingKind::BeltRed => BELT_RED_TICKS,
            BuildingKind::BeltBlue => BELT_BLUE_TICKS,
            _ => continue, // not a belt
        };

        let speed = 1.0 / move_ticks as f32;

        // Advance progress.
        let item = items.get_mut(id).unwrap();
        item.progress += speed;

        if item.progress >= 1.0 {
            let next_pos = item_pos.neighbor(building.direction);

            // Check if next tile is a belt (or underground belt) with space.
            let mut dest_pos = next_pos;
            let can_move = if let Some(next_tile) = grid.get_tile(next_pos) {
                if let Some(next_bid) = next_tile.building {
                    if let Some(next_b) = buildings.get(next_bid) {
                        if next_b.kind.is_belt() {
                            // Regular belt — move if empty.
                            grid.items_at(next_pos).is_empty()
                        } else if next_b.kind.is_underground_belt() {
                            // Underground belt entry — teleport to paired exit.
                            if let Some(exit_pos) = next_b.underground_pair {
                                if grid.items_at(exit_pos).is_empty() {
                                    dest_pos = exit_pos;
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false // unpaired underground belt
                            }
                        } else {
                            false // non-belt building
                        }
                    } else {
                        false
                    }
                } else {
                    false // empty tile
                }
            } else {
                false // out of bounds
            };

            let item = items.get_mut(id).unwrap();
            if can_move {
                grid.remove_item_from_tile(item_pos, id);
                item.pos = dest_pos;
                item.progress -= 1.0;
                grid.add_item_to_tile(dest_pos, id);
            } else {
                item.progress = 0.99;
            }
        }
    }
}
