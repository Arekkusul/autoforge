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

// These lints fire on pre-existing rendering/serialization signatures where the
// tuple shapes and argument lists are inherent to the drawing/save API and
// factoring them out would not improve clarity.
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

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
mod input;
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
mod simulation;
mod sound;
mod splitter;
#[allow(dead_code, non_snake_case)]
mod sprites;
#[allow(dead_code)]
mod story;
#[allow(dead_code)]
mod train;
#[allow(dead_code)]
mod types;
mod ui;

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
    // --- CLI flags ---
    // `--export-state` dumps a fresh game state as JSON to stdout and exits,
    // without entering the render loop. Useful for debugging deterministic
    // logic (grid/inventory/research shape) without a full play session.
    if std::env::args().any(|arg| arg == "--export-state") {
        let state = GameState::new(0);
        match save::export_state_json(&state) {
            Ok(json) => println!("{json}"),
            Err(e) => eprintln!("Failed to export game state: {e}"),
        }
        return;
    }

    // `--export-state-json [PATH]` writes a fresh game state snapshot to a JSON
    // file (defaulting to `autoforge_state_export.json` in the working directory)
    // and exits, without entering the render loop. Unlike `--export-state` (which
    // prints to stdout), this leaves a structured file on disk for tooling/tests
    // to inspect.
    let args: Vec<String> = std::env::args().collect();
    if let Some(idx) = args.iter().position(|arg| arg == "--export-state-json") {
        let path = args
            .get(idx + 1)
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("autoforge_state_export.json"));
        let state = GameState::new(0);
        match save::export_state_to_file(&state, &path) {
            Ok(()) => println!("Exported game state to {}", path.display()),
            Err(e) => eprintln!("Failed to export game state to {}: {e}", path.display()),
        }
        return;
    }

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
    state.toast(
        "Welcome to AutoForge! Press H for tutorial, F1 for help~".to_string(),
        120,
    );
    state.toast(
        "Starting supplies: 50 Iron, 30 Copper, 20 Stone, 30 Coal, 25 Gears".to_string(),
        150,
    );
    // Random tip of the day.
    let tips = [
        "Tip: Scroll wheel on a selected building to cycle tiers!",
        "Tip: Press Q on a building to copy its type (eyedropper)!",
        "Tip: Inserters grab from BEHIND and deliver FORWARD (R to rotate)!",
        "Tip: Click the crashed ship for lore messages~",
        "Tip: Press M for a map overview of your whole base!",
        "Tip: Furnaces need coal AND ore — use 2 inserters!",
        "Tip: Storage Chests auto-feed your building inventory!",
        "Tip: Press N to see your achievement roadmap!",
    ];
    let tip_idx = (macroquad::miniquad::date::now() * 1000.0) as usize % tips.len();
    state.toast(tips[tip_idx].to_string(), 200);

    // Start with a zoom-in animation from the cutscene.
    state.camera.zoom = 0.3;
    let mut startup_zoom_timer = 0.0f32;
    sfx.start_ambient();

    // --- Main game loop ---
    let mut autosave_timer = 0.0f32;

    loop {
        let dt = get_frame_time() as f64;

        // Startup zoom-in animation (0.3 → 1.0 over ~1 second).
        if startup_zoom_timer < 1.0 {
            startup_zoom_timer += dt as f32;
            let t = (startup_zoom_timer / 1.0).min(1.0);
            let ease = t * t * (3.0 - 2.0 * t); // smoothstep
            state.camera.zoom = 0.3 + ease * 0.7;
        }

        // Auto-save every 5 minutes (pauses when game is paused).
        if !state.paused {
            autosave_timer += dt as f32;
        }
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
            let over_minimap =
                mx > screen_width() - 160.0 && my > screen_height() - 380.0 && my < toolbar_y;
            let any_overlay = state.paused
                || state.show_recipes
                || state.show_research
                || state.show_stats
                || state.show_achievements
                || state.show_help
                || state.recipe_picker.is_some();

            if !over_toolbar && !over_status && !over_minimap && !any_overlay {
                if mx < edge_margin {
                    state.camera.target.x -= edge_speed;
                }
                if mx > screen_width() - edge_margin {
                    state.camera.target.x += edge_speed;
                }
                if my < edge_margin {
                    state.camera.target.y -= edge_speed;
                }
                if my > screen_height() - edge_margin {
                    state.camera.target.y += edge_speed;
                }
            }
        }

        // 1. Input (every frame, independent of simulation tick rate).
        input::handle_input(&mut state, &mut sfx);
        state.camera.update(get_frame_time());

        // 2. Fixed-timestep simulation (with game speed multiplier).
        // Capped at 5 ticks/frame to prevent stutters during lag spikes.
        if !state.paused {
            state.tick_accumulator += dt * state.game_speed as f64;
            if state.tick_accumulator > MAX_ACCUMULATOR {
                state.tick_accumulator = MAX_ACCUMULATOR;
            }
            let mut ticks_this_frame = 0u32;
            while state.tick_accumulator >= TICK_DURATION && ticks_this_frame < 5 {
                simulation::simulation_tick(&mut state, &sfx);
                state.tick_accumulator -= TICK_DURATION;
                ticks_this_frame += 1;
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
                zoom: vec2(
                    map_zoom * 2.0 / screen_width(),
                    map_zoom * 2.0 / screen_height(),
                ),
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
            draw_rectangle_lines(
                vis_min.x,
                vis_min.y,
                vis_max.x - vis_min.x,
                vis_max.y - vis_min.y,
                4.0,
                WHITE,
            );
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

        // Render combat FX (turret laser/bullet lines).
        for &(fx, fy, tx, ty, ticks, r, g, b) in &state.combat_fx {
            let alpha = ticks as f32 / 4.0;
            draw_line(fx, fy, tx, ty, 2.0, Color::new(r, g, b, alpha * 0.8));
            // Impact flash at target.
            draw_circle(tx, ty, 3.0, Color::new(1.0, 1.0, 1.0, alpha * 0.5));
        }

        // Render robot workers (pixel art sprites with trail).
        let robot_anim = ((state.stats.total_ticks / 3) % 2) as usize; // propeller animation
        for (start, target, progress) in &state.robots {
            let pos = *start + (*target - *start) * *progress;
            let robot_size = TILE_SIZE * 0.5;
            // Thruster trail (fading dots behind the robot).
            for t in 1..4 {
                let trail = *start + (*target - *start) * (*progress - t as f32 * 0.03).max(0.0);
                let alpha = 0.4 - t as f32 * 0.1;
                draw_circle(
                    trail.x,
                    trail.y,
                    2.0 - t as f32 * 0.4,
                    Color::new(0.5, 0.3, 0.8, alpha),
                );
            }
            // Robot sprite.
            draw_texture_ex(
                &atlas.tex,
                pos.x - robot_size * 0.5,
                pos.y - robot_size * 0.5,
                WHITE,
                DrawTextureParams {
                    source: Some(atlas.r_robot[robot_anim]),
                    dest_size: Some(Vec2::splat(robot_size)),
                    ..Default::default()
                },
            );
        }

        // Render trains (rectangles moving along their routes).
        for train in &state.trains.list {
            if !train.alive {
                continue;
            }
            let size = TILE_SIZE * 0.8;
            let tx = train.x - size * 0.5;
            let ty = train.y - size * 0.3;
            // Train body (dark rectangle with colored top).
            draw_rectangle(tx, ty, size, size * 0.6, Color::new(0.15, 0.15, 0.2, 0.9));
            draw_rectangle(
                tx + 2.0,
                ty + 2.0,
                size - 4.0,
                size * 0.3,
                Color::new(0.3, 0.5, 0.8, 0.9),
            );
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
        let enemies_attacking = state
            .enemies
            .list
            .iter()
            .any(|e| e.alive && e.attack_cooldown > 15);
        if enemies_attacking {
            let flash = (state.stats.total_ticks as f32 * 0.2).sin() * 0.08 + 0.05;
            draw_rectangle(
                -100000.0,
                -100000.0,
                200000.0,
                200000.0,
                Color::new(0.8, 0.0, 0.0, flash),
            );
        }

        // Range circle for hovered turrets/roboports (drawn in world space).
        if !state.camera.map_view && state.selected_building.is_none() {
            let hover_screen = Vec2::new(mouse_position().0, mouse_position().1);
            let hover_world = state.camera.screen_to_world(hover_screen);
            let hover_grid = grid::Grid::world_to_grid(hover_world);
            if let Some(tile) = state.grid.get_tile(hover_grid) {
                if let Some(bid) = tile.building {
                    if let Some(b) = state.buildings.get(bid) {
                        let center = grid::Grid::grid_to_world_center(b.pos);
                        match b.kind {
                            types::BuildingKind::GunTurret | types::BuildingKind::LaserTurret => {
                                draw_circle_lines(
                                    center.x,
                                    center.y,
                                    TILE_SIZE * 6.0,
                                    1.5,
                                    Color::new(1.0, 0.3, 0.3, 0.2),
                                );
                            }
                            types::BuildingKind::Roboport => {
                                draw_circle_lines(
                                    center.x,
                                    center.y,
                                    TILE_SIZE * 10.0,
                                    1.5,
                                    Color::new(0.3, 0.5, 1.0, 0.2),
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Screen-space UI overlay.
        set_default_camera();
        ui::draw_ui(&mut state, &atlas);

        // Performance warning when FPS drops (one-time toast).
        if get_fps() < 30
            && state.stats.total_ticks > 200
            && !state
                .notification_log
                .iter()
                .any(|m| m.contains("Performance mode"))
        {
            state.toast(
                "Performance mode enabled — zoom in for full detail".to_string(),
                120,
            );
        }

        next_frame().await;
    }
}
