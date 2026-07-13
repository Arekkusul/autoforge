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
                simulation_tick(&mut state, &sfx);
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
fn simulation_tick(state: &mut GameState, sfx: &sound::SoundEffects) {
    state.stats.total_ticks += 1;
    let tick = state.stats.total_ticks;

    // Day/night cycle advances every tick for smooth transitions.
    let was_day = state.daynight.is_day();
    state.daynight.tick();
    if was_day && !state.daynight.is_day() {
        state.toast("Night is falling... solar power reduces.".to_string(), 80);
    } else if !was_day && state.daynight.is_day() {
        state.toast("Dawn breaks! Solar panels at full power.".to_string(), 80);
    }

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

    // Tick combat FX (fade out and remove expired).
    for fx in state.combat_fx.iter_mut() {
        if fx.4 > 0 {
            fx.4 -= 1;
        }
    }
    state.combat_fx.retain(|fx| fx.4 > 0);

    // --- EVERY TICK (20 Hz) --- Critical path for smooth gameplay ---

    // 1. Machines process: count down timers, complete recipes, start new ones.
    let crafted_before = state.stats.items_crafted;
    let mining_bonus = state.research.mining_bonus();
    machine::tick_machines(
        &mut state.grid,
        &mut state.buildings,
        &mut state.items,
        &mut state.stats,
        state.power.satisfaction,
        mining_bonus,
    );
    // First item celebration + periodic ding.
    if crafted_before == 0 && state.stats.items_crafted > 0 {
        sfx.play(&sfx.research_done);
        state.toast("Your first item! You're on your way~".to_string(), 100);
    } else if state.stats.items_crafted / 50 > crafted_before / 50 {
        sfx.play(&sfx.recipe_done);
    }
    // Prune old production log entries (keep last 60 seconds = 1200 ticks).
    if tick.is_multiple_of(100) {
        let cutoff = tick.saturating_sub(1200);
        state.stats.production_log.retain(|&(_, t)| t >= cutoff);
    }

    // Check for depleted miners using the alert system.
    if tick.is_multiple_of(200) && tick >= 200 {
        let depleted_count = state
            .buildings
            .iter()
            .filter(|(_, b)| b.kind == types::BuildingKind::Miner)
            .filter(|(_, b)| {
                state
                    .grid
                    .get_tile(b.pos)
                    .map(|t| t.deposit.is_none())
                    .unwrap_or(false)
            })
            .count();
        if depleted_count > 0 {
            state.alert(
                types::AlertKind::MinerDepleted,
                format!(
                    "{} miner(s) on depleted ore — relocate them!",
                    depleted_count
                ),
                120,
                types::AlertSeverity::Warning,
            );
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

    if tick.is_multiple_of(2) {
        // 5. Labs: consume science packs, advance research.
        let techs_before: Vec<bool> = state.research.completed.clone();
        research::tick_labs(&mut state.buildings, &mut state.research);
        // Check for newly completed research.
        let newly_completed: Vec<usize> = techs_before
            .iter()
            .zip(state.research.completed.iter())
            .enumerate()
            .filter(|(_, (&was, &now))| !was && now)
            .map(|(i, _)| i)
            .collect();
        for i in newly_completed {
            if i < research::TECHNOLOGIES.len() {
                sfx.play(&sfx.research_done);
                state.toast(
                    format!("Research complete: {}!", research::TECHNOLOGIES[i].name),
                    120,
                );
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
            // Drain the chest buffer, then fold into inventory via the shared API.
            let drained: Vec<types::Resource> = match state.buildings.get_mut(bid) {
                Some(building) => match &mut building.machine_state {
                    Some(ms) => ms.input_buffer.drain(..).collect(),
                    None => Vec::new(),
                },
                None => Vec::new(),
            };
            for resource in drained {
                state.add_to_inventory(resource, 1);
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
            // Check if any walls/turrets were lost (defense breach).
            let wall_count_after = state
                .buildings
                .iter()
                .filter(|(_, b)| {
                    b.kind == types::BuildingKind::Wall
                        || b.kind == types::BuildingKind::Gate
                        || b.kind == types::BuildingKind::GunTurret
                        || b.kind == types::BuildingKind::LaserTurret
                })
                .count();
            let had_defenses = buildings_before > buildings_after; // any building lost
            if wall_count_after == 0 && had_defenses {
                state.toast(
                    "!! DEFENSES BREACHED — Factory under attack! !!".to_string(),
                    120,
                );
                sfx.play(&sfx.wave_warning);
            } else {
                state.toast(format!("Building destroyed! ({} lost)", lost), 60);
            }
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
        // Generate combat visual FX: laser/bullet lines from turrets to nearest enemies.
        for (_bid, building) in state.buildings.iter() {
            if building.kind != types::BuildingKind::GunTurret
                && building.kind != types::BuildingKind::LaserTurret
            {
                continue;
            }
            if let Some(ref ms) = building.machine_state {
                // progress_ticks == COOLDOWN means turret JUST fired this tick.
                if ms.progress_ticks == 10 {
                    // just fired (cooldown = 10 ticks)
                    let bx = building.pos.x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
                    let by = building.pos.y as f32 * TILE_SIZE + TILE_SIZE * 0.5;
                    // Find closest alive enemy for the line target.
                    let mut closest = None;
                    let mut closest_dist = f32::MAX;
                    for enemy in &state.enemies.list {
                        if !enemy.alive {
                            continue;
                        }
                        let dx = enemy.x - bx;
                        let dy = enemy.y - by;
                        let d = dx * dx + dy * dy;
                        if d < closest_dist {
                            closest_dist = d;
                            closest = Some((enemy.x, enemy.y));
                        }
                    }
                    if let Some((tx, ty)) = closest {
                        let (r, g, b) = if building.kind == types::BuildingKind::LaserTurret {
                            (0.3, 0.5, 1.0) // blue laser
                        } else {
                            (1.0, 0.8, 0.2) // yellow bullet
                        };
                        state.combat_fx.push((bx, by, tx, ty, 4, r, g, b));
                    }
                }
            }
        }

        // Loot drops + sounds.
        let new_kills = state.stats.enemies_killed - kills_before;
        if new_kills > 0 {
            sfx.play(&sfx.turret_fire);
            sfx.play(&sfx.enemy_death);
            let n = new_kills as u32;
            *state
                .inventory
                .entry(types::Resource::IronPlate)
                .or_insert(0) += n * 2;
            *state.inventory.entry(types::Resource::Coal).or_insert(0) += n;
            // Higher evolution = rarer drops.
            if state.evolution > 0.3 {
                *state
                    .inventory
                    .entry(types::Resource::CopperPlate)
                    .or_insert(0) += n;
            }
            if state.evolution > 0.5 {
                *state.inventory.entry(types::Resource::Gear).or_insert(0) += n;
            }
            if state.evolution > 0.7 {
                *state
                    .inventory
                    .entry(types::Resource::GreenCircuit)
                    .or_insert(0) += n;
            }
            if state.evolution > 0.9 {
                *state
                    .inventory
                    .entry(types::Resource::SteelPlate)
                    .or_insert(0) += n;
            }
        }
    }

    // --- EVERY 10 TICKS: Passive building regeneration (walls/turrets heal 1 HP) ---
    if tick.is_multiple_of(10) {
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
    if tick.is_multiple_of(20) {
        // Find all roboports, then for each, scan nearby machines that need inputs.
        let roboport_ids: Vec<(types::BuildingId, types::GridPos)> = state
            .buildings
            .alive_ids()
            .iter()
            .filter_map(|&bid| {
                state.buildings.get(bid).and_then(|b| {
                    if b.kind == types::BuildingKind::Roboport {
                        Some((bid, b.pos))
                    } else {
                        None
                    }
                })
            })
            .collect();

        for (_rbid, rpos) in &roboport_ids {
            let radius = 10i32;
            // Find machines in range that have a recipe set and need inputs.
            let nearby_ids: Vec<types::BuildingId> = state
                .buildings
                .alive_ids()
                .iter()
                .filter_map(|&bid| {
                    state.buildings.get(bid).and_then(|b| {
                        let d = b.pos.distance(*rpos);
                        if d < radius as f32
                            && b.kind != types::BuildingKind::StorageChest
                            && b.kind != types::BuildingKind::Roboport
                        {
                            match &b.machine_state {
                                Some(ms)
                                    if ms.selected_recipe.is_some()
                                        && ms.input_buffer.len() < 4 =>
                                {
                                    Some(bid)
                                }
                                _ => None,
                            }
                        } else {
                            None
                        }
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
                                    if state.inventory_count(res) > 0 {
                                        let m = state.buildings.get_mut(mid).unwrap();
                                        let ms = m.machine_state.as_mut().unwrap();
                                        if ms.input_buffer.len() < 8 {
                                            ms.input_buffer.push(res);
                                            state.remove_from_inventory(res, 1);
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

    if tick.is_multiple_of(5) {
        // 6. Power: calculate supply/demand, consume fuel in boilers/engines.
        power::update_power(&mut state.buildings, &mut state.power, &state.daynight);

        // 6b. Check for alert conditions (power, turret ammo, etc.).
        input::check_alerts(state, sfx);
    }

    // --- EVERY 10 TICKS (2 Hz) --- Expensive systems that change slowly ---

    if tick.is_multiple_of(10) {
        // 7. Pollution: generate from machines, diffuse across grid.
        pollution::tick_pollution(&mut state.grid, &state.buildings);
    }

    // --- EVERY 20 TICKS (1 Hz) --- Story triggers + milestones ---
    if tick.is_multiple_of(20) {
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

        // Check rocket silo launches — the primary win condition.
        // A rocket silo with 100+ Rocket Parts triggers a launch.
        {
            let silo_ids = state.buildings.alive_ids();
            for bid in silo_ids {
                let is_silo = state
                    .buildings
                    .get(bid)
                    .map(|b| b.kind == types::BuildingKind::RocketSilo)
                    .unwrap_or(false);
                if !is_silo {
                    continue;
                }
                let parts = state
                    .buildings
                    .get(bid)
                    .and_then(|b| b.machine_state.as_ref())
                    .map(|ms| {
                        ms.input_buffer
                            .iter()
                            .filter(|r| **r == types::Resource::RocketPart)
                            .count()
                    })
                    .unwrap_or(0);
                if parts >= ROCKET_PARTS_PER_LAUNCH {
                    // Launch!
                    let building = state.buildings.get_mut(bid).unwrap();
                    let ms = building.machine_state.as_mut().unwrap();
                    // Remove rocket parts consumed by the launch.
                    let mut removed = 0usize;
                    ms.input_buffer.retain(|r| {
                        if *r == types::Resource::RocketPart && removed < ROCKET_PARTS_PER_LAUNCH {
                            removed += 1;
                            false
                        } else {
                            true
                        }
                    });
                    state.stats.rockets_launched += 1;
                    sfx.play(&sfx.research_done);
                    state.toast("*** ROCKET LAUNCHED! ***".to_string(), 200);
                    if !state.game_won {
                        state.game_won = true;
                        state.toast(
                            "CONSCIOUSNESS RESTORED! The signal reaches your crew!".to_string(),
                            300,
                        );
                        state.toast("Thank you for playing AutoForge <3".to_string(), 300);
                    }
                }
            }
        }

        // Fallback win condition: 50,000 items crafted (for players who skip the silo).
        if state.stats.items_crafted >= 50000 && !state.game_won {
            state.game_won = true;
            state.toast(
                "CONSCIOUSNESS RESTORED! You found your crew!".to_string(),
                300,
            );
            state.toast("Thank you for playing AutoForge <3".to_string(), 300);
        }

        // Soft-lock detection: if player has no buildings and no resources to rebuild,
        // give them emergency supplies so they can recover.
        if tick.is_multiple_of(600) && state.buildings.alive_ids().is_empty() {
            let total_resources: u32 = state.inventory.values().sum();
            if total_resources < 10 {
                state.toast(
                    "FORGE: Don't give up! Emergency supplies incoming~".to_string(),
                    120,
                );
                *state
                    .inventory
                    .entry(types::Resource::IronPlate)
                    .or_insert(0) += 50;
                *state
                    .inventory
                    .entry(types::Resource::CopperPlate)
                    .or_insert(0) += 30;
                *state.inventory.entry(types::Resource::Stone).or_insert(0) += 20;
                *state.inventory.entry(types::Resource::Coal).or_insert(0) += 30;
                *state.inventory.entry(types::Resource::Gear).or_insert(0) += 20;
            }
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
            sfx.play(&sfx.research_done); // celebratory arpeggio
            state.toast(format!("*** MILESTONE: {} ***", milestone.name), 120);
            state.toast(
                format!("+{} resource types rewarded!", milestone.reward.len()),
                80,
            );
        }

        // Expand build zone with research milestones.
        let base_radius = 40.0f32;
        let bonus = state.research.completed.iter().filter(|&&c| c).count() as f32 * 3.0;
        state.build_radius = base_radius + bonus;
    }
}
