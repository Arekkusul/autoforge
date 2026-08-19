//! Bottom toolbar (building hotbar) and the selected-building info card
//! that floats above it.

use macroquad::prelude::*;

use crate::buildcost;
use crate::game::GameState;
use crate::sprites::SpriteAtlas;
use crate::types;

use super::{
    building_description, draw_panel, short_resource_name, PANEL_BG, PANEL_BORDER, SELECTED_BG,
    SELECTED_BORDER, TEXT_BRIGHT, TEXT_DIM,
};

pub(super) fn draw_toolbar(state: &GameState, atlas: &SpriteAtlas) {
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
            draw_rectangle(x + 2.0, y, slot_w - 4.0, slot_h, SELECTED_BG);
            draw_rectangle_lines(x + 2.0, y, slot_w - 4.0, slot_h, 2.0, SELECTED_BORDER);
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
            if is_selected { TEXT_BRIGHT } else { TEXT_DIM },
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
}

pub(super) fn draw_selected_building_info(state: &GameState) {
    let toolbar_y = screen_height() - 88.0;
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
            PANEL_BG,
        );
        draw_rectangle_lines(
            panel_x,
            toolbar_y - panel_h - 4.0,
            total_w,
            panel_h,
            1.0,
            PANEL_BORDER,
        );
        draw_text(
            &info,
            panel_x + 10.0,
            toolbar_y - panel_h + 14.0,
            18.0,
            TEXT_BRIGHT,
        );
        if !desc.is_empty() {
            draw_text(
                desc,
                panel_x + 10.0,
                toolbar_y - panel_h + 30.0,
                12.0,
                TEXT_DIM,
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
}
