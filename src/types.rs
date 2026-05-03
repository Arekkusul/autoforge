//! Core data types shared across all modules.
//!
//! This module defines the fundamental enums and structs that form the vocabulary
//! of AutoForge: directions, resources, building kinds, and grid coordinates.

use serde::{Deserialize, Serialize};

/// Cardinal direction for belt flow, building output, and inserter facing.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Direction {
    /// Up on screen (negative Y in world space).
    North,
    /// Right on screen (positive X).
    East,
    /// Down on screen (positive Y).
    South,
    /// Left on screen (negative X).
    West,
}

impl Direction {
    /// Returns the grid offset `(dx, dy)` for this direction.
    ///
    /// North moves toward row 0 (screen-up), so `dy = -1`.
    pub const fn delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::East => (1, 0),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
        }
    }

    /// Returns the direction rotated 90° clockwise.
    pub const fn rotated_cw(self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    /// Returns the direction rotated 90° counter-clockwise.
    pub const fn rotated_ccw(self) -> Direction {
        match self {
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
        }
    }

    /// Returns the opposite direction (180° rotation).
    pub const fn opposite(self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    /// Returns all four cardinal directions in order: N, E, S, W.
    pub const fn all() -> [Direction; 4] {
        [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
    }
}

/// Raw and processed resources that exist as items on belts and in buffers.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Resource {
    // Tier 1 — Raw ores
    IronOre,
    CopperOre,
    Coal,
    Stone,
    UraniumOre,

    // Tier 2 — Basic processed
    IronPlate,
    CopperPlate,
    StoneBrick,
    SteelPlate,

    // Tier 3 — Basic components
    Gear,
    Wire,
    Pipe,
    IronStick,
    Sulfur,
    Plastic,
    Battery,

    // Tier 4 — Intermediate products
    GreenCircuit,
    RedCircuit,
    BlueCircuit,
    EngineUnit,
    ElectricEngine,
    Rail,
    Concrete,

    // Tier 5 — Advanced products
    SpeedModule,
    EfficiencyModule,
    ProductivityModule,
    SolarPanelItem,
    AccumulatorItem,
    FlyingRobotFrame,
    Uranium235,
    Uranium238,
    NuclearFuelCell,
    RocketFuel,

    // Tier 6 — Science packs & rocket
    ScienceRed,
    ScienceGreen,
    ScienceBlue,
    SciencePurple,
    ScienceYellow,
    SpaceScience,
    RocketPart,
    Inserter,
    LowDensityStructure,

    // Ammo
    BasicAmmo,
    PiercingAmmo,
    Grenade,
}

impl Resource {
    /// Human-readable display name for UI and tooltips.
    pub const fn display_name(self) -> &'static str {
        match self {
            Resource::IronOre => "Iron Ore",
            Resource::CopperOre => "Copper Ore",
            Resource::Coal => "Coal",
            Resource::Stone => "Stone",
            Resource::UraniumOre => "Uranium Ore",
            Resource::IronPlate => "Iron Plate",
            Resource::CopperPlate => "Copper Plate",
            Resource::StoneBrick => "Stone Brick",
            Resource::SteelPlate => "Steel Plate",
            Resource::Gear => "Gear",
            Resource::Wire => "Wire",
            Resource::Pipe => "Pipe",
            Resource::IronStick => "Iron Stick",
            Resource::Sulfur => "Sulfur",
            Resource::Plastic => "Plastic",
            Resource::Battery => "Battery",
            Resource::GreenCircuit => "Green Circuit",
            Resource::RedCircuit => "Red Circuit",
            Resource::BlueCircuit => "Blue Circuit",
            Resource::EngineUnit => "Engine Unit",
            Resource::ElectricEngine => "Electric Engine",
            Resource::Rail => "Rail",
            Resource::Concrete => "Concrete",
            Resource::SpeedModule => "Speed Module",
            Resource::EfficiencyModule => "Efficiency Module",
            Resource::ProductivityModule => "Productivity Module",
            Resource::SolarPanelItem => "Solar Panel",
            Resource::AccumulatorItem => "Accumulator",
            Resource::FlyingRobotFrame => "Flying Robot Frame",
            Resource::Uranium235 => "Uranium-235",
            Resource::Uranium238 => "Uranium-238",
            Resource::NuclearFuelCell => "Nuclear Fuel Cell",
            Resource::RocketFuel => "Rocket Fuel",
            Resource::ScienceRed => "Science Pack (Red)",
            Resource::ScienceGreen => "Science Pack (Green)",
            Resource::ScienceBlue => "Science Pack (Blue)",
            Resource::SciencePurple => "Science Pack (Purple)",
            Resource::ScienceYellow => "Science Pack (Yellow)",
            Resource::SpaceScience => "Space Science Pack",
            Resource::RocketPart => "Rocket Part",
            Resource::Inserter => "Inserter",
            Resource::LowDensityStructure => "Low Density Structure",
            Resource::BasicAmmo => "Basic Ammo",
            Resource::PiercingAmmo => "Piercing Ammo",
            Resource::Grenade => "Grenade",
        }
    }
}

/// Natural resource deposit type found on terrain tiles.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum OreDeposit {
    Iron,
    Copper,
    Coal,
    Stone,
    Uranium,
    Tin,
    Gold,
    Sulfur,
    Crystal,
    /// Oil well — extracted by pump jack, produces crude oil (fluid).
    Oil,
}

impl OreDeposit {
    /// The [`Resource`] item produced when mining this deposit.
    ///
    /// Returns `None` for [`OreDeposit::Oil`] since oil is a fluid, not an item.
    pub const fn mined_resource(self) -> Option<Resource> {
        match self {
            OreDeposit::Iron => Some(Resource::IronOre),
            OreDeposit::Copper => Some(Resource::CopperOre),
            OreDeposit::Coal => Some(Resource::Coal),
            OreDeposit::Stone => Some(Resource::Stone),
            OreDeposit::Uranium => Some(Resource::UraniumOre),
            OreDeposit::Tin => Some(Resource::IronOre),      // placeholder until TinOre added
            OreDeposit::Gold => Some(Resource::CopperOre),   // placeholder until GoldOre added
            OreDeposit::Sulfur => Some(Resource::Stone),     // placeholder until SulfurOre added
            OreDeposit::Crystal => Some(Resource::Stone),    // placeholder until Crystal added
            OreDeposit::Oil => None,
        }
    }

    /// Human-readable display name.
    pub const fn display_name(self) -> &'static str {
        match self {
            OreDeposit::Iron => "Iron Deposit",
            OreDeposit::Copper => "Copper Deposit",
            OreDeposit::Coal => "Coal Deposit",
            OreDeposit::Stone => "Stone Deposit",
            OreDeposit::Uranium => "Uranium Deposit",
            OreDeposit::Tin => "Tin Deposit",
            OreDeposit::Gold => "Gold Deposit",
            OreDeposit::Sulfur => "Sulfur Deposit",
            OreDeposit::Crystal => "Crystal Deposit",
            OreDeposit::Oil => "Oil Well",
        }
    }
}

/// Fluid types that flow through pipes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum FluidType {
    Water,
    CrudeOil,
    PetroleumGas,
    HeavyOil,
    LightOil,
    SulfuricAcid,
    Lubricant,
}

impl FluidType {
    /// Human-readable display name.
    pub const fn display_name(self) -> &'static str {
        match self {
            FluidType::Water => "Water",
            FluidType::CrudeOil => "Crude Oil",
            FluidType::PetroleumGas => "Petroleum Gas",
            FluidType::HeavyOil => "Heavy Oil",
            FluidType::LightOil => "Light Oil",
            FluidType::SulfuricAcid => "Sulfuric Acid",
            FluidType::Lubricant => "Lubricant",
        }
    }
}

/// Terrain type for a tile (determines ground appearance and properties).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Terrain {
    /// Standard buildable ground.
    Grass,
    /// Sandy terrain, fewer trees.
    Desert,
    /// Dense forest — absorbs pollution, must be cleared to build.
    Forest,
    /// Impassable water — needed for water pumps.
    Water,
    /// Impassable cliff — removable with cliff explosives.
    Cliff,
}

impl Terrain {
    /// Whether buildings can be placed on this terrain.
    pub const fn is_buildable(self) -> bool {
        matches!(self, Terrain::Grass | Terrain::Desert)
    }

    /// Whether this terrain has trees (absorbs pollution).
    pub const fn has_trees(self) -> bool {
        matches!(self, Terrain::Forest)
    }
}

/// What kind of building occupies a tile.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum BuildingKind {
    // Logistics — belts
    BeltYellow,
    BeltRed,
    BeltBlue,
    UndergroundBeltYellow,
    UndergroundBeltRed,
    UndergroundBeltBlue,
    Splitter,

    // Logistics — inserters
    InserterRegular,
    InserterLong,
    InserterFast,
    InserterStack,

    // Storage
    StorageChest,

    // Production
    Miner,
    PumpJack,
    WaterPump,
    StoneFurnace,
    SteelFurnace,
    ElectricFurnace,
    AssemblerT1,
    AssemblerT2,
    AssemblerT3,
    ChemicalPlant,
    OilRefinery,
    Centrifuge,
    RocketSilo,
    Lab,

    // Power
    Boiler,
    SteamEngine,
    SolarPanel,
    Accumulator,
    NuclearReactor,

    // Military
    GunTurret,
    LaserTurret,
    Wall,
    Gate,
    Radar,

    // Fluids
    PipeSegment,
    UndergroundPipe,
    StorageTank,

    // Trains
    RailStraight,
    RailCurved,
    TrainStop,
    RailSignal,

    // Robots
    Roboport,
    Beacon,
}

impl BuildingKind {
    /// Human-readable display name for UI.
    pub fn display_name(self) -> &'static str {
        match self {
            BuildingKind::BeltYellow => "Belt (Yellow)",
            BuildingKind::BeltRed => "Belt (Red)",
            BuildingKind::BeltBlue => "Belt (Blue)",
            BuildingKind::UndergroundBeltYellow => "Underground Belt (Yellow)",
            BuildingKind::UndergroundBeltRed => "Underground Belt (Red)",
            BuildingKind::UndergroundBeltBlue => "Underground Belt (Blue)",
            BuildingKind::Splitter => "Splitter",
            BuildingKind::InserterRegular => "Inserter",
            BuildingKind::InserterLong => "Long Inserter",
            BuildingKind::InserterFast => "Fast Inserter",
            BuildingKind::InserterStack => "Stack Inserter",
            BuildingKind::StorageChest => "Storage Chest",
            BuildingKind::Miner => "Miner",
            BuildingKind::PumpJack => "Pump Jack",
            BuildingKind::WaterPump => "Water Pump",
            BuildingKind::StoneFurnace => "Stone Furnace",
            BuildingKind::SteelFurnace => "Steel Furnace",
            BuildingKind::ElectricFurnace => "Electric Furnace",
            BuildingKind::AssemblerT1 => "Assembler (Tier 1)",
            BuildingKind::AssemblerT2 => "Assembler (Tier 2)",
            BuildingKind::AssemblerT3 => "Assembler (Tier 3)",
            BuildingKind::ChemicalPlant => "Chemical Plant",
            BuildingKind::OilRefinery => "Oil Refinery",
            BuildingKind::Centrifuge => "Centrifuge",
            BuildingKind::RocketSilo => "Rocket Silo",
            BuildingKind::Lab => "Lab",
            BuildingKind::Boiler => "Boiler",
            BuildingKind::SteamEngine => "Steam Engine",
            BuildingKind::SolarPanel => "Solar Panel",
            BuildingKind::Accumulator => "Accumulator",
            BuildingKind::NuclearReactor => "Nuclear Reactor",
            BuildingKind::GunTurret => "Gun Turret",
            BuildingKind::LaserTurret => "Laser Turret",
            BuildingKind::Wall => "Wall",
            BuildingKind::Gate => "Gate",
            BuildingKind::Radar => "Radar",
            BuildingKind::PipeSegment => "Pipe",
            BuildingKind::UndergroundPipe => "Underground Pipe",
            BuildingKind::StorageTank => "Storage Tank",
            BuildingKind::RailStraight => "Rail (Straight)",
            BuildingKind::RailCurved => "Rail (Curved)",
            BuildingKind::TrainStop => "Train Stop",
            BuildingKind::RailSignal => "Rail Signal",
            BuildingKind::Roboport => "Roboport",
            BuildingKind::Beacon => "Beacon",
        }
    }

    /// Whether this building is a belt (any tier).
    pub const fn is_belt(self) -> bool {
        matches!(
            self,
            BuildingKind::BeltYellow | BuildingKind::BeltRed | BuildingKind::BeltBlue
        )
    }

    /// Whether this building is an underground belt (any tier).
    pub const fn is_underground_belt(self) -> bool {
        matches!(
            self,
            BuildingKind::UndergroundBeltYellow
                | BuildingKind::UndergroundBeltRed
                | BuildingKind::UndergroundBeltBlue
        )
    }

    /// Whether this building is an inserter (any tier).
    pub const fn is_inserter(self) -> bool {
        matches!(
            self,
            BuildingKind::InserterRegular
                | BuildingKind::InserterLong
                | BuildingKind::InserterFast
                | BuildingKind::InserterStack
        )
    }

    /// Whether this building requires electric power to operate.
    pub const fn needs_power(self) -> bool {
        matches!(
            self,
            BuildingKind::Miner
                | BuildingKind::PumpJack
                | BuildingKind::WaterPump
                | BuildingKind::ElectricFurnace
                | BuildingKind::AssemblerT1
                | BuildingKind::AssemblerT2
                | BuildingKind::AssemblerT3
                | BuildingKind::ChemicalPlant
                | BuildingKind::OilRefinery
                | BuildingKind::Centrifuge
                | BuildingKind::RocketSilo
                | BuildingKind::Lab
                | BuildingKind::LaserTurret
                | BuildingKind::Radar
                | BuildingKind::Roboport
                | BuildingKind::Beacon
                | BuildingKind::InserterRegular
                | BuildingKind::InserterLong
                | BuildingKind::InserterFast
                | BuildingKind::InserterStack
        )
    }

    /// Whether this building uses fuel (coal) instead of electric power.
    pub const fn needs_fuel(self) -> bool {
        matches!(
            self,
            BuildingKind::StoneFurnace | BuildingKind::SteelFurnace | BuildingKind::Boiler
        )
    }
}

/// Grid position in tile coordinates. Origin at top-left corner of the map.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct GridPos {
    /// Column (increases rightward).
    pub x: i32,
    /// Row (increases downward).
    pub y: i32,
}

impl GridPos {
    /// Creates a new grid position.
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Returns the neighboring tile in the given direction.
    pub const fn neighbor(self, dir: Direction) -> GridPos {
        let (dx, dy) = dir.delta();
        GridPos {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    /// Manhattan distance to another position.
    pub fn manhattan_distance(self, other: GridPos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// Euclidean distance to another position.
    pub fn distance(self, other: GridPos) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Generational handle to a building in the [`Buildings`](crate::building::Buildings) arena.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct BuildingId {
    /// Index into the arena's internal vector.
    pub index: u32,
    /// Generation counter — prevents use-after-free with stale handles.
    pub generation: u32,
}

/// Generational handle to an item in the [`ItemPool`](crate::item::ItemPool).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ItemId {
    /// Index into the pool's internal vector.
    pub index: u32,
    /// Generation counter.
    pub generation: u32,
}
