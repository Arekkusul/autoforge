use macroquad::prelude::*;

use crate::game::GameState;
use crate::sprites::SpriteAtlas;
use crate::types;

mod hud;
mod inventory;
mod popups;
mod recipe_research;
mod screens;
mod toolbar;
mod world_info;

use hud::{
    draw_controls_hint, draw_status_panel, draw_toast_notifications, draw_tutorial_overlay,
    draw_victory_overlay,
};
use inventory::draw_inventory_and_goals;
use popups::{draw_blueprint_picker, draw_recipe_picker};
use recipe_research::{draw_recipe_browser, draw_research_screen};
use screens::{draw_achievements_screen, draw_help_overlay, draw_stats_screen};
use toolbar::{draw_selected_building_info, draw_toolbar};
use world_info::{draw_hovered_tile_info, draw_minimap};

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

// Modern UI colors — clean, high contrast, readable. Shared across all UI
// drawing sections (hoisted here so section functions don't need to thread
// them through as parameters).
const PANEL_BG: Color = Color::new(0.08, 0.08, 0.12, 0.92);
const PANEL_BORDER: Color = Color::new(0.25, 0.25, 0.35, 0.6);
const TEXT_BRIGHT: Color = Color::new(0.98, 0.98, 0.98, 1.0);
const TEXT_DIM: Color = Color::new(0.65, 0.65, 0.7, 1.0);
const TEXT_ACCENT: Color = Color::new(0.45, 0.92, 0.55, 1.0);
const SELECTED_BG: Color = Color::new(0.15, 0.35, 0.55, 0.8);
const SELECTED_BORDER: Color = Color::new(0.45, 0.75, 1.0, 1.0);

/// Draws the user interface, including status panels, overlays, and tooltips.
pub fn draw_ui(state: &mut GameState, atlas: &SpriteAtlas) {
    draw_status_panel(state);

    draw_victory_overlay(state, atlas);

    draw_hovered_tile_info(state);

    draw_toolbar(state, atlas);

    draw_selected_building_info(state);

    draw_minimap(state);

    draw_blueprint_picker(state);

    draw_toast_notifications(state);

    draw_controls_hint(state);

    draw_tutorial_overlay(state);

    draw_inventory_and_goals(state);

    draw_recipe_picker(state);

    draw_stats_screen(state);

    draw_achievements_screen(state);

    draw_help_overlay(state);

    // --- Recipe Browser (E key) ---
    if state.show_recipes {
        draw_recipe_browser();
    }

    // --- Research screen overlay (Tab) ---
    if state.show_research {
        draw_research_screen(state);
    }
}
