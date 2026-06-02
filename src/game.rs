//! Top-level game state and initialization.
//!
//! [`GameState`] holds all data for a running game session: the grid, camera,
//! simulation state, and UI state. The main loop in [`crate::main`] owns one
//! `GameState` and passes it to input, simulation, and rendering systems.

use std::collections::HashMap;

use crate::building::Buildings;
use crate::camera::GameCamera;
use crate::recipe;
use crate::constants::*;
use crate::daynight::DayNightState;
use crate::enemy::Enemies;
use crate::grid::Grid;
use crate::item::ItemPool;
use crate::mapgen;
use crate::power::PowerState;
use crate::research::ResearchState;
use crate::story::StoryState;
use crate::train::Trains;
use crate::types::*;
use serde::{Deserialize, Serialize};

/// Persistent gameplay statistics tracked across the session.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct GameStats {
    /// Total simulation ticks elapsed.
    pub total_ticks: u64,
    /// Total rockets launched.
    pub rockets_launched: u32,
    /// Total items crafted (all types combined).
    pub items_crafted: u64,
    /// Total buildings placed over the session.
    pub buildings_placed: u64,
    /// Total enemies killed.
    pub enemies_killed: u64,
}

/// Complete game state for one session.
pub struct GameState {
    /// The world grid (terrain, deposits, buildings, pollution).
    pub grid: Grid,
    /// All placed buildings.
    pub buildings: Buildings,
    /// All items in the world (on belts, etc.).
    pub items: ItemPool,
    /// Camera position and zoom.
    pub camera: GameCamera,
    /// Gameplay statistics.
    pub stats: GameStats,
    /// Research/tech tree state.
    pub research: ResearchState,
    /// Power grid state (supply, demand, satisfaction).
    pub power: PowerState,
    /// Day/night cycle state.
    pub daynight: DayNightState,
    /// All enemies in the world.
    pub enemies: Enemies,
    /// All trains.
    pub trains: Trains,
    /// Enemy evolution factor (0.0 → 1.0).
    pub evolution: f64,
    /// Positions of enemy spawner nests.
    pub nests: Vec<GridPos>,

    // --- Player inventory (resources available for building) ---
    /// Player's stockpile of resources for constructing buildings.
    /// Collected by placing items into a "logistics" output (or starting resources).
    pub inventory: std::collections::HashMap<Resource, u32>,

    // --- Tutorial ---
    /// Current tutorial step (0 = not started, increments as player completes steps).
    pub tutorial_step: u32,
    /// Whether to show the tutorial overlay.
    pub show_tutorial: bool,
    /// Whether the recipe browser is open (E key).
    pub show_recipes: bool,
    /// Milestones completed (indexed by milestone ID).
    pub milestones_completed: Vec<bool>,
    /// Undo history stack (most recent placement at the end, max 20).
    pub undo_history: Vec<GridPos>,
    /// Last belt position placed (for auto-rotate during drag).
    pub last_belt_pos: Option<GridPos>,
    /// Production tracking: items produced in the last 1200 ticks (60 sec).
    pub production_log: Vec<(Resource, u64)>, // (resource, tick_produced)

    // --- Simulation timing ---
    /// Accumulated time for fixed-timestep simulation.
    pub tick_accumulator: f64,

    // --- UI state (not serialized) ---
    /// Currently selected building type for placement (None = no selection).
    pub selected_building: Option<BuildingKind>,
    /// Direction the next placed building will face.
    pub placement_direction: Direction,
    /// Whether the game is paused.
    pub paused: bool,
    /// Game speed multiplier (1 = normal, 2 = double, 3 = triple).
    pub game_speed: u32,
    /// Whether the research screen overlay is visible.
    pub show_research: bool,
    /// Toast notification messages (text, remaining display ticks).
    pub toasts: Vec<(String, u32)>,
    /// Notification history (last 20 messages for review).
    pub notification_log: Vec<String>,
    /// Whether the game has been won (all story complete).
    pub game_won: bool,
    /// Whether the help/keybinds overlay is showing (F1).
    pub show_help: bool,
    /// Whether the achievements screen is showing (N key).
    pub show_achievements: bool,
    /// Whether the production stats screen is showing (V key).
    pub show_stats: bool,
    /// Blueprint: stored buildings from a copy operation (relative positions + kinds).
    pub blueprint: Vec<(i32, i32, BuildingKind, Direction)>,
    /// Whether we're in blueprint paste mode.
    pub pasting_blueprint: bool,
    /// Brief placement flash effect (position + remaining ticks).
    pub placement_flash: Option<(GridPos, u32)>,
    /// Build zone radius (tiles from map center). Expands with research.
    pub build_radius: f32,
    /// Recipe picker: open for which building? (BuildingId, available recipes).
    pub recipe_picker: Option<(BuildingId, Vec<recipe::RecipeId>)>,
    /// Active robot workers (start pos, target pos, progress 0.0-1.0).
    pub robots: Vec<(macroquad::prelude::Vec2, macroquad::prelude::Vec2, f32)>,
    /// Combat visual effects: (from_x, from_y, to_x, to_y, ticks_remaining, color_r, color_g, color_b).
    pub combat_fx: Vec<(f32, f32, f32, f32, u32, f32, f32, f32)>,
    /// Narrative/story progression state.
    pub story: StoryState,
    /// Seed used for map generation (stored for save/load).
    pub seed: u64,
}

impl GameState {
    /// Creates a new game with a procedurally generated map.
    ///
    /// The `seed` determines map layout. Pass `0` to use the current system time.
    pub fn new(seed: u64) -> Self {
        let actual_seed = if seed == 0 {
            macroquad::miniquad::date::now().to_bits()
        } else {
            seed
        };

        let mut grid = Grid::new(GRID_WIDTH, GRID_HEIGHT);
        let nests = mapgen::generate_map(&mut grid, actual_seed);

        Self {
            grid,
            buildings: Buildings::new(),
            items: ItemPool::new(4096),
            camera: GameCamera::new(),
            stats: GameStats::default(),
            research: ResearchState::new(),
            power: PowerState::default(),
            daynight: DayNightState::default(),
            enemies: Enemies::new(),
            trains: Trains::new(),
            evolution: 0.0,
            nests,
            inventory: {
                let mut inv = HashMap::new();
                // Starter resources so the player can build their first machines.
                inv.insert(Resource::IronPlate, 50);
                inv.insert(Resource::CopperPlate, 30);
                inv.insert(Resource::Stone, 20);
                inv.insert(Resource::Coal, 30);
                inv.insert(Resource::Gear, 25);
                inv.insert(Resource::Wire, 10);
                inv.insert(Resource::GreenCircuit, 8);
                inv
            },
            tutorial_step: 0,
            show_tutorial: true,
            show_recipes: false,
            milestones_completed: vec![false; crate::milestones::MILESTONES.len()],
            undo_history: Vec::new(),
            last_belt_pos: None,
            production_log: Vec::new(),
            tick_accumulator: 0.0,
            selected_building: None,
            placement_direction: Direction::South,
            paused: false,
            game_speed: 1,
            show_research: false,
            toasts: Vec::new(),
            notification_log: Vec::new(),
            game_won: false,
            show_help: false,
            show_achievements: false,
            show_stats: false,
            blueprint: Vec::new(),
            pasting_blueprint: false,
            placement_flash: None,
            build_radius: 30.0,
            recipe_picker: None,
            robots: Vec::new(),
            combat_fx: Vec::new(),
            story: StoryState::new(),
            seed: actual_seed,
        }
    }

    /// Adds a toast notification that displays for `duration_ticks` simulation ticks.
    pub fn toast(&mut self, message: String, duration_ticks: u32) {
        // Log for history review.
        self.notification_log.push(message.clone());
        if self.notification_log.len() > 30 {
            self.notification_log.remove(0);
        }
        self.toasts.push((message, duration_ticks));
        if self.toasts.len() > 5 {
            self.toasts.remove(0);
        }
    }

    /// Decrements toast timers and removes expired ones. Call once per tick.
    pub fn tick_toasts(&mut self) {
        for toast in &mut self.toasts {
            if toast.1 > 0 {
                toast.1 -= 1;
            }
        }
        self.toasts.retain(|t| t.1 > 0);
    }
}
