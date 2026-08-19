//! Modal popup panels: blueprint library picker and recipe picker.

use macroquad::prelude::*;

use crate::game::GameState;
use crate::recipe;
use crate::types;

use super::{draw_panel, short_resource_name, TEXT_BRIGHT};

pub(super) fn draw_blueprint_picker(state: &mut GameState) {
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
                TEXT_BRIGHT,
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
}

pub(super) fn draw_recipe_picker(state: &mut GameState) {
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
}
