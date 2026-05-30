//! Global tuning constants for AutoForge.
//!
//! All gameplay-affecting numbers live here so they can be found and adjusted in one place.
//! Each constant documents its unit and purpose.

/// Tile size in world-space pixels. 48px gives crisp visuals at 1080p.
pub const TILE_SIZE: f32 = 48.0;

/// Sprite resolution in pixels (sprites are authored at this size, scaled to [`TILE_SIZE`]).
/// Production quality: 32×32 gives enough detail for proper shading and anti-aliasing.
pub const SPRITE_SIZE: u16 = 32;

/// Item sprite resolution in pixels.
pub const ITEM_SPRITE_SIZE: u16 = 8;

/// Enemy sprite resolution in pixels.
pub const ENEMY_SPRITE_SIZE: u16 = 12;

/// World grid dimensions in tiles.
pub const GRID_WIDTH: i32 = 512;
/// World grid dimensions in tiles.
pub const GRID_HEIGHT: i32 = 512;

/// Simulation ticks per second (fixed timestep).
pub const TICKS_PER_SECOND: u32 = 20;

/// Duration of one simulation tick in seconds.
pub const TICK_DURATION: f64 = 1.0 / TICKS_PER_SECOND as f64;

/// Maximum accumulated time before we cap (prevents spiral of death on lag).
pub const MAX_ACCUMULATOR: f64 = 0.25;

// ---------------------------------------------------------------------------
// Belt speeds (ticks per tile movement)
// ---------------------------------------------------------------------------

/// Yellow belt: moves an item one tile every N ticks.
pub const BELT_YELLOW_TICKS: u32 = 4;
/// Red belt: 2× yellow speed.
pub const BELT_RED_TICKS: u32 = 2;
/// Blue belt: 3× yellow speed.
pub const BELT_BLUE_TICKS: u32 = 1;

/// Underground belt max tunnel distance (yellow tier) in tiles.
pub const UNDERGROUND_RANGE_YELLOW: i32 = 5;
/// Underground belt max tunnel distance (red tier) in tiles.
pub const UNDERGROUND_RANGE_RED: i32 = 7;
/// Underground belt max tunnel distance (blue tier) in tiles.
pub const UNDERGROUND_RANGE_BLUE: i32 = 10;

// ---------------------------------------------------------------------------
// Machine processing durations (in ticks)
// ---------------------------------------------------------------------------

/// Miner: ticks to extract one ore.
pub const MINER_TICKS: u32 = 40;
/// Stone / Steel furnace: ticks to smelt one item.
pub const SMELTER_TICKS: u32 = 60;
/// Electric furnace: ticks to smelt one item (faster).
pub const ELECTRIC_SMELTER_TICKS: u32 = 40;
/// Assembler Tier 1: base ticks per recipe craft.
pub const ASSEMBLER_T1_TICKS: u32 = 80;
/// Assembler Tier 2: base ticks per recipe craft.
pub const ASSEMBLER_T2_TICKS: u32 = 53;
/// Assembler Tier 3: base ticks per recipe craft.
pub const ASSEMBLER_T3_TICKS: u32 = 40;
/// Chemical plant: base ticks per recipe.
pub const CHEMICAL_PLANT_TICKS: u32 = 60;
/// Oil refinery: ticks per refining cycle.
pub const REFINERY_TICKS: u32 = 100;
/// Centrifuge: ticks per uranium processing cycle.
pub const CENTRIFUGE_TICKS: u32 = 200;
/// Lab: ticks to consume one science pack.
pub const LAB_TICKS: u32 = 60;

// ---------------------------------------------------------------------------
// Machine buffer capacities
// ---------------------------------------------------------------------------

/// Maximum items in a machine input or output buffer.
pub const MACHINE_BUFFER_CAP: usize = 8;
/// Storage chest capacity (number of item stacks).
pub const STORAGE_CHEST_STACKS: usize = 48;
/// Items per stack in a storage chest.
pub const STACK_SIZE: u32 = 50;
/// Storage tank capacity for fluids (units).
pub const TANK_CAPACITY: f32 = 25_000.0;

// ---------------------------------------------------------------------------
// Power system
// ---------------------------------------------------------------------------

/// Power produced by one steam engine (kW equivalent units).
pub const STEAM_ENGINE_POWER: f32 = 900.0;
/// Power produced by one solar panel at peak (daytime).
pub const SOLAR_PANEL_POWER: f32 = 60.0;
/// Energy stored in one accumulator (kJ equivalent).
pub const ACCUMULATOR_CAPACITY: f32 = 5000.0;
/// Power produced by one nuclear reactor (kW).
pub const NUCLEAR_REACTOR_POWER: f32 = 40_000.0;
/// Ticks of fuel from one coal in a boiler.
pub const COAL_FUEL_TICKS: u32 = 120;
/// Ticks of fuel from one nuclear fuel cell.
pub const NUCLEAR_FUEL_CELL_TICKS: u32 = 4000;

/// Base power draw of a miner (kW).
pub const MINER_POWER_DRAW: f32 = 90.0;
/// Base power draw of an electric smelter (kW).
pub const ELECTRIC_SMELTER_POWER_DRAW: f32 = 180.0;
/// Base power draw of an assembler (kW).
pub const ASSEMBLER_POWER_DRAW: f32 = 150.0;
/// Base power draw of a chemical plant (kW).
pub const CHEMICAL_PLANT_POWER_DRAW: f32 = 210.0;
/// Base power draw of an oil refinery (kW).
pub const REFINERY_POWER_DRAW: f32 = 420.0;
/// Base power draw of a lab (kW).
pub const LAB_POWER_DRAW: f32 = 60.0;
/// Base power draw of a laser turret (kW).
pub const LASER_TURRET_POWER_DRAW: f32 = 800.0;
/// Base power draw of a radar (kW).
pub const RADAR_POWER_DRAW: f32 = 300.0;
/// Base power draw of a centrifuge (kW).
pub const CENTRIFUGE_POWER_DRAW: f32 = 350.0;
/// Base power draw of a roboport (kW).
pub const ROBOPORT_POWER_DRAW: f32 = 500.0;
/// Base power draw of a beacon (kW).
pub const BEACON_POWER_DRAW: f32 = 480.0;
/// Base power draw of a rocket silo (kW).
pub const ROCKET_SILO_POWER_DRAW: f32 = 4000.0;

// ---------------------------------------------------------------------------
// Day / night cycle
// ---------------------------------------------------------------------------

/// Duration of full daytime in seconds.
pub const DAY_DURATION_SECS: f32 = 420.0;
/// Duration of full nighttime in seconds.
pub const NIGHT_DURATION_SECS: f32 = 180.0;
/// Total day+night cycle in seconds.
pub const FULL_CYCLE_SECS: f32 = DAY_DURATION_SECS + NIGHT_DURATION_SECS;

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

/// Minimum camera zoom level.
pub const ZOOM_MIN: f32 = 0.25;
/// Maximum camera zoom level.
pub const ZOOM_MAX: f32 = 4.0;
/// Zoom sensitivity per scroll tick.
pub const ZOOM_SPEED: f32 = 0.1;
/// Camera pan speed in world pixels per second at zoom = 1.
pub const PAN_SPEED: f32 = 400.0;

// ---------------------------------------------------------------------------
// Pollution & enemies
// ---------------------------------------------------------------------------

/// Pollution generated per tick by a powered machine, multiplied by power draw.
pub const POLLUTION_PER_KW_PER_TICK: f32 = 0.0001;
/// Fraction of pollution that diffuses to each neighbor per tick.
pub const POLLUTION_DIFFUSION_RATE: f32 = 0.02;
/// Pollution absorbed by one tree tile per tick.
pub const TREE_ABSORPTION_PER_TICK: f32 = 0.005;

/// Evolution increase per tick from time alone.
pub const EVOLUTION_TIME_FACTOR: f64 = 0.000004;
/// Evolution increase per unit of pollution absorbed by a nest.
pub const EVOLUTION_POLLUTION_FACTOR: f64 = 0.000015;
/// Evolution increase when a nest is destroyed.
pub const EVOLUTION_NEST_DESTROY: f64 = 0.005;

/// Wall hit points.
pub const WALL_HP: f32 = 500.0;
/// Gate hit points.
pub const GATE_HP: f32 = 500.0;

// ---------------------------------------------------------------------------
// Map generation
// ---------------------------------------------------------------------------

/// How many ore clusters to generate per resource type.
pub const ORE_CLUSTERS_PER_TYPE: u32 = 25;
/// Minimum ore cluster radius in tiles.
pub const ORE_CLUSTER_RADIUS_MIN: i32 = 3;
/// Maximum ore cluster radius in tiles.
pub const ORE_CLUSTER_RADIUS_MAX: i32 = 8;
/// Number of water bodies to generate.
pub const WATER_BODY_COUNT: u32 = 12;
/// Number of tree patches to generate.
pub const TREE_PATCH_COUNT: u32 = 40;
/// Number of enemy nests at map start.
pub const INITIAL_NEST_COUNT: u32 = 20;
/// Minimum distance (tiles) from map center for nests to spawn.
pub const NEST_MIN_DISTANCE: f32 = 120.0;

// ---------------------------------------------------------------------------
// Animation
// ---------------------------------------------------------------------------

/// Ticks per animation frame for belts.
pub const BELT_ANIM_SPEED: u32 = 4;
/// Ticks per animation frame for active machines.
pub const MACHINE_ANIM_SPEED: u32 = 8;
/// Ticks per animation frame for enemies.
pub const ENEMY_ANIM_SPEED: u32 = 6;

// ---------------------------------------------------------------------------
// Inserter speeds (ticks per operation)
// ---------------------------------------------------------------------------

/// Regular inserter: ticks per swing.
pub const INSERTER_REGULAR_TICKS: u32 = 16;
/// Long inserter: ticks per swing.
pub const INSERTER_LONG_TICKS: u32 = 20;
/// Fast inserter: ticks per swing.
pub const INSERTER_FAST_TICKS: u32 = 8;
/// Stack inserter: ticks per swing (moves multiple items).
pub const INSERTER_STACK_TICKS: u32 = 12;
/// Items moved per swing by a stack inserter.
pub const STACK_INSERTER_COUNT: u32 = 4;

// ---------------------------------------------------------------------------
// Window defaults
// ---------------------------------------------------------------------------

/// Default window width in pixels (1080p).
pub const WINDOW_WIDTH: i32 = 1920;
/// Default window height in pixels (1080p).
pub const WINDOW_HEIGHT: i32 = 1080;
