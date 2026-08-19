//! Recipe browser (E key) and research tree (Tab key) overlay screens.

use macroquad::prelude::*;

use crate::game::GameState;
use crate::recipe;
use crate::research;
use crate::types;

use super::{draw_panel, short_resource_name};

/// Draws the recipe browser overlay (E key).
pub(super) fn draw_recipe_browser() {
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
pub(super) fn draw_research_screen(state: &GameState) {
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
    // Overall tech-tree completion summary.
    draw_text(
        &format!(
            "Tech tree: {:.0}% complete  |  {} available now",
            state.research.percent_complete() * 100.0,
            state.research.available_techs().len(),
        ),
        px + pw - 300.0,
        py + 55.0,
        13.0,
        Color::new(0.55, 0.8, 0.95, 0.9),
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
        // Total science-pack cost to complete this technology.
        if !completed {
            draw_text(
                &format!("{} sci", state.research.total_science_cost(i)),
                px + pw - 70.0,
                y,
                12.0,
                Color::new(0.7, 0.65, 0.45, 0.8),
            );
        }

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
