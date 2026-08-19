//! Full-screen overlay panels: production stats (V), achievements (N), help (F1).

use macroquad::prelude::*;

use crate::game::GameState;
use crate::milestones;
use crate::types;

use super::{draw_panel, fmt_num, short_resource_name, TEXT_ACCENT, TEXT_BRIGHT, TEXT_DIM};

pub(super) fn draw_stats_screen(state: &mut GameState) {
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
            TEXT_BRIGHT,
        );
        sy += 20.0;
        draw_text(
            &format!("Items crafted: {}", state.stats.items_crafted),
            sx,
            sy,
            14.0,
            TEXT_BRIGHT,
        );
        sy += 20.0;
        draw_text(
            &format!("Production rate: {:.1}/min", items_per_min),
            sx,
            sy,
            14.0,
            TEXT_ACCENT,
        );
        sy += 20.0;
        draw_text(
            &format!("Buildings placed: {}", state.stats.buildings_placed),
            sx,
            sy,
            14.0,
            TEXT_BRIGHT,
        );
        sy += 20.0;
        draw_text(
            &format!("Enemies killed: {}", state.stats.enemies_killed),
            sx,
            sy,
            14.0,
            TEXT_BRIGHT,
        );
        sy += 20.0;
        draw_text(
            &format!(
                "Inventory: {} items ({} stacks)",
                fmt_num(state.total_inventory_items()),
                state.inventory_stacks()
            ),
            sx,
            sy,
            14.0,
            TEXT_BRIGHT,
        );
        sy += 20.0;
        draw_text(
            &format!(
                "Belt throughput: Y {:.0}/s  R {:.0}/s  B {:.0}/s",
                types::BuildingKind::BeltYellow.belt_items_per_second(),
                types::BuildingKind::BeltRed.belt_items_per_second(),
                types::BuildingKind::BeltBlue.belt_items_per_second(),
            ),
            sx,
            sy,
            14.0,
            TEXT_DIM,
        );
        sy += 20.0;
        draw_text(
            &format!("Rockets launched: {}", state.stats.rockets_launched),
            sx,
            sy,
            14.0,
            TEXT_BRIGHT,
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
        draw_text(&seed_str, sx, sy, 13.0, TEXT_DIM);
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
        // Per-resource production rates (items/min from the last 60 seconds).
        let rates = state.stats.production_rates(1200);
        if !rates.is_empty() {
            draw_text(
                "Production (per min):",
                sx,
                sy,
                14.0,
                Color::new(0.95, 0.82, 0.35, 0.9),
            );
            sy += 18.0;
            for (res, rate) in rates.iter().take(10) {
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
            draw_text(&format!("{}: {}", name, count), sx, sy, 13.0, TEXT_DIM);
            sy += 16.0;
        }
    }
}

pub(super) fn draw_achievements_screen(state: &GameState) {
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
            TEXT_DIM,
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
                TEXT_ACCENT
            } else if is_next {
                Color::new(0.95, 0.85, 0.35, 1.0)
            } else {
                TEXT_DIM
            };
            draw_text(
                &format!("{} {}", icon, milestone.name),
                ax + 18.0,
                ay,
                14.0,
                name_color,
            );
            draw_text(milestone.description, ax + 180.0, ay, 11.0, TEXT_DIM);

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
}

pub(super) fn draw_help_overlay(state: &GameState) {
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
}
