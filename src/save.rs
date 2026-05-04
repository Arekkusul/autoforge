//! Save and load game state to/from JSON files.
//!
//! Uses flat, non-referential structs for serialization so that building/item IDs
//! are reassigned during load. The save file is human-readable JSON stored next
//! to the executable.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::building::{Building, Buildings, MachineState};
use crate::game::{GameState, GameStats};
use crate::grid::Grid;
use crate::item::ItemPool;
use crate::recipe::RecipeId;
use crate::types::*;

/// Top-level save data structure.
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub seed: u64,
    pub grid_width: i32,
    pub grid_height: i32,
    pub stats: GameStats,
    pub evolution: f64,
    pub nests: Vec<(i32, i32)>,
    pub tiles: Vec<SaveTile>,
    pub buildings: Vec<SaveBuilding>,
    pub items: Vec<SaveItem>,
}

/// Serialized tile (only non-default tiles are saved for efficiency).
#[derive(Serialize, Deserialize)]
pub struct SaveTile {
    pub x: i32,
    pub y: i32,
    pub terrain: Terrain,
    pub deposit: Option<OreDeposit>,
    pub ore_amount: u32,
    pub ore_origin: bool,
    pub pollution: f32,
}

/// Serialized building.
#[derive(Serialize, Deserialize)]
pub struct SaveBuilding {
    pub kind: BuildingKind,
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
    pub hp: f32,
    pub max_hp: f32,
    pub input_buffer: Vec<Resource>,
    pub output_buffer: Vec<Resource>,
    pub progress_ticks: u32,
    pub total_ticks: u32,
    pub fuel_ticks: u32,
    pub selected_recipe: Option<usize>,
}

/// Serialized item on a belt.
#[derive(Serialize, Deserialize)]
pub struct SaveItem {
    pub resource: Resource,
    pub x: i32,
    pub y: i32,
    pub progress: f32,
}

/// Returns the save file path (next to the executable).
fn save_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_default();
    path.push("autoforge_save.bin");
    path
}

fn save_path_json() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_default();
    path.push("autoforge_save.json");
    path
}

/// Saves the current game state to disk.
///
/// Returns `true` on success.
pub fn save_game(state: &GameState) -> bool {
    let mut save = SaveData {
        version: 1,
        seed: state.seed,
        grid_width: state.grid.width,
        grid_height: state.grid.height,
        stats: state.stats.clone(),
        evolution: state.evolution,
        nests: state.nests.iter().map(|p| (p.x, p.y)).collect(),
        tiles: Vec::new(),
        buildings: Vec::new(),
        items: Vec::new(),
    };

    // Save tiles that differ from default (have deposits, pollution, or non-grass terrain).
    for y in 0..state.grid.height {
        for x in 0..state.grid.width {
            let pos = GridPos::new(x, y);
            if let Some(tile) = state.grid.get_tile(pos) {
                if tile.deposit.is_some()
                    || tile.terrain != Terrain::Grass
                    || tile.pollution > 0.001
                    || tile.ore_origin
                {
                    save.tiles.push(SaveTile {
                        x,
                        y,
                        terrain: tile.terrain,
                        deposit: tile.deposit,
                        ore_amount: tile.ore_amount,
                        ore_origin: tile.ore_origin,
                        pollution: tile.pollution,
                    });
                }
            }
        }
    }

    // Save buildings.
    for (_, b) in state.buildings.iter() {
        let ms = b.machine_state.as_ref();
        save.buildings.push(SaveBuilding {
            kind: b.kind,
            x: b.pos.x,
            y: b.pos.y,
            direction: b.direction,
            hp: b.hp,
            max_hp: b.max_hp,
            input_buffer: ms.map(|m| m.input_buffer.clone()).unwrap_or_default(),
            output_buffer: ms.map(|m| m.output_buffer.clone()).unwrap_or_default(),
            progress_ticks: ms.map(|m| m.progress_ticks).unwrap_or(0),
            total_ticks: ms.map(|m| m.total_ticks).unwrap_or(0),
            fuel_ticks: ms.map(|m| m.fuel_ticks).unwrap_or(0),
            selected_recipe: ms.and_then(|m| m.selected_recipe.map(|r| r.0)),
        });
    }

    // Save items.
    for (_, item) in state.items.iter() {
        save.items.push(SaveItem {
            resource: item.resource,
            x: item.pos.x,
            y: item.pos.y,
            progress: item.progress,
        });
    }

    // Save as binary (bincode) — much smaller and faster than JSON.
    match bincode::serialize(&save) {
        Ok(bytes) => {
            if fs::write(save_path(), bytes).is_ok() {
                return true;
            }
        }
        Err(_) => {}
    }
    false
}

/// Loads a saved game from disk, replacing the current state.
///
/// Returns `true` on success.
pub fn load_game(state: &mut GameState) -> bool {
    // Try binary (bincode) first, fall back to JSON for old saves.
    let save: SaveData = if let Ok(bytes) = fs::read(save_path()) {
        match bincode::deserialize(&bytes) {
            Ok(s) => s,
            Err(_) => return false,
        }
    } else if let Ok(json) = fs::read_to_string(save_path_json()) {
        match serde_json::from_str(&json) {
            Ok(s) => s,
            Err(_) => return false,
        }
    } else {
        return false;
    };

    // Rebuild grid.
    let mut grid = Grid::new(save.grid_width, save.grid_height);
    for st in &save.tiles {
        if let Some(tile) = grid.get_tile_mut(GridPos::new(st.x, st.y)) {
            tile.terrain = st.terrain;
            tile.deposit = st.deposit;
            tile.ore_amount = st.ore_amount;
            tile.ore_origin = st.ore_origin;
            tile.pollution = st.pollution;
        }
    }

    // Rebuild buildings.
    let mut buildings = Buildings::new();
    for sb in &save.buildings {
        let needs_ms = !sb.kind.is_belt()
            && !matches!(sb.kind, BuildingKind::Wall | BuildingKind::Gate);

        let b = Building {
            kind: sb.kind,
            pos: GridPos::new(sb.x, sb.y),
            direction: sb.direction,
            machine_state: if needs_ms {
                Some(MachineState {
                    input_buffer: sb.input_buffer.clone(),
                    output_buffer: sb.output_buffer.clone(),
                    progress_ticks: sb.progress_ticks,
                    total_ticks: sb.total_ticks,
                    fuel_ticks: sb.fuel_ticks,
                    selected_recipe: sb.selected_recipe.map(RecipeId),
                })
            } else {
                None
            },
            hp: sb.hp,
            max_hp: sb.max_hp,
            underground_pair: None,
        };
        buildings.place(b, &mut grid);
    }

    // Rebuild items.
    let mut items = ItemPool::new(4096);
    for si in &save.items {
        let pos = GridPos::new(si.x, si.y);
        let id = items.spawn(si.resource, pos);
        if let Some(item) = items.get_mut(id) {
            item.progress = si.progress;
        }
        grid.add_item_to_tile(pos, id);
    }

    state.grid = grid;
    state.buildings = buildings;
    state.items = items;
    state.stats = save.stats;
    state.evolution = save.evolution;
    state.nests = save.nests.iter().map(|&(x, y)| GridPos::new(x, y)).collect();
    state.seed = save.seed;

    true
}
