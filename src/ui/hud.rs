//! Always-on HUD chrome: status panel, victory/pause overlays, toast
//! notifications, the controls-hint strip, and the tutorial overlay.

use macroquad::prelude::*;

use crate::game::GameState;
use crate::research;
use crate::save;
use crate::sprites::SpriteAtlas;
use crate::types;

use super::{draw_panel, fmt_num, TEXT_DIM};

pub(super) fn draw_status_panel(state: &mut GameState) {
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
        TEXT_DIM,
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
        draw_text("No demand", cx + bar_w + 6.0, cy + 10.0, 11.0, TEXT_DIM);
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
        TEXT_DIM,
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
        TEXT_DIM,
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
            TEXT_DIM,
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
}

pub(super) fn draw_victory_overlay(state: &mut GameState, atlas: &SpriteAtlas) {
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
}

pub(super) fn draw_toast_notifications(state: &GameState) {
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
}

pub(super) fn draw_controls_hint(state: &GameState) {
    let toolbar_y = screen_height() - 88.0;
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
}

pub(super) fn draw_tutorial_overlay(state: &GameState) {
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
}
