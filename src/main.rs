//! # AutoForge
//!
//! A 2D top-down factory automation game built in Rust with macroquad.
//!
//! Mine resources, smelt ores, assemble products, research technologies,
//! and defend your factory from hostile creatures — all rendered with
//! procedurally generated pixel art.
//!
//! ## Architecture
//!
//! The game uses a fixed-timestep simulation (20 TPS) decoupled from rendering.
//! Data is stored in flat arrays for cache efficiency, with generational arenas
//! for entities (buildings, items). All sprites are generated at startup from
//! const pixel data — no external asset files.
//!
//! ## Module overview
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`constants`] | All tuning numbers in one place |
//! | [`types`] | Core enums and structs (Resource, BuildingKind, GridPos, etc.) |
//! | [`grid`] | Flat tile grid, spatial item index, coordinate math |
//! | [`mapgen`] | Procedural world generation (biomes, ores, water, nests) |
//! | [`camera`] | Pan/zoom camera with screen↔world conversion |
//! | [`sprites`] | Palette + pixel art sprite generation |
//! | [`render`] | Frustum-culled world drawing |
//! | [`game`] | GameState struct tying everything together |

use macroquad::prelude::*;

mod atlas;
mod batcher;
mod belt;
mod buildcost;
mod building;
mod camera;
mod combat;
mod constants;
mod cutscene;
mod daynight;
mod enemy;
mod fluid;
mod game;
mod grid;
mod inserter;
mod item;
mod machine;
mod mapgen;
mod milestones;
mod pollution;
mod power;
mod recipe;
mod render;
mod research;
mod save;
mod splitter;
mod story;
mod train;
mod sprites;
mod types;

use constants::*;
use game::GameState;
use sprites::SpriteAtlas;

/// Window configuration — called by macroquad before the window opens.
fn window_conf() -> Conf {
    Conf {
        window_title: "AutoForge".to_string(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // --- Startup ---
    // Request VSync to cap frame rate and reduce power usage on low-end devices.
    // macroquad respects the display refresh rate by default (60Hz typically).
    let atlas = SpriteAtlas::generate();
    let mut intro = cutscene::CutsceneState::new();

    // --- Intro cutscene loop ---
    while !intro.is_done() {
        let dt = get_frame_time();
        intro.update(dt);
        intro.draw();
        next_frame().await;
    }

    // --- Game initialization (after cutscene) ---
    let mut state = GameState::new(0);
    state.toast("Welcome to AutoForge! Press F1 for help~".to_string(), 120);
    state.toast("Click toolbar to select buildings. Press E for recipes!".to_string(), 150);

    // --- Main game loop ---
    let mut autosave_timer = 0.0f32;

    loop {
        let dt = get_frame_time() as f64;

        // Auto-save every 5 minutes.
        autosave_timer += dt as f32;
        if autosave_timer > 300.0 {
            autosave_timer = 0.0;
            if save::save_game(&state) {
                state.toast("Auto-saved!".to_string(), 60);
            }
        }

        // Edge-scroll: move camera when mouse is near screen edges.
        {
            let edge_margin = 10.0;
            let edge_speed = 300.0 * get_frame_time() / state.camera.zoom;
            let (mx, my) = mouse_position();
            if mx < edge_margin { state.camera.target.x -= edge_speed; }
            if mx > screen_width() - edge_margin { state.camera.target.x += edge_speed; }
            if my < edge_margin { state.camera.target.y -= edge_speed; }
            if my > screen_height() - edge_margin { state.camera.target.y += edge_speed; }
        }

        // 1. Input (every frame, independent of simulation tick rate).
        handle_input(&mut state);
        state.camera.update(get_frame_time());

        // 2. Fixed-timestep simulation (with game speed multiplier).
        if !state.paused {
            state.tick_accumulator += dt * state.game_speed as f64;
            if state.tick_accumulator > MAX_ACCUMULATOR {
                state.tick_accumulator = MAX_ACCUMULATOR;
            }
            while state.tick_accumulator >= TICK_DURATION {
                simulation_tick(&mut state);
                state.tick_accumulator -= TICK_DURATION;
            }
        }

        // 3. Render (every frame at display refresh rate).
        clear_background(Color::new(0.08, 0.08, 0.10, 1.0));

        // World-space rendering (affected by camera).
        set_camera(&state.camera.to_macroquad_camera());
        render::draw_world(
            &state.grid,
            &state.buildings,
            &state.items,
            &state.enemies,
            &state.camera,
            &atlas,
            state.stats.total_ticks,
        );
        render::draw_ghost_preview(
            &state.grid,
            &state.camera,
            &atlas,
            state.selected_building,
            state.placement_direction,
        );
        render::draw_night_overlay(state.daynight.darkness());

        // Placement flash effect (brief white glow on last placed building).
        if let Some((pos, ticks)) = state.placement_flash {
            let alpha = ticks as f32 / 10.0 * 0.4;
            let world = grid::Grid::grid_to_world(pos);
            draw_rectangle(
                world.x - 2.0,
                world.y - 2.0,
                TILE_SIZE + 4.0,
                TILE_SIZE + 4.0,
                Color::new(0.8, 0.9, 1.0, alpha),
            );
        }

        // Render robot workers (small dots moving from ship to target).
        for (start, target, progress) in &state.robots {
            let pos = *start + (*target - *start) * *progress;
            // Robot body (small cute circle).
            draw_circle(pos.x, pos.y, 4.0, Color::new(0.4, 0.6, 0.9, 0.9));
            draw_circle(pos.x, pos.y, 2.0, Color::new(0.7, 0.8, 1.0, 0.9));
            // Trail.
            let trail_pos = *start + (*target - *start) * (*progress - 0.05).max(0.0);
            draw_circle(trail_pos.x, trail_pos.y, 2.0, Color::new(0.4, 0.6, 0.9, 0.4));
        }

        // Build zone indicator (faint circle around ship).
        if state.selected_building.is_some() {
            let center_world = Vec2::new(
                state.grid.width as f32 * TILE_SIZE * 0.5,
                state.grid.height as f32 * TILE_SIZE * 0.5,
            );
            let radius_world = state.build_radius * TILE_SIZE;
            // Draw faint circle showing build zone boundary.
            let segments = 64;
            for i in 0..segments {
                let a1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
                let a2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
                draw_line(
                    center_world.x + a1.cos() * radius_world,
                    center_world.y + a1.sin() * radius_world,
                    center_world.x + a2.cos() * radius_world,
                    center_world.y + a2.sin() * radius_world,
                    1.5,
                    Color::new(0.3, 0.5, 0.9, 0.3),
                );
            }
        }

        // Red vignette flash when enemies are actively attacking buildings.
        let enemies_attacking = state.enemies.list.iter()
            .any(|e| e.alive && e.attack_cooldown > 15);
        if enemies_attacking {
            let flash = (state.stats.total_ticks as f32 * 0.2).sin() * 0.08 + 0.05;
            draw_rectangle(-100000.0, -100000.0, 200000.0, 200000.0,
                Color::new(0.8, 0.0, 0.0, flash));
        }

        // Screen-space UI overlay.
        set_default_camera();
        draw_ui(&state, &atlas);

        next_frame().await;
    }
}

/// Handles player input for building selection, placement, and hotkeys.
fn handle_input(state: &mut GameState) {
    // Pause toggle
    if is_key_pressed(KeyCode::Space) {
        state.paused = !state.paused;
    }

    // Rotate placement direction
    if is_key_pressed(KeyCode::R) {
        state.placement_direction = state.placement_direction.rotated_cw();
    }

    // Deselect
    if is_key_pressed(KeyCode::Escape) {
        state.selected_building = None;
    }

    // Toggle research screen
    if is_key_pressed(KeyCode::Tab) {
        state.show_research = !state.show_research;
    }

    // Toggle tutorial
    if is_key_pressed(KeyCode::H) {
        state.show_tutorial = !state.show_tutorial;
    }

    // Toggle full help overlay
    if is_key_pressed(KeyCode::F1) {
        state.show_help = !state.show_help;
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
        && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper) || is_key_down(KeyCode::RightSuper))
    {
        if let Some(pos) = state.last_placed {
            if let Some(tile) = state.grid.get_tile(pos) {
                if let Some(bid) = tile.building {
                    if let Some(b) = state.buildings.get(bid) {
                        buildcost::refund_cost(&mut state.inventory, b.kind);
                    }
                    state.buildings.remove(bid, &mut state.grid);
                    state.last_placed = None;
                    state.toast("Undone!".to_string(), 30);
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
        state.show_recipes = !state.show_recipes;
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
            state.toast("Game loaded!".to_string(), 60);
        } else {
            state.toast("No save file found.".to_string(), 60);
        }
    }

    // Quick-select buildings (basic set available from start).
    if is_key_pressed(KeyCode::Key1) {
        state.selected_building = Some(types::BuildingKind::BeltYellow);
    }
    if is_key_pressed(KeyCode::Key2) {
        state.selected_building = Some(types::BuildingKind::Miner);
    }
    if is_key_pressed(KeyCode::Key3) {
        state.selected_building = Some(types::BuildingKind::StoneFurnace);
    }
    if is_key_pressed(KeyCode::Key4) {
        state.selected_building = Some(types::BuildingKind::InserterRegular);
    }
    if is_key_pressed(KeyCode::Key5) {
        state.selected_building = Some(types::BuildingKind::AssemblerT1);
    }
    if is_key_pressed(KeyCode::Key6) {
        state.selected_building = Some(types::BuildingKind::Boiler);
    }
    if is_key_pressed(KeyCode::Key7) {
        state.selected_building = Some(types::BuildingKind::SteamEngine);
    }
    if is_key_pressed(KeyCode::Key8) {
        state.selected_building = Some(types::BuildingKind::Lab);
    }
    if is_key_pressed(KeyCode::Key9) {
        state.selected_building = Some(types::BuildingKind::StorageChest);
    }
    if is_key_pressed(KeyCode::Key0) {
        state.selected_building = Some(types::BuildingKind::Splitter);
    }
    // U: Underground belt
    if is_key_pressed(KeyCode::U) {
        state.selected_building = Some(types::BuildingKind::UndergroundBeltYellow);
    }
    // T: Gun Turret
    if is_key_pressed(KeyCode::T) {
        state.selected_building = Some(types::BuildingKind::GunTurret);
    }
    // G: Wall
    if is_key_pressed(KeyCode::G) {
        state.selected_building = Some(types::BuildingKind::Wall);
    }
    // C: Chemical Plant
    if is_key_pressed(KeyCode::C) {
        state.selected_building = Some(types::BuildingKind::ChemicalPlant);
    }
    // P: Solar Panel
    if is_key_pressed(KeyCode::P) {
        state.selected_building = Some(types::BuildingKind::SolarPanel);
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
                }
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

    // Left click with no selection: interact with existing building (cycle recipe)
    // or interact with the crashed ship at map center.
    if state.selected_building.is_none() && is_mouse_button_pressed(MouseButton::Left) {
        // Check if clicking on the crashed ship (within 3 tiles of map center).
        let center = types::GridPos::new(state.grid.width / 2, state.grid.height / 2);
        if grid_pos.distance(center) < 4.0 {
            let lore_messages = [
                "The hull is cold. Scorched from atmospheric entry.",
                "You can see cryo pod fragments inside... empty.",
                "The ship's name: 'Horizon's Promise'. Your ship.",
                "Data core intact but encrypted. You need more processing power.",
                "A photo is stuck to the console: Dr. Vasquez and her team, smiling.",
            ];
            let idx = (state.stats.total_ticks / 100) as usize % lore_messages.len();
            state.toast(lore_messages[idx].to_string(), 100);
        }

        if let Some(tile) = state.grid.get_tile(grid_pos) {
            if let Some(bid) = tile.building {
                if let Some(b) = state.buildings.get(bid) {
                    // If it's an assembler or chemical plant, cycle recipe.
                    if b.kind == types::BuildingKind::AssemblerT1
                        || b.kind == types::BuildingKind::AssemblerT2
                        || b.kind == types::BuildingKind::AssemblerT3
                        || b.kind == types::BuildingKind::ChemicalPlant
                    {
                        let available = recipe::recipes_for_machine(b.kind);
                        if !available.is_empty() {
                            let current = b.machine_state.as_ref().and_then(|ms| ms.selected_recipe);
                            let next_idx = if let Some(cur) = current {
                                let pos = available.iter().position(|r| r.0 == cur.0).unwrap_or(0);
                                (pos + 1) % available.len()
                            } else {
                                0
                            };
                            let new_recipe = available[next_idx];
                            let building = state.buildings.get_mut(bid).unwrap();
                            if let Some(ms) = &mut building.machine_state {
                                ms.selected_recipe = Some(new_recipe);
                                // Clear input buffer when changing recipe to avoid junk items.
                                ms.input_buffer.clear();
                            }
                            let name = recipe::RECIPES[new_recipe.0].name;
                            state.toast(format!("Recipe set: {}", name), 60);
                        }
                    }
                }
            }
        }
    }

    // Middle-click: hand-insert item from inventory into a machine.
    // Inserts the first useful item (Coal for furnaces, or recipe inputs for assemblers).
    if is_mouse_button_pressed(MouseButton::Middle) {
        if let Some(tile) = state.grid.get_tile(grid_pos) {
            if let Some(bid) = tile.building {
                if let Some(building) = state.buildings.get(bid) {
                    if building.machine_state.is_some() && !building.kind.is_belt() && !building.kind.is_inserter() {
                        let kind = building.kind;
                        // Determine what to insert.
                        let to_insert = if kind.needs_fuel() {
                            // Furnaces: insert Coal first, then ore.
                            if state.inventory.get(&types::Resource::Coal).copied().unwrap_or(0) > 0 {
                                Some(types::Resource::Coal)
                            } else if state.inventory.get(&types::Resource::IronOre).copied().unwrap_or(0) > 0 {
                                Some(types::Resource::IronOre)
                            } else {
                                None
                            }
                        } else {
                            // Assemblers: insert first available resource from inventory
                            // that matches the locked recipe's inputs.
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
                                // No recipe set — try iron plate (most common).
                                if state.inventory.get(&types::Resource::IronPlate).copied().unwrap_or(0) > 0 {
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
                }
                return;
            }

            // Auto-rotate belts during drag-placement based on movement direction.
            if kind.is_belt() {
                if let Some(last_pos) = state.last_belt_pos {
                    if last_pos != grid_pos {
                        let dx = grid_pos.x - last_pos.x;
                        let dy = grid_pos.y - last_pos.y;
                        if dx.abs() >= dy.abs() {
                            state.placement_direction = if dx > 0 { types::Direction::East } else { types::Direction::West };
                        } else {
                            state.placement_direction = if dy > 0 { types::Direction::South } else { types::Direction::North };
                        }
                    }
                }
            }

            let needs_machine_state = !kind.is_belt()
                && !kind.is_underground_belt()
                && !matches!(
                    kind,
                    types::BuildingKind::Wall | types::BuildingKind::Gate
                );

            // Underground belt pairing logic.
            let underground_pair = if kind.is_underground_belt() {
                find_underground_pair(
                    &state.buildings,
                    grid_pos,
                    state.placement_direction,
                    kind,
                )
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

            if let Some(_new_bid) = state.buildings.place(b, &mut state.grid) {
                // Deduct cost from inventory.
                buildcost::pay_cost(&mut state.inventory, kind);
                state.last_placed = Some(grid_pos);
                state.placement_flash = Some((grid_pos, 10));
                state.stats.buildings_placed += 1;

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
                // Refund cost.
                if let Some(b) = state.buildings.get(bid) {
                    buildcost::refund_cost(&mut state.inventory, b.kind);
                }
                state.buildings.remove(bid, &mut state.grid);
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

/// Runs one simulation tick (called at fixed [`TICKS_PER_SECOND`] rate).
///
/// **Performance strategy**: Systems are frequency-gated based on how fast they
/// need to update. Critical systems (belts, inserters) run every tick. Slower
/// systems (pollution, enemies) run less frequently to reduce CPU load on
/// low-end devices.
///
/// | System | Frequency | Reason |
/// |--------|-----------|--------|
/// | Belts | Every tick | Smooth visual movement |
/// | Inserters | Every tick | Must keep up with belts |
/// | Machines | Every tick | Timer countdown accuracy |
/// | Labs | Every 2 ticks | Not time-critical |
/// | Enemies | Every 2 ticks | Movement still smooth at 10 updates/sec |
/// | Combat | Every 2 ticks | Turret fire rate is already 10+tick cooldowns |
/// | Pollution | Every 10 ticks | Changes very slowly, expensive diffusion |

/// Finds an unpaired underground belt entry in the opposite direction within range.
///
/// When placing an underground belt exit, we look backward along the facing direction
/// for an entry that doesn't have a pair yet.
fn find_underground_pair(
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

fn simulation_tick(state: &mut GameState) {
    state.stats.total_ticks += 1;
    let tick = state.stats.total_ticks;

    // Day/night cycle advances every tick for smooth transitions.
    state.daynight.tick();

    // Tick toast notifications.
    state.tick_toasts();

    // Tick robot workers (move from ship to target).
    for robot in &mut state.robots {
        robot.2 += 0.05; // 20 ticks to reach target
    }
    state.robots.retain(|r| r.2 < 1.0);

    // Tick placement flash.
    if let Some((_, ref mut ticks)) = state.placement_flash {
        if *ticks > 0 {
            *ticks -= 1;
        } else {
            state.placement_flash = None;
        }
    }

    // --- EVERY TICK (20 Hz) --- Critical path for smooth gameplay ---

    // 1. Machines process: count down timers, complete recipes, start new ones.
    machine::tick_machines(
        &mut state.grid,
        &mut state.buildings,
        &mut state.items,
        &mut state.stats,
    );

    // 2. Machine output: eject finished items onto output belts.
    machine::tick_machine_output(&mut state.grid, &mut state.buildings, &mut state.items);

    // 3. Inserters: move items between belts and machines.
    inserter::tick_inserters(&mut state.grid, &mut state.buildings, &mut state.items);

    // 4. Belts: advance item progress, transfer between tiles.
    belt::tick_belts(&mut state.grid, &state.buildings, &mut state.items);

    // 4b. Splitters: route items at split points.
    splitter::tick_splitters(&mut state.grid, &mut state.buildings, &mut state.items);

    // 4c. Pump jacks: extract oil from deposits.
    fluid::tick_pump_jacks(&state.grid, &mut state.buildings);

    // --- EVERY 2 TICKS (10 Hz) --- Still responsive, saves 50% CPU for these ---

    if tick % 2 == 0 {
        // 5. Labs: consume science packs, advance research.
        research::tick_labs(&mut state.buildings, &mut state.research);

        // 5b. Storage chests feed player inventory (the key progression mechanic).
        // Any items in a StorageChest's input_buffer are added to the player's inventory.
        let chest_ids = state.buildings.alive_ids();
        for bid in chest_ids {
            let kind = state.buildings.get(bid).map(|b| b.kind);
            if kind != Some(types::BuildingKind::StorageChest) {
                continue;
            }
            if let Some(building) = state.buildings.get_mut(bid) {
                if let Some(ms) = &mut building.machine_state {
                    // Move all items from chest buffer into player inventory.
                    for resource in ms.input_buffer.drain(..) {
                        *state.inventory.entry(resource).or_insert(0) += 1;
                    }
                }
            }
        }

        // 5c. Mark first wave for story.
        if !state.story.first_wave_arrived && state.enemies.wave_number > 0 {
            state.story.first_wave_arrived = true;
        }

        // Wave warning toast.
        if state.enemies.wave_warned && state.enemies.list.iter().filter(|e| e.alive).count() == 0 {
            state.toast("!! WAVE INCOMING — Prepare defenses! !!".to_string(), 100);
            // Only show once per warning cycle (wave_warned resets after spawn).
        }

        // 6. Enemy AI: movement, attacking buildings.
        enemy::tick_enemies(
            &mut state.grid,
            &mut state.buildings,
            &mut state.enemies,
            &state.nests,
            &mut state.evolution,
            tick,
            &mut state.stats.enemies_killed,
        );

        // 7. Trains: move along rails, wait at stops.
        train::tick_trains(&state.grid, &state.buildings, &mut state.trains);

        // 8. Combat: turrets shoot enemies.
        let kills_before = state.stats.enemies_killed;
        combat::tick_combat(
            &state.grid,
            &mut state.buildings,
            &mut state.enemies,
            &mut state.stats.enemies_killed,
        );
        // Loot drops: enemies killed give small resource bonus.
        let new_kills = state.stats.enemies_killed - kills_before;
        if new_kills > 0 {
            *state.inventory.entry(types::Resource::IronPlate).or_insert(0) += new_kills as u32;
            *state.inventory.entry(types::Resource::Coal).or_insert(0) += new_kills as u32;
        }
    }

    // --- EVERY 10 TICKS: Passive building regeneration (walls/turrets heal 1 HP) ---
    if tick % 10 == 0 {
        let ids = state.buildings.alive_ids();
        for bid in ids {
            if let Some(b) = state.buildings.get_mut(bid) {
                if b.hp < b.max_hp && b.hp > 0.0 {
                    b.hp = (b.hp + 0.5).min(b.max_hp);
                }
            }
        }
    }

    // --- EVERY 5 TICKS (4 Hz) --- Medium frequency systems ---

    if tick % 5 == 0 {
        // 6. Power: calculate supply/demand, consume fuel in boilers/engines.
        power::update_power(&mut state.buildings, &mut state.power, &state.daynight);
    }

    // --- EVERY 10 TICKS (2 Hz) --- Expensive systems that change slowly ---

    if tick % 10 == 0 {
        // 7. Pollution: generate from machines, diffuse across grid.
        pollution::tick_pollution(&mut state.grid, &state.buildings);
    }

    // --- EVERY 20 TICKS (1 Hz) --- Story triggers + milestones ---
    if tick % 20 == 0 {
        let new_beats = story::check_story_triggers(
            &mut state.story,
            state.stats.items_crafted,
            state.stats.enemies_killed,
            &state.research.completed,
            tick,
        );
        for (text, subtext) in new_beats {
            state.toast(text, 120);
            state.toast(subtext, 160);
        }

        // Check win condition (final story beat = consciousness restored).
        if state.stats.items_crafted >= 50000 && !state.game_won {
            state.game_won = true;
            state.toast("CONSCIOUSNESS RESTORED! You found your crew!".to_string(), 300);
            state.toast("Thank you for playing AutoForge <3".to_string(), 300);
        }

        // Check milestones.
        let new_milestones = milestones::check_milestones(
            &state.milestones_completed,
            state.stats.items_crafted,
            state.stats.enemies_killed,
            &state.research.completed,
            &state.inventory,
            tick,
        );
        for idx in new_milestones {
            state.milestones_completed[idx] = true;
            let milestone = &milestones::MILESTONES[idx];
            // Award rewards to inventory.
            for &(resource, count) in milestone.reward {
                *state.inventory.entry(resource).or_insert(0) += count;
            }
            state.toast(format!("*** MILESTONE: {} ***", milestone.name), 120);
            state.toast(format!("+{} resource types rewarded!", milestone.reward.len()), 80);
        }
    }
}

/// Draws the screen-space UI overlay.
///
/// Layout:
/// - **Top-left**: Game title, tick count, direction indicator
/// - **Top-right**: Hovered tile info panel (dark background)
/// - **Bottom**: Categorized toolbar with sprite icons + labels
/// - **Bottom-right**: Controls hint (fades out at low zoom)
fn draw_ui(state: &GameState, atlas: &SpriteAtlas) {
    // Modern UI colors — clean, high contrast, readable.
    let panel_bg = Color::new(0.08, 0.08, 0.12, 0.92);
    let panel_border = Color::new(0.25, 0.25, 0.35, 0.6);
    let text_bright = Color::new(0.98, 0.98, 0.98, 1.0);
    let text_dim = Color::new(0.65, 0.65, 0.7, 1.0);
    let text_accent = Color::new(0.45, 0.92, 0.55, 1.0);
    let selected_bg = Color::new(0.15, 0.35, 0.55, 0.8);
    let selected_border = Color::new(0.45, 0.75, 1.0, 1.0);

    // --- Top-left: game info ---
    draw_rectangle(5.0, 5.0, 400.0, 85.0, panel_bg);
    draw_rectangle_lines(5.0, 5.0, 400.0, 85.0, 1.0, panel_border);
    draw_text("AUTOFORGE", 15.0, 28.0, 28.0, text_accent);
    draw_text(
        &format!("Time: {}:{:02}  |  FPS: {}",
            state.stats.total_ticks / 1200, // minutes
            (state.stats.total_ticks / 20) % 60, // seconds
            get_fps()),
        15.0,
        48.0,
        16.0,
        text_dim,
    );

    // Power status
    let power_color = if state.power.satisfaction >= 0.99 {
        Color::new(0.3, 0.9, 0.3, 1.0)
    } else if state.power.satisfaction >= 0.5 {
        Color::new(0.9, 0.9, 0.2, 1.0)
    } else {
        Color::new(0.9, 0.2, 0.2, 1.0)
    };
    draw_text(
        &format!(
            "Power: {:.0}/{:.0} kW ({:.0}%)",
            state.power.supply,
            state.power.demand,
            state.power.satisfaction * 100.0,
        ),
        15.0,
        66.0,
        16.0,
        power_color,
    );

    // Direction indicator + enemies killed
    let dir_text = match state.placement_direction {
        types::Direction::North => "N",
        types::Direction::East => "E",
        types::Direction::South => "S",
        types::Direction::West => "W",
    };
    draw_text(&format!("Dir: {} [R]", dir_text), 300.0, 28.0, 20.0, text_bright);
    draw_text(
        &format!("Kills: {}", state.stats.enemies_killed),
        300.0,
        48.0,
        14.0,
        text_dim,
    );
    // Day/night time
    let dn_color = if state.daynight.is_day() {
        Color::new(0.9, 0.85, 0.3, 1.0)
    } else {
        Color::new(0.4, 0.4, 0.7, 1.0)
    };
    let speed_str = if state.game_speed > 1 { format!(" [{}x]", state.game_speed) } else { String::new() };
    draw_text(&format!("{}{}", state.daynight.display(), speed_str), 15.0, 82.0, 14.0, dn_color);

    // Pause menu overlay
    if state.paused {
        let cx = screen_width() * 0.5;
        let cy = screen_height() * 0.5;
        let pw = 300.0;
        let ph = 220.0;

        // Dark overlay behind everything.
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.05, 0.5));

        // Menu panel.
        draw_rectangle(cx - pw * 0.5, cy - ph * 0.5, pw, ph, Color::new(0.08, 0.06, 0.14, 0.95));
        draw_rectangle_lines(cx - pw * 0.5, cy - ph * 0.5, pw, ph, 2.0, Color::new(0.4, 0.3, 0.7, 0.8));

        draw_text("PAUSED", cx - 55.0, cy - ph * 0.5 + 40.0, 36.0, Color::new(0.9, 0.85, 0.4, 1.0));

        // Menu items.
        let items = [
            "Space — Resume",
            "F5 — Save Game",
            "F9 — Load Game",
            "E — Recipe Book",
            "Tab — Research",
            "+/- — Game Speed",
            "H — Toggle Tutorial",
        ];
        for (i, item) in items.iter().enumerate() {
            draw_text(
                item,
                cx - pw * 0.5 + 30.0,
                cy - ph * 0.5 + 70.0 + i as f32 * 22.0,
                17.0,
                Color::new(0.8, 0.8, 0.85, 0.9),
            );
        }
    }

    // --- Top-right: hovered tile info panel ---
    let mouse_screen = Vec2::new(mouse_position().0, mouse_position().1);
    let mouse_world = state.camera.screen_to_world(mouse_screen);
    let grid_pos = grid::Grid::world_to_grid(mouse_world);

    let panel_w = 280.0;
    let panel_x = screen_width() - panel_w - 10.0;

    if let Some(tile) = state.grid.get_tile(grid_pos) {
        let mut lines: Vec<(String, Color)> = Vec::new();
        lines.push((format!("({}, {})", grid_pos.x, grid_pos.y), text_dim));
        lines.push((format!("{:?}", tile.terrain), text_bright));

        if let Some(deposit) = tile.deposit {
            let amount_str = if tile.ore_amount == u32::MAX {
                "Infinite".to_string()
            } else {
                format!("{}", tile.ore_amount)
            };
            lines.push((
                format!("{} ({})", deposit.display_name(), amount_str),
                Color::new(0.9, 0.7, 0.3, 1.0),
            ));
        }

        if let Some(bid) = tile.building {
            if let Some(b) = state.buildings.get(bid) {
                lines.push((b.kind.display_name().to_string(), text_accent));
                lines.push((format!("Facing: {:?}", b.direction), text_dim));
                if let Some(ref ms) = b.machine_state {
                    // Show locked recipe name for assemblers.
                    if let Some(rid) = ms.selected_recipe {
                        if rid.0 < recipe::RECIPES.len() {
                            lines.push((
                                format!("Recipe: {}", recipe::RECIPES[rid.0].name),
                                Color::new(0.9, 0.8, 0.4, 1.0),
                            ));
                        }
                    } else if b.kind == types::BuildingKind::AssemblerT1
                        || b.kind == types::BuildingKind::AssemblerT2
                        || b.kind == types::BuildingKind::AssemblerT3
                        || b.kind == types::BuildingKind::ChemicalPlant
                    {
                        lines.push(("Recipe: Auto (feed items)".to_string(), text_dim));
                    }
                    if !ms.input_buffer.is_empty() {
                        lines.push((format!("In: {} items", ms.input_buffer.len()), text_dim));
                    }
                    if !ms.output_buffer.is_empty() {
                        lines.push((format!("Out: {} items", ms.output_buffer.len()), text_dim));
                    }
                    if ms.fuel_ticks > 0 {
                        lines.push((format!("Fuel: {} ticks", ms.fuel_ticks), text_dim));
                    }
                }
            }
        }

        // Show items on this tile (on belts).
        let items_here = state.grid.items_at(grid_pos);
        if !items_here.is_empty() {
            for &item_id in items_here.iter().take(3) {
                if let Some(item) = state.items.get(item_id) {
                    lines.push((
                        format!("Item: {}", item.resource.display_name()),
                        Color::new(0.9, 0.85, 0.5, 1.0),
                    ));
                }
            }
        }

        let panel_h = 12.0 + lines.len() as f32 * 22.0 + 8.0;
        draw_rectangle(panel_x, 5.0, panel_w, panel_h, panel_bg);
        draw_rectangle_lines(panel_x, 5.0, panel_w, panel_h, 1.0, panel_border);

        for (i, (text, color)) in lines.iter().enumerate() {
            draw_text(text, panel_x + 12.0, 26.0 + i as f32 * 22.0, 18.0, *color);
        }
    }

    // --- Bottom: categorized toolbar with sprite icons ---
    let toolbar_h = 80.0;
    let toolbar_y = screen_height() - toolbar_h;
    draw_rectangle(0.0, toolbar_y, screen_width(), toolbar_h, panel_bg);
    draw_line(0.0, toolbar_y, screen_width(), toolbar_y, 2.0, panel_border);

    // Toolbar items: (hotkey label, display name, kind, texture)
    let toolbar_items: Vec<(&str, &str, types::BuildingKind, &Texture2D)> = vec![
        ("1", "Belt", types::BuildingKind::BeltYellow, &atlas.belt_yellow[0]),
        ("2", "Miner", types::BuildingKind::Miner, &atlas.miner),
        ("3", "Furnace", types::BuildingKind::StoneFurnace, &atlas.stone_furnace),
        ("4", "Inserter", types::BuildingKind::InserterRegular, &atlas.inserter),
        ("5", "Assembler", types::BuildingKind::AssemblerT1, &atlas.assembler),
        ("6", "Boiler", types::BuildingKind::Boiler, &atlas.boiler),
        ("7", "Engine", types::BuildingKind::SteamEngine, &atlas.steam_engine),
        ("8", "Lab", types::BuildingKind::Lab, &atlas.lab),
        ("9", "Chest", types::BuildingKind::StorageChest, &atlas.chest),
        ("0", "Splitter", types::BuildingKind::Splitter, &atlas.chest),
        ("T", "Turret", types::BuildingKind::GunTurret, &atlas.gun_turret),
        ("G", "Wall", types::BuildingKind::Wall, &atlas.wall),
        ("C", "Chemical", types::BuildingKind::ChemicalPlant, &atlas.assembler),
        ("P", "Solar", types::BuildingKind::SolarPanel, &atlas.solar_panel),
    ];

    let slot_w = 76.0;
    let slot_h = 66.0;
    let total_w = toolbar_items.len() as f32 * slot_w;
    let start_x = (screen_width() - total_w) * 0.5; // center the toolbar

    for (i, (hotkey, name, kind, tex)) in toolbar_items.iter().enumerate() {
        let x = start_x + i as f32 * slot_w;
        let y = toolbar_y + 5.0;
        let is_selected = state.selected_building == Some(*kind);

        // Slot background
        if is_selected {
            draw_rectangle(x + 2.0, y, slot_w - 4.0, slot_h, selected_bg);
            draw_rectangle_lines(x + 2.0, y, slot_w - 4.0, slot_h, 2.0, selected_border);
        } else {
            draw_rectangle(x + 2.0, y, slot_w - 4.0, slot_h, Color::new(0.12, 0.12, 0.15, 0.6));
            draw_rectangle_lines(x + 2.0, y, slot_w - 4.0, slot_h, 1.0, Color::new(0.3, 0.3, 0.3, 0.5));
        }

        // Sprite icon (centered in top portion of slot)
        let icon_size = 34.0;
        let icon_x = x + (slot_w - icon_size) * 0.5;
        let icon_y = y + 3.0;
        draw_texture_ex(
            tex,
            icon_x,
            icon_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::splat(icon_size)),
                ..Default::default()
            },
        );

        // Hotkey badge (top-left corner with background)
        draw_rectangle(x + 3.0, y + 1.0, 16.0, 16.0, Color::new(0.0, 0.0, 0.0, 0.6));
        draw_text(hotkey, x + 6.0, y + 14.0, 16.0, Color::new(1.0, 1.0, 0.5, 0.9));

        // Label below icon
        let label_w = measure_text(name, None, 14, 1.0).width;
        draw_text(
            name,
            x + (slot_w - label_w) * 0.5,
            y + slot_h - 5.0,
            14.0,
            if is_selected { text_bright } else { text_dim },
        );
    }

    // --- Selected building name + cost (above toolbar, centered) ---
    if let Some(kind) = state.selected_building {
        let name = kind.display_name();
        // Show cost.
        let cost = buildcost::building_cost(kind);
        let cost_str: String = cost
            .iter()
            .map(|(r, c)| format!("{}x {}", c, match r {
                types::Resource::IronPlate => "Iron",
                types::Resource::CopperPlate => "Copper",
                types::Resource::Stone => "Stone",
                types::Resource::StoneBrick => "Brick",
                types::Resource::Coal => "Coal",
                types::Resource::Gear => "Gear",
                types::Resource::Wire => "Wire",
                types::Resource::GreenCircuit => "Circuit",
                types::Resource::SteelPlate => "Steel",
                types::Resource::Pipe => "Pipe",
                types::Resource::Battery => "Battery",
                types::Resource::Concrete => "Concrete",
                _ => "?",
            }))
            .collect::<Vec<_>>()
            .join(" + ");

        let info = format!("{} — {:?}", name, state.placement_direction);
        let can_afford = buildcost::can_afford(&state.inventory, kind);
        let cost_color = if can_afford {
            Color::new(0.5, 0.9, 0.5, 0.9)
        } else {
            Color::new(0.9, 0.3, 0.3, 0.9)
        };

        let total_w = measure_text(&info, None, 20, 1.0).width.max(measure_text(&cost_str, None, 14, 1.0).width) + 28.0;
        let panel_x = (screen_width() - total_w) * 0.5;

        draw_rectangle(panel_x, toolbar_y - 50.0, total_w, 46.0, panel_bg);
        draw_rectangle_lines(panel_x, toolbar_y - 50.0, total_w, 46.0, 1.0, panel_border);
        draw_text(&info, panel_x + 10.0, toolbar_y - 32.0, 20.0, text_bright);
        draw_text(&cost_str, panel_x + 10.0, toolbar_y - 12.0, 14.0, cost_color);
    }

    // --- Minimap (top-right corner, below info panel) ---
    {
        let mm_size = 140.0;
        let mm_x = screen_width() - mm_size - 10.0;
        let mm_y = screen_height() - 210.0 - mm_size; // above toolbar area
        let panel_bg = Color::new(0.06, 0.06, 0.08, 0.9);

        draw_rectangle(mm_x - 4.0, mm_y - 18.0, mm_size + 8.0, mm_size + 22.0, panel_bg);
        draw_rectangle_lines(mm_x - 4.0, mm_y - 18.0, mm_size + 8.0, mm_size + 22.0, 1.0, Color::new(0.3, 0.25, 0.5, 0.7));
        draw_text("MAP", mm_x + mm_size * 0.5 - 15.0, mm_y - 4.0, 14.0, Color::new(0.6, 0.55, 0.8, 0.8));

        // Draw a simplified view of the map (each pixel = 4 tiles).
        let tiles_per_pixel = 4;
        let map_pixels = (mm_size as i32).min(state.grid.width / tiles_per_pixel);

        // Center the minimap on the camera position.
        let cam_grid = grid::Grid::world_to_grid(state.camera.target);
        let half_range = map_pixels * tiles_per_pixel / 2;

        for py in 0..map_pixels {
            for px in 0..map_pixels {
                let gx = cam_grid.x - half_range + px * tiles_per_pixel;
                let gy = cam_grid.y - half_range + py * tiles_per_pixel;
                let pos = types::GridPos::new(gx, gy);

                let color = if let Some(tile) = state.grid.get_tile(pos) {
                    if tile.building.is_some() {
                        Color::new(0.5, 0.5, 0.8, 1.0) // buildings = blue dots
                    } else if tile.deposit.is_some() {
                        Color::new(0.7, 0.5, 0.2, 1.0) // ore = orange
                    } else if tile.terrain == types::Terrain::Water {
                        Color::new(0.2, 0.3, 0.6, 1.0) // water = dark blue
                    } else if tile.terrain == types::Terrain::Forest {
                        Color::new(0.1, 0.4, 0.1, 1.0) // forest = green
                    } else if tile.pollution > 0.5 {
                        Color::new(0.4, 0.4, 0.1, 0.8) // pollution = yellow-green
                    } else {
                        Color::new(0.15, 0.15, 0.12, 1.0) // ground = dark
                    }
                } else {
                    Color::new(0.05, 0.05, 0.05, 1.0) // out of bounds
                };

                let screen_px = mm_x + px as f32 * (mm_size / map_pixels as f32);
                let screen_py = mm_y + py as f32 * (mm_size / map_pixels as f32);
                let pixel_size = mm_size / map_pixels as f32;
                draw_rectangle(screen_px, screen_py, pixel_size, pixel_size, color);
            }
        }

        // Draw enemies as red dots on minimap.
        for enemy in &state.enemies.list {
            if !enemy.alive { continue; }
            let eg = grid::Grid::world_to_grid(Vec2::new(enemy.x, enemy.y));
            let rpx = (eg.x - (cam_grid.x - half_range)) as f32 / (map_pixels * tiles_per_pixel) as f32 * mm_size;
            let rpy = (eg.y - (cam_grid.y - half_range)) as f32 / (map_pixels * tiles_per_pixel) as f32 * mm_size;
            if rpx >= 0.0 && rpx < mm_size && rpy >= 0.0 && rpy < mm_size {
                draw_circle(mm_x + rpx, mm_y + rpy, 2.0, Color::new(1.0, 0.1, 0.1, 0.9));
            }
        }

        // Camera viewport rectangle.
        let (vis_min, vis_max) = state.camera.visible_bounds();
        let vis_min_g = grid::Grid::world_to_grid(vis_min);
        let vis_max_g = grid::Grid::world_to_grid(vis_max);
        let rx = (vis_min_g.x - (cam_grid.x - half_range)) as f32 / (map_pixels * tiles_per_pixel) as f32 * mm_size;
        let ry = (vis_min_g.y - (cam_grid.y - half_range)) as f32 / (map_pixels * tiles_per_pixel) as f32 * mm_size;
        let rw = (vis_max_g.x - vis_min_g.x) as f32 / (map_pixels * tiles_per_pixel) as f32 * mm_size;
        let rh = (vis_max_g.y - vis_min_g.y) as f32 / (map_pixels * tiles_per_pixel) as f32 * mm_size;
        draw_rectangle_lines(mm_x + rx, mm_y + ry, rw, rh, 1.0, WHITE);
    }

    // --- Toast notifications (center-top area) ---
    if !state.toasts.is_empty() {
        let cx = screen_width() * 0.5;
        for (i, (msg, remaining)) in state.toasts.iter().enumerate() {
            let alpha = (*remaining as f32 / 20.0).min(1.0); // fade out last 20 ticks
            let y = 70.0 + i as f32 * 26.0;
            let w = measure_text(msg, None, 20, 1.0).width;
            draw_rectangle(
                cx - w * 0.5 - 12.0,
                y - 16.0,
                w + 24.0,
                24.0,
                Color::new(0.1, 0.1, 0.15, 0.85 * alpha),
            );
            draw_text(
                msg,
                cx - w * 0.5,
                y,
                20.0,
                Color::new(1.0, 1.0, 1.0, alpha),
            );
        }
    }

    // --- Bottom-right: controls hint ---
    let help_x = screen_width() - 280.0;
    let help_y = toolbar_y - 80.0;
    let hint_color = Color::new(0.5, 0.5, 0.5, 0.6);
    let hints = [
        "WASD: Pan | Scroll: Zoom | Edge: Scroll",
        "LClick: Place | RClick: Remove (hold=drag)",
        "R: Rotate | Q: Copy | Ctrl+Z: Undo",
        "E: Recipes | Tab: Research | H: Tutorial",
        "Space: Pause | +/-: Speed | F5/F9: Save/Load",
        "Middle-Click: Hand-insert item into machine",
    ];
    for (i, line) in hints.iter().enumerate() {
        draw_text(line, help_x, help_y + i as f32 * 18.0, 15.0, hint_color);
    }

    // --- Tutorial overlay ---
    if state.show_tutorial && state.tutorial_step < 6 {
        let tut_bg = Color::new(0.05, 0.03, 0.1, 0.9);
        let tut_w = 380.0;
        let tut_h = 80.0;
        let tut_x = (screen_width() - tut_w) * 0.5;
        let tut_y = 100.0;

        draw_rectangle(tut_x, tut_y, tut_w, tut_h, tut_bg);
        draw_rectangle_lines(tut_x, tut_y, tut_w, tut_h, 2.0, Color::new(0.5, 0.4, 0.8, 0.8));

        let tutorial_text = match state.tutorial_step {
            0 => ("Welcome! Click a building in the toolbar below", "or press 1-8 to select it. Press E for recipes!"),
            1 => ("Place a MINER on an ore deposit (big rocks)", "Face it toward where you want items to go (R to rotate)"),
            2 => ("Place BELTS from the miner's output arrow", "Items will flow along them automatically!"),
            3 => ("Place an INSERTER between belt and machine", "It grabs from behind, places forward (R to rotate)"),
            4 => ("Place a FURNACE! Feed it ore AND coal for fuel", "Use 2 inserters: one for ore, one for coal"),
            5 => ("Put items into a STORAGE CHEST to collect them!", "Chest contents go to your inventory for building!"),
            _ => ("", ""),
        };

        draw_text(tutorial_text.0, tut_x + 15.0, tut_y + 30.0, 20.0, Color::new(0.95, 0.9, 1.0, 1.0));
        draw_text(tutorial_text.1, tut_x + 15.0, tut_y + 55.0, 16.0, Color::new(0.7, 0.65, 0.85, 0.9));
    }

    // --- Inventory display (left side, below main panel) ---
    {
        let inv_x = 5.0;
        let inv_y = 100.0;
        let inv_bg = Color::new(0.06, 0.06, 0.08, 0.8);
        let inv_w = 180.0;

        // Show all resources the player has (dynamic list).
        let all_resources: &[(types::Resource, &str)] = &[
            (types::Resource::IronPlate, "Iron Plate"),
            (types::Resource::CopperPlate, "Copper Plate"),
            (types::Resource::Stone, "Stone"),
            (types::Resource::Coal, "Coal"),
            (types::Resource::StoneBrick, "Stone Brick"),
            (types::Resource::SteelPlate, "Steel Plate"),
            (types::Resource::Gear, "Gear"),
            (types::Resource::Wire, "Wire"),
            (types::Resource::GreenCircuit, "Green Circuit"),
            (types::Resource::RedCircuit, "Red Circuit"),
            (types::Resource::BlueCircuit, "Blue Circuit"),
            (types::Resource::Pipe, "Pipe"),
            (types::Resource::IronStick, "Iron Stick"),
            (types::Resource::Sulfur, "Sulfur"),
            (types::Resource::Plastic, "Plastic"),
            (types::Resource::Battery, "Battery"),
            (types::Resource::BasicAmmo, "Ammo"),
            (types::Resource::ScienceRed, "Red Science"),
            (types::Resource::ScienceGreen, "Green Science"),
        ];
        // Only show resources the player actually has.
        let show_resources: Vec<&(types::Resource, &str)> = all_resources
            .iter()
            .filter(|(r, _)| state.inventory.get(r).copied().unwrap_or(0) > 0)
            .collect();

        let inv_h = 18.0 + show_resources.len().min(12) as f32 * 16.0;
        draw_rectangle(inv_x, inv_y, inv_w, inv_h, inv_bg);

        draw_text("Inventory", inv_x + 8.0, inv_y + 14.0, 14.0, Color::new(0.7, 0.6, 0.9, 1.0));
        for (i, (resource, name)) in show_resources.iter().enumerate() {
            if i >= 12 { break; } // max 12 visible
            let count = state.inventory.get(resource).copied().unwrap_or(0);
            let y = inv_y + 28.0 + i as f32 * 16.0;
            let color = if count > 0 {
                Color::new(0.85, 0.85, 0.9, 1.0)
            } else {
                Color::new(0.4, 0.4, 0.4, 0.6)
            };
            draw_text(&format!("{}: {}", name, count), inv_x + 10.0, y, 13.0, color);
        }
    }

    // --- Next Goal Indicator (below inventory on left) ---
    {
        let goal_x = 5.0;
        let goal_y = 330.0;
        let goal_bg = Color::new(0.08, 0.06, 0.14, 0.88);
        draw_rectangle(goal_x, goal_y, 220.0, 90.0, goal_bg);
        draw_rectangle_lines(goal_x, goal_y, 220.0, 90.0, 1.0, Color::new(0.4, 0.3, 0.6, 0.5));
        draw_text("Next Goal:", goal_x + 8.0, goal_y + 16.0, 14.0, Color::new(0.9, 0.7, 0.3, 1.0));

        let goal_text = if state.tutorial_step < 5 {
            "Follow the tutorial above ^"
        } else if state.research.current_tech.is_some() {
            "Feed science packs to Labs!"
        } else if state.stats.items_crafted < 5 {
            "Feed ore+coal into Furnace via Inserters"
        } else if !state.research.completed[0] {
            "Craft Red Science (Assembler) > feed Lab"
        } else if !state.research.completed[6] {
            "Craft Green Circuits (needs Wire!)"
        } else if !state.research.completed[7] {
            "Research Advanced Electronics"
        } else {
            "Expand and optimize!"
        };
        draw_text(goal_text, goal_x + 8.0, goal_y + 36.0, 13.0, Color::new(0.85, 0.85, 0.95, 1.0));

        // Production rate (items/min).
        let items_per_min = if state.stats.total_ticks > 0 {
            state.stats.items_crafted as f32 / (state.stats.total_ticks as f32 / 1200.0)
        } else {
            0.0
        };
        draw_text(
            &format!("Production: {:.1}/min", items_per_min),
            goal_x + 8.0,
            goal_y + 56.0,
            12.0,
            Color::new(0.6, 0.8, 0.6, 0.9),
        );
        // Additional stats.
        let enemy_count = state.enemies.list.iter().filter(|e| e.alive).count();
        let building_count = state.buildings.alive_ids().len();
        draw_text(
            &format!("Buildings: {}  |  Enemies: {}", building_count, enemy_count),
            goal_x + 8.0,
            goal_y + 74.0,
            11.0,
            Color::new(0.5, 0.6, 0.7, 0.8),
        );
        // Tip about controls.
        draw_text(
            "E:Recipes Tab:Research Mid:Insert",
            goal_x + 8.0,
            goal_y + 88.0,
            10.0,
            Color::new(0.4, 0.4, 0.6, 0.6),
        );
    }

    // --- Help overlay (F1) ---
    if state.show_help {
        let sw = screen_width();
        let sh = screen_height();
        let pw = (sw * 0.6).min(600.0);
        let ph = (sh * 0.75).min(500.0);
        let px = (sw - pw) * 0.5;
        let py = (sh - ph) * 0.5;

        draw_rectangle(px, py, pw, ph, Color::new(0.05, 0.04, 0.08, 0.95));
        draw_rectangle_lines(px, py, pw, ph, 2.0, Color::new(0.4, 0.3, 0.7, 0.8));
        draw_text("AUTOFORGE — HELP (F1 to close)", px + 20.0, py + 30.0, 24.0, Color::new(0.9, 0.8, 0.4, 1.0));

        let help = [
            ("BUILDING", ""),
            ("1-9, 0", "Select building from toolbar"),
            ("Click toolbar", "Select building"),
            ("Left Click", "Place building (costs resources)"),
            ("Right Click", "Remove building (hold to mass-delete)"),
            ("R", "Rotate direction before placing"),
            ("Q", "Copy building type from world (eyedropper)"),
            ("Ctrl+Z", "Undo last placement"),
            ("", ""),
            ("INTERACTION", ""),
            ("Left Click (no selection)", "Click assembler to cycle recipe"),
            ("Middle Click", "Hand-insert item from inventory into machine"),
            ("", ""),
            ("NAVIGATION", ""),
            ("WASD / Arrows", "Pan camera"),
            ("Scroll wheel", "Zoom (toward cursor)"),
            ("Edge of screen", "Auto-scroll camera"),
            ("", ""),
            ("MENUS", ""),
            ("E", "Recipe book (all crafting recipes)"),
            ("Tab", "Research tree (unlock technologies)"),
            ("H", "Toggle tutorial"),
            ("Space", "Pause (shows pause menu)"),
            ("+/-", "Game speed (1x to 5x)"),
            ("F1", "This help screen"),
            ("F5 / F9", "Save / Load game"),
        ];

        for (i, (key, desc)) in help.iter().enumerate() {
            let y = py + 55.0 + i as f32 * 17.0;
            if y > py + ph - 15.0 { break; }
            if key.is_empty() { continue; }
            if desc.is_empty() {
                // Section header
                draw_text(key, px + 20.0, y, 16.0, Color::new(0.7, 0.6, 0.9, 1.0));
            } else {
                draw_text(key, px + 20.0, y, 14.0, Color::new(0.9, 0.85, 0.4, 0.9));
                draw_text(desc, px + 200.0, y, 14.0, Color::new(0.75, 0.75, 0.8, 0.8));
            }
        }
    }

    // --- Recipe Browser (E key) ---
    if state.show_recipes {
        draw_recipe_browser();
    }

    // --- Research screen overlay (Tab) ---
    if state.show_research {
        draw_research_screen(state);
    }
}

/// Draws the recipe browser overlay (E key).
fn draw_recipe_browser() {
    let panel_bg = Color::new(0.04, 0.04, 0.06, 0.95);
    let border = Color::new(0.4, 0.3, 0.6, 0.9);

    let sw = screen_width();
    let sh = screen_height();
    let pw = (sw * 0.75).min(800.0);
    let ph = (sh * 0.85).min(700.0);
    let px = (sw - pw) * 0.5;
    let py = (sh - ph) * 0.5;

    draw_rectangle(px, py, pw, ph, panel_bg);
    draw_rectangle_lines(px, py, pw, ph, 2.0, border);

    draw_text("RECIPE BOOK", px + 20.0, py + 32.0, 28.0, Color::new(0.9, 0.7, 1.0, 1.0));
    draw_text(
        "How to build your factory — press E to close",
        px + 20.0,
        py + 52.0,
        14.0,
        Color::new(0.6, 0.6, 0.7, 0.8),
    );

    // Recipe entries.
    let recipes: &[(&str, &str, &str)] = &[
        // (Name, Inputs, Output)
        ("== SMELTING (Furnace) ==", "", ""),
        ("Iron Plate", "1x Iron Ore", "1x Iron Plate"),
        ("Copper Plate", "1x Copper Ore", "1x Copper Plate"),
        ("Stone Brick", "2x Stone", "1x Stone Brick"),
        ("Steel Plate", "5x Iron Plate", "1x Steel Plate"),
        ("", "", ""),
        ("== BASIC ASSEMBLY ==", "", ""),
        ("Gear", "2x Iron Plate", "1x Gear"),
        ("Wire", "1x Copper Plate", "2x Wire"),
        ("Green Circuit", "1x Iron Plate + 3x Wire", "1x Green Circuit"),
        ("Pipe", "1x Iron Plate", "1x Pipe"),
        ("Iron Stick", "1x Iron Plate", "2x Iron Stick"),
        ("Basic Ammo", "1x Iron Plate", "1x Ammo"),
        ("", "", ""),
        ("== SCIENCE PACKS ==", "", ""),
        ("Red Science", "1x Gear + 1x Copper Plate", "1x Red Science"),
        ("Inserter (item)", "1x Circuit + 1x Gear + 1x Iron", "1x Inserter"),
        ("Green Science", "1x Inserter + 1x Iron Plate", "1x Green Science"),
        ("Blue Science", "1x Piercing Ammo + 1x Grenade + 2x Brick", "1x Blue Science"),
        ("", "", ""),
        ("== CHEMICAL PLANT ==", "", ""),
        ("Sulfur", "2x Coal + 1x Iron Plate", "2x Sulfur"),
        ("Plastic", "1x Coal + 1x Copper Plate", "2x Plastic"),
        ("Battery", "1x Copper + 1x Iron + 1x Sulfur", "1x Battery"),
        ("Rocket Fuel", "5x Coal + 1x Steel Plate", "1x Rocket Fuel"),
    ];

    let col_name = px + 20.0;
    let col_input = px + 180.0;
    let col_output = px + pw - 180.0;
    let start_y = py + 75.0;
    let row_h = 22.0;

    // Column headers.
    draw_text("Recipe", col_name, start_y - 5.0, 14.0, Color::new(0.7, 0.7, 0.8, 0.7));
    draw_text("Inputs", col_input, start_y - 5.0, 14.0, Color::new(0.7, 0.7, 0.8, 0.7));
    draw_text("Output", col_output, start_y - 5.0, 14.0, Color::new(0.7, 0.7, 0.8, 0.7));

    for (i, (name, inputs, output)) in recipes.iter().enumerate() {
        let y = start_y + 10.0 + i as f32 * row_h;
        if y > py + ph - 20.0 {
            break;
        }

        if name.starts_with("==") {
            // Section header.
            draw_text(name, col_name, y, 16.0, Color::new(0.9, 0.75, 0.4, 1.0));
        } else if !name.is_empty() {
            draw_text(name, col_name, y, 14.0, Color::new(0.9, 0.9, 0.95, 1.0));
            draw_text(inputs, col_input, y, 13.0, Color::new(0.7, 0.8, 0.7, 0.9));
            draw_text(output, col_output, y, 13.0, Color::new(0.5, 0.9, 0.5, 0.9));
        }
    }
}

/// Draws the research screen overlay.
fn draw_research_screen(state: &GameState) {
    let panel_bg = Color::new(0.04, 0.04, 0.06, 0.95);
    let border = Color::new(0.3, 0.3, 0.4, 0.9);

    let sw = screen_width();
    let sh = screen_height();
    let pw = (sw * 0.7).min(700.0);
    let ph = (sh * 0.8).min(600.0);
    let px = (sw - pw) * 0.5;
    let py = (sh - ph) * 0.5;

    // Background
    draw_rectangle(px, py, pw, ph, panel_bg);
    draw_rectangle_lines(px, py, pw, ph, 2.0, border);

    // Title
    draw_text("RESEARCH", px + 20.0, py + 35.0, 32.0, Color::new(0.4, 0.85, 0.4, 1.0));
    draw_text(
        "Click a technology to start researching. Tab to close.",
        px + 20.0,
        py + 55.0,
        14.0,
        Color::new(0.6, 0.6, 0.6, 1.0),
    );

    // Current research
    if let Some(idx) = state.research.current_tech {
        let tech = &research::TECHNOLOGIES[idx];
        let progress_pct = if tech.units_needed > 0 {
            state.research.progress as f32 / tech.units_needed as f32
        } else {
            0.0
        };
        draw_text(
            &format!("Researching: {} ({}/{})", tech.name, state.research.progress, tech.units_needed),
            px + 20.0,
            py + 80.0,
            18.0,
            YELLOW,
        );
        // Progress bar
        draw_rectangle(px + 20.0, py + 85.0, pw - 40.0, 8.0, Color::new(0.2, 0.2, 0.2, 1.0));
        draw_rectangle(px + 20.0, py + 85.0, (pw - 40.0) * progress_pct, 8.0, GREEN);
    } else {
        draw_text("No active research", px + 20.0, py + 80.0, 18.0, Color::new(0.6, 0.6, 0.6, 1.0));
    }

    // Tech list
    let start_y = py + 110.0;
    let row_h = 28.0;
    let col1 = px + 20.0;
    let col2 = px + 220.0;

    for (i, tech) in research::TECHNOLOGIES.iter().enumerate() {
        let y = start_y + i as f32 * row_h;
        if y > py + ph - 20.0 {
            break; // clip to panel
        }

        let completed = state.research.completed[i];
        let is_current = state.research.current_tech == Some(i);
        let can_research = state.research.can_research(i);

        let color = if completed {
            Color::new(0.3, 0.8, 0.3, 1.0) // green = done
        } else if is_current {
            YELLOW
        } else if can_research {
            WHITE
        } else {
            Color::new(0.4, 0.4, 0.4, 0.6) // gray = locked
        };

        let status = if completed {
            "[DONE]"
        } else if is_current {
            "[...]"
        } else if can_research {
            "[READY]"
        } else {
            "[LOCKED]"
        };

        draw_text(&format!("{} {}", tech.name, status), col1, y, 16.0, color);
        draw_text(tech.description, col2, y, 14.0, Color::new(0.5, 0.5, 0.6, 0.8));

        // Click to start research
        if can_research && !is_current {
            let mouse = Vec2::new(mouse_position().0, mouse_position().1);
            if mouse.x >= col1
                && mouse.x <= col1 + 400.0
                && mouse.y >= y - 14.0
                && mouse.y <= y + 4.0
            {
                // Highlight on hover
                draw_rectangle(col1 - 5.0, y - 14.0, 400.0, row_h - 2.0, Color::new(0.2, 0.3, 0.5, 0.3));
            }
        }
    }
}
