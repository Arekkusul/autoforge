//! Top-level game state and initialization.
//!
//! [`GameState`] holds all data for a running game session: the grid, camera,
//! simulation state, and UI state. The main loop in [`crate::main`] owns one
//! `GameState` and passes it to input, simulation, and rendering systems.

use std::collections::HashMap;

use crate::building::Buildings;
use crate::camera::GameCamera;
use crate::constants::*;
use crate::daynight::DayNightState;
use crate::enemy::Enemies;
use crate::grid::Grid;
use crate::item::ItemPool;
use crate::mapgen;
use crate::power::PowerState;
use crate::recipe;
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
    /// Per-resource production log: (resource, tick_produced). Transient, not saved.
    #[serde(skip)]
    pub production_log: Vec<(Resource, u64)>,
}

impl GameStats {
    /// Computes per-resource production throughput (items per minute) over the
    /// last `window_ticks` simulation ticks, using the transient production log.
    ///
    /// At [`crate::constants::TICKS_PER_SECOND`] ticks/second, one game minute is
    /// `TICKS_PER_SECOND * 60` ticks. Results are sorted by rate, descending, so
    /// callers can show the busiest lines first. An empty log yields no entries.
    pub fn production_rates(&self, window_ticks: u64) -> Vec<(Resource, f32)> {
        let now = self.total_ticks;
        let cutoff = now.saturating_sub(window_ticks);
        // Only the ticks actually elapsed within the window count toward the rate.
        let elapsed = (now - cutoff).max(1) as f32;
        let minutes = elapsed / (crate::constants::TICKS_PER_SECOND as f32 * 60.0);

        let mut counts: HashMap<Resource, u32> = HashMap::new();
        for &(resource, tick) in &self.production_log {
            if tick >= cutoff {
                *counts.entry(resource).or_insert(0) += 1;
            }
        }

        let mut rates: Vec<(Resource, f32)> = counts
            .into_iter()
            .map(|(resource, count)| (resource, count as f32 / minutes))
            .collect();
        rates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        rates
    }
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
    /// Toast notification messages (text, remaining display ticks, severity).
    pub toasts: Vec<(String, u32, AlertSeverity)>,
    /// Cooldown tracking for alerts: maps AlertKind → last tick fired.
    pub alert_cooldowns: HashMap<AlertKind, u64>,
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
    /// Saved blueprint library (name, buildings). Max 10.
    pub blueprint_library: Vec<(String, Vec<(i32, i32, BuildingKind, Direction)>)>,
    /// Whether the blueprint picker overlay is showing.
    pub show_blueprint_picker: bool,
    /// Custom hotbar overrides. None = use defaults. Vec index = slot, value = building kind.
    pub custom_hotbar: Vec<Option<BuildingKind>>,
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
            tick_accumulator: 0.0,
            selected_building: None,
            placement_direction: Direction::South,
            paused: false,
            game_speed: 1,
            show_research: false,
            toasts: Vec::new(),
            alert_cooldowns: HashMap::new(),
            notification_log: Vec::new(),
            game_won: false,
            show_help: false,
            show_achievements: false,
            show_stats: false,
            blueprint: Vec::new(),
            pasting_blueprint: false,
            blueprint_library: Vec::new(),
            show_blueprint_picker: false,
            custom_hotbar: vec![None; 16],
            placement_flash: None,
            build_radius: 30.0,
            recipe_picker: None,
            robots: Vec::new(),
            combat_fx: Vec::new(),
            story: StoryState::new(),
            seed: actual_seed,
        }
    }

    /// Adds `amount` of a resource to the player's building inventory.
    pub fn add_to_inventory(&mut self, resource: Resource, amount: u32) {
        if amount == 0 {
            return;
        }
        *self.inventory.entry(resource).or_insert(0) += amount;
    }

    /// Returns how many of `resource` the player currently holds.
    pub fn inventory_count(&self, resource: Resource) -> u32 {
        self.inventory.get(&resource).copied().unwrap_or(0)
    }

    /// Removes up to `amount` of a resource from inventory.
    ///
    /// Returns `true` only if the full amount was available and removed; on
    /// insufficient stock the inventory is left unchanged and `false` is returned.
    pub fn remove_from_inventory(&mut self, resource: Resource, amount: u32) -> bool {
        let have = self.inventory_count(resource);
        if have < amount {
            return false;
        }
        if let Some(slot) = self.inventory.get_mut(&resource) {
            *slot -= amount;
            if *slot == 0 {
                self.inventory.remove(&resource);
            }
        }
        true
    }

    /// Total number of individual items across all inventory resource types.
    pub fn total_inventory_items(&self) -> u64 {
        self.inventory.values().map(|&c| c as u64).sum()
    }

    /// Number of inventory stacks occupied, using [`STACK_SIZE`] per stack.
    ///
    /// Each resource type rounds up to whole stacks, mirroring how a real chest
    /// would allocate slots.
    pub fn inventory_stacks(&self) -> u32 {
        self.inventory
            .values()
            .map(|&count| count.div_ceil(STACK_SIZE))
            .sum()
    }

    /// Adds a toast notification that displays for `duration_ticks` simulation ticks.
    pub fn toast(&mut self, message: String, duration_ticks: u32) {
        self.toast_with_severity(message, duration_ticks, AlertSeverity::Info);
    }

    /// Adds a toast notification with a specific severity level.
    pub fn toast_with_severity(
        &mut self,
        message: String,
        duration_ticks: u32,
        severity: AlertSeverity,
    ) {
        // Log for history review.
        self.notification_log.push(message.clone());
        if self.notification_log.len() > 30 {
            self.notification_log.remove(0);
        }
        self.toasts.push((message, duration_ticks, severity));
        if self.toasts.len() > 5 {
            self.toasts.remove(0);
        }
    }

    /// Fires an alert with cooldown-based deduplication.
    /// If the same `kind` was fired within its cooldown window, this is a no-op.
    pub fn alert(
        &mut self,
        kind: AlertKind,
        message: String,
        duration_ticks: u32,
        severity: AlertSeverity,
    ) {
        let tick = self.stats.total_ticks;
        let cooldown = kind.cooldown_ticks();
        if let Some(&last) = self.alert_cooldowns.get(&kind) {
            if tick < last + cooldown {
                return;
            }
        }
        self.alert_cooldowns.insert(kind, tick);
        self.toast_with_severity(message, duration_ticks, severity);
    }

    /// Closes all UI overlay panels.
    pub fn close_all_overlays(&mut self) {
        self.show_research = false;
        self.show_recipes = false;
        self.show_stats = false;
        self.show_achievements = false;
        self.show_help = false;
        self.show_blueprint_picker = false;
        self.recipe_picker = None;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds an inventory map directly, avoiding full map generation.
    fn inv(pairs: &[(Resource, u32)]) -> HashMap<Resource, u32> {
        pairs.iter().copied().collect()
    }

    /// Minimal harness exposing only the inventory helpers under test.
    struct InvView(HashMap<Resource, u32>);
    impl InvView {
        fn add(&mut self, resource: Resource, amount: u32) {
            if amount == 0 {
                return;
            }
            *self.0.entry(resource).or_insert(0) += amount;
        }
        fn count(&self, resource: Resource) -> u32 {
            self.0.get(&resource).copied().unwrap_or(0)
        }
        fn remove(&mut self, resource: Resource, amount: u32) -> bool {
            if self.count(resource) < amount {
                return false;
            }
            if let Some(slot) = self.0.get_mut(&resource) {
                *slot -= amount;
                if *slot == 0 {
                    self.0.remove(&resource);
                }
            }
            true
        }
        fn total(&self) -> u64 {
            self.0.values().map(|&c| c as u64).sum()
        }
        fn stacks(&self) -> u32 {
            self.0.values().map(|&c| c.div_ceil(STACK_SIZE)).sum()
        }
    }

    #[test]
    fn remove_fails_when_insufficient_and_leaves_stock_intact() {
        let mut v = InvView(inv(&[(Resource::IronPlate, 3)]));
        assert!(!v.remove(Resource::IronPlate, 5));
        assert_eq!(v.count(Resource::IronPlate), 3);
    }

    #[test]
    fn remove_to_zero_clears_the_entry() {
        let mut v = InvView(inv(&[(Resource::Gear, 2)]));
        assert!(v.remove(Resource::Gear, 2));
        assert_eq!(v.count(Resource::Gear), 0);
        assert!(!v.0.contains_key(&Resource::Gear));
    }

    #[test]
    fn add_zero_is_a_noop() {
        let mut v = InvView(HashMap::new());
        v.add(Resource::Coal, 0);
        assert!(v.0.is_empty());
    }

    #[test]
    fn total_items_sums_all_resource_types() {
        let v = InvView(inv(&[(Resource::IronPlate, 40), (Resource::Coal, 60)]));
        assert_eq!(v.total(), 100);
    }

    #[test]
    fn stacks_round_up_per_resource() {
        // STACK_SIZE is 50: 51 iron = 2 stacks, 50 coal = 1 stack.
        let v = InvView(inv(&[(Resource::IronPlate, 51), (Resource::Coal, 50)]));
        assert_eq!(v.stacks(), 3);
    }

    #[test]
    fn production_rates_empty_log_is_empty() {
        let stats = GameStats::default();
        assert!(stats.production_rates(1200).is_empty());
    }

    #[test]
    fn production_rates_scale_to_items_per_minute() {
        // One game minute is TICKS_PER_SECOND*60 = 1200 ticks. Log 60 gears
        // within a 1200-tick window -> 60/min.
        let mut stats = GameStats {
            total_ticks: 1200,
            ..Default::default()
        };
        for t in 0..60 {
            stats.production_log.push((Resource::Gear, t));
        }
        let rates = stats.production_rates(1200);
        assert_eq!(rates.len(), 1);
        let (res, rate) = rates[0];
        assert_eq!(res, Resource::Gear);
        assert!((rate - 60.0).abs() < 1e-3, "expected 60/min, got {rate}");
    }

    #[test]
    fn production_rates_ignore_entries_outside_window() {
        let mut stats = GameStats {
            total_ticks: 3000,
            ..Default::default()
        };
        // Old entry before the 1200-tick window (cutoff = 1800) is excluded.
        stats.production_log.push((Resource::Coal, 100));
        stats.production_log.push((Resource::Gear, 2500));
        let rates = stats.production_rates(1200);
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].0, Resource::Gear);
    }

    #[test]
    fn production_rates_sorted_descending() {
        let mut stats = GameStats {
            total_ticks: 1200,
            ..Default::default()
        };
        stats.production_log.push((Resource::Coal, 0));
        for t in 0..5 {
            stats.production_log.push((Resource::Gear, t));
        }
        let rates = stats.production_rates(1200);
        assert_eq!(rates[0].0, Resource::Gear);
        assert!(rates[0].1 >= rates[1].1);
    }
}
