use macroquad::prelude::*;

use crate::buildcost;
use crate::building;
use crate::constants;
use crate::constants::{GATE_HP, WALL_HP};
use crate::game::GameState;
use crate::grid;
use crate::recipe;
use crate::research;
use crate::save;
use crate::sound;
use crate::types;

/// Handles player input for building selection, placement, and hotkeys.
pub fn handle_input(state: &mut GameState, sfx: &mut sound::SoundEffects) {
    // Pause toggle
    if is_key_pressed(KeyCode::Space) {
        state.paused = !state.paused;
    }

    // Rotate placement direction
    if is_key_pressed(KeyCode::R) {
        state.placement_direction = state.placement_direction.rotated_cw();
        // Brief flash on the ghost preview position to confirm rotation.
        if state.selected_building.is_some() {
            let dir_name = match state.placement_direction {
                types::Direction::North => "North",
                types::Direction::East => "East",
                types::Direction::South => "South",
                types::Direction::West => "West",
            };
            state.toast(format!("Facing: {}", dir_name), 20);
        }
    }

    // X button click: close overlays. X button is at (px + pw - 28, py + 4, 24x20).
    // If an X button is hit, consume the click (return early) so it doesn't also place a building.
    if is_mouse_button_pressed(MouseButton::Left) {
        let mx = mouse_position().0;
        let my = mouse_position().1;
        let sw = screen_width();
        let sh = screen_height();

        // Helper: check if mouse is inside X button region for a panel at (px, py, pw).
        let hit_x = |px: f32, py: f32, pw: f32| -> bool {
            let bx = px + pw - 28.0;
            let by = py + 4.0;
            mx >= bx && mx <= bx + 24.0 && my >= by && my <= by + 20.0
        };

        let mut consumed = false;

        if state.show_tutorial && state.tutorial_step < 6 {
            let pw = 400.0f32;
            let py = 100.0f32;
            let px = (sw - pw) * 0.5;
            if hit_x(px, py, pw) {
                state.show_tutorial = false;
                consumed = true;
            }
        }
        if state.recipe_picker.is_some() {
            let pw = 340.0f32;
            let ph: f32 =
                50.0 + state.recipe_picker.as_ref().map(|r| r.1.len()).unwrap_or(0) as f32 * 28.0;
            let px = (sw - pw) * 0.5;
            let py = (sh - ph.min(500.0)) * 0.5;
            if hit_x(px, py, pw) {
                state.recipe_picker = None;
                consumed = true;
            }
        }
        if state.show_help {
            let pw = (sw * 0.6).min(600.0);
            let ph = (sh * 0.75).min(500.0);
            let px = (sw - pw) * 0.5;
            let py = (sh - ph) * 0.5;
            if hit_x(px, py, pw) {
                state.show_help = false;
                consumed = true;
            }
        }
        if state.show_recipes {
            let pw = (sw * 0.75).min(800.0);
            let ph = (sh * 0.85).min(700.0);
            let px = (sw - pw) * 0.5;
            let py = (sh - ph) * 0.5;
            if hit_x(px, py, pw) {
                state.show_recipes = false;
                consumed = true;
            }
        }
        if state.show_research {
            let pw = (sw * 0.7).min(700.0);
            let ph = (sh * 0.8).min(600.0);
            let px = (sw - pw) * 0.5;
            let py = (sh - ph) * 0.5;
            if hit_x(px, py, pw) {
                state.show_research = false;
                consumed = true;
            }
        }
        if state.show_achievements {
            let pw = (sw * 0.5).min(500.0);
            let ph = (sh * 0.7).min(450.0);
            let px = (sw - pw) * 0.5;
            let py = (sh - ph) * 0.5;
            if hit_x(px, py, pw) {
                state.show_achievements = false;
                consumed = true;
            }
        }
        if state.show_stats {
            let pw = (sw * 0.5).min(480.0);
            let ph = (sh * 0.6).min(400.0);
            let px = (sw - pw) * 0.5;
            let py = (sh - ph) * 0.5;
            if hit_x(px, py, pw) {
                state.show_stats = false;
                consumed = true;
            }
        }
        if consumed {
            return;
        }
    }

    // Escape: close the topmost overlay, or deselect building.
    if is_key_pressed(KeyCode::Escape) {
        if state.recipe_picker.is_some() {
            state.recipe_picker = None;
        } else if state.show_stats {
            state.show_stats = false;
        } else if state.show_achievements {
            state.show_achievements = false;
        } else if state.show_help {
            state.show_help = false;
        } else if state.show_recipes {
            state.show_recipes = false;
        } else if state.show_research {
            state.show_research = false;
        } else if state.show_tutorial {
            state.show_tutorial = false;
        } else {
            state.selected_building = None;
        }
    }

    // Toggle research screen
    if is_key_pressed(KeyCode::Tab) {
        let was = state.show_research;
        state.close_all_overlays();
        state.show_research = !was;
    }

    // Toggle tutorial
    if is_key_pressed(KeyCode::H) {
        state.show_tutorial = !state.show_tutorial;
    }

    // F2: Toggle sound mute
    if is_key_pressed(KeyCode::F2) {
        if sfx.volume > 0.0 {
            sfx.volume = 0.0;
            macroquad::audio::stop_sound(&sfx.ambient);
            state.toast("Sound: OFF".to_string(), 40);
        } else {
            sfx.volume = 0.5;
            sfx.start_ambient();
            state.toast("Sound: ON".to_string(), 40);
        }
    }

    // Toggle full help overlay
    if is_key_pressed(KeyCode::F1) {
        let was = state.show_help;
        state.close_all_overlays();
        state.show_help = !was;
    }

    // Toggle achievements screen
    if is_key_pressed(KeyCode::N) {
        let was = state.show_achievements;
        state.close_all_overlays();
        state.show_achievements = !was;
    }

    // Toggle production stats
    if is_key_pressed(KeyCode::V) {
        let was = state.show_stats;
        state.close_all_overlays();
        state.show_stats = !was;
    }

    // Blueprint: B to copy, Ctrl+B to save to library, Shift+B to open library picker.
    if is_key_pressed(KeyCode::B) {
        let ctrl = is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper)
            || is_key_down(KeyCode::RightSuper);
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);

        if ctrl {
            // Ctrl+B: save current blueprint to library.
            if state.blueprint.is_empty() {
                state.toast(
                    "No blueprint to save! Press B first to copy buildings.".to_string(),
                    60,
                );
            } else if state.blueprint_library.len() >= 10 {
                state.toast("Blueprint library full! (max 10)".to_string(), 60);
            } else {
                let name = format!("BP {}", state.blueprint_library.len() + 1);
                state
                    .blueprint_library
                    .push((name.clone(), state.blueprint.clone()));
                state.toast(
                    format!(
                        "Blueprint saved as '{}' ({} buildings)",
                        name,
                        state.blueprint.len()
                    ),
                    80,
                );
            }
        } else if shift {
            // Shift+B: toggle blueprint library picker.
            if state.blueprint_library.is_empty() {
                state.toast(
                    "No saved blueprints! Copy with B, save with Ctrl+B.".to_string(),
                    60,
                );
            } else {
                state.show_blueprint_picker = !state.show_blueprint_picker;
            }
        } else if state.pasting_blueprint {
            // Cancel paste mode.
            state.pasting_blueprint = false;
            state.toast("Blueprint paste cancelled.".to_string(), 40);
        } else {
            // Copy buildings within 5 tiles of cursor.
            let mouse_screen = Vec2::new(mouse_position().0, mouse_position().1);
            let mouse_world = state.camera.screen_to_world(mouse_screen);
            let center = grid::Grid::world_to_grid(mouse_world);
            let mut bp = Vec::new();
            for (_, b) in state.buildings.iter() {
                let dx = b.pos.x - center.x;
                let dy = b.pos.y - center.y;
                if dx.abs() <= 5 && dy.abs() <= 5 {
                    bp.push((dx, dy, b.kind, b.direction));
                }
            }
            if bp.is_empty() {
                state.toast("No buildings to copy nearby.".to_string(), 40);
            } else {
                state.toast(
                    format!(
                        "Copied {} buildings (5-tile radius)! Click to paste, B to cancel.",
                        bp.len()
                    ),
                    80,
                );
                state.blueprint = bp;
                state.pasting_blueprint = true;
                state.selected_building = None;
            }
        }
    }

    // Blueprint picker: number keys 1-9 select a saved blueprint.
    if state.show_blueprint_picker {
        let keys = [
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
            KeyCode::Key8,
            KeyCode::Key9,
        ];
        for (i, &key) in keys.iter().enumerate() {
            if is_key_pressed(key) && i < state.blueprint_library.len() {
                let (name, bp) = state.blueprint_library[i].clone();
                state.blueprint = bp;
                state.pasting_blueprint = true;
                state.show_blueprint_picker = false;
                state.selected_building = None;
                state.toast(
                    format!("Loaded '{}' — click to paste, B to cancel", name),
                    80,
                );
                break;
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            state.show_blueprint_picker = false;
        }
    }

    // Home key: center camera on map center (factory area)
    if is_key_pressed(KeyCode::Home) {
        state.camera.target = macroquad::prelude::Vec2::new(
            state.grid.width as f32 * constants::TILE_SIZE * 0.5,
            state.grid.height as f32 * constants::TILE_SIZE * 0.5,
        );
        state.camera.zoom = 1.0;
    }

    // Undo last placement (Ctrl+Z or Cmd+Z).
    if is_key_pressed(KeyCode::Z)
        && (is_key_down(KeyCode::LeftControl)
            || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper)
            || is_key_down(KeyCode::RightSuper))
    {
        if let Some(pos) = state.undo_history.pop() {
            if let Some(tile) = state.grid.get_tile(pos) {
                if let Some(bid) = tile.building {
                    if let Some(b) = state.buildings.get(bid) {
                        buildcost::refund_cost(&mut state.inventory, b.kind);
                        // Return installed modules to inventory.
                        if let Some(ms) = &b.machine_state {
                            for &m in &ms.modules {
                                *state.inventory.entry(m).or_insert(0) += 1;
                            }
                        }
                    }
                    state.buildings.remove(bid, &mut state.grid);
                    sfx.play(&sfx.remove);
                    let remaining = state.undo_history.len();
                    state.toast(format!("Undone! ({} more)", remaining), 30);
                }
            }
        }
    }

    // Game speed: + to increase, - to decrease.
    if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
        state.game_speed = (state.game_speed + 1).min(5);
        state.toast(format!("Speed: {}x", state.game_speed), 40);
    }
    if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
        state.game_speed = (state.game_speed - 1).max(1);
        state.toast(format!("Speed: {}x", state.game_speed), 40);
    }

    // Toggle recipe browser
    if is_key_pressed(KeyCode::E) {
        let was = state.show_recipes;
        state.close_all_overlays();
        state.show_recipes = !was;
    }

    // Save (F5)
    if is_key_pressed(KeyCode::F5) {
        if save::save_game(state) {
            state.toast("Game saved!".to_string(), 60);
        } else {
            state.toast("Save failed!".to_string(), 60);
        }
    }

    // Load (F9)
    if is_key_pressed(KeyCode::F9) {
        if save::load_game(state) {
            // Reset UI state after loading.
            state.close_all_overlays();
            state.toasts.clear();
            state.selected_building = None;
            state.toast("Game loaded!".to_string(), 60);
        } else {
            state.toast("No save file found.".to_string(), 60);
        }
    }

    // Screenshot (F12)
    if is_key_pressed(KeyCode::F12) {
        let img = get_screen_data();
        let filename = format!(
            "autoforge_screenshot_{}.png",
            (macroquad::miniquad::date::now() * 1000.0) as u64
        );
        img.export_png(&filename);
        state.toast(format!("Screenshot saved: {}", filename), 80);
    }

    // Quick-select buildings. Shift+key remaps the slot to current selection.
    {
        let default_hotbar: &[(KeyCode, types::BuildingKind)] = &[
            (KeyCode::Key1, types::BuildingKind::BeltYellow),
            (KeyCode::Key2, types::BuildingKind::Miner),
            (KeyCode::Key3, types::BuildingKind::StoneFurnace),
            (KeyCode::Key4, types::BuildingKind::InserterRegular),
            (KeyCode::Key5, types::BuildingKind::AssemblerT1),
            (KeyCode::Key6, types::BuildingKind::Boiler),
            (KeyCode::Key7, types::BuildingKind::SteamEngine),
            (KeyCode::Key8, types::BuildingKind::Lab),
            (KeyCode::Key9, types::BuildingKind::StorageChest),
            (KeyCode::Key0, types::BuildingKind::Splitter),
            (KeyCode::T, types::BuildingKind::GunTurret),
            (KeyCode::G, types::BuildingKind::Wall),
            (KeyCode::U, types::BuildingKind::UndergroundBeltYellow),
            (KeyCode::C, types::BuildingKind::ChemicalPlant),
            (KeyCode::L, types::BuildingKind::LaserTurret),
            (KeyCode::P, types::BuildingKind::SolarPanel),
        ];
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        for (i, &(key, default_kind)) in default_hotbar.iter().enumerate() {
            if is_key_pressed(key) {
                if shift && state.selected_building.is_some() {
                    // Remap this slot to the currently selected building.
                    let current = state.selected_building.unwrap();
                    if i < state.custom_hotbar.len() {
                        state.custom_hotbar[i] = Some(current);
                        state.toast(format!("Slot remapped to {}", current.display_name()), 40);
                    }
                } else {
                    // Select the building for this slot (custom override or default).
                    let kind = if i < state.custom_hotbar.len() {
                        state.custom_hotbar[i].unwrap_or(default_kind)
                    } else {
                        default_kind
                    };
                    state.selected_building = Some(kind);
                }
            }
        }
    }

    // Scroll wheel cycles through building tiers when a tiered building is selected.
    if let Some(kind) = state.selected_building {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y.abs() > 0.1 {
            let up = wheel_y > 0.0;
            let next = match kind {
                // Belt tiers
                types::BuildingKind::BeltYellow if up => Some(types::BuildingKind::BeltRed),
                types::BuildingKind::BeltRed if up => Some(types::BuildingKind::BeltBlue),
                types::BuildingKind::BeltBlue if !up => Some(types::BuildingKind::BeltRed),
                types::BuildingKind::BeltRed if !up => Some(types::BuildingKind::BeltYellow),
                // Inserter tiers
                types::BuildingKind::InserterRegular if up => {
                    Some(types::BuildingKind::InserterLong)
                }
                types::BuildingKind::InserterLong if up => Some(types::BuildingKind::InserterFast),
                types::BuildingKind::InserterFast if up => Some(types::BuildingKind::InserterStack),
                types::BuildingKind::InserterStack if !up => {
                    Some(types::BuildingKind::InserterFast)
                }
                types::BuildingKind::InserterFast if !up => Some(types::BuildingKind::InserterLong),
                types::BuildingKind::InserterLong if !up => {
                    Some(types::BuildingKind::InserterRegular)
                }
                // Assembler tiers
                types::BuildingKind::AssemblerT1 if up => Some(types::BuildingKind::AssemblerT2),
                types::BuildingKind::AssemblerT2 if up => Some(types::BuildingKind::AssemblerT3),
                types::BuildingKind::AssemblerT3 if !up => Some(types::BuildingKind::AssemblerT2),
                types::BuildingKind::AssemblerT2 if !up => Some(types::BuildingKind::AssemblerT1),
                // Furnace tiers
                types::BuildingKind::StoneFurnace if up => Some(types::BuildingKind::SteelFurnace),
                types::BuildingKind::SteelFurnace if up => {
                    Some(types::BuildingKind::ElectricFurnace)
                }
                types::BuildingKind::ElectricFurnace if !up => {
                    Some(types::BuildingKind::SteelFurnace)
                }
                types::BuildingKind::SteelFurnace if !up => Some(types::BuildingKind::StoneFurnace),
                // Underground belt tiers
                types::BuildingKind::UndergroundBeltYellow if up => {
                    Some(types::BuildingKind::UndergroundBeltRed)
                }
                types::BuildingKind::UndergroundBeltRed if up => {
                    Some(types::BuildingKind::UndergroundBeltBlue)
                }
                types::BuildingKind::UndergroundBeltBlue if !up => {
                    Some(types::BuildingKind::UndergroundBeltRed)
                }
                types::BuildingKind::UndergroundBeltRed if !up => {
                    Some(types::BuildingKind::UndergroundBeltYellow)
                }
                _ => None,
            };
            if let Some(new_kind) = next {
                state.selected_building = Some(new_kind);
                state.toast(format!("Selected: {:?}", new_kind), 20);
            }
        }
    }

    // Eyedropper (Q): pick building type from hovered tile.
    if is_key_pressed(KeyCode::Q) {
        let mouse_screen = Vec2::new(mouse_position().0, mouse_position().1);
        let mouse_world = state.camera.screen_to_world(mouse_screen);
        let grid_pos = grid::Grid::world_to_grid(mouse_world);
        if let Some(tile) = state.grid.get_tile(grid_pos) {
            if let Some(bid) = tile.building {
                if let Some(b) = state.buildings.get(bid) {
                    state.selected_building = Some(b.kind);
                    state.placement_direction = b.direction;
                    state.toast(format!("Picked: {:?}", b.kind), 30);
                }
            } else {
                state.selected_building = None;
                state.toast("Deselected building".to_string(), 20);
            }
        }
    }

    // Handle research screen clicks
    if state.show_research && is_mouse_button_pressed(MouseButton::Left) {
        let sw = screen_width();
        let sh = screen_height();
        let pw = (sw * 0.7).min(700.0);
        let ph = (sh * 0.8).min(600.0);
        let px = (sw - pw) * 0.5;
        let py = (sh - ph) * 0.5;
        let start_y = py + 110.0;
        let row_h = 28.0;
        let col1 = px + 20.0;
        let mouse = Vec2::new(mouse_position().0, mouse_position().1);

        for (i, _tech) in research::TECHNOLOGIES.iter().enumerate() {
            let y = start_y + i as f32 * row_h;
            if mouse.x >= col1
                && mouse.x <= col1 + 400.0
                && mouse.y >= y - 14.0
                && mouse.y <= y + 4.0
            {
                state.research.start_research(i);
                break;
            }
        }
        return; // don't process building placement when research screen is open
    }

    // Handle toolbar clicks (select building by clicking).
    let toolbar_y = screen_height() - 80.0;
    if mouse_position().1 > toolbar_y && is_mouse_button_pressed(MouseButton::Left) {
        let toolbar_kinds: &[types::BuildingKind] = &[
            types::BuildingKind::BeltYellow,
            types::BuildingKind::Miner,
            types::BuildingKind::StoneFurnace,
            types::BuildingKind::InserterRegular,
            types::BuildingKind::AssemblerT1,
            types::BuildingKind::Boiler,
            types::BuildingKind::SteamEngine,
            types::BuildingKind::Lab,
            types::BuildingKind::StorageChest,
            types::BuildingKind::Splitter,
            types::BuildingKind::GunTurret,
            types::BuildingKind::Wall,
            types::BuildingKind::ChemicalPlant,
            types::BuildingKind::SolarPanel,
        ];
        let slot_w = 76.0;
        let total_w = toolbar_kinds.len() as f32 * slot_w;
        let start_x = (screen_width() - total_w) * 0.5;
        let mx = mouse_position().0;

        for (i, &kind) in toolbar_kinds.iter().enumerate() {
            let x = start_x + i as f32 * slot_w;
            if mx >= x && mx < x + slot_w {
                state.selected_building = Some(kind);
                // Start tutorial on first click.
                if state.tutorial_step == 0 {
                    state.tutorial_step = 1;
                }
                break;
            }
        }
        return;
    }

    // Don't process mouse placement if cursor is over the toolbar.
    if mouse_position().1 > toolbar_y {
        return;
    }

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

/// Periodic alert checks for critical game events (power, ammo, etc.).
/// Called every tick but internally gates on appropriate frequencies.
pub fn check_alerts(state: &mut GameState, sfx: &sound::SoundEffects) {
    let tick = state.stats.total_ticks;
    if !tick.is_multiple_of(100) {
        return;
    }

    // --- Power alerts ---
    if state.power.satisfaction < 0.1 && state.power.demand > 0.0 {
        state.alert(
            types::AlertKind::PowerBlackout,
            "BLACKOUT -- No power! Build more generators.".to_string(),
            120,
            types::AlertSeverity::Critical,
        );
        sfx.play(&sfx.wave_warning);
    } else if state.power.satisfaction < 0.5 && state.power.demand > 0.0 {
        state.alert(
            types::AlertKind::PowerBrownout,
            format!(
                "Low power: {:.0}% -- machines slowing down",
                state.power.satisfaction * 100.0
            ),
            100,
            types::AlertSeverity::Warning,
        );
    }

    // --- Turret ammo alerts ---
    let mut empty_turrets = 0u32;
    let mut low_turrets = 0u32;
    for (_, b) in state.buildings.iter() {
        if b.kind != types::BuildingKind::GunTurret {
            continue;
        }
        if let Some(ms) = &b.machine_state {
            let ammo = ms
                .input_buffer
                .iter()
                .filter(|r| {
                    matches!(
                        r,
                        types::Resource::BasicAmmo | types::Resource::PiercingAmmo
                    )
                })
                .count();
            if ammo == 0 {
                empty_turrets += 1;
            } else if ammo <= 2 {
                low_turrets += 1;
            }
        }
    }
    let enemies_alive = state.enemies.list.iter().any(|e| e.alive);
    if empty_turrets > 0 && enemies_alive {
        state.alert(
            types::AlertKind::TurretAmmoEmpty,
            format!("{} turret(s) have NO ammo!", empty_turrets),
            120,
            types::AlertSeverity::Critical,
        );
        sfx.play(&sfx.wave_warning);
    } else if low_turrets > 0 {
        state.alert(
            types::AlertKind::TurretAmmoLow,
            format!("{} turret(s) running low on ammo", low_turrets),
            80,
            types::AlertSeverity::Warning,
        );
    }
}
