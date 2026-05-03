//! Building placement, storage, and management.
//!
//! Buildings are stored in a generational arena (like [`ItemPool`](crate::item::ItemPool))
//! for O(1) access by [`BuildingId`]. The [`Buildings`] struct handles placement
//! validation, grid registration, and removal.

use serde::{Deserialize, Serialize};

use crate::grid::Grid;
use crate::recipe::RecipeId;
use crate::types::*;

/// Machine-specific state for production buildings (smelters, assemblers, etc.).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MachineState {
    /// Items waiting to be processed.
    pub input_buffer: Vec<Resource>,
    /// Items finished, waiting to be ejected onto an output belt.
    pub output_buffer: Vec<Resource>,
    /// Ticks remaining on the current crafting job. 0 = idle.
    pub progress_ticks: u32,
    /// Total ticks for the current recipe (used for progress bar rendering).
    pub total_ticks: u32,
    /// Fuel remaining (for coal-fueled buildings like stone/steel furnace, boiler).
    pub fuel_ticks: u32,
    /// The recipe this machine is currently crafting. `None` = idle / auto-detect.
    pub selected_recipe: Option<RecipeId>,
}

impl MachineState {
    /// Creates a new empty machine state.
    pub fn new() -> Self {
        Self {
            input_buffer: Vec::new(),
            output_buffer: Vec::new(),
            progress_ticks: 0,
            total_ticks: 0,
            fuel_ticks: 0,
            selected_recipe: None,
        }
    }
}

/// A building placed on the grid.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Building {
    /// What kind of building this is.
    pub kind: BuildingKind,
    /// Grid position of this building.
    pub pos: GridPos,
    /// Direction the building faces (output direction for machines, flow for belts).
    pub direction: Direction,
    /// Machine state (only for production buildings, `None` for belts/walls/etc.).
    pub machine_state: Option<MachineState>,
    /// Hit points (for destructible buildings like walls, turrets).
    pub hp: f32,
    /// Maximum hit points.
    pub max_hp: f32,
    /// For underground belt entry: position of the paired exit. `None` if unpaired.
    pub underground_pair: Option<GridPos>,
}

/// Internal slot in the building arena.
struct BuildingSlot {
    building: Building,
    alive: bool,
    generation: u32,
}

/// Arena storage for all placed buildings.
///
/// Uses generational indices for safe, O(1) access by [`BuildingId`].
pub struct Buildings {
    slots: Vec<BuildingSlot>,
    free_list: Vec<u32>,
}

impl Buildings {
    /// Creates a new empty building storage.
    pub fn new() -> Self {
        Self {
            slots: Vec::with_capacity(1024),
            free_list: Vec::new(),
        }
    }

    /// Attempts to place a building on the grid.
    ///
    /// Returns `Some(BuildingId)` on success, or `None` if the tile is occupied
    /// or otherwise invalid for this building type.
    pub fn place(&mut self, building: Building, grid: &mut Grid) -> Option<BuildingId> {
        let pos = building.pos;

        // Validate placement.
        let tile = grid.get_tile(pos)?;
        if tile.building.is_some() {
            return None; // tile already occupied
        }
        // Don't place non-belt buildings on tiles that have items (would orphan them).
        if !building.kind.is_belt() && !grid.items_at(pos).is_empty() {
            return None;
        }
        if !tile.terrain.is_buildable() {
            // Exception: water pump can be placed adjacent to water.
            if building.kind != BuildingKind::WaterPump {
                return None;
            }
        }
        // Miners must be on a deposit.
        if building.kind == BuildingKind::Miner {
            if tile.deposit.is_none() || tile.deposit == Some(OreDeposit::Oil) {
                return None;
            }
        }
        // Pump jacks must be on oil.
        if building.kind == BuildingKind::PumpJack {
            if tile.deposit != Some(OreDeposit::Oil) {
                return None;
            }
        }

        // Insert into arena.
        let id = if let Some(index) = self.free_list.pop() {
            let i = index as usize;
            let gen = self.slots[i].generation + 1;
            self.slots[i] = BuildingSlot {
                building,
                alive: true,
                generation: gen,
            };
            BuildingId {
                index,
                generation: gen,
            }
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(BuildingSlot {
                building,
                alive: true,
                generation: 0,
            });
            BuildingId {
                index,
                generation: 0,
            }
        };

        // Register on grid.
        if let Some(tile) = grid.get_tile_mut(pos) {
            tile.building = Some(id);
        }

        Some(id)
    }

    /// Removes a building from the grid and returns it to the free list.
    pub fn remove(&mut self, id: BuildingId, grid: &mut Grid) {
        let i = id.index as usize;
        if i < self.slots.len()
            && self.slots[i].alive
            && self.slots[i].generation == id.generation
        {
            let pos = self.slots[i].building.pos;
            self.slots[i].alive = false;
            self.free_list.push(id.index);

            // Unregister from grid.
            if let Some(tile) = grid.get_tile_mut(pos) {
                tile.building = None;
            }
        }
    }

    /// Returns a reference to a building if the handle is valid.
    pub fn get(&self, id: BuildingId) -> Option<&Building> {
        let i = id.index as usize;
        if i < self.slots.len()
            && self.slots[i].alive
            && self.slots[i].generation == id.generation
        {
            Some(&self.slots[i].building)
        } else {
            None
        }
    }

    /// Returns a mutable reference to a building if the handle is valid.
    pub fn get_mut(&mut self, id: BuildingId) -> Option<&mut Building> {
        let i = id.index as usize;
        if i < self.slots.len()
            && self.slots[i].alive
            && self.slots[i].generation == id.generation
        {
            Some(&mut self.slots[i].building)
        } else {
            None
        }
    }

    /// Iterates over all alive buildings with their IDs.
    pub fn iter(&self) -> impl Iterator<Item = (BuildingId, &Building)> {
        self.slots.iter().enumerate().filter_map(|(i, slot)| {
            if slot.alive {
                Some((
                    BuildingId {
                        index: i as u32,
                        generation: slot.generation,
                    },
                    &slot.building,
                ))
            } else {
                None
            }
        })
    }

    /// Collects all alive building IDs (for iteration that needs mutation).
    pub fn alive_ids(&self) -> Vec<BuildingId> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| {
                if slot.alive {
                    Some(BuildingId {
                        index: i as u32,
                        generation: slot.generation,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
