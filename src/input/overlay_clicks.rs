//! Click handling for the research screen and the bottom toolbar.
//! Returns `true` if the click was consumed here, signaling the caller
//! to skip building-placement input for the rest of this frame.

use macroquad::prelude::*;

use crate::game::GameState;
use crate::research;
use crate::types;

pub(super) fn handle_overlay_and_toolbar_clicks(state: &mut GameState) -> bool {
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
        return true; // don't process building placement when research screen is open
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
        return true;
    }

    // Don't process mouse placement if cursor is over the toolbar.
    if mouse_position().1 > toolbar_y {
        return true;
    }

    false
}
