//! Left-side inventory panel and the roadmap/milestone goal panel below it.

use macroquad::prelude::*;

use crate::game::GameState;
use crate::milestones;
use crate::types;

use super::{draw_panel, short_resource_name, TEXT_BRIGHT};

/// Draws the inventory panel and, positioned dynamically below it, the
/// roadmap goal panel showing the next uncompleted milestone.
pub(super) fn draw_inventory_and_goals(state: &GameState) {
    let status_h = if state.research.current_tech.is_some() {
        138.0
    } else {
        120.0
    };

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
                        TEXT_BRIGHT,
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
}
