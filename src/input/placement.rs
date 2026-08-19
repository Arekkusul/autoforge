//! Building placement, removal, blueprint pasting, and hand-inserting
//! items into machines — everything gated on a cursor grid position.

use macroquad::prelude::*;

use crate::buildcost;
use crate::building;
use crate::constants;
use crate::constants::{GATE_HP, WALL_HP};
use crate::game::GameState;
use crate::grid;
use crate::recipe;
use crate::sound;
use crate::types;

pub(super) fn handle_placement_and_removal(state: &mut GameState, sfx: &mut sound::SoundEffects) {
    let mouse_screen = Vec2::new(mouse_position().0, mouse_position().1);
    let mouse_world = state.camera.screen_to_world(mouse_screen);
    let grid_pos = grid::Grid::world_to_grid(mouse_world);

    // Handle recipe picker clicks (if open).
    if state.recipe_picker.is_some() && is_mouse_button_pressed(MouseButton::Left) {
        let sw = screen_width();
        let sh = screen_height();
        let pw = 340.0;
        let picker_recipes = state.recipe_picker.as_ref().unwrap().1.clone();
        let picker_bid = state.recipe_picker.as_ref().unwrap().0;
        let ph = 50.0 + picker_recipes.len() as f32 * 28.0;
        let px = (sw - pw) * 0.5;
        let py = (sh - ph) * 0.5;
        let mx = mouse_position().0;
        let my = mouse_position().1;

        let mut selected = false;
        for (i, rid) in picker_recipes.iter().enumerate() {
            let ry = py + 45.0 + i as f32 * 28.0;
            if mx >= px + 10.0 && mx <= px + pw - 10.0 && my >= ry - 10.0 && my <= ry + 16.0 {
                // Selected this recipe!
                if let Some(building) = state.buildings.get_mut(picker_bid) {
                    if let Some(ms) = &mut building.machine_state {
                        ms.selected_recipe = Some(*rid);
                        ms.input_buffer.clear();
                    }
                }
                let name = recipe::RECIPES[rid.0].name;
                state.toast(format!("Recipe set: {}", name), 60);
                selected = true;
                break;
            }
        }
        state.recipe_picker = None; // Close picker after any click
        if selected { /* already handled */ }
    }

    // (Recipe picker Escape is handled in the unified Escape handler above.)

    // Left click with no selection: interact with existing building (open recipe picker)
    // or interact with the crashed ship at map center.
    if state.selected_building.is_none()
        && state.recipe_picker.is_none()
        && is_mouse_button_pressed(MouseButton::Left)
    {
        // Check if clicking on the crashed ship (within 3 tiles of map center).
        let center = types::GridPos::new(state.grid.width / 2, state.grid.height / 2);
        if grid_pos.distance(center) < 4.0 {
            let lore_messages = [
                "The hull is cold. Scorched from atmospheric entry.",
                "You can see cryo pod fragments inside... empty.",
                "The ship's name: 'Horizon's Promise'. Your ship.",
                "Data core intact but encrypted. You need more processing power.",
                "A photo is stuck to the console: Dr. Vasquez and her team, smiling.",
                "The emergency beacon is pulsing faintly... someone might hear it.",
                "FORGE's memory banks are scattered across the crash site.",
                "The colony manifest: 4,000 souls in cryosleep. Destination: New Horizon.",
            ];
            // Cycle through lore on each click (uses notification count as click counter).
            let idx = state.notification_log.len() % lore_messages.len();
            state.toast(lore_messages[idx].to_string(), 120);
        }

        if let Some(tile) = state.grid.get_tile(grid_pos) {
            if let Some(bid) = tile.building {
                if let Some(b) = state.buildings.get(bid) {
                    // Click TrainStop → trains not yet fully implemented.
                    if b.kind == types::BuildingKind::TrainStop {
                        state.toast("Trains coming in a future update!".to_string(), 60);
                    } else
                    // If it's an assembler or chemical plant, open recipe picker popup.
                    if b.kind == types::BuildingKind::AssemblerT1
                        || b.kind == types::BuildingKind::AssemblerT2
                        || b.kind == types::BuildingKind::AssemblerT3
                        || b.kind == types::BuildingKind::ChemicalPlant
                    {
                        let available = recipe::recipes_for_machine(b.kind);
                        if !available.is_empty() {
                            state.recipe_picker = Some((bid, available));
                        }
                    }
                }
            }
        }
    }

    // Middle-click: hand-insert item from inventory into a machine.
    // Priority: modules first (if machine has open slots), then regular items.
    if is_mouse_button_pressed(MouseButton::Middle) {
        if let Some(tile) = state.grid.get_tile(grid_pos) {
            if let Some(bid) = tile.building {
                if let Some(building) = state.buildings.get(bid) {
                    if building.machine_state.is_some()
                        && !building.kind.is_belt()
                        && !building.kind.is_inserter()
                    {
                        let kind = building.kind;

                        // Try to insert a module first (if machine has open slots).
                        let module_slots = building
                            .machine_state
                            .as_ref()
                            .map(|ms| ms.modules.len())
                            .unwrap_or(0);
                        let mut inserted_module = false;
                        if module_slots < building::MAX_MODULE_SLOTS {
                            let module_types = [
                                types::Resource::SpeedModule,
                                types::Resource::EfficiencyModule,
                                types::Resource::ProductivityModule,
                            ];
                            for &mt in &module_types {
                                if state.inventory.get(&mt).copied().unwrap_or(0) > 0 {
                                    let building = state.buildings.get_mut(bid).unwrap();
                                    let ms = building.machine_state.as_mut().unwrap();
                                    ms.modules.push(mt);
                                    *state.inventory.entry(mt).or_insert(0) -= 1;
                                    state.toast(
                                        format!("Module installed: {}", mt.display_name()),
                                        60,
                                    );
                                    inserted_module = true;
                                    break;
                                }
                            }
                        }

                        if !inserted_module {
                            // Regular hand-insert: Coal for furnaces, recipe inputs for assemblers.
                            let building = state.buildings.get(bid).unwrap();
                            let to_insert = if kind.needs_fuel() {
                                if state
                                    .inventory
                                    .get(&types::Resource::Coal)
                                    .copied()
                                    .unwrap_or(0)
                                    > 0
                                {
                                    Some(types::Resource::Coal)
                                } else if state
                                    .inventory
                                    .get(&types::Resource::IronOre)
                                    .copied()
                                    .unwrap_or(0)
                                    > 0
                                {
                                    Some(types::Resource::IronOre)
                                } else {
                                    None
                                }
                            } else {
                                let ms = building.machine_state.as_ref().unwrap();
                                if let Some(rid) = ms.selected_recipe {
                                    let recipe_inputs = recipe::RECIPES[rid.0].inputs;
                                    recipe_inputs.iter().find_map(|(res, _)| {
                                        if state.inventory.get(res).copied().unwrap_or(0) > 0 {
                                            Some(*res)
                                        } else {
                                            None
                                        }
                                    })
                                } else {
                                    if state
                                        .inventory
                                        .get(&types::Resource::IronPlate)
                                        .copied()
                                        .unwrap_or(0)
                                        > 0
                                    {
                                        Some(types::Resource::IronPlate)
                                    } else {
                                        None
                                    }
                                }
                            };

                            if let Some(resource) = to_insert {
                                let building = state.buildings.get_mut(bid).unwrap();
                                let ms = building.machine_state.as_mut().unwrap();
                                if ms.input_buffer.len() < 8 {
                                    ms.input_buffer.push(resource);
                                    *state.inventory.entry(resource).or_insert(0) -= 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Blueprint paste: click to stamp buildings at cursor position.
    if state.pasting_blueprint && is_mouse_button_pressed(MouseButton::Left) {
        let mouse_screen = Vec2::new(mouse_position().0, mouse_position().1);
        let mouse_world = state.camera.screen_to_world(mouse_screen);
        let center = grid::Grid::world_to_grid(mouse_world);
        let mut placed = 0u32;
        for &(dx, dy, kind, dir) in &state.blueprint.clone() {
            let pos = types::GridPos::new(center.x + dx, center.y + dy);
            if !buildcost::can_afford(&state.inventory, kind) {
                continue;
            }
            let needs_ms = !kind.is_belt()
                && !kind.is_underground_belt()
                && !matches!(kind, types::BuildingKind::Wall | types::BuildingKind::Gate);
            let b = building::Building {
                kind,
                pos,
                direction: dir,
                machine_state: if needs_ms {
                    Some(building::MachineState::new())
                } else {
                    None
                },
                hp: 100.0,
                max_hp: 100.0,
                underground_pair: None,
            };
            if state.buildings.place(b, &mut state.grid).is_some() {
                buildcost::pay_cost(&mut state.inventory, kind);
                placed += 1;
            }
        }
        if placed > 0 {
            state.toast(format!("Pasted {} buildings!", placed), 60);
        }
        state.pasting_blueprint = false;
    }

    // Reset belt drag tracking when mouse is released.
    if !is_mouse_button_down(MouseButton::Left) {
        state.last_belt_pos = None;
    }

    // Left click: place building.
    // Hold left click for drag-placing belts.
    let should_place = if let Some(kind) = state.selected_building {
        if kind.is_belt() {
            is_mouse_button_down(MouseButton::Left)
        } else {
            is_mouse_button_pressed(MouseButton::Left)
        }
    } else {
        false
    };

    if should_place {
        if let Some(kind) = state.selected_building {
            // Check build zone — must be within radius of the ship (map center).
            let center = types::GridPos::new(state.grid.width / 2, state.grid.height / 2);
            let dist = grid_pos.distance(center);
            if dist > state.build_radius {
                if is_mouse_button_pressed(MouseButton::Left) {
                    state.toast("Too far from ship! Expand with research.".to_string(), 50);
                }
                return;
            }

            // Check if player can afford this building.
            if !buildcost::can_afford(&state.inventory, kind) {
                if is_mouse_button_pressed(MouseButton::Left) {
                    state.toast("Not enough resources!".to_string(), 40);
                    sfx.play(&sfx.error);
                }
                return;
            }

            // Auto-rotate belts during drag-placement based on movement direction.
            // When direction changes (corner), retroactively update the previous belt
            // to face the new direction so items flow through the corner correctly.
            if kind.is_belt() {
                if let Some(last_pos) = state.last_belt_pos {
                    if last_pos != grid_pos {
                        let dx = grid_pos.x - last_pos.x;
                        let dy = grid_pos.y - last_pos.y;
                        let new_dir = if dx.abs() >= dy.abs() {
                            if dx > 0 {
                                types::Direction::East
                            } else {
                                types::Direction::West
                            }
                        } else {
                            if dy > 0 {
                                types::Direction::South
                            } else {
                                types::Direction::North
                            }
                        };

                        // If direction changed, update the PREVIOUS belt to face the new
                        // direction (creating a proper corner where items enter from the
                        // side and exit in the new direction).
                        if new_dir != state.placement_direction {
                            if let Some(tile) = state.grid.get_tile(last_pos) {
                                if let Some(bid) = tile.building {
                                    if let Some(prev_belt) = state.buildings.get_mut(bid) {
                                        if prev_belt.kind.is_belt() {
                                            prev_belt.direction = new_dir;
                                        }
                                    }
                                }
                            }
                        }

                        state.placement_direction = new_dir;
                    }
                }
            }

            let needs_machine_state = !kind.is_belt()
                && !kind.is_underground_belt()
                && !matches!(kind, types::BuildingKind::Wall | types::BuildingKind::Gate);

            // Underground belt pairing logic.
            let underground_pair = if kind.is_underground_belt() {
                find_underground_pair(&state.buildings, grid_pos, state.placement_direction, kind)
            } else {
                None
            };

            let (hp, max_hp) = match kind {
                types::BuildingKind::Wall => (WALL_HP, WALL_HP),
                types::BuildingKind::Gate => (GATE_HP, GATE_HP),
                types::BuildingKind::GunTurret => (200.0, 200.0),
                types::BuildingKind::LaserTurret => (200.0, 200.0),
                _ => (100.0, 100.0),
            };

            let b = building::Building {
                kind,
                pos: grid_pos,
                direction: state.placement_direction,
                machine_state: if needs_machine_state {
                    Some(building::MachineState::new())
                } else {
                    None
                },
                hp,
                max_hp,
                underground_pair,
            };

            // Belt upgrade: if placing a belt over an existing belt, remove the old one first.
            if kind.is_belt() {
                if let Some(tile) = state.grid.get_tile(grid_pos) {
                    if let Some(old_bid) = tile.building {
                        if let Some(old_b) = state.buildings.get(old_bid) {
                            if old_b.kind.is_belt() && old_b.kind != kind {
                                // Refund old belt, remove it, then place the new one.
                                buildcost::refund_cost(&mut state.inventory, old_b.kind);
                                state.buildings.remove(old_bid, &mut state.grid);
                            }
                        }
                    }
                }
            }

            // Pre-check for specific error messages before attempting placement.
            if is_mouse_button_pressed(MouseButton::Left) {
                if let Some(tile) = state.grid.get_tile(grid_pos) {
                    if tile.building.is_some() {
                        // Only toast on initial click, not drag
                    } else if !tile.terrain.is_buildable() && kind != types::BuildingKind::WaterPump
                    {
                        state.toast(
                            "Can't build here — terrain is not buildable".to_string(),
                            40,
                        );
                    } else if kind == types::BuildingKind::Miner
                        && (tile.deposit.is_none() || tile.deposit == Some(types::OreDeposit::Oil))
                    {
                        state.toast("Miner must be placed on an ore deposit".to_string(), 50);
                    } else if kind == types::BuildingKind::PumpJack
                        && tile.deposit != Some(types::OreDeposit::Oil)
                    {
                        state.toast("Pump jack must be placed on an oil well".to_string(), 50);
                    }
                }
            }

            if let Some(_new_bid) = state.buildings.place(b, &mut state.grid) {
                // Deduct cost from inventory.
                buildcost::pay_cost(&mut state.inventory, kind);
                state.undo_history.push(grid_pos);
                if state.undo_history.len() > 20 {
                    state.undo_history.remove(0);
                }
                state.placement_flash = Some((grid_pos, 10));
                state.stats.buildings_placed += 1;
                // Throttle placement sound during belt drag (play max every 4 ticks).
                if !kind.is_belt() || state.stats.total_ticks.is_multiple_of(4) {
                    sfx.play(&sfx.place);
                }

                // Spawn robot worker from ship to placement site.
                let ship_center = macroquad::prelude::Vec2::new(
                    state.grid.width as f32 * constants::TILE_SIZE * 0.5,
                    state.grid.height as f32 * constants::TILE_SIZE * 0.5,
                );
                let target = grid::Grid::grid_to_world_center(grid_pos);
                state.robots.push((ship_center, target, 0.0));
                if kind.is_belt() {
                    state.last_belt_pos = Some(grid_pos);
                }

                // Advance tutorial.
                if state.tutorial_step == 1 && kind == types::BuildingKind::Miner {
                    state.tutorial_step = 2;
                } else if state.tutorial_step == 2 && kind.is_belt() {
                    state.tutorial_step = 3;
                } else if state.tutorial_step == 3 && kind.is_inserter() {
                    state.tutorial_step = 4;
                } else if state.tutorial_step == 4 && kind == types::BuildingKind::StoneFurnace {
                    state.tutorial_step = 5;
                } else if state.tutorial_step == 5 && kind == types::BuildingKind::StorageChest {
                    state.tutorial_step = 6;
                    state.show_tutorial = false;
                    state.toast(
                        "Tutorial complete! Press N for your roadmap~".to_string(),
                        120,
                    );
                }

                // Track first miner for story.
                if kind == types::BuildingKind::Miner && !state.story.first_miner_placed {
                    state.story.first_miner_placed = true;
                }
                // If this is an underground belt exit, update the entry to point to us.
                if kind.is_underground_belt() {
                    if let Some(pair_pos) = underground_pair {
                        // We are the exit — find the entry at pair_pos and set its pair to us.
                        if let Some(tile) = state.grid.get_tile(pair_pos) {
                            if let Some(entry_bid) = tile.building {
                                if let Some(entry) = state.buildings.get_mut(entry_bid) {
                                    entry.underground_pair = Some(grid_pos);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Right click: remove building and refund resources.
    // Hold right click to mass-delete (drag to demolish).
    if is_mouse_button_pressed(MouseButton::Right) || is_mouse_button_down(MouseButton::Right) {
        if let Some(tile) = state.grid.get_tile(grid_pos) {
            if let Some(bid) = tile.building {
                // Refund cost and return installed modules.
                if let Some(b) = state.buildings.get(bid) {
                    buildcost::refund_cost(&mut state.inventory, b.kind);
                    if let Some(ms) = &b.machine_state {
                        for &m in &ms.modules {
                            *state.inventory.entry(m).or_insert(0) += 1;
                        }
                    }
                }
                state.buildings.remove(bid, &mut state.grid);
                sfx.play(&sfx.remove);
                // Also despawn any items on that tile.
                let item_ids: Vec<types::ItemId> = state.grid.items_at(grid_pos).to_vec();
                for item_id in item_ids {
                    state.items.despawn(item_id);
                    state.grid.remove_item_from_tile(grid_pos, item_id);
                }
            }
        }
    }
}

/// Finds an unpaired underground belt entry in the opposite direction within range.
///
/// When placing an underground belt exit, we look backward along the facing direction
/// for an entry that doesn't have a pair yet.
/// Finds an unpaired underground belt entry in the opposite direction within range.
///
/// When placing an underground belt exit, we look backward along the facing direction
/// for an entry that doesn't have a pair yet.
pub fn find_underground_pair(
    buildings: &building::Buildings,
    pos: types::GridPos,
    dir: types::Direction,
    kind: types::BuildingKind,
) -> Option<types::GridPos> {
    let max_range = match kind {
        types::BuildingKind::UndergroundBeltYellow => constants::UNDERGROUND_RANGE_YELLOW,
        types::BuildingKind::UndergroundBeltRed => constants::UNDERGROUND_RANGE_RED,
        types::BuildingKind::UndergroundBeltBlue => constants::UNDERGROUND_RANGE_BLUE,
        _ => return None,
    };

    // Look backward (opposite of our direction) for an unpaired entry.
    let search_dir = dir.opposite();
    let mut check = pos;
    for _ in 1..=max_range {
        check = check.neighbor(search_dir);
        // Check if there's an unpaired underground belt of the same kind facing the same direction.
        for (_bid, b) in buildings.iter() {
            if b.pos == check
                && b.kind == kind
                && b.direction == dir
                && b.underground_pair.is_none()
            {
                return Some(check);
            }
        }
    }
    None
}
