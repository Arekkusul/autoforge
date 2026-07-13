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
    // v2 fields (optional for backward compat with old saves).
    #[serde(default)]
    pub inventory: Vec<(Resource, u32)>,
    #[serde(default)]
    pub research_completed: Vec<bool>,
    #[serde(default)]
    pub research_current: Option<usize>,
    #[serde(default)]
    pub research_progress: u32,
    #[serde(default)]
    pub story_triggered: Vec<bool>,
    #[serde(default)]
    pub story_first_miner: bool,
    #[serde(default)]
    pub story_first_wave: bool,
    #[serde(default)]
    pub daynight_time: f32,
    #[serde(default)]
    pub game_speed: u32,
    #[serde(default)]
    pub build_radius: f32,
    #[serde(default)]
    pub game_won: bool,
    #[serde(default)]
    pub milestones_completed: Vec<bool>,
    #[serde(default)]
    pub tutorial_step: u32,
    #[serde(default)]
    pub blueprint_library: Vec<(String, Vec<(i32, i32, BuildingKind, Direction)>)>,
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
    #[serde(default)]
    pub modules: Vec<Resource>,
}

/// Serialized item on a belt.
#[derive(Serialize, Deserialize)]
pub struct SaveItem {
    pub resource: Resource,
    pub x: i32,
    pub y: i32,
    pub progress: f32,
}

impl SaveData {
    /// Serializes this save to a binary (bincode) byte buffer.
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserializes a save from a binary (bincode) byte buffer.
    pub fn from_bytes(bytes: &[u8]) -> Result<SaveData, bincode::Error> {
        bincode::deserialize(bytes)
    }
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
        version: 2,
        seed: state.seed,
        grid_width: state.grid.width,
        grid_height: state.grid.height,
        stats: state.stats.clone(),
        evolution: state.evolution,
        nests: state.nests.iter().map(|p| (p.x, p.y)).collect(),
        tiles: Vec::new(),
        buildings: Vec::new(),
        items: Vec::new(),
        // v2: preserve critical state across save/load.
        inventory: state.inventory.iter().map(|(r, c)| (*r, *c)).collect(),
        research_completed: state.research.completed.clone(),
        research_current: state.research.current_tech,
        research_progress: state.research.progress,
        story_triggered: state.story.triggered.clone(),
        story_first_miner: state.story.first_miner_placed,
        story_first_wave: state.story.first_wave_arrived,
        daynight_time: state.daynight.time,
        game_speed: state.game_speed,
        build_radius: state.build_radius,
        game_won: state.game_won,
        milestones_completed: state.milestones_completed.clone(),
        tutorial_step: state.tutorial_step,
        blueprint_library: state.blueprint_library.clone(),
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
            modules: ms.map(|m| m.modules.clone()).unwrap_or_default(),
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

    // Save as binary (bincode) — atomic write via temp file + rename.
    if let Ok(bytes) = save.to_bytes() {
        let temp = save_path().with_extension("tmp");
        if fs::write(&temp, &bytes).is_ok() {
            if fs::rename(&temp, save_path()).is_ok() {
                return true;
            }
            // Rename failed — try direct write as fallback.
            let _ = fs::remove_file(&temp);
        }
        // Fallback: direct write (less safe but works on all platforms).
        if fs::write(save_path(), bytes).is_ok() {
            return true;
        }
    }
    false
}

/// Loads a saved game from disk, replacing the current state.
///
/// Returns `true` on success.
pub fn load_game(state: &mut GameState) -> bool {
    // Try binary (bincode) first, fall back to JSON for old saves.
    let save: SaveData = if let Ok(bytes) = fs::read(save_path()) {
        match SaveData::from_bytes(&bytes) {
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
        let needs_ms =
            !sb.kind.is_belt() && !matches!(sb.kind, BuildingKind::Wall | BuildingKind::Gate);

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
                    selected_recipe: sb
                        .selected_recipe
                        .filter(|&idx| idx < crate::recipe::RECIPES.len())
                        .map(RecipeId),
                    modules: sb.modules.clone(),
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

    // Reconstruct underground belt pairs.
    let ug_ids = buildings.alive_ids();
    for bid in &ug_ids {
        let (kind, pos, dir) = match buildings.get(*bid) {
            Some(b) if b.kind.is_underground_belt() => (b.kind, b.pos, b.direction),
            _ => continue,
        };
        // Search forward for a matching underground belt exit.
        let mut check = pos;
        for _ in 1..=6 {
            check = check.neighbor(dir);
            if let Some(tile) = grid.get_tile(check) {
                if let Some(other_bid) = tile.building {
                    if let Some(other) = buildings.get(other_bid) {
                        if other.kind == kind && other.direction == dir && *bid != other_bid {
                            if let Some(b) = buildings.get_mut(*bid) {
                                b.underground_pair = Some(check);
                            }
                            break;
                        }
                    }
                }
            }
        }
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
    state.nests = save
        .nests
        .iter()
        .map(|&(x, y)| GridPos::new(x, y))
        .collect();
    state.seed = save.seed;

    // Restore v2 fields (gracefully handles old saves via serde defaults).
    if !save.inventory.is_empty() {
        state.inventory.clear();
        for (r, c) in &save.inventory {
            state.inventory.insert(*r, *c);
        }
    }
    if !save.research_completed.is_empty() {
        // Ensure completed vec is large enough for all technologies (forward compat).
        let mut completed = save.research_completed;
        completed.resize(crate::research::TECHNOLOGIES.len(), false);
        state.research.completed = completed;
        state.research.current_tech = save
            .research_current
            .filter(|&idx| idx < crate::research::TECHNOLOGIES.len());
        state.research.progress = save.research_progress;
    }
    if !save.story_triggered.is_empty() {
        // Ensure triggered vec is large enough for all story beats.
        let mut triggered = save.story_triggered;
        triggered.resize(crate::story::STORY_BEATS.len(), false);
        state.story.triggered = triggered;
        state.story.first_miner_placed = save.story_first_miner;
        state.story.first_wave_arrived = save.story_first_wave;
    }
    if save.daynight_time > 0.0 {
        state.daynight.time = save.daynight_time;
    }
    if save.game_speed > 0 {
        state.game_speed = save.game_speed;
    }
    if save.build_radius > 0.0 {
        state.build_radius = save.build_radius;
    }
    state.game_won = save.game_won;

    // Milestones: restore and resize for forward compat.
    if !save.milestones_completed.is_empty() {
        let mut mc = save.milestones_completed;
        mc.resize(crate::milestones::MILESTONES.len(), false);
        state.milestones_completed = mc;
    }

    // Tutorial state.
    if save.tutorial_step > 0 {
        state.tutorial_step = save.tutorial_step;
        state.show_tutorial = state.tutorial_step < 6;
    }

    // Blueprint library.
    state.blueprint_library = save.blueprint_library;

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_save() -> SaveData {
        SaveData {
            version: 2,
            seed: 0xDEAD_BEEF,
            grid_width: 64,
            grid_height: 64,
            stats: GameStats {
                total_ticks: 12_345,
                items_crafted: 999,
                buildings_placed: 42,
                ..Default::default()
            },
            evolution: 0.25,
            nests: vec![(10, 20), (30, 40)],
            tiles: vec![SaveTile {
                x: 5,
                y: 6,
                terrain: Terrain::Grass,
                deposit: Some(OreDeposit::Iron),
                ore_amount: 500,
                ore_origin: true,
                pollution: 1.5,
            }],
            buildings: vec![SaveBuilding {
                kind: BuildingKind::Miner,
                x: 5,
                y: 6,
                direction: Direction::South,
                hp: 100.0,
                max_hp: 100.0,
                input_buffer: vec![Resource::Coal],
                output_buffer: vec![Resource::IronOre, Resource::IronOre],
                progress_ticks: 10,
                total_ticks: 40,
                fuel_ticks: 100,
                selected_recipe: Some(0),
                modules: vec![Resource::SpeedModule],
            }],
            items: vec![SaveItem {
                resource: Resource::IronPlate,
                x: 7,
                y: 8,
                progress: 0.5,
            }],
            inventory: vec![(Resource::IronPlate, 50), (Resource::Coal, 30)],
            research_completed: vec![true, false, true],
            research_current: Some(3),
            research_progress: 7,
            story_triggered: vec![true, true, false],
            story_first_miner: true,
            story_first_wave: false,
            daynight_time: 123.4,
            game_speed: 2,
            build_radius: 45.0,
            game_won: false,
            milestones_completed: vec![true, false],
            tutorial_step: 4,
            blueprint_library: vec![(
                "line".to_string(),
                vec![(0, 0, BuildingKind::BeltYellow, Direction::East)],
            )],
        }
    }

    #[test]
    fn round_trip_preserves_core_fields() {
        let save = sample_save();
        let bytes = save.to_bytes().expect("serialize");
        let back = SaveData::from_bytes(&bytes).expect("deserialize");

        assert_eq!(back.version, save.version);
        assert_eq!(back.seed, save.seed);
        assert_eq!(back.grid_width, save.grid_width);
        assert_eq!(back.stats.items_crafted, save.stats.items_crafted);
        assert_eq!(back.stats.total_ticks, save.stats.total_ticks);
        assert_eq!(back.evolution, save.evolution);
        assert_eq!(back.nests, save.nests);
        assert_eq!(back.inventory, save.inventory);
        assert_eq!(back.research_completed, save.research_completed);
        assert_eq!(back.research_current, save.research_current);
    }

    #[test]
    fn round_trip_preserves_buildings_and_items() {
        let save = sample_save();
        let back = SaveData::from_bytes(&save.to_bytes().unwrap()).unwrap();

        assert_eq!(back.buildings.len(), 1);
        let b = &back.buildings[0];
        assert_eq!(b.kind, BuildingKind::Miner);
        assert_eq!(b.output_buffer, vec![Resource::IronOre, Resource::IronOre]);
        assert_eq!(b.selected_recipe, Some(0));
        assert_eq!(b.modules, vec![Resource::SpeedModule]);

        assert_eq!(back.items.len(), 1);
        assert_eq!(back.items[0].resource, Resource::IronPlate);
        assert_eq!(back.items[0].progress, 0.5);
    }

    #[test]
    fn serialization_is_deterministic() {
        let save = sample_save();
        assert_eq!(save.to_bytes().unwrap(), save.to_bytes().unwrap());
    }

    #[test]
    fn truncated_bytes_fail_gracefully() {
        // A valid save prefix cut short must not panic — it returns an error.
        let bytes = sample_save().to_bytes().unwrap();
        let truncated = &bytes[..bytes.len() / 2];
        assert!(SaveData::from_bytes(truncated).is_err());
    }

    #[test]
    fn empty_bytes_fail_gracefully() {
        assert!(SaveData::from_bytes(&[]).is_err());
    }
}
