//! Keyboard hotkey handling: pause, rotate, overlay toggles, blueprint
//! keys, save/load, hotbar quick-select, tier-cycle scroll, and the
//! eyedropper. Returns `true` if a hotkey consumed input (X-button close
//! click), signaling the caller to skip the rest of this frame's input.

use macroquad::prelude::*;

use crate::buildcost;
use crate::constants;
use crate::game::GameState;
use crate::grid;
use crate::save;
use crate::sound;
use crate::types;

pub(super) fn handle_hotkeys(state: &mut GameState, sfx: &mut sound::SoundEffects) -> bool {
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
            return true;
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

    false
}
