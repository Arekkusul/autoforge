use macroquad::prelude::*;

use crate::buildcost;
use crate::building;
use crate::constants::{MACHINE_BUFFER_CAP, TILE_SIZE};
use crate::game::GameState;
use crate::grid;
use crate::milestones;
use crate::recipe;
use crate::research;
use crate::save;
use crate::sprites::SpriteAtlas;
use crate::types;

fn draw_panel(x: f32, y: f32, w: f32, h: f32, title: Option<&str>, closeable: bool) -> (f32, f32) {
    let bg = Color::new(0.08, 0.08, 0.12, 0.92);
    let border = Color::new(0.25, 0.30, 0.40, 0.50);
    let title_color = Color::new(0.95, 0.82, 0.35, 1.0);

    // Subtle rounded feel: slightly brighter inner border.
    draw_rectangle(x, y, w, h, bg);
    draw_rectangle_lines(x, y, w, h, 1.0, border);
    draw_rectangle_lines(
        x + 1.0,
        y + 1.0,
        w - 2.0,
        h - 2.0,
        1.0,
        Color::new(0.15, 0.15, 0.22, 0.3),
    );

    let mut content_y = y + 10.0;
    if let Some(t) = title {
        draw_text(t, x + 12.0, y + 24.0, 18.0, title_color);
        content_y = y + 34.0;
    }
    if closeable {
        draw_close_button(x, y, w);
    }
    (x + 10.0, content_y)
}

/// Draws a clickable X close button at the top-right of a panel.
fn draw_close_button(px: f32, py: f32, pw: f32) {
    let bx = px + pw - 28.0;
    let by = py + 4.0;
    let hover = mouse_position().0 >= bx
        && mouse_position().0 <= bx + 24.0
        && mouse_position().1 >= by
        && mouse_position().1 <= by + 20.0;
    let bg = if hover {
        Color::new(0.7, 0.2, 0.2, 0.8)
    } else {
        Color::new(0.4, 0.2, 0.2, 0.6)
    };
    draw_rectangle(bx, by, 24.0, 20.0, bg);
    draw_text(
        "X",
        bx + 7.0,
        by + 15.0,
        18.0,
        Color::new(1.0, 1.0, 1.0, 0.9),
    );
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
fn building_description(kind: types::BuildingKind) -> &'static str {
    match kind {
        types::BuildingKind::BeltYellow => "Moves items at 1x speed",
        types::BuildingKind::BeltRed => "Moves items at 2x speed",
        types::BuildingKind::BeltBlue => "Moves items at 3x speed",
        types::BuildingKind::Miner => "Extracts ore from deposits",
        types::BuildingKind::StoneFurnace => "Smelts ore into plates (needs coal)",
        types::BuildingKind::SteelFurnace => "1.5x smelting speed (needs coal)",
        types::BuildingKind::ElectricFurnace => "2x smelting speed (uses power)",
        types::BuildingKind::InserterRegular => "Moves items between buildings",
        types::BuildingKind::InserterLong => "Long-reach inserter",
        types::BuildingKind::InserterFast => "2x inserter speed",
        types::BuildingKind::InserterStack => "Moves 4 items at once",
        types::BuildingKind::AssemblerT1 => "Crafts items from recipes",
        types::BuildingKind::AssemblerT2 => "1.33x crafting speed",
        types::BuildingKind::AssemblerT3 => "2x crafting speed",
        types::BuildingKind::Lab => "Consumes science packs for research",
        types::BuildingKind::Boiler => "Burns coal to heat water",
        types::BuildingKind::SteamEngine => "Generates 900kW from steam",
        types::BuildingKind::SolarPanel => "Generates 60kW during daytime",
        types::BuildingKind::StorageChest => "Stores items, feeds your inventory",
        types::BuildingKind::GunTurret => "Shoots enemies (needs ammo)",
        types::BuildingKind::LaserTurret => "Shoots enemies (uses power)",
        types::BuildingKind::Wall => "Blocks enemies, absorbs damage",
        types::BuildingKind::ChemicalPlant => "Advanced chemical recipes",
        types::BuildingKind::Roboport => "Auto-delivers items to nearby machines",
        types::BuildingKind::NuclearReactor => "40,000kW nuclear power",
        types::BuildingKind::Splitter => "Splits belt items left/right",
        _ => "",
    }
}

fn fmt_num(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(c);
    }
    result
}

pub fn draw_ui(state: &mut GameState, atlas: &SpriteAtlas) {
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
        let w = measure_text(label, None, 20, 1.0).width;
        draw_rectangle(
            (screen_width() - w) * 0.5 - 12.0,
            6.0,
            w + 24.0,
            28.0,
            Color::new(0.1, 0.1, 0.15, 0.85),
        );
        draw_text(
            label,
            (screen_width() - w) * 0.5,
            26.0,
            20.0,
            Color::new(0.9, 0.8, 0.3, 1.0),
        );
    }

    let status_h = if state.research.current_tech.is_some() {
        138.0
    } else {
        120.0
    };
    let (cx, mut cy) = draw_panel(8.0, 8.0, 250.0, status_h, Some("FORGE"), false);

    // Line 1: Time + FPS (standardized 14px body)
    draw_text(
        &format!(
            "Play {}:{:02}  |  FPS: {}",
            state.stats.total_ticks / 1200,
            (state.stats.total_ticks / 20) % 60,
            get_fps()
        ),
        cx,
        cy + 4.0,
        14.0,
        text_dim,
    );
    // Speed badge (highlighted when not 1x).
    if state.game_speed > 1 {
        let speed_text = format!("{}x", state.game_speed);
        let stw = measure_text(&speed_text, None, 14, 1.0).width;
        draw_rectangle(
            cx + 172.0,
            cy - 4.0,
            stw + 10.0,
            18.0,
            Color::new(0.8, 0.6, 0.1, 0.8),
        );
        draw_text(
            &speed_text,
            cx + 177.0,
            cy + 8.0,
            14.0,
            Color::new(1.0, 1.0, 1.0, 1.0),
        );
    }
    cy += 20.0;

    // Line 2: Power bar (wider, taller for readability)
    let bar_w = 150.0;
    let bar_h = 12.0;
    let power_fill = state.power.satisfaction;
    let power_color = if power_fill >= 0.9 {
        Color::new(0.3, 0.85, 0.3, 1.0)
    } else if power_fill >= 0.5 {
        Color::new(0.9, 0.8, 0.2, 1.0)
    } else {
        Color::new(0.9, 0.2, 0.2, 1.0)
    };
    draw_rectangle(cx, cy, bar_w, bar_h, Color::new(0.15, 0.15, 0.2, 0.8));
    draw_rectangle(cx, cy, bar_w * power_fill, bar_h, power_color);
    if state.power.demand > 0.0 {
        draw_text(
            &format!(
                "{:.0}% ({:.0}/{:.0}kW)",
                power_fill * 100.0,
                state.power.supply,
                state.power.demand
            ),
            cx + bar_w + 6.0,
            cy + 10.0,
            11.0,
            power_color,
        );
    } else {
        draw_text("No demand", cx + bar_w + 6.0, cy + 10.0, 11.0, text_dim);
    }
    cy += 18.0;

    // Line 3: Items crafted + production rate
    let items_per_min = if state.stats.total_ticks > 1200 {
        state.stats.items_crafted as f32 / (state.stats.total_ticks as f32 / 1200.0)
    } else {
        0.0
    };
    draw_text(
        &format!(
            "Items: {}  ({:.0}/min)",
            fmt_num(state.stats.items_crafted),
            items_per_min
        ),
        cx,
        cy + 4.0,
        14.0,
        text_dim,
    );
    cy += 18.0;

    // Line 4: Day/Night + Direction
    let dn_color = if state.daynight.is_day() {
        Color::new(0.9, 0.82, 0.3, 1.0)
    } else {
        Color::new(0.4, 0.4, 0.7, 1.0)
    };
    let dir_text = match state.placement_direction {
        types::Direction::North => "N",
        types::Direction::East => "E",
        types::Direction::South => "S",
        types::Direction::West => "W",
    };
    draw_text(&state.daynight.display(), cx, cy + 4.0, 13.0, dn_color);
    // Wave info with estimated time to next wave.
    let next_wave_ticks =
        (6000 + state.enemies.wave_number as u64 * 1200).saturating_sub(state.stats.total_ticks);
    let wave_eta = if next_wave_ticks > 0 {
        format!(" ~{}s", next_wave_ticks / 20)
    } else {
        String::new()
    };
    draw_text(
        &format!(
            "Dir:{}  Kills:{}  Wave:{}{}",
            dir_text, state.stats.enemies_killed, state.enemies.wave_number, wave_eta
        ),
        cx + 85.0,
        cy + 4.0,
        11.0,
        text_dim,
    );

    // Line 5: Current research (if active).
    if let Some(tech_idx) = state.research.current_tech {
        if tech_idx < research::TECHNOLOGIES.len() {
            cy += 16.0;
            let tech = &research::TECHNOLOGIES[tech_idx];
            let pct = if tech.units_needed > 0 {
                (state.research.progress as f32 / tech.units_needed as f32 * 100.0) as u32
            } else {
                100
            };
            let research_color = Color::new(0.4, 0.7, 1.0, 0.9);
            draw_text(
                &format!("Research: {} ({}%)", tech.name, pct),
                cx,
                cy + 4.0,
                12.0,
                research_color,
            );
        }
    }

    // Pause menu overlay (uses unified panel)
    if state.paused {
        let sw = screen_width();
        let sh = screen_height();
        let pw = 320.0;
        let ph = 360.0;
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
        // Quit button.
        let quit_y = cy + 8.0;
        let quit_w = 120.0;
        let quit_x = px + (pw - quit_w) * 0.5;
        let (qmx, qmy) = mouse_position();
        let quit_hover =
            qmx >= quit_x && qmx <= quit_x + quit_w && qmy >= quit_y && qmy <= quit_y + 24.0;
        let quit_bg = if quit_hover {
            Color::new(0.5, 0.15, 0.15, 0.8)
        } else {
            Color::new(0.3, 0.1, 0.1, 0.6)
        };
        draw_rectangle(quit_x, quit_y, quit_w, 24.0, quit_bg);
        draw_rectangle_lines(
            quit_x,
            quit_y,
            quit_w,
            24.0,
            1.0,
            Color::new(0.5, 0.2, 0.2, 0.5),
        );
        draw_text(
            "Save & Quit",
            quit_x + 16.0,
            quit_y + 17.0,
            14.0,
            Color::new(0.9, 0.7, 0.7, 0.9),
        );
        if quit_hover && is_mouse_button_pressed(MouseButton::Left) {
            save::save_game(state);
            std::process::exit(0);
        }

        draw_text(
            "You can still place buildings while paused!",
            px + 30.0,
            py + ph - 36.0,
            11.0,
            Color::new(0.5, 0.7, 0.5, 0.6),
        );
        draw_text(
            "Click outside or press Space to resume",
            px + 40.0,
            py + ph - 20.0,
            12.0,
            text_dim,
        );
        draw_text(
            "AutoForge v0.2.0",
            px + pw - 110.0,
            py + ph - 8.0,
            11.0,
            Color::new(0.4, 0.4, 0.5, 0.5),
        );

        // Click outside the pause panel to unpause.
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx < px || mx > px + pw || my < py || my > py + ph {
                state.paused = false;
            }
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
        let blink_frame = if (get_time() * 0.3).fract() > 0.92 {
            1
        } else {
            0
        };
        draw_texture_ex(
            &atlas.tex,
            avatar_x,
            cy,
            WHITE,
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

        draw_text(
            "I found them. All 4,000 colonists. Alive. Safe.",
            cx,
            cy,
            18.0,
            gold,
        );
        cy += 22.0;
        draw_text(
            "Thank you for helping me remember who I am. <3",
            cx,
            cy,
            14.0,
            Color::new(1.0, 0.7, 0.85, 0.9),
        );
        cy += 30.0;

        // Stats.
        let playtime_min = state.stats.total_ticks / 1200;
        let playtime_sec = (state.stats.total_ticks / 20) % 60;
        draw_text(
            &format!("Playtime:  {}:{:02}", playtime_min, playtime_sec),
            cx,
            cy,
            14.0,
            bright,
        );
        cy += 20.0;
        draw_text(
            &format!("Items Crafted:  {}", fmt_num(state.stats.items_crafted)),
            cx,
            cy,
            14.0,
            bright,
        );
        cy += 20.0;
        draw_text(
            &format!("Buildings Placed:  {}", state.stats.buildings_placed),
            cx,
            cy,
            14.0,
            bright,
        );
        cy += 20.0;
        draw_text(
            &format!("Enemies Defeated:  {}", state.stats.enemies_killed),
            cx,
            cy,
            14.0,
            bright,
        );
        cy += 30.0;

        draw_text("Press any key to continue playing~", cx, cy, 13.0, dim);

        // Dismiss on any key/click.
        if is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::Escape)
            || is_key_pressed(KeyCode::Enter)
            || is_mouse_button_pressed(MouseButton::Left)
        {
            state.game_won = false; // Dismiss victory screen, keep playing.
        }
    }

    // --- Top-right: hovered tile info panel ---
    let mouse_screen = Vec2::new(mouse_position().0, mouse_position().1);
    let mouse_world = state.camera.screen_to_world(mouse_screen);
    let grid_pos = grid::Grid::world_to_grid(mouse_world);

    let panel_w = 300.0;
    let panel_x = screen_width() - panel_w - 10.0;

    if let Some(tile) = state.grid.get_tile(grid_pos) {
        let mut lines: Vec<(String, Color)> = Vec::new();
        lines.push((format!("({}, {})", grid_pos.x, grid_pos.y), text_dim));
        let terrain_info = if tile.terrain == types::Terrain::Forest {
            format!("{:?} (absorbs pollution)", tile.terrain)
        } else if tile.terrain == types::Terrain::Water {
            format!("{:?} (not buildable)", tile.terrain)
        } else {
            format!("{:?}", tile.terrain)
        };
        lines.push((terrain_info, text_bright));

        // Show pollution level if significant.
        if tile.pollution > 0.1 {
            let pol_color = if tile.pollution > 1.0 {
                Color::new(0.9, 0.4, 0.2, 0.9)
            } else {
                Color::new(0.7, 0.6, 0.2, 0.8)
            };
            lines.push((format!("Pollution: {:.1}", tile.pollution), pol_color));
        }

        if let Some(deposit) = tile.deposit {
            let amount_str = if tile.ore_amount == u32::MAX {
                "Infinite".to_string()
            } else {
                fmt_num(tile.ore_amount as u64)
            };
            lines.push((
                format!("{} ({})", deposit.display_name(), amount_str),
                Color::new(0.9, 0.7, 0.3, 1.0),
            ));
        }

        if let Some(bid) = tile.building {
            if let Some(b) = state.buildings.get(bid) {
                lines.push((b.kind.display_name().to_string(), text_accent));
                // One-liner description.
                let desc = building_description(b.kind);
                if !desc.is_empty() {
                    lines.push((desc.to_string(), Color::new(0.6, 0.6, 0.7, 0.7)));
                }
                lines.push((
                    format!("Facing: {:?}  (R to rotate)", b.direction),
                    text_dim,
                ));
                // Miner: show what ore it's extracting.
                if b.kind == types::BuildingKind::Miner || b.kind == types::BuildingKind::PumpJack {
                    if let Some(deposit) = tile.deposit {
                        lines.push((
                            format!("Mining: {}", deposit.display_name()),
                            Color::new(0.9, 0.7, 0.3, 0.9),
                        ));
                    } else {
                        lines.push((
                            "No ore remaining!".to_string(),
                            Color::new(0.9, 0.3, 0.3, 0.9),
                        ));
                    }
                }
                // Show HP if damaged (with auto-repair hint).
                if b.hp < b.max_hp {
                    let hp_pct = (b.hp / b.max_hp * 100.0) as u32;
                    let hp_color = if hp_pct > 50 {
                        Color::new(0.9, 0.8, 0.2, 1.0)
                    } else {
                        Color::new(0.9, 0.3, 0.3, 1.0)
                    };
                    lines.push((
                        format!(
                            "HP: {}% ({:.0}/{:.0}) — auto-repairing",
                            hp_pct, b.hp, b.max_hp
                        ),
                        hp_color,
                    ));
                }
                // Belt speed info.
                if b.kind.is_belt() {
                    let (tier, speed) = match b.kind {
                        types::BuildingKind::BeltYellow => ("Yellow", "1x"),
                        types::BuildingKind::BeltRed => ("Red", "2x"),
                        types::BuildingKind::BeltBlue => ("Blue", "3x"),
                        _ => ("", ""),
                    };
                    if !tier.is_empty() {
                        lines.push((
                            format!("{} belt — {} speed (scroll to upgrade)", tier, speed),
                            Color::new(0.7, 0.7, 0.5, 0.8),
                        ));
                    }
                }
                // Splitter next-output indicator.
                if b.kind == types::BuildingKind::Splitter {
                    if let Some(ref ms) = b.machine_state {
                        let next_dir = if ms.fuel_ticks % 2 == 0 {
                            "straight"
                        } else {
                            "right"
                        };
                        lines.push((
                            format!("Next item goes: {}", next_dir),
                            Color::new(0.6, 0.7, 0.8, 0.8),
                        ));
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
                            let inputs: String = r
                                .inputs
                                .iter()
                                .map(|(res, c)| format!("{}x {}", c, short_resource_name(*res)))
                                .collect::<Vec<_>>()
                                .join(" + ");
                            lines.push((
                                format!("Needs: {}", inputs),
                                Color::new(0.7, 0.8, 0.7, 0.9),
                            ));
                            // Show what comes OUT.
                            let outputs: String = r
                                .outputs
                                .iter()
                                .map(|(res, c)| format!("{}x {}", c, short_resource_name(*res)))
                                .collect::<Vec<_>>()
                                .join(" + ");
                            lines.push((
                                format!("Makes: {}", outputs),
                                Color::new(0.5, 0.9, 0.5, 0.9),
                            ));
                        }
                    } else if b.kind == types::BuildingKind::AssemblerT1
                        || b.kind == types::BuildingKind::AssemblerT2
                        || b.kind == types::BuildingKind::AssemblerT3
                        || b.kind == types::BuildingKind::ChemicalPlant
                    {
                        lines.push((
                            "Click to set recipe!".to_string(),
                            Color::new(0.9, 0.7, 0.3, 1.0),
                        ));
                    }
                    // Progress/status indicator (different for inserters vs machines).
                    if b.kind.is_inserter() {
                        if ms.progress_ticks > 0 {
                            lines.push(("Swinging...".to_string(), Color::new(0.4, 0.8, 1.0, 0.9)));
                        } else {
                            lines.push(("Ready".to_string(), Color::new(0.6, 0.6, 0.4, 0.8)));
                        }
                    } else if ms.progress_ticks > 0 && ms.total_ticks > 0 {
                        let pct = ((ms.total_ticks - ms.progress_ticks) as f32
                            / ms.total_ticks as f32
                            * 100.0) as u32;
                        lines.push((
                            format!("Progress: {}%", pct),
                            Color::new(0.4, 0.8, 1.0, 0.9),
                        ));
                    } else if ms.selected_recipe.is_some() && ms.progress_ticks == 0 {
                        lines.push((
                            "Idle — waiting for inputs".to_string(),
                            Color::new(0.6, 0.6, 0.4, 0.8),
                        ));
                    }
                    // Buffer contents — show item names for chests, counts for machines.
                    if !ms.input_buffer.is_empty() {
                        if b.kind == types::BuildingKind::StorageChest {
                            // Show first few item names.
                            let items: String = ms
                                .input_buffer
                                .iter()
                                .take(4)
                                .map(|r| short_resource_name(*r))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let more = if ms.input_buffer.len() > 4 {
                                format!(" +{}", ms.input_buffer.len() - 4)
                            } else {
                                String::new()
                            };
                            lines.push((format!("Stored: {}{}", items, more), text_dim));
                        } else {
                            lines.push((
                                format!("Input: {}/{}", ms.input_buffer.len(), MACHINE_BUFFER_CAP),
                                text_dim,
                            ));
                        }
                    }
                    if !ms.output_buffer.is_empty() {
                        lines.push((
                            format!("Output: {}/{}", ms.output_buffer.len(), MACHINE_BUFFER_CAP),
                            text_dim,
                        ));
                    }
                    if ms.fuel_ticks > 0 {
                        lines.push((
                            format!("Fuel: {:.1}s", ms.fuel_ticks as f32 / 20.0),
                            text_dim,
                        ));
                    }
                    // Show installed modules.
                    if !ms.modules.is_empty() {
                        let mod_names: String = ms
                            .modules
                            .iter()
                            .map(|m| short_resource_name(*m))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let slots_left = building::MAX_MODULE_SLOTS - ms.modules.len();
                        let slot_info = if slots_left > 0 {
                            format!(" ({} slot free)", slots_left)
                        } else {
                            String::new()
                        };
                        lines.push((
                            format!("Modules: {}{}", mod_names, slot_info),
                            Color::new(0.6, 0.8, 1.0, 0.9),
                        ));
                    } else if !b.kind.is_belt()
                        && !b.kind.is_inserter()
                        && b.kind != types::BuildingKind::StorageChest
                        && b.kind != types::BuildingKind::Wall
                        && b.kind != types::BuildingKind::Gate
                    {
                        lines.push((
                            "Modules: empty (middle-click to add)".to_string(),
                            Color::new(0.5, 0.5, 0.6, 0.6),
                        ));
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
            if !enemy.alive {
                continue;
            }
            let dx = enemy.x - mouse_world.x;
            let dy = enemy.y - mouse_world.y;
            if dx * dx + dy * dy < (TILE_SIZE * TILE_SIZE) {
                let hp_pct = (enemy.hp / enemy.kind.max_hp() * 100.0) as u32;
                lines.push((
                    format!("{:?} — HP: {}%", enemy.kind, hp_pct),
                    Color::new(1.0, 0.4, 0.3, 1.0),
                ));
                lines.push((
                    format!(
                        "Dmg: {:.0}  Spd: {:.1}",
                        enemy.kind.damage(),
                        enemy.kind.speed()
                    ),
                    Color::new(0.8, 0.5, 0.4, 0.8),
                ));
                break; // only show one enemy
            }
        }

        // Check for nearby enemy nests.
        for nest_pos in &state.nests {
            let dist = grid_pos.distance(*nest_pos);
            if dist < 3.0 {
                lines.push((
                    "Enemy Nest nearby!".to_string(),
                    Color::new(0.8, 0.2, 0.2, 0.9),
                ));
                break;
            }
        }

        // Only show panel if there's meaningful info (skip bare grass tiles).
        let has_info = tile.deposit.is_some()
            || tile.building.is_some()
            || !items_here.is_empty()
            || lines.len() > 2;
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
    let default_toolbar: Vec<(&str, types::BuildingKind)> = vec![
        ("1", types::BuildingKind::BeltYellow),
        ("2", types::BuildingKind::Miner),
        ("3", types::BuildingKind::StoneFurnace),
        ("4", types::BuildingKind::InserterRegular),
        ("5", types::BuildingKind::AssemblerT1),
        ("6", types::BuildingKind::Boiler),
        ("7", types::BuildingKind::SteamEngine),
        ("8", types::BuildingKind::Lab),
        ("9", types::BuildingKind::StorageChest),
        ("0", types::BuildingKind::Splitter),
        ("T", types::BuildingKind::GunTurret),
        ("G", types::BuildingKind::Wall),
        ("U", types::BuildingKind::UndergroundBeltYellow),
        ("C", types::BuildingKind::ChemicalPlant),
        ("L", types::BuildingKind::LaserTurret),
        ("P", types::BuildingKind::SolarPanel),
    ];
    // Apply custom hotbar overrides.
    let toolbar_items: Vec<(&str, &str, types::BuildingKind, Rect)> = default_toolbar
        .iter()
        .enumerate()
        .map(|(i, &(key, default_kind))| {
            let kind = if i < state.custom_hotbar.len() {
                state.custom_hotbar[i].unwrap_or(default_kind)
            } else {
                default_kind
            };
            let label = kind.display_name();
            // Truncate label for toolbar display.
            let short = if label.len() > 8 { &label[..8] } else { label };
            let sprite = crate::render::building_src_rect(kind, atlas, 0, 0);
            (key, short, kind, sprite)
        })
        .collect();

    let slot_w = (screen_width() / toolbar_items.len() as f32).min(76.0);
    let slot_h = 66.0;
    let total_w = toolbar_items.len() as f32 * slot_w;
    let start_x = (screen_width() - total_w) * 0.5;

    // Pre-count buildings by kind for toolbar badges.
    let mut kind_counts: std::collections::HashMap<types::BuildingKind, u32> =
        std::collections::HashMap::new();
    for (_, b) in state.buildings.iter() {
        *kind_counts.entry(b.kind).or_insert(0) += 1;
    }

    for (i, (hotkey, name, kind, src_rect)) in toolbar_items.iter().enumerate() {
        let x = start_x + i as f32 * slot_w;
        let y = toolbar_y + 5.0;
        let is_selected = state.selected_building == Some(*kind);

        // Slot background
        if is_selected {
            draw_rectangle(x + 2.0, y, slot_w - 4.0, slot_h, selected_bg);
            draw_rectangle_lines(x + 2.0, y, slot_w - 4.0, slot_h, 2.0, selected_border);
        } else {
            draw_rectangle(
                x + 2.0,
                y,
                slot_w - 4.0,
                slot_h,
                Color::new(0.12, 0.12, 0.15, 0.6),
            );
            draw_rectangle_lines(
                x + 2.0,
                y,
                slot_w - 4.0,
                slot_h,
                1.0,
                Color::new(0.3, 0.3, 0.3, 0.5),
            );
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
        draw_text(
            hotkey,
            x + 6.0,
            y + 14.0,
            16.0,
            Color::new(1.0, 1.0, 0.5, 0.9),
        );

        // Placed count badge (top-right corner, only if > 0).
        let count = kind_counts.get(kind).copied().unwrap_or(0);
        if count > 0 {
            let count_str = format!("{}", count);
            let cw = measure_text(&count_str, None, 10, 1.0).width;
            draw_rectangle(
                x + slot_w - cw - 8.0,
                y + 1.0,
                cw + 6.0,
                12.0,
                Color::new(0.0, 0.0, 0.0, 0.5),
            );
            draw_text(
                &count_str,
                x + slot_w - cw - 5.0,
                y + 11.0,
                10.0,
                Color::new(0.6, 0.7, 0.8, 0.7),
            );
        }

        // Label below icon (auto-shrink font if slot is narrow).
        let label_font = if slot_w < 68.0 { 11.0 } else { 13.0 };
        let label_w = measure_text(name, None, label_font as u16, 1.0).width;
        draw_text(
            name,
            x + (slot_w - label_w).max(0.0) * 0.5,
            y + slot_h - 5.0,
            label_font,
            if is_selected { text_bright } else { text_dim },
        );

        // Hover highlight + cost tooltip (only shows when mouse is stationary over slot).
        let (mx, my) = mouse_position();
        if !is_selected && mx >= x && mx < x + slot_w && my >= y && my < y + slot_h {
            draw_rectangle(
                x + 2.0,
                y,
                slot_w - 4.0,
                slot_h,
                Color::new(0.2, 0.2, 0.3, 0.3),
            );
            // Show cost tooltip with fade-in effect.
            let cost = buildcost::building_cost(*kind);
            if !cost.is_empty() {
                let cost_str: String = cost
                    .iter()
                    .map(|(r, c)| format!("{}x{}", c, short_resource_name(*r)))
                    .collect::<Vec<_>>()
                    .join(" ");
                let can_afford = buildcost::can_afford(&state.inventory, *kind);
                let color = if can_afford {
                    Color::new(0.5, 0.9, 0.5, 0.85)
                } else {
                    Color::new(0.9, 0.4, 0.4, 0.85)
                };
                let tw = measure_text(&cost_str, None, 12, 1.0).width;
                let tx = x + (slot_w - tw) * 0.5;
                draw_rectangle(
                    tx - 4.0,
                    y - 20.0,
                    tw + 8.0,
                    18.0,
                    Color::new(0.05, 0.05, 0.08, 0.85),
                );
                draw_rectangle_lines(
                    tx - 4.0,
                    y - 20.0,
                    tw + 8.0,
                    18.0,
                    1.0,
                    Color::new(0.2, 0.2, 0.3, 0.4),
                );
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
            .map(|(r, c)| {
                format!(
                    "{}x {}",
                    c,
                    match r {
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
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(" + ");

        let info = format!("{} — {:?}", name, state.placement_direction);
        let desc = building_description(kind);
        let can_afford = buildcost::can_afford(&state.inventory, kind);
        let cost_color = if can_afford {
            Color::new(0.5, 0.9, 0.5, 0.9)
        } else {
            Color::new(0.9, 0.3, 0.3, 0.9)
        };

        let total_w = measure_text(&info, None, 18, 1.0)
            .width
            .max(measure_text(&cost_str, None, 13, 1.0).width)
            .max(measure_text(desc, None, 12, 1.0).width)
            + 28.0;
        let panel_x = (screen_width() - total_w) * 0.5;
        let panel_h = if desc.is_empty() { 48.0 } else { 62.0 };

        draw_rectangle(
            panel_x,
            toolbar_y - panel_h - 4.0,
            total_w,
            panel_h,
            panel_bg,
        );
        draw_rectangle_lines(
            panel_x,
            toolbar_y - panel_h - 4.0,
            total_w,
            panel_h,
            1.0,
            panel_border,
        );
        draw_text(
            &info,
            panel_x + 10.0,
            toolbar_y - panel_h + 14.0,
            18.0,
            text_bright,
        );
        if !desc.is_empty() {
            draw_text(
                desc,
                panel_x + 10.0,
                toolbar_y - panel_h + 30.0,
                12.0,
                text_dim,
            );
        }
        draw_text(
            &cost_str,
            panel_x + 10.0,
            toolbar_y - 10.0,
            13.0,
            cost_color,
        );
    }

    // --- Minimap (top-right corner, below info panel) ---
    {
        let mm_size = 140.0;
        let mm_x = screen_width() - mm_size - 10.0;
        let mm_y = screen_height() - 210.0 - mm_size; // above toolbar area
        let _panel_bg = Color::new(0.06, 0.06, 0.08, 0.9);

        draw_panel(
            mm_x - 4.0,
            mm_y - 24.0,
            mm_size + 8.0,
            mm_size + 56.0,
            Some("Map"),
            false,
        );

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
            let npx = (nest_pos.x - (cam_grid.x - half_range)) as f32
                / (map_pixels * tiles_per_pixel) as f32
                * mm_size;
            let npy = (nest_pos.y - (cam_grid.y - half_range)) as f32
                / (map_pixels * tiles_per_pixel) as f32
                * mm_size;
            if npx >= 0.0 && npx < mm_size && npy >= 0.0 && npy < mm_size {
                draw_circle(mm_x + npx, mm_y + npy, 3.0, Color::new(0.6, 0.1, 0.1, 0.8));
            }
        }

        // Draw enemies as red dots on minimap.
        for enemy in &state.enemies.list {
            if !enemy.alive {
                continue;
            }
            let eg = grid::Grid::world_to_grid(Vec2::new(enemy.x, enemy.y));
            let rpx = (eg.x - (cam_grid.x - half_range)) as f32
                / (map_pixels * tiles_per_pixel) as f32
                * mm_size;
            let rpy = (eg.y - (cam_grid.y - half_range)) as f32
                / (map_pixels * tiles_per_pixel) as f32
                * mm_size;
            if rpx >= 0.0 && rpx < mm_size && rpy >= 0.0 && rpy < mm_size {
                draw_circle(mm_x + rpx, mm_y + rpy, 2.0, Color::new(1.0, 0.1, 0.1, 0.9));
            }
        }

        // Camera viewport rectangle.
        let (vis_min, vis_max) = state.camera.visible_bounds();
        let vis_min_g = grid::Grid::world_to_grid(vis_min);
        let vis_max_g = grid::Grid::world_to_grid(vis_max);
        let rx = (vis_min_g.x - (cam_grid.x - half_range)) as f32
            / (map_pixels * tiles_per_pixel) as f32
            * mm_size;
        let ry = (vis_min_g.y - (cam_grid.y - half_range)) as f32
            / (map_pixels * tiles_per_pixel) as f32
            * mm_size;
        let rw =
            (vis_max_g.x - vis_min_g.x) as f32 / (map_pixels * tiles_per_pixel) as f32 * mm_size;
        let rh =
            (vis_max_g.y - vis_min_g.y) as f32 / (map_pixels * tiles_per_pixel) as f32 * mm_size;
        draw_rectangle_lines(mm_x + rx, mm_y + ry, rw, rh, 1.0, WHITE);

        // Click minimap to teleport camera.
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx >= mm_x && mx < mm_x + mm_size && my >= mm_y && my < mm_y + mm_size {
                let frac_x = (mx - mm_x) / mm_size;
                let frac_y = (my - mm_y) / mm_size;
                let target_gx = cam_grid.x - half_range
                    + (frac_x * (map_pixels * tiles_per_pixel) as f32) as i32;
                let target_gy = cam_grid.y - half_range
                    + (frac_y * (map_pixels * tiles_per_pixel) as f32) as i32;
                state.camera.target =
                    grid::Grid::grid_to_world_center(types::GridPos::new(target_gx, target_gy));
            }
        }

        // Minimap legend (color key).
        let lg_y = mm_y + mm_size + 4.0;
        let lg_items: &[(&str, Color)] = &[
            ("Bld", Color::new(0.5, 0.5, 0.8, 1.0)),
            ("Ore", Color::new(0.7, 0.5, 0.2, 1.0)),
            ("H2O", Color::new(0.2, 0.3, 0.6, 1.0)),
            ("Pol", Color::new(0.4, 0.4, 0.1, 1.0)),
            ("Nest", Color::new(0.6, 0.1, 0.1, 1.0)),
        ];
        for (i, (label, color)) in lg_items.iter().enumerate() {
            let lx = mm_x + i as f32 * 28.0;
            draw_rectangle(lx, lg_y, 6.0, 6.0, *color);
            draw_text(label, lx + 8.0, lg_y + 7.0, 9.0, text_dim);
        }
    }

    // --- Blueprint library picker (centered overlay) ---
    if state.show_blueprint_picker && !state.blueprint_library.is_empty() {
        let sw = screen_width();
        let sh = screen_height();
        let pw = 320.0;
        let ph = 50.0 + state.blueprint_library.len() as f32 * 28.0;
        let px = (sw - pw) * 0.5;
        let py = (sh - ph) * 0.5;
        let (cx, mut cy) = draw_panel(px, py, pw, ph, Some("Blueprint Library"), true);
        cy += 4.0;

        let (mx, my) = mouse_position();
        let mut selected_bp: Option<(
            String,
            Vec<(i32, i32, types::BuildingKind, types::Direction)>,
        )> = None;
        for (i, (name, bp)) in state.blueprint_library.iter().enumerate() {
            let ry = cy + i as f32 * 28.0;
            let hover = mx >= px + 5.0 && mx <= px + pw - 5.0 && my >= ry - 2.0 && my <= ry + 22.0;
            if hover {
                draw_rectangle(
                    px + 5.0,
                    ry - 2.0,
                    pw - 10.0,
                    24.0,
                    Color::new(0.2, 0.3, 0.5, 0.4),
                );
            }
            let key_label = format!("[{}]", i + 1);
            draw_text(
                &key_label,
                cx,
                ry + 14.0,
                14.0,
                Color::new(0.5, 0.7, 1.0, 0.9),
            );
            draw_text(
                &format!("{} — {} buildings", name, bp.len()),
                cx + 28.0,
                ry + 14.0,
                14.0,
                text_bright,
            );

            if hover && is_mouse_button_pressed(MouseButton::Left) {
                selected_bp = Some((name.clone(), bp.clone()));
            }
        }

        // Apply selection after loop (avoids borrow conflict).
        if let Some((name, bp)) = selected_bp {
            state.blueprint = bp;
            state.pasting_blueprint = true;
            state.show_blueprint_picker = false;
            state.selected_building = None;
            state.toast(
                format!("Loaded '{}' — click to paste, B to cancel", name),
                80,
            );
        }

        // Close button handling.
        let bx = px + pw - 28.0;
        let by = py + 4.0;
        if is_mouse_button_pressed(MouseButton::Left)
            && mx >= bx
            && mx <= bx + 24.0
            && my >= by
            && my <= by + 20.0
        {
            state.show_blueprint_picker = false;
        }
    }

    // --- Toast notifications (center-top area, max 3 visible) ---
    if !state.toasts.is_empty() {
        let cx = screen_width() * 0.5;
        for (i, (msg, remaining, severity)) in state.toasts.iter().take(3).enumerate() {
            let alpha = (*remaining as f32 / 20.0).min(1.0); // fade out last 20 ticks
            let y = 40.0 + i as f32 * 26.0;
            let w = measure_text(msg, None, 20, 1.0).width;
            let (bg_color, text_color) = match severity {
                types::AlertSeverity::Info => (
                    Color::new(0.1, 0.1, 0.15, 0.85 * alpha),
                    Color::new(1.0, 1.0, 1.0, alpha),
                ),
                types::AlertSeverity::Warning => (
                    Color::new(0.5, 0.35, 0.05, 0.9 * alpha),
                    Color::new(1.0, 0.9, 0.3, alpha),
                ),
                types::AlertSeverity::Critical => (
                    Color::new(0.5, 0.1, 0.05, 0.9 * alpha),
                    Color::new(1.0, 0.4, 0.3, alpha),
                ),
            };
            draw_rectangle(cx - w * 0.5 - 12.0, y - 16.0, w + 24.0, 24.0, bg_color);
            draw_text(msg, cx - w * 0.5, y, 20.0, text_color);
        }
    }

    // --- Bottom-right: controls hint (hidden when any overlay is active) ---
    let any_overlay = state.paused
        || state.show_recipes
        || state.show_research
        || state.show_stats
        || state.show_achievements
        || state.show_help
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
            "F5:Save F9:Load F12:Screenshot F1:Help",
        ];
        for (i, line) in hints.iter().enumerate() {
            draw_text(line, help_x, help_y + i as f32 * 18.0, 14.0, hint_color);
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
            0 => (
                "Welcome! Click a building in the toolbar below",
                "or press 1-8 to select it. Press E for recipes!",
            ),
            1 => (
                "Place a MINER on an ore deposit (big rocks)",
                "Face it toward where you want items to go (R to rotate)",
            ),
            2 => (
                "Place BELTS from the miner's output arrow",
                "Items will flow along them automatically!",
            ),
            3 => (
                "Place an INSERTER between belt and machine",
                "It grabs from behind, places forward (R to rotate)",
            ),
            4 => (
                "Place a FURNACE! Feed it ore AND coal for fuel",
                "Use 2 inserters: one for ore, one for coal",
            ),
            5 => (
                "Put items into a STORAGE CHEST to collect them!",
                "Chest contents go to your inventory for building!",
            ),
            _ => ("", ""),
        };

        draw_text(
            tutorial_text.0,
            tut_x + 15.0,
            tut_y + 30.0,
            20.0,
            Color::new(0.95, 0.9, 1.0, 1.0),
        );
        draw_text(
            tutorial_text.1,
            tut_x + 15.0,
            tut_y + 55.0,
            16.0,
            Color::new(0.7, 0.65, 0.85, 0.9),
        );
    }

    // --- Inventory (left side, below status, compact two-column) ---
    // Dynamically shows ALL resources the player has (not a hardcoded subset).
    let inv_start_y = 8.0 + status_h + 4.0; // below status panel
    let mut inv_panel_bottom = inv_start_y;
    {
        // Resource category ordering: plates first, then ores, intermediates, science, ammo.
        fn resource_sort_key(r: &types::Resource) -> u32 {
            match r {
                types::Resource::IronPlate
                | types::Resource::CopperPlate
                | types::Resource::SteelPlate => 0,
                types::Resource::Stone | types::Resource::Coal | types::Resource::StoneBrick => 1,
                types::Resource::IronOre
                | types::Resource::CopperOre
                | types::Resource::UraniumOre => 2,
                types::Resource::Gear
                | types::Resource::Wire
                | types::Resource::Pipe
                | types::Resource::IronStick => 3,
                types::Resource::GreenCircuit
                | types::Resource::RedCircuit
                | types::Resource::BlueCircuit => 4,
                types::Resource::ScienceRed
                | types::Resource::ScienceGreen
                | types::Resource::ScienceBlue
                | types::Resource::SciencePurple
                | types::Resource::ScienceYellow => 5,
                types::Resource::BasicAmmo
                | types::Resource::PiercingAmmo
                | types::Resource::Grenade => 6,
                _ => 7,
            }
        }
        let mut show: Vec<(types::Resource, &str, u32)> = state
            .inventory
            .iter()
            .filter(|(_, &c)| c > 0)
            .map(|(r, &c)| (*r, short_resource_name(*r), c))
            .collect();
        show.sort_by(|a, b| {
            resource_sort_key(&a.0)
                .cmp(&resource_sort_key(&b.0))
                .then(b.2.cmp(&a.2))
        });

        if !show.is_empty() {
            let rows = show.len().div_ceil(2); // two columns
            let inv_h = 34.0 + rows.min(10) as f32 * 16.0;
            let (ix, mut iy) = draw_panel(8.0, inv_start_y, 220.0, inv_h, Some("Inventory"), false);
            inv_panel_bottom = inv_start_y + inv_h;

            for chunk in show.chunks(2) {
                for (col, (_, name, count)) in chunk.iter().enumerate() {
                    let x = ix + col as f32 * 102.0;
                    draw_text(
                        &format!("{}: {}", name, count),
                        x,
                        iy + 4.0,
                        12.0,
                        text_bright,
                    );
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
        let panel_h = if next.is_some() { 130.0 } else { 70.0 };
        let (gx, _gy) = draw_panel(goal_x, goal_y, 220.0, panel_h, Some("Roadmap"), false);

        if let Some(idx) = next {
            let m = &milestones::MILESTONES[idx];
            let (pr, pg, pb) = m.phase.color();
            let phase_color = Color::new(pr, pg, pb, 0.9);

            // Phase label.
            draw_text(m.phase.label(), gx, goal_y + 18.0, 12.0, phase_color);

            // Milestone name (gold, prominent).
            draw_text(
                m.name,
                gx,
                goal_y + 38.0,
                17.0,
                Color::new(0.95, 0.82, 0.35, 1.0),
            );

            // Description.
            draw_text(
                m.description,
                gx,
                goal_y + 56.0,
                13.0,
                Color::new(0.85, 0.85, 0.95, 1.0),
            );

            // Hint (how to do it — enlarged for readability).
            draw_text(
                m.hint,
                gx,
                goal_y + 74.0,
                11.0,
                Color::new(0.6, 0.7, 0.8, 0.85),
            );

            // Reward preview.
            let reward_str: String = m
                .reward
                .iter()
                .map(|(r, c)| format!("{}x{}", c, short_resource_name(*r)))
                .collect::<Vec<_>>()
                .join(" ");
            draw_text(
                &format!("Reward: {}", reward_str),
                gx,
                goal_y + 90.0,
                12.0,
                Color::new(0.5, 0.9, 0.5, 0.8),
            );

            // Progress bar toward 50k items.
            let progress = (state.stats.items_crafted as f32 / 50000.0).min(1.0);
            let bar_x = gx;
            let bar_y = goal_y + 100.0;
            let bar_w = 200.0;
            let bar_h = 8.0;
            draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.15, 0.15, 0.2, 0.8));
            let bar_color = if progress >= 0.9 {
                Color::new(0.3, 0.9, 0.3, 0.9)
            } else if progress >= 0.5 {
                Color::new(0.9, 0.8, 0.2, 0.9)
            } else {
                Color::new(0.4, 0.5, 0.7, 0.9)
            };
            draw_rectangle(bar_x, bar_y, bar_w * progress, bar_h, bar_color);
            let completed_count = state.milestones_completed.iter().filter(|&&c| c).count();
            draw_text(
                &format!(
                    "{}/{} milestones  |  {:.0}% to FORGE",
                    completed_count,
                    milestones::MILESTONES.len(),
                    progress * 100.0
                ),
                bar_x,
                bar_y + 18.0,
                11.0,
                Color::new(0.5, 0.6, 0.7, 0.75),
            );
        } else {
            // All milestones done!
            draw_text(
                "All milestones complete!",
                gx + 8.0,
                goal_y + 20.0,
                14.0,
                Color::new(0.5, 0.9, 0.5, 1.0),
            );
            draw_text(
                "FORGE consciousness restored <3",
                gx + 8.0,
                goal_y + 40.0,
                12.0,
                Color::new(0.9, 0.7, 0.85, 0.9),
            );
        }
    }

    // --- Recipe picker popup (click assembler to open) ---
    if let Some((bid, ref recipes)) = state.recipe_picker {
        let sw = screen_width();
        let sh = screen_height();
        let pw: f32 = 340.0;
        let ph: f32 = 50.0 + recipes.len() as f32 * 28.0;
        let capped_ph: f32 = ph.min(screen_height() * 0.7).min(600.0);
        let px = (sw - pw) * 0.5;
        let py = (sh - capped_ph) * 0.5;

        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.4));
        draw_panel(px, py, pw, capped_ph, Some("Select Recipe"), true);
        draw_text(
            "Click to select, Esc to cancel",
            px + 20.0,
            py + 40.0,
            12.0,
            Color::new(0.6, 0.6, 0.65, 0.7),
        );

        // Show current recipe (if any).
        if let Some(b) = state.buildings.get(bid) {
            if let Some(ref ms) = b.machine_state {
                if let Some(cur) = ms.selected_recipe {
                    if cur.0 < recipe::RECIPES.len() {
                        draw_text(
                            &format!("Current: {}", recipe::RECIPES[cur.0].name),
                            px + 180.0,
                            py + 25.0,
                            14.0,
                            Color::new(0.5, 0.8, 0.5, 0.8),
                        );
                    }
                }
            }
        }

        // Recipe list with inputs → outputs.
        let mx = mouse_position().0;
        let my = mouse_position().1;
        for (i, rid) in recipes.iter().enumerate() {
            let ry = py + 55.0 + i as f32 * 28.0;
            if ry > py + capped_ph - 10.0 {
                break;
            }

            let r = &recipe::RECIPES[rid.0];

            // Hover highlight.
            if mx >= px + 10.0 && mx <= px + pw - 10.0 && my >= ry - 10.0 && my <= ry + 16.0 {
                draw_rectangle(
                    px + 5.0,
                    ry - 10.0,
                    pw - 10.0,
                    26.0,
                    Color::new(0.2, 0.3, 0.5, 0.4),
                );
            }

            // Recipe name + craftability indicator.
            let can_craft = r
                .inputs
                .iter()
                .all(|(res, count)| state.inventory.get(res).copied().unwrap_or(0) >= *count);
            let name_color = if can_craft {
                Color::new(0.5, 0.95, 0.5, 1.0) // green = you have all inputs
            } else {
                Color::new(0.9, 0.9, 0.95, 1.0) // white = can't craft yet
            };
            draw_text(r.name, px + 15.0, ry + 4.0, 14.0, name_color);

            // Inputs → Output with per-input availability coloring.
            let inputs: String = r
                .inputs
                .iter()
                .map(|(res, c)| {
                    let have = state.inventory.get(res).copied().unwrap_or(0);
                    let sym = if have >= *c { "+" } else { "-" };
                    format!("{}{}x{}", sym, c, short_resource_name(*res))
                })
                .collect::<Vec<_>>()
                .join(" ");
            let outputs: String = r
                .outputs
                .iter()
                .map(|(res, c)| format!("{}x{}", c, short_resource_name(*res)))
                .collect::<Vec<_>>()
                .join("+");
            let flow = format!("{} -> {}", inputs, outputs);
            draw_text(
                &flow,
                px + 180.0,
                ry + 4.0,
                12.0,
                Color::new(0.6, 0.7, 0.6, 0.8),
            );
        }
    }

    // --- Production Stats screen (V key) ---
    if state.show_stats {
        let sw = screen_width();
        let sh = screen_height();
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.4));
        let pw = (sw * 0.5).min(500.0);
        let ph = (sh * 0.65).min(450.0);
        let px = (sw - pw) * 0.5;
        let py = (sh - ph) * 0.5;

        let (sx, mut sy) = draw_panel(px, py, pw, ph, Some("Production Stats"), true);

        // General stats.
        let playtime_min = state.stats.total_ticks / 1200;
        let playtime_sec = (state.stats.total_ticks / 20) % 60;
        let items_per_min = if state.stats.total_ticks > 0 {
            state.stats.items_crafted as f32 / (state.stats.total_ticks as f32 / 1200.0)
        } else {
            0.0
        };

        draw_text(
            &format!("Playtime: {}:{:02}", playtime_min, playtime_sec),
            sx,
            sy,
            14.0,
            text_bright,
        );
        sy += 20.0;
        draw_text(
            &format!("Items crafted: {}", state.stats.items_crafted),
            sx,
            sy,
            14.0,
            text_bright,
        );
        sy += 20.0;
        draw_text(
            &format!("Production rate: {:.1}/min", items_per_min),
            sx,
            sy,
            14.0,
            text_accent,
        );
        sy += 20.0;
        draw_text(
            &format!("Buildings placed: {}", state.stats.buildings_placed),
            sx,
            sy,
            14.0,
            text_bright,
        );
        sy += 20.0;
        draw_text(
            &format!("Enemies killed: {}", state.stats.enemies_killed),
            sx,
            sy,
            14.0,
            text_bright,
        );
        sy += 20.0;
        draw_text(
            &format!("Rockets launched: {}", state.stats.rockets_launched),
            sx,
            sy,
            14.0,
            text_bright,
        );
        sy += 20.0;
        let evo_pct = (state.evolution * 100.0) as u32;
        let evo_color = if evo_pct > 70 {
            Color::new(0.9, 0.3, 0.3, 1.0)
        } else if evo_pct > 40 {
            Color::new(0.9, 0.7, 0.2, 1.0)
        } else {
            Color::new(0.5, 0.8, 0.5, 1.0)
        };
        draw_text(
            &format!("Enemy evolution: {}%", evo_pct),
            sx,
            sy,
            14.0,
            evo_color,
        );
        sy += 20.0;
        let seed_str = format!("World seed: {}", state.seed);
        draw_text(&seed_str, sx, sy, 13.0, text_dim);
        // Click seed to copy to toast for easy sharing.
        let seed_w = measure_text(&seed_str, None, 13, 1.0).width;
        let (smx, smy) = mouse_position();
        if smx >= sx && smx <= sx + seed_w && smy >= sy - 12.0 && smy <= sy + 4.0 {
            draw_rectangle(
                sx - 2.0,
                sy - 12.0,
                seed_w + 4.0,
                16.0,
                Color::new(0.2, 0.2, 0.3, 0.4),
            );
            if is_mouse_button_pressed(MouseButton::Left) {
                state.toast(
                    format!("Seed: {} (share this to recreate your map!)", state.seed),
                    200,
                );
            }
        }
        sy += 24.0;

        // Building count by type.
        let total_buildings = state.buildings.alive_ids().len();
        draw_text(
            &format!("Buildings ({} total):", total_buildings),
            sx,
            sy,
            14.0,
            Color::new(0.95, 0.82, 0.35, 0.9),
        );
        sy += 18.0;
        // Per-resource production rates (items/min from last 60 seconds).
        let tick = state.stats.total_ticks;
        let window = 1200u64; // 60 seconds
        let cutoff = tick.saturating_sub(window);
        let mut res_counts: std::collections::HashMap<types::Resource, u32> =
            std::collections::HashMap::new();
        for &(res, t) in &state.stats.production_log {
            if t >= cutoff {
                *res_counts.entry(res).or_insert(0) += 1;
            }
        }
        if !res_counts.is_empty() {
            draw_text(
                "Production (per min):",
                sx,
                sy,
                14.0,
                Color::new(0.95, 0.82, 0.35, 0.9),
            );
            sy += 18.0;
            let mut sorted_res: Vec<(types::Resource, u32)> = res_counts.into_iter().collect();
            sorted_res.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
            let elapsed_mins = if tick > cutoff {
                (tick - cutoff) as f32 / 1200.0
            } else {
                1.0
            };
            for (res, count) in sorted_res.iter().take(10) {
                let rate = *count as f32 / elapsed_mins;
                draw_text(
                    &format!("{}: {:.1}/min", res.display_name(), rate),
                    sx,
                    sy,
                    12.0,
                    Color::new(0.6, 0.85, 0.6, 0.9),
                );
                sy += 15.0;
            }
            sy += 4.0;
        }

        let mut counts: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
        for (_, b) in state.buildings.iter() {
            *counts.entry(b.kind.display_name()).or_insert(0) += 1;
        }
        let mut sorted: Vec<(&&str, &u32)> = counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (name, count) in sorted.iter().take(12) {
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
        let (ax, mut ay) = draw_panel(
            px,
            py,
            pw,
            ph,
            Some("Achievement Roadmap — N to close"),
            true,
        );

        let completed = state.milestones_completed.iter().filter(|&&c| c).count();
        let total = milestones::MILESTONES.len();
        draw_text(
            &format!("{}/{} completed", completed, total),
            ax + 200.0,
            ay - 6.0,
            13.0,
            text_dim,
        );
        ay += 4.0;

        let mut last_phase = None;
        for (i, milestone) in milestones::MILESTONES.iter().enumerate() {
            if ay > py + ph - 16.0 {
                break;
            }

            // Phase header.
            if last_phase != Some(milestone.phase) {
                last_phase = Some(milestone.phase);
                let (pr, pg, pb) = milestone.phase.color();
                ay += 4.0;
                draw_text(
                    milestone.phase.label(),
                    ax,
                    ay,
                    14.0,
                    Color::new(pr, pg, pb, 0.9),
                );
                ay += 18.0;
            }

            let done = state.milestones_completed.get(i).copied().unwrap_or(false);
            let is_next = milestones::next_milestone(&state.milestones_completed) == Some(i);

            // Highlight the next goal.
            if is_next {
                draw_rectangle(
                    ax - 4.0,
                    ay - 12.0,
                    pw - 16.0,
                    36.0,
                    Color::new(0.15, 0.2, 0.3, 0.5),
                );
                draw_text(
                    ">>>",
                    ax - 2.0,
                    ay + 4.0,
                    12.0,
                    Color::new(0.9, 0.8, 0.3, 0.9),
                );
            }

            let icon = if done { "[X]" } else { "[ ]" };
            let name_color = if done {
                text_accent
            } else if is_next {
                Color::new(0.95, 0.85, 0.35, 1.0)
            } else {
                text_dim
            };
            draw_text(
                &format!("{} {}", icon, milestone.name),
                ax + 18.0,
                ay,
                14.0,
                name_color,
            );
            draw_text(milestone.description, ax + 180.0, ay, 11.0, text_dim);

            // Show reward for uncompleted milestones.
            if !done {
                let reward_str: String = milestone
                    .reward
                    .iter()
                    .map(|(r, c)| format!("{}x{}", c, short_resource_name(*r)))
                    .collect::<Vec<_>>()
                    .join(" ");
                draw_text(
                    &format!("Reward: {}", reward_str),
                    ax + 180.0,
                    ay + 14.0,
                    11.0,
                    Color::new(0.4, 0.8, 0.4, 0.6),
                );
            }

            ay += if is_next { 38.0 } else { 24.0 };
        }
    }

    // --- Help overlay (F1) ---
    if state.show_help {
        let sw = screen_width();
        let sh = screen_height();
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.4));
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
            ("B", "Blueprint (copies 5-tile radius, click to paste)"),
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
            ("F12", "Screenshot"),
        ];

        for (i, (key, desc)) in help.iter().enumerate() {
            let y = py + 55.0 + i as f32 * 17.0;
            if y > py + ph - 15.0 {
                break;
            }
            if key.is_empty() {
                continue;
            }
            if desc.is_empty() {
                // Section header
                draw_text(key, px + 20.0, y, 18.0, Color::new(0.7, 0.6, 0.9, 1.0));
            } else {
                draw_text(key, px + 20.0, y, 14.0, Color::new(0.9, 0.85, 0.4, 0.9));
                draw_text(desc, px + 200.0, y, 14.0, Color::new(0.75, 0.75, 0.8, 0.8));
            }
        }
        // Credits at bottom.
        draw_text(
            "AutoForge v0.2.0 — A narrative factory automation game",
            px + 20.0,
            py + ph - 28.0,
            12.0,
            Color::new(0.5, 0.5, 0.6, 0.5),
        );
        draw_text(
            "Built with Rust + macroquad | MIT License",
            px + 20.0,
            py + ph - 12.0,
            11.0,
            Color::new(0.4, 0.4, 0.5, 0.4),
        );
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
    let col_input = px + 170.0;
    let col_output = px + pw - 250.0;
    let col_raw = px + pw - 120.0;
    let start_y = py + 75.0;
    let row_h = 18.0;

    // Column headers.
    let header = Color::new(0.7, 0.7, 0.8, 0.7);
    draw_text("Recipe", col_name, start_y - 5.0, 14.0, header);
    draw_text("Inputs", col_input, start_y - 5.0, 14.0, header);
    draw_text("Output", col_output, start_y - 5.0, 14.0, header);
    draw_text("Raw ore", col_raw, start_y - 5.0, 14.0, header);

    for (i, r) in recipe::RECIPES.iter().enumerate() {
        let y = start_y + 10.0 + i as f32 * row_h;
        if y > py + ph - 20.0 {
            break;
        }

        // Recipe name.
        draw_text(r.name, col_name, y, 13.0, Color::new(0.9, 0.9, 0.95, 1.0));

        // Inputs.
        let inputs: String = r
            .inputs
            .iter()
            .map(|(res, c)| format!("{}x{}", c, short_resource_name(*res)))
            .collect::<Vec<_>>()
            .join("+");
        draw_text(&inputs, col_input, y, 12.0, Color::new(0.7, 0.8, 0.7, 0.9));

        // Outputs.
        let outputs: String = r
            .outputs
            .iter()
            .map(|(res, c)| format!("{}x{}", c, short_resource_name(*res)))
            .collect::<Vec<_>>()
            .join("+");
        draw_text(
            &outputs,
            col_output,
            y,
            12.0,
            Color::new(0.5, 0.9, 0.5, 0.9),
        );

        // Total raw-material cost of the primary output, expanded to ores.
        if let Some(&(primary, _)) = r.outputs.first() {
            let raw = recipe::raw_material_cost(primary);
            let mut entries: Vec<(types::Resource, f32)> = raw.into_iter().collect();
            entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let raw_text: String = entries
                .iter()
                .map(|(res, amt)| format!("{:.1}{}", amt, short_resource_name(*res)))
                .collect::<Vec<_>>()
                .join("+");
            draw_text(&raw_text, col_raw, y, 11.0, Color::new(0.8, 0.7, 0.5, 0.85));
        }
    }
}

/// Draws the research screen overlay.
fn draw_research_screen(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();

    // Darken background for modal consistency.
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.4));

    let pw = (sw * 0.75).min(750.0);
    let ph = (sh * 0.9).min(850.0);
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
            &format!(
                "Researching: {} ({}/{})",
                tech.name, state.research.progress, tech.units_needed
            ),
            px + 20.0,
            py + 80.0,
            18.0,
            YELLOW,
        );
        // Progress bar
        draw_rectangle(
            px + 20.0,
            py + 85.0,
            pw - 40.0,
            8.0,
            Color::new(0.2, 0.2, 0.2, 1.0),
        );
        draw_rectangle(px + 20.0, py + 85.0, (pw - 40.0) * progress_pct, 8.0, GREEN);
    } else {
        draw_text(
            "No active research",
            px + 20.0,
            py + 80.0,
            18.0,
            Color::new(0.6, 0.6, 0.6, 1.0),
        );
    }

    // Tech list with prerequisite lines.
    let start_y = py + 110.0;
    let row_h = 24.0;
    let col1 = px + 20.0;
    let col2 = px + 220.0;

    // Draw prerequisite connection lines FIRST (behind text).
    for (i, tech) in research::TECHNOLOGIES.iter().enumerate() {
        let y = start_y + i as f32 * row_h;
        if y > py + ph - 20.0 {
            break;
        }
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

        draw_text(&format!("{} {}", tech.name, status), col1, y, 14.0, color);
        draw_text(
            tech.description,
            col2,
            y,
            13.0,
            Color::new(0.5, 0.5, 0.6, 0.8),
        );

        // Click to start research
        if can_research && !is_current {
            let mouse = Vec2::new(mouse_position().0, mouse_position().1);
            if mouse.x >= col1
                && mouse.x <= col1 + pw - 40.0
                && mouse.y >= y - 16.0
                && mouse.y <= y + 8.0
            {
                // Highlight on hover (full row width).
                draw_rectangle(
                    col1 - 5.0,
                    y - 16.0,
                    pw - 40.0,
                    row_h,
                    Color::new(0.2, 0.3, 0.5, 0.3),
                );
            }
        }
    }
}
