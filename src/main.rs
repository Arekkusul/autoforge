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

#[allow(dead_code)]
mod belt;
mod buildcost;
#[allow(dead_code)]
mod building;
#[allow(dead_code)]
mod camera;
mod combat;
#[allow(dead_code)]
mod constants;
#[allow(dead_code)]
mod cutscene;
mod daynight;
#[allow(dead_code)]
mod enemy;
#[allow(dead_code)]
mod fluid;
#[allow(dead_code)]
mod game;
#[allow(dead_code)]
mod grid;
mod inserter;
#[allow(dead_code)]
mod item;
mod machine;
mod mapgen;
#[allow(dead_code)]
mod milestones;
mod pollution;
#[allow(dead_code)]
mod power;
mod recipe;
mod render;
#[allow(dead_code)]
mod research;
mod save;
mod sound;
mod splitter;
#[allow(dead_code)]
mod story;
#[allow(dead_code)]
mod train;
#[allow(dead_code, non_snake_case)]
mod sprites;
#[allow(dead_code)]
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
    let mut sfx = sound::SoundEffects::generate().await;
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
    state.toast("Welcome to AutoForge! Press H for tutorial, F1 for help~".to_string(), 120);
    state.toast("Starting supplies: 50 Iron, 30 Copper, 20 Stone, 30 Coal, 25 Gears".to_string(), 150);

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
        // Disabled when mouse is over UI panels (toolbar, status, minimap) to prevent
        // accidental scrolling when clicking UI elements.
        {
            let edge_margin = 10.0;
            let edge_speed = 300.0 * get_frame_time().min(0.05) / state.camera.zoom;
            let (mx, my) = mouse_position();
            let toolbar_y = screen_height() - 88.0;
            let over_toolbar = my > toolbar_y;
            let over_status = mx < 250.0 && my < 120.0;
            let over_minimap = mx > screen_width() - 160.0 && my > screen_height() - 380.0 && my < toolbar_y;
            let any_overlay = state.paused || state.show_recipes || state.show_research
                || state.show_stats || state.show_achievements || state.show_help
                || state.recipe_picker.is_some();

            if !over_toolbar && !over_status && !over_minimap && !any_overlay {
                if mx < edge_margin { state.camera.target.x -= edge_speed; }
                if mx > screen_width() - edge_margin { state.camera.target.x += edge_speed; }
                if my < edge_margin { state.camera.target.y -= edge_speed; }
                if my > screen_height() - edge_margin { state.camera.target.y += edge_speed; }
            }
        }

        // 1. Input (every frame, independent of simulation tick rate).
        handle_input(&mut state, &mut sfx);
        state.camera.update(get_frame_time());

        // 2. Fixed-timestep simulation (with game speed multiplier).
        if !state.paused {
            state.tick_accumulator += dt * state.game_speed as f64;
            if state.tick_accumulator > MAX_ACCUMULATOR {
                state.tick_accumulator = MAX_ACCUMULATOR;
            }
            while state.tick_accumulator >= TICK_DURATION {
                simulation_tick(&mut state, &sfx);
                state.tick_accumulator -= TICK_DURATION;
            }
        }

        // 3. Render (every frame at display refresh rate).
        clear_background(Color::new(0.08, 0.08, 0.10, 1.0));

        // World-space rendering (affected by camera).
        if state.camera.map_view {
            // Map overview: zoom way out to show entire base area.
            let map_target = Vec2::new(
                state.grid.width as f32 * TILE_SIZE * 0.5,
                state.grid.height as f32 * TILE_SIZE * 0.5,
            );
            let map_zoom = 0.15;
            let map_cam = Camera2D {
                target: map_target,
                zoom: vec2(map_zoom * 2.0 / screen_width(), map_zoom * 2.0 / screen_height()),
                ..Default::default()
            };
            set_camera(&map_cam);
            // Create a temporary camera with map-view zoom for correct frustum culling.
            let mut map_camera = camera::GameCamera::new();
            map_camera.target = map_target;
            map_camera.zoom = map_zoom;
            render::draw_world(
                &state.grid,
                &state.buildings,
                &state.items,
                &state.enemies,
                &map_camera,
                &atlas,
                state.stats.total_ticks,
                state.power.satisfaction,
            );
            // Draw camera viewport rectangle on the overview.
            let (vis_min, vis_max) = state.camera.visible_bounds();
            draw_rectangle_lines(vis_min.x, vis_min.y, vis_max.x - vis_min.x, vis_max.y - vis_min.y, 4.0, WHITE);
        } else {
            set_camera(&state.camera.to_macroquad_camera());
            render::draw_world(
                &state.grid,
                &state.buildings,
                &state.items,
                &state.enemies,
                &state.camera,
                &atlas,
                state.stats.total_ticks,
                state.power.satisfaction,
            );
            render::draw_ghost_preview(
                &state.grid,
                &state.camera,
                &atlas,
                state.selected_building,
                state.placement_direction,
            );
            // Build zone circle (subtle outline when a building is selected).
            if state.selected_building.is_some() {
                let cx = state.grid.width as f32 * TILE_SIZE * 0.5;
                let cy = state.grid.height as f32 * TILE_SIZE * 0.5;
                let radius = state.build_radius * TILE_SIZE;
                draw_circle_lines(cx, cy, radius, 1.5, Color::new(0.3, 0.5, 0.8, 0.15));
            }
        }
        render::draw_night_overlay(state.daynight.darkness(), &state.buildings, &state.camera);

        // Placement flash effect (bright glow expanding outward).
        if let Some((pos, ticks)) = state.placement_flash {
            let t = ticks as f32 / 10.0;
            let expand = (1.0 - t) * 4.0; // expands as it fades
            let world = grid::Grid::grid_to_world(pos);
            draw_rectangle(
                world.x - 2.0 - expand,
                world.y - 2.0 - expand,
                TILE_SIZE + 4.0 + expand * 2.0,
                TILE_SIZE + 4.0 + expand * 2.0,
                Color::new(0.6, 0.85, 1.0, t * 0.6),
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

        // Render trains (rectangles moving along their routes).
        for train in &state.trains.list {
            if !train.alive { continue; }
            let size = TILE_SIZE * 0.8;
            let tx = train.x - size * 0.5;
            let ty = train.y - size * 0.3;
            // Train body (dark rectangle with colored top).
            draw_rectangle(tx, ty, size, size * 0.6, Color::new(0.15, 0.15, 0.2, 0.9));
            draw_rectangle(tx + 2.0, ty + 2.0, size - 4.0, size * 0.3, Color::new(0.3, 0.5, 0.8, 0.9));
            // Headlight.
            let (dx, dy) = train.direction.delta();
            let hx = train.x + dx as f32 * size * 0.4;
            let hy = train.y + dy as f32 * size * 0.3;
            draw_circle(hx, hy, 3.0, Color::new(1.0, 0.9, 0.3, 0.8));
            // Label.
            if state.camera.zoom >= 0.8 {
                draw_text("TRAIN", tx, ty - 4.0, 10.0, Color::new(0.6, 0.7, 0.9, 0.6));
            }
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
        draw_ui(&mut state, &atlas);

        next_frame().await;
    }
}

/// Handles player input for building selection, placement, and hotkeys.
fn handle_input(state: &mut GameState, sfx: &mut sound::SoundEffects) {
    // Pause toggle
    if is_key_pressed(KeyCode::Space) {
        state.paused = !state.paused;
    }

    // Rotate placement direction
    if is_key_pressed(KeyCode::R) {
        state.placement_direction = state.placement_direction.rotated_cw();
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
            if hit_x(px, py, pw) { state.show_tutorial = false; consumed = true; }
        }
        if state.recipe_picker.is_some() {
            let pw = 340.0f32;
            let ph: f32 = 50.0 + state.recipe_picker.as_ref().map(|r| r.1.len()).unwrap_or(0) as f32 * 28.0;
            let px = (sw - pw) * 0.5;
            let py = (sh - ph.min(500.0)) * 0.5;
            if hit_x(px, py, pw) { state.recipe_picker = None; consumed = true; }
        }
        if state.show_help {
            let pw = (sw * 0.6).min(600.0);
            let ph = (sh * 0.75).min(500.0);
            let px = (sw - pw) * 0.5;
            let py = (sh - ph) * 0.5;
            if hit_x(px, py, pw) { state.show_help = false; consumed = true; }
        }
        if state.show_recipes {
            let pw = (sw * 0.75).min(800.0);
            let ph = (sh * 0.85).min(700.0);
            let px = (sw - pw) * 0.5;
            let py = (sh - ph) * 0.5;
            if hit_x(px, py, pw) { state.show_recipes = false; consumed = true; }
        }
        if state.show_research {
            let pw = (sw * 0.7).min(700.0);
            let ph = (sh * 0.8).min(600.0);
            let px = (sw - pw) * 0.5;
            let py = (sh - ph) * 0.5;
            if hit_x(px, py, pw) { state.show_research = false; consumed = true; }
        }
        if state.show_achievements {
            let pw = (sw * 0.5).min(500.0);
            let ph = (sh * 0.7).min(450.0);
            let px = (sw - pw) * 0.5;
            let py = (sh - ph) * 0.5;
            if hit_x(px, py, pw) { state.show_achievements = false; consumed = true; }
        }
        if state.show_stats {
            let pw = (sw * 0.5).min(480.0);
            let ph = (sh * 0.6).min(400.0);
            let px = (sw - pw) * 0.5;
            let py = (sh - ph) * 0.5;
            if hit_x(px, py, pw) { state.show_stats = false; consumed = true; }
        }
        if consumed { return; }
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

    // Helper: close all overlays (ensures mutual exclusivity).
    fn close_all_overlays(state: &mut GameState) {
        state.show_research = false;
        state.show_recipes = false;
        state.show_stats = false;
        state.show_achievements = false;
        state.show_help = false;
        state.recipe_picker = None;
    }

    // Toggle research screen
    if is_key_pressed(KeyCode::Tab) {
        let was = state.show_research;
        close_all_overlays(state);
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
            state.toast("Sound: OFF".to_string(), 40);
        } else {
            sfx.volume = 0.5;
            state.toast("Sound: ON".to_string(), 40);
        }
    }

    // Toggle full help overlay
    if is_key_pressed(KeyCode::F1) {
        let was = state.show_help;
        close_all_overlays(state);
        state.show_help = !was;
    }

    // Toggle achievements screen
    if is_key_pressed(KeyCode::N) {
        let was = state.show_achievements;
        close_all_overlays(state);
        state.show_achievements = !was;
    }

    // Toggle production stats
    if is_key_pressed(KeyCode::V) {
        let was = state.show_stats;
        close_all_overlays(state);
        state.show_stats = !was;
    }

    // Blueprint: B to copy buildings near cursor, then click to paste.
    if is_key_pressed(KeyCode::B) {
        if state.pasting_blueprint {
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
                state.toast(format!("Copied {} buildings! Click to paste.", bp.len()), 60);
                state.blueprint = bp;
                state.pasting_blueprint = true;
                state.selected_building = None;
            }
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
        && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)
            || is_key_down(KeyCode::LeftSuper) || is_key_down(KeyCode::RightSuper))
    {
        if let Some(pos) = state.undo_history.pop() {
            if let Some(tile) = state.grid.get_tile(pos) {
                if let Some(bid) = tile.building {
                    if let Some(b) = state.buildings.get(bid) {
                        buildcost::refund_cost(&mut state.inventory, b.kind);
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
        close_all_overlays(state);
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
            close_all_overlays(state);
            state.toasts.clear();
            state.selected_building = None;
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
    // L: Laser Turret
    if is_key_pressed(KeyCode::L) {
        state.selected_building = Some(types::BuildingKind::LaserTurret);
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
                types::BuildingKind::InserterRegular if up => Some(types::BuildingKind::InserterLong),
                types::BuildingKind::InserterLong if up => Some(types::BuildingKind::InserterFast),
                types::BuildingKind::InserterFast if up => Some(types::BuildingKind::InserterStack),
                types::BuildingKind::InserterStack if !up => Some(types::BuildingKind::InserterFast),
                types::BuildingKind::InserterFast if !up => Some(types::BuildingKind::InserterLong),
                types::BuildingKind::InserterLong if !up => Some(types::BuildingKind::InserterRegular),
                // Assembler tiers
                types::BuildingKind::AssemblerT1 if up => Some(types::BuildingKind::AssemblerT2),
                types::BuildingKind::AssemblerT2 if up => Some(types::BuildingKind::AssemblerT3),
                types::BuildingKind::AssemblerT3 if !up => Some(types::BuildingKind::AssemblerT2),
                types::BuildingKind::AssemblerT2 if !up => Some(types::BuildingKind::AssemblerT1),
                // Furnace tiers
                types::BuildingKind::StoneFurnace if up => Some(types::BuildingKind::SteelFurnace),
                types::BuildingKind::SteelFurnace if up => Some(types::BuildingKind::ElectricFurnace),
                types::BuildingKind::ElectricFurnace if !up => Some(types::BuildingKind::SteelFurnace),
                types::BuildingKind::SteelFurnace if !up => Some(types::BuildingKind::StoneFurnace),
                // Underground belt tiers
                types::BuildingKind::UndergroundBeltYellow if up => Some(types::BuildingKind::UndergroundBeltRed),
                types::BuildingKind::UndergroundBeltRed if up => Some(types::BuildingKind::UndergroundBeltBlue),
                types::BuildingKind::UndergroundBeltBlue if !up => Some(types::BuildingKind::UndergroundBeltRed),
                types::BuildingKind::UndergroundBeltRed if !up => Some(types::BuildingKind::UndergroundBeltYellow),
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
    if state.selected_building.is_none() && state.recipe_picker.is_none() && is_mouse_button_pressed(MouseButton::Left) {
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

    // Blueprint paste: click to stamp buildings at cursor position.
    if state.pasting_blueprint && is_mouse_button_pressed(MouseButton::Left) {
        let mouse_screen = Vec2::new(mouse_position().0, mouse_position().1);
        let mouse_world = state.camera.screen_to_world(mouse_screen);
        let center = grid::Grid::world_to_grid(mouse_world);
        let mut placed = 0u32;
        for &(dx, dy, kind, dir) in &state.blueprint.clone() {
            let pos = types::GridPos::new(center.x + dx, center.y + dy);
            if !buildcost::can_afford(&state.inventory, kind) { continue; }
            let needs_ms = !kind.is_belt() && !kind.is_underground_belt()
                && !matches!(kind, types::BuildingKind::Wall | types::BuildingKind::Gate);
            let b = building::Building {
                kind, pos, direction: dir,
                machine_state: if needs_ms { Some(building::MachineState::new()) } else { None },
                hp: 100.0, max_hp: 100.0, underground_pair: None,
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
                            if dx > 0 { types::Direction::East } else { types::Direction::West }
                        } else {
                            if dy > 0 { types::Direction::South } else { types::Direction::North }
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
                    } else if !tile.terrain.is_buildable() && kind != types::BuildingKind::WaterPump {
                        state.toast("Can't build here — terrain is not buildable".to_string(), 40);
                    } else if kind == types::BuildingKind::Miner && (tile.deposit.is_none() || tile.deposit == Some(types::OreDeposit::Oil)) {
                        state.toast("Miner must be placed on an ore deposit".to_string(), 50);
                    } else if kind == types::BuildingKind::PumpJack && tile.deposit != Some(types::OreDeposit::Oil) {
                        state.toast("Pump jack must be placed on an oil well".to_string(), 50);
                    }
                }
            }

            if let Some(_new_bid) = state.buildings.place(b, &mut state.grid) {
                // Deduct cost from inventory.
                buildcost::pay_cost(&mut state.inventory, kind);
                state.undo_history.push(grid_pos);
                if state.undo_history.len() > 20 { state.undo_history.remove(0); }
                state.placement_flash = Some((grid_pos, 10));
                state.stats.buildings_placed += 1;
                sfx.play(&sfx.place);

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
                    state.toast("Tutorial complete! Press N for your roadmap~".to_string(), 120);
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

fn simulation_tick(state: &mut GameState, sfx: &sound::SoundEffects) {
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
    let crafted_before = state.stats.items_crafted;
    machine::tick_machines(
        &mut state.grid,
        &mut state.buildings,
        &mut state.items,
        &mut state.stats,
        state.power.satisfaction,
    );
    // Play a subtle ding every 50 items crafted (not every single craft — too noisy).
    if state.stats.items_crafted / 50 > crafted_before / 50 {
        sfx.play(&sfx.recipe_done);
    }

    // Check for depleted miners (every 200 ticks to avoid spam).
    if tick % 200 == 0 {
        let depleted: Vec<(i32, i32)> = state.buildings.iter()
            .filter(|(_, b)| b.kind == types::BuildingKind::Miner)
            .filter(|(_, b)| {
                state.grid.get_tile(b.pos)
                    .map(|t| t.deposit.is_none())
                    .unwrap_or(false)
                    && b.machine_state.as_ref()
                        .map(|ms| ms.progress_ticks == 0 && ms.output_buffer.is_empty())
                        .unwrap_or(false)
            })
            .map(|(_, b)| (b.pos.x, b.pos.y))
            .collect();
        for (x, y) in depleted.into_iter().take(1) {
            state.toast(format!("Miner at ({},{}) — ore depleted!", x, y), 120);
        }
    }

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
        let techs_before: Vec<bool> = state.research.completed.clone();
        research::tick_labs(&mut state.buildings, &mut state.research);
        // Check for newly completed research.
        let newly_completed: Vec<usize> = techs_before.iter()
            .zip(state.research.completed.iter())
            .enumerate()
            .filter(|(_, (&was, &now))| !was && now)
            .map(|(i, _)| i)
            .collect();
        for i in newly_completed {
            if i < research::TECHNOLOGIES.len() {
                sfx.play(&sfx.research_done);
                state.toast(format!("Research complete: {}!", research::TECHNOLOGIES[i].name), 120);
            }
        }

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
        let wave_before = state.enemies.wave_number;
        let buildings_before = state.buildings.alive_ids().len();
        enemy::tick_enemies(
            &mut state.grid,
            &mut state.buildings,
            &mut state.enemies,
            &state.nests,
            &mut state.evolution,
            tick,
            &mut state.stats.enemies_killed,
        );
        let buildings_after = state.buildings.alive_ids().len();
        if buildings_after < buildings_before {
            let lost = buildings_before - buildings_after;
            state.toast(format!("Building destroyed! ({} lost)", lost), 60);
        }

        if state.enemies.wave_number > wave_before {
            sfx.play(&sfx.wave_warning);
        }

        // 7. Trains: disabled pending full implementation (no cargo loading/unloading yet).
        // train::tick_trains(&state.grid, &state.buildings, &mut state.trains);

        // 8. Combat: turrets shoot enemies.
        let kills_before = state.stats.enemies_killed;
        combat::tick_combat(
            &state.grid,
            &mut state.buildings,
            &mut state.enemies,
            &mut state.stats.enemies_killed,
        );
        // Loot drops + sounds.
        let new_kills = state.stats.enemies_killed - kills_before;
        if new_kills > 0 {
            sfx.play(&sfx.turret_fire);
            sfx.play(&sfx.enemy_death);
            let n = new_kills as u32;
            *state.inventory.entry(types::Resource::IronPlate).or_insert(0) += n * 2;
            *state.inventory.entry(types::Resource::Coal).or_insert(0) += n;
            // Higher evolution = rarer drops.
            if state.evolution > 0.3 {
                *state.inventory.entry(types::Resource::CopperPlate).or_insert(0) += n;
            }
            if state.evolution > 0.5 {
                *state.inventory.entry(types::Resource::Gear).or_insert(0) += n;
            }
            if state.evolution > 0.7 {
                *state.inventory.entry(types::Resource::GreenCircuit).or_insert(0) += n;
            }
            if state.evolution > 0.9 {
                *state.inventory.entry(types::Resource::SteelPlate).or_insert(0) += n;
            }
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

    // --- EVERY 20 TICKS: Roboport logistics (auto-distribute items) ---
    if tick % 20 == 0 {
        // Find all roboports, then for each, scan nearby machines that need inputs.
        let roboport_ids: Vec<(types::BuildingId, types::GridPos)> = state.buildings.alive_ids()
            .iter()
            .filter_map(|&bid| {
                state.buildings.get(bid).and_then(|b| {
                    if b.kind == types::BuildingKind::Roboport { Some((bid, b.pos)) } else { None }
                })
            })
            .collect();

        for (_rbid, rpos) in &roboport_ids {
            let radius = 10i32;
            // Find machines in range that have a recipe set and need inputs.
            let nearby_ids: Vec<types::BuildingId> = state.buildings.alive_ids()
                .iter()
                .filter_map(|&bid| {
                    state.buildings.get(bid).and_then(|b| {
                        let d = b.pos.distance(*rpos);
                        if d < radius as f32 && b.machine_state.is_some()
                            && b.kind != types::BuildingKind::StorageChest
                            && b.kind != types::BuildingKind::Roboport
                        {
                            let ms = b.machine_state.as_ref().unwrap();
                            if ms.selected_recipe.is_some() && ms.input_buffer.len() < 4 {
                                Some(bid)
                            } else { None }
                        } else { None }
                    })
                })
                .collect();

            // For each needy machine, try to supply from player inventory (simulating bot delivery).
            for mid in nearby_ids {
                if let Some(machine) = state.buildings.get(mid) {
                    if let Some(ms) = &machine.machine_state {
                        if let Some(rid) = ms.selected_recipe {
                            if rid.0 < recipe::RECIPES.len() {
                                let recipe_inputs = recipe::RECIPES[rid.0].inputs;
                                // Deliver one unit of each needed input from inventory.
                                for &(res, _count) in recipe_inputs {
                                    let have = state.inventory.get(&res).copied().unwrap_or(0);
                                    if have > 0 {
                                        let m = state.buildings.get_mut(mid).unwrap();
                                        let ms = m.machine_state.as_mut().unwrap();
                                        if ms.input_buffer.len() < 8 {
                                            ms.input_buffer.push(res);
                                            *state.inventory.entry(res).or_insert(0) -= 1;
                                        }
                                        break; // one item per tick per machine
                                    }
                                }
                            }
                        }
                    }
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
            state.stats.buildings_placed,
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

        // Expand build zone with research milestones.
        let base_radius = 40.0f32;
        let bonus = state.research.completed.iter().filter(|&&c| c).count() as f32 * 3.0;
        state.build_radius = base_radius + bonus;
    }
}

/// Draws the screen-space UI overlay.
///
/// Layout:
/// - **Top-left**: Game title, tick count, direction indicator
/// - **Top-right**: Hovered tile info panel (dark background)
/// - **Bottom**: Categorized toolbar with sprite icons + labels
/// - **Bottom-right**: Controls hint (fades out at low zoom)
/// Unified panel helper — consistent background, border, title, close button.
/// Returns content origin (x, y) with 8px internal padding.
fn draw_panel(x: f32, y: f32, w: f32, h: f32, title: Option<&str>, closeable: bool) -> (f32, f32) {
    let bg = Color::new(0.08, 0.08, 0.12, 0.92);
    let border = Color::new(0.25, 0.30, 0.40, 0.50);
    let title_color = Color::new(0.95, 0.82, 0.35, 1.0);

    draw_rectangle(x, y, w, h, bg);
    draw_rectangle_lines(x, y, w, h, 1.0, border);

    let mut content_y = y + 8.0;
    if let Some(t) = title {
        draw_text(t, x + 12.0, y + 22.0, 16.0, title_color);
        content_y = y + 30.0;
    }
    if closeable {
        draw_close_button(x, y, w);
    }
    (x + 8.0, content_y)
}

/// Draws a clickable X close button at the top-right of a panel.
fn draw_close_button(px: f32, py: f32, pw: f32) {
    let bx = px + pw - 28.0;
    let by = py + 4.0;
    let hover = mouse_position().0 >= bx && mouse_position().0 <= bx + 24.0
        && mouse_position().1 >= by && mouse_position().1 <= by + 20.0;
    let bg = if hover {
        Color::new(0.7, 0.2, 0.2, 0.8)
    } else {
        Color::new(0.4, 0.2, 0.2, 0.6)
    };
    draw_rectangle(bx, by, 24.0, 20.0, bg);
    draw_text("X", bx + 7.0, by + 15.0, 18.0, Color::new(1.0, 1.0, 1.0, 0.9));
}

/// Short display name for a resource (for compact recipe display).
fn short_resource_name(r: types::Resource) -> &'static str {
    match r {
        types::Resource::IronOre => "Iron",
        types::Resource::CopperOre => "Copper",
        types::Resource::Coal => "Coal",
        types::Resource::Stone => "Stone",
        types::Resource::IronPlate => "Fe",
        types::Resource::CopperPlate => "Cu",
        types::Resource::SteelPlate => "Steel",
        types::Resource::StoneBrick => "Brick",
        types::Resource::Gear => "Gear",
        types::Resource::Wire => "Wire",
        types::Resource::GreenCircuit => "GrnC",
        types::Resource::RedCircuit => "RedC",
        types::Resource::BlueCircuit => "BluC",
        types::Resource::Pipe => "Pipe",
        types::Resource::IronStick => "Stick",
        types::Resource::Sulfur => "Sulfur",
        types::Resource::Plastic => "Plstc",
        types::Resource::Battery => "Batt",
        types::Resource::EngineUnit => "Engine",
        types::Resource::ScienceRed => "RedSci",
        types::Resource::ScienceGreen => "GrnSci",
        types::Resource::ScienceBlue => "BluSci",
        types::Resource::BasicAmmo => "Ammo",
        types::Resource::PiercingAmmo => "PAmmo",
        types::Resource::Grenade => "Gren",
        types::Resource::Inserter => "Ins",
        types::Resource::Rail => "Rail",
        types::Resource::Concrete => "Conc",
        _ => "?",
    }
}

/// Format a number with comma separators (e.g., 12345 → "12,345").
fn fmt_num(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 { result.push(','); }
        result.push(c);
    }
    result
}

fn draw_ui(state: &mut GameState, atlas: &SpriteAtlas) {
    // Modern UI colors — clean, high contrast, readable.
    let panel_bg = Color::new(0.08, 0.08, 0.12, 0.92);
    let panel_border = Color::new(0.25, 0.25, 0.35, 0.6);
    let text_bright = Color::new(0.98, 0.98, 0.98, 1.0);
    let text_dim = Color::new(0.65, 0.65, 0.7, 1.0);
    let text_accent = Color::new(0.45, 0.92, 0.55, 1.0);
    let selected_bg = Color::new(0.15, 0.35, 0.55, 0.8);
    let selected_border = Color::new(0.45, 0.75, 1.0, 1.0);

    // --- Top-left: Status Panel (compact, 4 lines) ---
    // Map view indicator.
    if state.camera.map_view {
        let label = "MAP VIEW — Press M or Esc to return";
        let w = measure_text(label, None, 22, 1.0).width;
        draw_rectangle((screen_width() - w) * 0.5 - 12.0, 6.0, w + 24.0, 28.0, Color::new(0.1, 0.1, 0.15, 0.85));
        draw_text(label, (screen_width() - w) * 0.5, 26.0, 22.0, Color::new(0.9, 0.8, 0.3, 1.0));
    }

    let (cx, mut cy) = draw_panel(8.0, 8.0, 240.0, 112.0, Some("FORGE"), false);

    // Line 1: Time + FPS
    draw_text(
        &format!("{}:{:02} | FPS:{}", state.stats.total_ticks / 1200, (state.stats.total_ticks / 20) % 60, get_fps()),
        cx, cy + 4.0, 13.0, text_dim,
    );
    // Speed badge (highlighted when not 1x).
    if state.game_speed > 1 {
        let speed_text = format!("{}x", state.game_speed);
        let sw = measure_text(&speed_text, None, 14, 1.0).width;
        draw_rectangle(cx + 165.0, cy - 4.0, sw + 10.0, 16.0, Color::new(0.8, 0.6, 0.1, 0.8));
        draw_text(&speed_text, cx + 170.0, cy + 7.0, 14.0, Color::new(1.0, 1.0, 1.0, 1.0));
    }
    cy += 18.0;

    // Line 2: Power bar (visual, not just text)
    let bar_w = 140.0;
    let bar_h = 10.0;
    let power_fill = state.power.satisfaction;
    let power_color = if power_fill >= 0.9 { Color::new(0.3, 0.85, 0.3, 1.0) }
        else if power_fill >= 0.5 { Color::new(0.9, 0.8, 0.2, 1.0) }
        else { Color::new(0.9, 0.2, 0.2, 1.0) };
    draw_rectangle(cx, cy, bar_w, bar_h, Color::new(0.15, 0.15, 0.2, 0.8));
    draw_rectangle(cx, cy, bar_w * power_fill, bar_h, power_color);
    draw_text(&format!("{:.0}%", power_fill * 100.0), cx + bar_w + 4.0, cy + 9.0, 12.0, power_color);
    draw_text("Power", cx + bar_w + 30.0, cy + 9.0, 11.0, text_dim);
    cy += 16.0;

    // Line 3: Items crafted + production rate
    let items_per_min = if state.stats.total_ticks > 1200 {
        state.stats.items_crafted as f32 / (state.stats.total_ticks as f32 / 1200.0)
    } else { 0.0 };
    draw_text(
        &format!("Items: {} ({:.0}/min)", fmt_num(state.stats.items_crafted), items_per_min),
        cx, cy + 4.0, 12.0, text_dim,
    );
    cy += 16.0;

    // Line 4: Day/Night + Direction
    let dn_color = if state.daynight.is_day() { Color::new(0.9, 0.82, 0.3, 1.0) } else { Color::new(0.4, 0.4, 0.7, 1.0) };
    let dir_text = match state.placement_direction {
        types::Direction::North => "N", types::Direction::East => "E",
        types::Direction::South => "S", types::Direction::West => "W",
    };
    draw_text(&state.daynight.display(), cx, cy + 4.0, 12.0, dn_color);
    draw_text(&format!("Dir:{} | Kills:{}", dir_text, state.stats.enemies_killed), cx + 80.0, cy + 4.0, 12.0, text_dim);

    // Pause menu overlay (uses unified panel)
    if state.paused {
        let sw = screen_width();
        let sh = screen_height();
        let pw = 320.0;
        let ph = 310.0;
        let px = (sw - pw) * 0.5;
        let py = (sh - ph) * 0.5;

        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.05, 0.5));
        let (cx, mut cy) = draw_panel(px, py, pw, ph, Some("PAUSED"), false);

        let items = [
            ("Space", "Resume"),
            ("F5", "Save Game"),
            ("F9", "Load Game"),
            ("E", "Recipe Book"),
            ("Tab", "Research Tree"),
            ("N", "Achievements"),
            ("V", "Production Stats"),
            ("B", "Blueprint"),
            ("+/-", "Game Speed"),
            ("F1", "Help / Controls"),
            ("F2", "Mute Sound"),
        ];
        cy += 4.0;
        for (key, desc) in &items {
            draw_text(key, cx, cy, 14.0, Color::new(0.95, 0.82, 0.35, 0.9));
            draw_text(desc, cx + 50.0, cy, 14.0, Color::new(0.8, 0.8, 0.85, 0.9));
            cy += 24.0;
        }
    }

    // --- Victory screen overlay ---
    if state.game_won {
        let sw = screen_width();
        let sh = screen_height();
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.05, 0.65));

        let pw = 460.0f32.min(sw * 0.8);
        let ph = 360.0f32.min(sh * 0.75);
        let px = (sw - pw) * 0.5;
        let py = (sh - ph) * 0.5;
        let (cx, mut cy) = draw_panel(px, py, pw, ph, Some("CONSCIOUSNESS RESTORED"), false);

        // FORGE avatar from atlas.
        let avatar_size = 64.0;
        let avatar_x = px + pw * 0.5 - avatar_size * 0.5;
        let blink_frame = if (get_time() * 0.3).fract() > 0.92 { 1 } else { 0 };
        draw_texture_ex(
            &atlas.tex, avatar_x, cy, WHITE,
            DrawTextureParams {
                source: Some(atlas.r_forge_avatar[blink_frame]),
                dest_size: Some(Vec2::splat(avatar_size)),
                ..Default::default()
            },
        );
        cy += avatar_size + 8.0;

        // Epilogue text.
        let gold = Color::new(0.95, 0.82, 0.35, 1.0);
        let bright = Color::new(0.9, 0.9, 0.95, 1.0);
        let dim = Color::new(0.6, 0.6, 0.7, 0.9);

        draw_text("I found them. All 4,000 colonists. Alive. Safe.", cx, cy, 16.0, gold);
        cy += 22.0;
        draw_text("Thank you for helping me remember who I am. <3", cx, cy, 14.0, Color::new(1.0, 0.7, 0.85, 0.9));
        cy += 30.0;

        // Stats.
        let playtime_min = state.stats.total_ticks / 1200;
        let playtime_sec = (state.stats.total_ticks / 20) % 60;
        draw_text(&format!("Playtime:  {}:{:02}", playtime_min, playtime_sec), cx, cy, 14.0, bright);
        cy += 20.0;
        draw_text(&format!("Items Crafted:  {}", fmt_num(state.stats.items_crafted)), cx, cy, 14.0, bright);
        cy += 20.0;
        draw_text(&format!("Buildings Placed:  {}", state.stats.buildings_placed), cx, cy, 14.0, bright);
        cy += 20.0;
        draw_text(&format!("Enemies Defeated:  {}", state.stats.enemies_killed), cx, cy, 14.0, bright);
        cy += 30.0;

        draw_text("Press any key to continue playing~", cx, cy, 13.0, dim);

        // Dismiss on key press (after initial display).
        if state.stats.items_crafted > 50100 {
            // Only dismiss after a brief delay so the screen is actually seen.
            if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Escape)
                || is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left)
            {
                state.game_won = false; // Dismiss victory screen, keep playing.
            }
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
                lines.push((format!("Facing: {:?}  (R to rotate)", b.direction), text_dim));
                // Belt speed info.
                if b.kind.is_belt() {
                    let (tier, speed) = match b.kind {
                        types::BuildingKind::BeltYellow => ("Yellow", "1x"),
                        types::BuildingKind::BeltRed => ("Red", "2x"),
                        types::BuildingKind::BeltBlue => ("Blue", "3x"),
                        _ => ("", ""),
                    };
                    if !tier.is_empty() {
                        lines.push((format!("{} belt — {} speed (scroll to upgrade)", tier, speed),
                            Color::new(0.7, 0.7, 0.5, 0.8)));
                    }
                }
                if let Some(ref ms) = b.machine_state {
                    // Show recipe with inputs → outputs clearly.
                    if let Some(rid) = ms.selected_recipe {
                        if rid.0 < recipe::RECIPES.len() {
                            let r = &recipe::RECIPES[rid.0];
                            lines.push((
                                format!("Recipe: {}", r.name),
                                Color::new(0.9, 0.8, 0.4, 1.0),
                            ));
                            // Show what goes IN.
                            let inputs: String = r.inputs.iter()
                                .map(|(res, c)| format!("{}x {}", c, short_resource_name(*res)))
                                .collect::<Vec<_>>().join(" + ");
                            lines.push((format!("Needs: {}", inputs), Color::new(0.7, 0.8, 0.7, 0.9)));
                            // Show what comes OUT.
                            let outputs: String = r.outputs.iter()
                                .map(|(res, c)| format!("{}x {}", c, short_resource_name(*res)))
                                .collect::<Vec<_>>().join(" + ");
                            lines.push((format!("Makes: {}", outputs), Color::new(0.5, 0.9, 0.5, 0.9)));
                        }
                    } else if b.kind == types::BuildingKind::AssemblerT1
                        || b.kind == types::BuildingKind::AssemblerT2
                        || b.kind == types::BuildingKind::AssemblerT3
                        || b.kind == types::BuildingKind::ChemicalPlant
                    {
                        lines.push(("Click to set recipe!".to_string(), Color::new(0.9, 0.7, 0.3, 1.0)));
                    }
                    // Progress/status indicator (different for inserters vs machines).
                    if b.kind.is_inserter() {
                        if ms.progress_ticks > 0 {
                            lines.push(("Swinging...".to_string(), Color::new(0.4, 0.8, 1.0, 0.9)));
                        } else {
                            lines.push(("Ready".to_string(), Color::new(0.6, 0.6, 0.4, 0.8)));
                        }
                    } else if ms.progress_ticks > 0 && ms.total_ticks > 0 {
                        let pct = ((ms.total_ticks - ms.progress_ticks) as f32 / ms.total_ticks as f32 * 100.0) as u32;
                        lines.push((format!("Progress: {}%", pct), Color::new(0.4, 0.8, 1.0, 0.9)));
                    } else if ms.selected_recipe.is_some() && ms.progress_ticks == 0 {
                        lines.push(("Idle — waiting for inputs".to_string(), Color::new(0.6, 0.6, 0.4, 0.8)));
                    }
                    // Buffer contents — show item names for chests, counts for machines.
                    if !ms.input_buffer.is_empty() {
                        if b.kind == types::BuildingKind::StorageChest {
                            // Show first few item names.
                            let items: String = ms.input_buffer.iter().take(4)
                                .map(|r| short_resource_name(*r))
                                .collect::<Vec<_>>().join(", ");
                            let more = if ms.input_buffer.len() > 4 { format!(" +{}", ms.input_buffer.len() - 4) } else { String::new() };
                            lines.push((format!("Stored: {}{}", items, more), text_dim));
                        } else {
                            lines.push((format!("Input: {}/{}", ms.input_buffer.len(), MACHINE_BUFFER_CAP), text_dim));
                        }
                    }
                    if !ms.output_buffer.is_empty() {
                        lines.push((format!("Output: {}/{}", ms.output_buffer.len(), MACHINE_BUFFER_CAP), text_dim));
                    }
                    if ms.fuel_ticks > 0 {
                        lines.push((format!("Fuel: {:.1}s", ms.fuel_ticks as f32 / 20.0), text_dim));
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

        // Check for enemies near the cursor.
        let mouse_world = state.camera.screen_to_world(mouse_screen);
        for enemy in &state.enemies.list {
            if !enemy.alive { continue; }
            let dx = enemy.x - mouse_world.x;
            let dy = enemy.y - mouse_world.y;
            if dx * dx + dy * dy < (TILE_SIZE * TILE_SIZE) {
                let hp_pct = (enemy.hp / enemy.kind.max_hp() * 100.0) as u32;
                lines.push((format!("{:?} — HP: {}%", enemy.kind, hp_pct),
                    Color::new(1.0, 0.4, 0.3, 1.0)));
                lines.push((format!("Dmg: {:.0}  Spd: {:.1}", enemy.kind.damage(), enemy.kind.speed()),
                    Color::new(0.8, 0.5, 0.4, 0.8)));
                break; // only show one enemy
            }
        }

        // Only show panel if there's meaningful info (skip bare grass tiles).
        let has_info = tile.deposit.is_some() || tile.building.is_some() || !items_here.is_empty() || lines.len() > 2;
        let panel_h = 8.0 + lines.len() as f32 * 20.0 + 8.0;
        if has_info || lines.len() > 2 {
            let (tx, mut ty) = draw_panel(panel_x, 8.0, panel_w, panel_h, None, false);
            for (text, color) in &lines {
                draw_text(text, tx, ty + 4.0, 14.0, *color);
                ty += 20.0;
            }
        }
    }

    // --- Bottom: Toolbar (unified panel) ---
    let toolbar_h = 88.0;
    let toolbar_y = screen_height() - toolbar_h;
    draw_panel(0.0, toolbar_y, screen_width(), toolbar_h, None, false);

    // Toolbar items: (hotkey label, display name, kind, atlas source rect)
    let toolbar_items: Vec<(&str, &str, types::BuildingKind, Rect)> = vec![
        ("1", "Belt", types::BuildingKind::BeltYellow, atlas.r_belt_yellow[0]),
        ("2", "Miner", types::BuildingKind::Miner, atlas.r_miner[0]),
        ("3", "Furnace", types::BuildingKind::StoneFurnace, atlas.r_stone_furnace[0]),
        ("4", "Inserter", types::BuildingKind::InserterRegular, atlas.r_inserter),
        ("5", "Assembler", types::BuildingKind::AssemblerT1, atlas.r_assembler[0]),
        ("6", "Boiler", types::BuildingKind::Boiler, atlas.r_boiler),
        ("7", "Engine", types::BuildingKind::SteamEngine, atlas.r_steam_engine),
        ("8", "Lab", types::BuildingKind::Lab, atlas.r_lab[0]),
        ("9", "Chest", types::BuildingKind::StorageChest, atlas.r_chest),
        ("0", "Splitter", types::BuildingKind::Splitter, atlas.r_splitter),
        ("T", "Turret", types::BuildingKind::GunTurret, atlas.r_gun_turret),
        ("G", "Wall", types::BuildingKind::Wall, atlas.r_wall),
        ("U", "UG Belt", types::BuildingKind::UndergroundBeltYellow, atlas.r_underground_belt),
        ("C", "Chemical", types::BuildingKind::ChemicalPlant, atlas.r_chemical_plant),
        ("L", "Laser", types::BuildingKind::LaserTurret, atlas.r_laser_turret),
        ("P", "Solar", types::BuildingKind::SolarPanel, atlas.r_solar_panel),
    ];

    let slot_w = (screen_width() / toolbar_items.len() as f32).min(76.0);
    let slot_h = 66.0;
    let total_w = toolbar_items.len() as f32 * slot_w;
    let start_x = (screen_width() - total_w) * 0.5; // center the toolbar

    for (i, (hotkey, name, kind, src_rect)) in toolbar_items.iter().enumerate() {
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
            &atlas.tex,
            icon_x,
            icon_y,
            WHITE,
            DrawTextureParams {
                source: Some(*src_rect),
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

        // Hover tooltip: show cost preview when hovering an unselected slot.
        let (mx, my) = mouse_position();
        if !is_selected && mx >= x && mx < x + slot_w && my >= y && my < y + slot_h {
            draw_rectangle(x + 2.0, y, slot_w - 4.0, slot_h, Color::new(0.2, 0.2, 0.3, 0.3));
            let cost = buildcost::building_cost(*kind);
            if !cost.is_empty() {
                let cost_str: String = cost.iter()
                    .map(|(r, c)| format!("{}x{}", c, short_resource_name(*r)))
                    .collect::<Vec<_>>().join(" ");
                let can_afford = buildcost::can_afford(&state.inventory, *kind);
                let color = if can_afford {
                    Color::new(0.5, 0.9, 0.5, 0.9)
                } else {
                    Color::new(0.9, 0.4, 0.4, 0.9)
                };
                let tw = measure_text(&cost_str, None, 12, 1.0).width;
                let tx = x + (slot_w - tw) * 0.5;
                draw_rectangle(tx - 4.0, y - 18.0, tw + 8.0, 16.0, Color::new(0.05, 0.05, 0.08, 0.9));
                draw_text(&cost_str, tx, y - 6.0, 12.0, color);
            }
        }
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
        let _panel_bg = Color::new(0.06, 0.06, 0.08, 0.9);

        draw_panel(mm_x - 4.0, mm_y - 24.0, mm_size + 8.0, mm_size + 32.0, Some("Map"), false);

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
                    } else if tile.pollution > 0.15 {
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

        // Draw enemy nests as dark red diamonds on minimap.
        for nest_pos in &state.nests {
            let npx = (nest_pos.x - (cam_grid.x - half_range)) as f32 / (map_pixels * tiles_per_pixel) as f32 * mm_size;
            let npy = (nest_pos.y - (cam_grid.y - half_range)) as f32 / (map_pixels * tiles_per_pixel) as f32 * mm_size;
            if npx >= 0.0 && npx < mm_size && npy >= 0.0 && npy < mm_size {
                draw_circle(mm_x + npx, mm_y + npy, 3.0, Color::new(0.6, 0.1, 0.1, 0.8));
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

        // Click minimap to teleport camera.
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx >= mm_x && mx < mm_x + mm_size && my >= mm_y && my < mm_y + mm_size {
                let frac_x = (mx - mm_x) / mm_size;
                let frac_y = (my - mm_y) / mm_size;
                let target_gx = cam_grid.x - half_range + (frac_x * (map_pixels * tiles_per_pixel) as f32) as i32;
                let target_gy = cam_grid.y - half_range + (frac_y * (map_pixels * tiles_per_pixel) as f32) as i32;
                state.camera.target = grid::Grid::grid_to_world_center(types::GridPos::new(target_gx, target_gy));
            }
        }
    }

    // --- Toast notifications (center-top area, max 3 visible) ---
    if !state.toasts.is_empty() {
        let cx = screen_width() * 0.5;
        for (i, (msg, remaining)) in state.toasts.iter().take(3).enumerate() {
            let alpha = (*remaining as f32 / 20.0).min(1.0); // fade out last 20 ticks
            let y = 40.0 + i as f32 * 26.0;
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

    // --- Bottom-right: controls hint (hidden when any overlay is active) ---
    let any_overlay = state.paused || state.show_recipes || state.show_research
        || state.show_stats || state.show_achievements || state.show_help
        || state.recipe_picker.is_some();
    if !any_overlay {
        let help_x = screen_width() - 280.0;
        let help_y = toolbar_y - 120.0;
        let hint_color = Color::new(0.5, 0.5, 0.5, 0.6);
        let hints = [
            "WASD: Pan | Scroll: Zoom | M: Map",
            "LClick: Place | RClick: Remove (hold=drag)",
            "R: Rotate | Q: Copy | Ctrl+Z: Undo",
            "E: Recipes | Tab: Research | H: Tutorial",
            "Space: Pause | +/-: Speed | F2: Mute",
            "F5: Save | F9: Load | F1: Help",
        ];
        for (i, line) in hints.iter().enumerate() {
            draw_text(line, help_x, help_y + i as f32 * 18.0, 15.0, hint_color);
        }
    }

    // --- Tutorial overlay (unified panel) ---
    if state.show_tutorial && state.tutorial_step < 6 {
        let tut_w = 400.0;
        let tut_h = 80.0;
        let tut_x = (screen_width() - tut_w) * 0.5;
        let tut_y = 100.0;

        draw_panel(tut_x, tut_y, tut_w, tut_h, Some("Tutorial"), true);

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

    // --- Inventory (left side, below status, compact two-column) ---
    let mut inv_panel_bottom = 128.0; // default if inventory empty
    {
        let all_resources: &[(types::Resource, &str)] = &[
            (types::Resource::IronPlate, "Fe"), (types::Resource::CopperPlate, "Cu"),
            (types::Resource::Stone, "Stone"), (types::Resource::Coal, "Coal"),
            (types::Resource::StoneBrick, "Brick"), (types::Resource::SteelPlate, "Steel"),
            (types::Resource::Gear, "Gear"), (types::Resource::Wire, "Wire"),
            (types::Resource::GreenCircuit, "GrnC"), (types::Resource::RedCircuit, "RedC"),
            (types::Resource::BlueCircuit, "BluC"), (types::Resource::Pipe, "Pipe"),
            (types::Resource::Sulfur, "Sulf"), (types::Resource::Plastic, "Plst"),
            (types::Resource::Battery, "Batt"), (types::Resource::BasicAmmo, "Ammo"),
            (types::Resource::ScienceRed, "RSci"), (types::Resource::ScienceGreen, "GSci"),
        ];
        let show: Vec<(&types::Resource, &&str, u32)> = all_resources.iter()
            .filter_map(|(r, n)| {
                let c = state.inventory.get(r).copied().unwrap_or(0);
                if c > 0 { Some((r, n, c)) } else { None }
            }).collect();

        if !show.is_empty() {
            let rows = (show.len() + 1) / 2; // two columns
            let inv_h = 30.0 + rows.min(8) as f32 * 16.0;
            let (ix, mut iy) = draw_panel(8.0, 128.0, 200.0, inv_h, Some("Inventory"), false);
            inv_panel_bottom = 128.0 + inv_h;

            for chunk in show.chunks(2) {
                for (col, (_, name, count)) in chunk.iter().enumerate() {
                    let x = ix + col as f32 * 96.0;
                    draw_text(&format!("{}: {}", name, count), x, iy + 4.0, 12.0, text_bright);
                }
                iy += 16.0;
            }
        }
    }

    // --- Roadmap Goal Panel (below inventory, dynamically positioned) ---
    {
        let goal_x = 8.0;
        let goal_y = inv_panel_bottom + 8.0;

        // Find the next uncompleted milestone.
        let next = milestones::next_milestone(&state.milestones_completed);
        let panel_h = if next.is_some() { 115.0 } else { 70.0 };
        let (gx, _gy) = draw_panel(goal_x, goal_y, 210.0, panel_h, Some("Roadmap"), false);
        let gx = gx - 4.0;

        if let Some(idx) = next {
            let m = &milestones::MILESTONES[idx];
            let (pr, pg, pb) = m.phase.color();
            let phase_color = Color::new(pr, pg, pb, 0.9);

            // Phase label.
            draw_text(m.phase.label(), gx + 8.0, goal_y + 16.0, 11.0, phase_color);

            // Milestone name (gold).
            draw_text(m.name, gx + 8.0, goal_y + 34.0, 16.0, Color::new(0.95, 0.82, 0.35, 1.0));

            // Description.
            draw_text(m.description, gx + 8.0, goal_y + 52.0, 12.0, Color::new(0.85, 0.85, 0.95, 1.0));

            // Hint (how to do it).
            draw_text(m.hint, gx + 8.0, goal_y + 68.0, 10.0, Color::new(0.6, 0.7, 0.8, 0.8));

            // Reward preview.
            let reward_str: String = m.reward.iter()
                .map(|(r, c)| format!("{}x{}", c, short_resource_name(*r)))
                .collect::<Vec<_>>().join(" ");
            draw_text(&format!("Reward: {}", reward_str), gx + 8.0, goal_y + 84.0, 10.0,
                Color::new(0.5, 0.9, 0.5, 0.7));

            // Progress bar toward 50k items.
            let progress = (state.stats.items_crafted as f32 / 50000.0).min(1.0);
            let bar_x = gx + 8.0;
            let bar_y = goal_y + 92.0;
            let bar_w = 194.0;
            let bar_h = 6.0;
            draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.15, 0.15, 0.2, 0.8));
            let bar_color = if progress >= 0.9 { Color::new(0.3, 0.9, 0.3, 0.9) }
                else if progress >= 0.5 { Color::new(0.9, 0.8, 0.2, 0.9) }
                else { Color::new(0.4, 0.5, 0.7, 0.9) };
            draw_rectangle(bar_x, bar_y, bar_w * progress, bar_h, bar_color);
            let completed_count = state.milestones_completed.iter().filter(|&&c| c).count();
            draw_text(
                &format!("{}/{} milestones | {:.0}% to FORGE", completed_count, milestones::MILESTONES.len(), progress * 100.0),
                bar_x, bar_y + 16.0, 9.0, Color::new(0.5, 0.6, 0.7, 0.7));
        } else {
            // All milestones done!
            draw_text("All milestones complete!", gx + 8.0, goal_y + 20.0, 14.0, Color::new(0.5, 0.9, 0.5, 1.0));
            draw_text("FORGE consciousness restored <3", gx + 8.0, goal_y + 40.0, 12.0, Color::new(0.9, 0.7, 0.85, 0.9));
        }

    }

    // --- Recipe picker popup (click assembler to open) ---
    if let Some((bid, ref recipes)) = state.recipe_picker {
        let sw = screen_width();
        let sh = screen_height();
        let pw: f32 = 340.0;
        let ph: f32 = 50.0 + recipes.len() as f32 * 28.0;
        let capped_ph: f32 = ph.min(500.0);
        let px = (sw - pw) * 0.5;
        let py = (sh - capped_ph) * 0.5;

        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.4));
        draw_panel(px, py, pw, capped_ph, Some("Select Recipe"), true);
        draw_text("Click to select, Esc to cancel", px + 20.0, py + 40.0, 12.0,
            Color::new(0.6, 0.6, 0.65, 0.7));

        // Show current recipe (if any).
        if let Some(b) = state.buildings.get(bid) {
            if let Some(ref ms) = b.machine_state {
                if let Some(cur) = ms.selected_recipe {
                    if cur.0 < recipe::RECIPES.len() {
                        draw_text(&format!("Current: {}", recipe::RECIPES[cur.0].name),
                            px + 180.0, py + 25.0, 14.0, Color::new(0.5, 0.8, 0.5, 0.8));
                    }
                }
            }
        }

        // Recipe list with inputs → outputs.
        let mx = mouse_position().0;
        let my = mouse_position().1;
        for (i, rid) in recipes.iter().enumerate() {
            let ry = py + 55.0 + i as f32 * 28.0;
            if ry > py + capped_ph - 10.0 { break; }

            let r = &recipe::RECIPES[rid.0];

            // Hover highlight.
            if mx >= px + 10.0 && mx <= px + pw - 10.0 && my >= ry - 10.0 && my <= ry + 16.0 {
                draw_rectangle(px + 5.0, ry - 10.0, pw - 10.0, 26.0,
                    Color::new(0.2, 0.3, 0.5, 0.4));
            }

            // Recipe name + craftability indicator.
            let can_craft = r.inputs.iter().all(|(res, count)| {
                state.inventory.get(res).copied().unwrap_or(0) >= *count
            });
            let name_color = if can_craft {
                Color::new(0.5, 0.95, 0.5, 1.0) // green = you have all inputs
            } else {
                Color::new(0.9, 0.9, 0.95, 1.0) // white = can't craft yet
            };
            draw_text(r.name, px + 15.0, ry + 4.0, 15.0, name_color);

            // Inputs → Output with per-input availability coloring.
            let inputs: String = r.inputs.iter()
                .map(|(res, c)| {
                    let have = state.inventory.get(res).copied().unwrap_or(0);
                    let sym = if have >= *c { "+" } else { "-" };
                    format!("{}{}x{}", sym, c, short_resource_name(*res))
                })
                .collect::<Vec<_>>().join(" ");
            let outputs: String = r.outputs.iter()
                .map(|(res, c)| format!("{}x{}", c, short_resource_name(*res)))
                .collect::<Vec<_>>().join("+");
            let flow = format!("{} -> {}", inputs, outputs);
            draw_text(&flow, px + 180.0, ry + 4.0, 12.0, Color::new(0.6, 0.7, 0.6, 0.8));
        }
    }

    // --- Production Stats screen (V key) ---
    if state.show_stats {
        let sw = screen_width();
        let sh = screen_height();
        let pw = (sw * 0.5).min(480.0);
        let ph = (sh * 0.6).min(400.0);
        let px = (sw - pw) * 0.5;
        let py = (sh - ph) * 0.5;

        let (sx, mut sy) = draw_panel(px, py, pw, ph, Some("Production Stats"), true);

        // General stats.
        let playtime_min = state.stats.total_ticks / 1200;
        let playtime_sec = (state.stats.total_ticks / 20) % 60;
        let items_per_min = if state.stats.total_ticks > 0 {
            state.stats.items_crafted as f32 / (state.stats.total_ticks as f32 / 1200.0)
        } else { 0.0 };

        draw_text(&format!("Playtime: {}:{:02}", playtime_min, playtime_sec), sx, sy, 14.0, text_bright);
        sy += 20.0;
        draw_text(&format!("Items crafted: {}", state.stats.items_crafted), sx, sy, 14.0, text_bright);
        sy += 20.0;
        draw_text(&format!("Production rate: {:.1}/min", items_per_min), sx, sy, 14.0, text_accent);
        sy += 20.0;
        draw_text(&format!("Buildings placed: {}", state.stats.buildings_placed), sx, sy, 14.0, text_bright);
        sy += 20.0;
        draw_text(&format!("Enemies killed: {}", state.stats.enemies_killed), sx, sy, 14.0, text_bright);
        sy += 20.0;
        draw_text(&format!("Rockets launched: {}", state.stats.rockets_launched), sx, sy, 14.0, text_bright);
        sy += 24.0;

        // Building count by type.
        draw_text("Building Counts:", sx, sy, 14.0, Color::new(0.95, 0.82, 0.35, 0.9));
        sy += 18.0;
        let mut counts: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
        for (_, b) in state.buildings.iter() {
            *counts.entry(b.kind.display_name()).or_insert(0) += 1;
        }
        let mut sorted: Vec<(&&str, &u32)> = counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (name, count) in sorted.iter().take(10) {
            draw_text(&format!("{}: {}", name, count), sx, sy, 13.0, text_dim);
            sy += 16.0;
        }
    }

    // --- Achievements screen (N key) ---
    if state.show_achievements {
        let sw = screen_width();
        let sh = screen_height();
        let pw = (sw * 0.6).min(560.0);
        let ph = (sh * 0.8).min(600.0);
        let px = (sw - pw) * 0.5;
        let py = (sh - ph) * 0.5;

        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.4));
        let (ax, mut ay) = draw_panel(px, py, pw, ph, Some("Achievement Roadmap — N to close"), true);

        let completed = state.milestones_completed.iter().filter(|&&c| c).count();
        let total = milestones::MILESTONES.len();
        draw_text(
            &format!("{}/{} completed", completed, total),
            ax + 200.0, ay - 6.0, 13.0, text_dim,
        );
        ay += 4.0;

        let mut last_phase = None;
        for (i, milestone) in milestones::MILESTONES.iter().enumerate() {
            if ay > py + ph - 16.0 { break; }

            // Phase header.
            if last_phase != Some(milestone.phase) {
                last_phase = Some(milestone.phase);
                let (pr, pg, pb) = milestone.phase.color();
                ay += 4.0;
                draw_text(milestone.phase.label(), ax, ay, 14.0, Color::new(pr, pg, pb, 0.9));
                ay += 18.0;
            }

            let done = state.milestones_completed.get(i).copied().unwrap_or(false);
            let is_next = milestones::next_milestone(&state.milestones_completed) == Some(i);

            // Highlight the next goal.
            if is_next {
                draw_rectangle(ax - 4.0, ay - 12.0, pw - 16.0, 36.0, Color::new(0.15, 0.2, 0.3, 0.5));
                draw_text(">>>", ax - 2.0, ay + 4.0, 12.0, Color::new(0.9, 0.8, 0.3, 0.9));
            }

            let icon = if done { "[X]" } else { "[ ]" };
            let name_color = if done { text_accent }
                else if is_next { Color::new(0.95, 0.85, 0.35, 1.0) }
                else { text_dim };
            draw_text(&format!("{} {}", icon, milestone.name), ax + 18.0, ay, 14.0, name_color);
            draw_text(milestone.description, ax + 180.0, ay, 11.0, text_dim);

            // Show reward for uncompleted milestones.
            if !done {
                let reward_str: String = milestone.reward.iter()
                    .map(|(r, c)| format!("{}x{}", c, short_resource_name(*r)))
                    .collect::<Vec<_>>().join(" ");
                draw_text(&format!("Reward: {}", reward_str), ax + 180.0, ay + 14.0, 10.0,
                    Color::new(0.4, 0.8, 0.4, 0.6));
            }

            ay += if is_next { 38.0 } else { 24.0 };
        }
    }

    // --- Help overlay (F1) ---
    if state.show_help {
        let sw = screen_width();
        let sh = screen_height();
        let pw = (sw * 0.65).min(620.0);
        let ph = (sh * 0.85).min(650.0);
        let px = (sw - pw) * 0.5;
        let py = (sh - ph) * 0.5;

        draw_panel(px, py, pw, ph, Some("Help — F1 to close"), true);
        let help = [
            ("BUILDING", ""),
            ("1-9, 0", "Select building from toolbar"),
            ("T/G/U/C/L/P", "Turret/Wall/UGBelt/Chem/Laser/Solar"),
            ("Left Click", "Place (hold to drag belts)"),
            ("Right Click", "Remove (hold to mass-delete)"),
            ("R", "Rotate direction"),
            ("Q", "Copy building from world (eyedropper)"),
            ("Ctrl+Z", "Undo last placement (20 levels)"),
            ("B", "Blueprint copy/paste"),
            ("Esc", "Deselect building"),
            ("", ""),
            ("INTERACTION", ""),
            ("Click assembler", "Open recipe picker"),
            ("Middle Click", "Hand-insert item into machine"),
            ("Click ship", "Read ship lore messages"),
            ("Click minimap", "Teleport camera"),
            ("", ""),
            ("NAVIGATION", ""),
            ("WASD / Arrows", "Pan camera"),
            ("Scroll wheel", "Zoom (toward cursor)"),
            ("Home", "Center camera on base"),
            ("M", "Map overview (zoom out)"),
            ("", ""),
            ("MENUS & SYSTEM", ""),
            ("E", "Recipe book"),
            ("Tab", "Research tree"),
            ("N", "Achievements"),
            ("V", "Production stats"),
            ("H", "Tutorial"),
            ("Space", "Pause"),
            ("+/-", "Game speed (1x–5x)"),
            ("F1", "This help screen"),
            ("F2", "Mute/unmute sound"),
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
    let sw = screen_width();
    let sh = screen_height();

    // Darken background for modal consistency.
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.4));

    let pw = (sw * 0.75).min(800.0);
    let ph = (sh * 0.85).min(700.0);
    let px = (sw - pw) * 0.5;
    let py = (sh - ph) * 0.5;

    draw_panel(px, py, pw, ph, Some("Recipe Book — E to close"), true);
    draw_text(
        "What goes in, what comes out",
        px + 20.0,
        py + 48.0,
        13.0,
        Color::new(0.6, 0.6, 0.7, 0.7),
    );

    // Dynamically generated from the actual RECIPES array — always complete.
    let col_name = px + 20.0;
    let col_input = px + 180.0;
    let col_output = px + pw - 180.0;
    let start_y = py + 75.0;
    let row_h = 18.0;

    // Column headers.
    draw_text("Recipe", col_name, start_y - 5.0, 14.0, Color::new(0.7, 0.7, 0.8, 0.7));
    draw_text("Inputs", col_input, start_y - 5.0, 14.0, Color::new(0.7, 0.7, 0.8, 0.7));
    draw_text("Output", col_output, start_y - 5.0, 14.0, Color::new(0.7, 0.7, 0.8, 0.7));

    for (i, r) in recipe::RECIPES.iter().enumerate() {
        let y = start_y + 10.0 + i as f32 * row_h;
        if y > py + ph - 20.0 {
            break;
        }

        // Recipe name.
        draw_text(r.name, col_name, y, 13.0, Color::new(0.9, 0.9, 0.95, 1.0));

        // Inputs.
        let inputs: String = r.inputs.iter()
            .map(|(res, c)| format!("{}x{}", c, short_resource_name(*res)))
            .collect::<Vec<_>>().join("+");
        draw_text(&inputs, col_input, y, 12.0, Color::new(0.7, 0.8, 0.7, 0.9));

        // Outputs.
        let outputs: String = r.outputs.iter()
            .map(|(res, c)| format!("{}x{}", c, short_resource_name(*res)))
            .collect::<Vec<_>>().join("+");
        draw_text(&outputs, col_output, y, 12.0, Color::new(0.5, 0.9, 0.5, 0.9));
    }
}

/// Draws the research screen overlay.
fn draw_research_screen(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();

    // Darken background for modal consistency.
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.4));

    let pw = (sw * 0.7).min(700.0);
    let ph = (sh * 0.8).min(600.0);
    let px = (sw - pw) * 0.5;
    let py = (sh - ph) * 0.5;

    draw_panel(px, py, pw, ph, Some("Research — Tab to close"), true);
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

    // Tech list with prerequisite lines.
    let start_y = py + 110.0;
    let row_h = 28.0;
    let col1 = px + 20.0;
    let col2 = px + 220.0;

    // Draw prerequisite connection lines FIRST (behind text).
    for (i, tech) in research::TECHNOLOGIES.iter().enumerate() {
        let y = start_y + i as f32 * row_h;
        if y > py + ph - 20.0 { break; }
        for &prereq in tech.prerequisites {
            let prereq_y = start_y + prereq as f32 * row_h;
            let line_color = if state.research.completed[prereq] {
                Color::new(0.3, 0.6, 0.3, 0.4) // green = satisfied
            } else {
                Color::new(0.5, 0.2, 0.2, 0.3) // red = unsatisfied
            };
            draw_line(col1 - 5.0, prereq_y, col1 - 5.0, y, 1.5, line_color);
            draw_line(col1 - 5.0, y, col1, y, 1.5, line_color);
        }
    }

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
