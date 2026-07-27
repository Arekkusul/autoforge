//! Belt item movement system.
//!
//! Items on belts advance their `progress` each tick. When progress reaches 1.0,
//! the item attempts to move to the next tile in the belt's direction. If the
//! next tile is a belt with no item, the transfer succeeds. If blocked, the item
//! waits at progress ~1.0 until space opens.
//!
//! Items are rendered with interpolated positions between ticks for smooth motion.

use crate::building::Buildings;
use crate::grid::Grid;
use crate::item::ItemPool;

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
        let move_ticks = match building.kind.belt_move_ticks() {
            Some(ticks) => ticks,
            None => continue, // not a belt
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::building::Building;
    use crate::types::*;

    /// Builds a bare belt/underground building (no machine state).
    fn belt_building(
        pos: GridPos,
        dir: Direction,
        kind: BuildingKind,
        underground_pair: Option<GridPos>,
    ) -> Building {
        Building {
            kind,
            pos,
            direction: dir,
            machine_state: None,
            hp: 1.0,
            max_hp: 1.0,
            underground_pair,
        }
    }

    /// Places a belt tile and returns its id.
    fn place_belt(
        grid: &mut Grid,
        buildings: &mut Buildings,
        pos: GridPos,
        dir: Direction,
        kind: BuildingKind,
    ) -> BuildingId {
        buildings
            .place(belt_building(pos, dir, kind, None), grid)
            .expect("belt placement should succeed on empty grass")
    }

    /// Spawns an item directly onto a tile and registers it in the spatial index.
    fn place_item(grid: &mut Grid, items: &mut ItemPool, pos: GridPos, res: Resource) -> ItemId {
        let id = items.spawn(res, pos);
        grid.add_item_to_tile(pos, id);
        id
    }

    fn world() -> (Grid, Buildings, ItemPool) {
        (Grid::new(16, 16), Buildings::new(), ItemPool::new(64))
    }

    #[test]
    fn yellow_belt_advances_quarter_tile_per_tick() {
        let (mut grid, mut buildings, mut items) = world();
        let a = GridPos::new(1, 1);
        let b = GridPos::new(2, 1);
        place_belt(&mut grid, &mut buildings, a, Direction::East, BuildingKind::BeltYellow);
        place_belt(&mut grid, &mut buildings, b, Direction::East, BuildingKind::BeltYellow);
        let id = place_item(&mut grid, &mut items, a, Resource::IronOre);

        // Yellow belt is 4 ticks/tile -> 0.25 progress per tick.
        for _ in 0..3 {
            tick_belts(&mut grid, &buildings, &mut items);
        }
        let item = items.get(id).unwrap();
        assert_eq!(item.pos, a, "item should still be on the first tile after 3 ticks");
        assert!((item.progress - 0.75).abs() < 1e-4, "progress={}", item.progress);

        // The 4th tick pushes progress to 1.0 and hands off to the next tile.
        tick_belts(&mut grid, &buildings, &mut items);
        let item = items.get(id).unwrap();
        assert_eq!(item.pos, b, "item should move to next belt on the 4th tick");
        assert!(item.progress < 0.01, "progress resets after moving: {}", item.progress);
        assert!(grid.items_at(a).is_empty(), "source tile spatial index cleared");
        assert_eq!(grid.items_at(b), &[id], "dest tile spatial index updated");
    }

    #[test]
    fn blue_belt_moves_every_tick() {
        let (mut grid, mut buildings, mut items) = world();
        let a = GridPos::new(1, 1);
        let b = GridPos::new(2, 1);
        place_belt(&mut grid, &mut buildings, a, Direction::East, BuildingKind::BeltBlue);
        place_belt(&mut grid, &mut buildings, b, Direction::East, BuildingKind::BeltBlue);
        let id = place_item(&mut grid, &mut items, a, Resource::CopperOre);

        // Blue belt is 1 tick/tile -> full progress in a single tick.
        tick_belts(&mut grid, &buildings, &mut items);
        assert_eq!(items.get(id).unwrap().pos, b);
    }

    #[test]
    fn item_at_end_of_line_clamps_and_is_not_lost() {
        let (mut grid, mut buildings, mut items) = world();
        let a = GridPos::new(1, 1);
        // Only one belt; the tile ahead (2,1) has no belt, so the item cannot advance.
        place_belt(&mut grid, &mut buildings, a, Direction::East, BuildingKind::BeltYellow);
        let id = place_item(&mut grid, &mut items, a, Resource::Coal);

        for _ in 0..10 {
            tick_belts(&mut grid, &buildings, &mut items);
        }
        let item = items.get(id).expect("blocked item must remain alive, never dropped");
        assert_eq!(item.pos, a, "blocked item stays put");
        assert!((item.progress - 0.99).abs() < 1e-4, "clamped to 0.99: {}", item.progress);
        assert_eq!(items.alive_ids().len(), 1, "no item duplication or loss");
    }

    #[test]
    fn item_blocked_by_occupied_downstream_belt() {
        let (mut grid, mut buildings, mut items) = world();
        let a = GridPos::new(1, 1);
        let b = GridPos::new(2, 1);
        // A -> B, but B's downstream (3,1) is not a belt, so B's item is stuck at B,
        // which in turn blocks A's item from advancing.
        place_belt(&mut grid, &mut buildings, a, Direction::East, BuildingKind::BeltYellow);
        place_belt(&mut grid, &mut buildings, b, Direction::East, BuildingKind::BeltYellow);
        let id_a = place_item(&mut grid, &mut items, a, Resource::IronOre);
        let id_b = place_item(&mut grid, &mut items, b, Resource::CopperOre);

        for _ in 0..12 {
            tick_belts(&mut grid, &buildings, &mut items);
        }
        // Both items survive on distinct tiles — never merged onto one tile.
        assert_eq!(items.get(id_a).unwrap().pos, a);
        assert_eq!(items.get(id_b).unwrap().pos, b);
        assert_eq!(grid.items_at(a), &[id_a]);
        assert_eq!(grid.items_at(b), &[id_b]);
        assert_eq!(items.alive_ids().len(), 2, "no duplication under back-pressure");
    }

    #[test]
    fn underground_belt_teleports_to_paired_exit() {
        let (mut grid, mut buildings, mut items) = world();
        let a = GridPos::new(1, 1);
        let entry = GridPos::new(2, 1);
        let exit = GridPos::new(6, 1);
        place_belt(&mut grid, &mut buildings, a, Direction::East, BuildingKind::BeltYellow);
        // Underground entry paired to a far exit tile.
        buildings
            .place(
                belt_building(
                    entry,
                    Direction::East,
                    BuildingKind::UndergroundBeltYellow,
                    Some(exit),
                ),
                &mut grid,
            )
            .expect("underground entry placement");
        buildings
            .place(
                belt_building(exit, Direction::East, BuildingKind::UndergroundBeltYellow, None),
                &mut grid,
            )
            .expect("underground exit placement");
        let id = place_item(&mut grid, &mut items, a, Resource::Stone);

        // 4 ticks to cross the yellow belt, then hand off into the underground network.
        for _ in 0..4 {
            tick_belts(&mut grid, &buildings, &mut items);
        }
        let item = items.get(id).unwrap();
        assert_eq!(item.pos, exit, "item teleports to the paired exit, skipping the buried span");
        assert!(grid.items_at(entry).is_empty(), "item never rests on the entry tile");
        assert_eq!(grid.items_at(exit), &[id]);
    }

    #[test]
    fn item_count_is_conserved_across_many_ticks() {
        let (mut grid, mut buildings, mut items) = world();
        // A 4-tile belt line running east.
        for x in 1..=4 {
            place_belt(
                &mut grid,
                &mut buildings,
                GridPos::new(x, 1),
                Direction::East,
                BuildingKind::BeltYellow,
            );
        }
        place_item(&mut grid, &mut items, GridPos::new(1, 1), Resource::IronPlate);
        place_item(&mut grid, &mut items, GridPos::new(3, 1), Resource::CopperPlate);

        for _ in 0..40 {
            tick_belts(&mut grid, &buildings, &mut items);
        }
        assert_eq!(items.alive_ids().len(), 2, "belt movement never creates or destroys items");
    }
}
