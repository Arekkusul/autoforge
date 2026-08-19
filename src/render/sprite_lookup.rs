//! Lookups from game data to atlas source rects and LOD fallback colors.

use macroquad::prelude::*;

use crate::sprites::SpriteAtlas;
use crate::types::*;

/// Returns the rotation angle (radians) for a direction.
///
/// Sprites are drawn facing North (up) by default.
pub(super) fn direction_to_rotation(dir: Direction) -> f32 {
    match dir {
        Direction::North => 0.0,
        Direction::East => std::f32::consts::FRAC_PI_2,
        Direction::South => std::f32::consts::PI,
        Direction::West => -std::f32::consts::FRAC_PI_2,
    }
}

/// Returns the atlas source rect for a building kind.
/// `anim_frame` cycles belt animation; `machine_frame` cycles machine animation (0=idle, 1=active).
pub fn building_src_rect(
    kind: BuildingKind,
    atlas: &SpriteAtlas,
    anim_frame: usize,
    machine_frame: usize,
) -> Rect {
    let mf = machine_frame.min(1);
    match kind {
        BuildingKind::BeltYellow => atlas.r_belt_yellow[anim_frame],
        BuildingKind::BeltRed => atlas.r_belt_red[anim_frame],
        BuildingKind::BeltBlue => atlas.r_belt_blue[anim_frame],
        BuildingKind::Miner => atlas.r_miner[mf],
        BuildingKind::PumpJack => atlas.r_pump_jack,
        BuildingKind::StoneFurnace => atlas.r_stone_furnace[mf],
        BuildingKind::SteelFurnace => atlas.r_steel_furnace[mf],
        BuildingKind::ElectricFurnace => atlas.r_electric_furnace,
        BuildingKind::AssemblerT1 | BuildingKind::AssemblerT2 | BuildingKind::AssemblerT3 => {
            atlas.r_assembler[mf]
        }
        BuildingKind::ChemicalPlant => atlas.r_chemical_plant,
        BuildingKind::OilRefinery => atlas.r_oil_refinery,
        BuildingKind::Lab => atlas.r_lab[mf],
        BuildingKind::Boiler => atlas.r_boiler,
        BuildingKind::SteamEngine => atlas.r_steam_engine,
        BuildingKind::SolarPanel => atlas.r_solar_panel,
        BuildingKind::NuclearReactor => atlas.r_nuclear_reactor,
        BuildingKind::Accumulator => atlas.r_accumulator,
        BuildingKind::StorageChest => atlas.r_chest,
        BuildingKind::GunTurret => atlas.r_gun_turret,
        BuildingKind::LaserTurret => atlas.r_laser_turret,
        BuildingKind::Wall | BuildingKind::Gate => atlas.r_wall,
        BuildingKind::InserterRegular
        | BuildingKind::InserterLong
        | BuildingKind::InserterFast
        | BuildingKind::InserterStack => atlas.r_inserter,
        BuildingKind::UndergroundBeltYellow
        | BuildingKind::UndergroundBeltRed
        | BuildingKind::UndergroundBeltBlue => atlas.r_underground_belt,
        BuildingKind::Splitter => atlas.r_splitter,
        BuildingKind::WaterPump => atlas.r_water_pump,
        BuildingKind::Centrifuge => atlas.r_chemical_plant,
        BuildingKind::RocketSilo => atlas.r_rocket_silo,
        BuildingKind::Radar => atlas.r_radar,
        BuildingKind::PipeSegment | BuildingKind::UndergroundPipe | BuildingKind::StorageTank => {
            atlas.r_pipe
        }
        BuildingKind::RailStraight | BuildingKind::RailCurved => atlas.r_rail,
        BuildingKind::TrainStop | BuildingKind::RailSignal => atlas.r_train_stop,
        BuildingKind::Roboport => atlas.r_roboport,
        BuildingKind::Beacon => atlas.r_beacon,
    }
}

/// Returns the atlas source rect for an item resource.
pub(super) fn item_src_rect(resource: Resource, atlas: &SpriteAtlas) -> Rect {
    match resource {
        Resource::IronOre => atlas.r_item_iron_ore,
        Resource::CopperOre => atlas.r_item_copper_ore,
        Resource::Coal => atlas.r_item_coal,
        Resource::Stone => atlas.r_item_stone,
        Resource::IronPlate => atlas.r_item_iron_plate,
        Resource::CopperPlate => atlas.r_item_copper_plate,
        Resource::SteelPlate => atlas.r_item_steel_plate,
        Resource::StoneBrick => atlas.r_item_stone_brick,
        Resource::Gear => atlas.r_item_gear,
        Resource::Wire => atlas.r_item_wire,
        Resource::Pipe => atlas.r_item_pipe,
        Resource::IronStick => atlas.r_item_iron_stick,
        Resource::GreenCircuit => atlas.r_item_green_circuit,
        Resource::RedCircuit => atlas.r_item_red_circuit,
        Resource::BlueCircuit => atlas.r_item_blue_circuit,
        Resource::ScienceRed => atlas.r_item_science_red,
        Resource::ScienceGreen => atlas.r_item_science_green,
        Resource::ScienceBlue => atlas.r_item_science_blue,
        Resource::SciencePurple => atlas.r_item_science_purple,
        Resource::ScienceYellow => atlas.r_item_science_yellow,
        Resource::Sulfur => atlas.r_item_sulfur,
        Resource::Plastic => atlas.r_item_plastic,
        Resource::Battery => atlas.r_item_battery,
        Resource::BasicAmmo | Resource::PiercingAmmo => atlas.r_item_ammo,
        Resource::Grenade => atlas.r_item_grenade,
        Resource::EngineUnit | Resource::ElectricEngine => atlas.r_item_engine,
        Resource::RocketPart => atlas.r_item_rocket_part,
        Resource::RocketFuel => atlas.r_item_rocket_fuel,
        Resource::Inserter => atlas.r_item_inserter,
        Resource::SpeedModule | Resource::EfficiencyModule | Resource::ProductivityModule => {
            atlas.r_item_speed_module
        }
        Resource::UraniumOre => atlas.r_item_uranium_ore,
        Resource::Rail => atlas.r_item_rail,
        Resource::Concrete => atlas.r_item_concrete,
        Resource::SolarPanelItem => atlas.r_item_solar_panel,
        Resource::AccumulatorItem => atlas.r_item_accumulator,
        Resource::FlyingRobotFrame => atlas.r_item_robot_frame,
        Resource::Uranium235 => atlas.r_item_uranium_235,
        Resource::Uranium238 => atlas.r_item_uranium_238,
        Resource::NuclearFuelCell => atlas.r_item_nuclear_fuel,
        Resource::LowDensityStructure => atlas.r_item_low_density,
        _ => atlas.r_item_iron_ore, // SpaceScience only
    }
}

/// Returns a simple color for LOD 2 building rendering (colored rectangle).
/// Returns a color for items at LOD 1 (colored dots when zoomed out).
pub(super) fn item_lod_color(resource: Resource) -> Color {
    match resource {
        Resource::IronOre | Resource::IronPlate => Color::new(0.55, 0.55, 0.6, 1.0),
        Resource::CopperOre | Resource::CopperPlate => Color::new(0.85, 0.55, 0.2, 1.0),
        Resource::Coal => Color::new(0.2, 0.2, 0.2, 1.0),
        Resource::Stone | Resource::StoneBrick => Color::new(0.6, 0.6, 0.55, 1.0),
        Resource::Gear => Color::new(0.5, 0.5, 0.55, 1.0),
        Resource::Wire => Color::new(0.8, 0.5, 0.15, 1.0),
        Resource::GreenCircuit => Color::new(0.2, 0.8, 0.3, 1.0),
        Resource::RedCircuit => Color::new(0.8, 0.2, 0.2, 1.0),
        Resource::ScienceRed => Color::new(1.0, 0.2, 0.2, 1.0),
        Resource::ScienceGreen => Color::new(0.2, 1.0, 0.2, 1.0),
        _ => Color::new(0.7, 0.7, 0.3, 1.0),
    }
}

pub(super) fn building_lod_color(kind: BuildingKind) -> Color {
    match kind {
        BuildingKind::BeltYellow => Color::new(0.45, 0.4, 0.15, 1.0),
        BuildingKind::BeltRed => Color::new(0.55, 0.2, 0.15, 1.0),
        BuildingKind::BeltBlue => Color::new(0.15, 0.2, 0.55, 1.0),
        BuildingKind::Miner | BuildingKind::PumpJack => Color::new(0.6, 0.4, 0.1, 1.0),
        BuildingKind::StoneFurnace | BuildingKind::SteelFurnace | BuildingKind::ElectricFurnace => {
            Color::new(0.55, 0.35, 0.15, 1.0) // warm orange-brown (distinct from red belts)
        }
        BuildingKind::AssemblerT1 | BuildingKind::AssemblerT2 | BuildingKind::AssemblerT3 => {
            Color::new(0.2, 0.3, 0.55, 1.0) // darker blue (distinct from solar)
        }
        BuildingKind::Lab => Color::new(0.45, 0.2, 0.55, 1.0),
        BuildingKind::GunTurret => Color::new(0.45, 0.45, 0.4, 1.0), // warm gray
        BuildingKind::LaserTurret => Color::new(0.3, 0.4, 0.7, 1.0), // blue-gray (distinct from gun)
        BuildingKind::Wall | BuildingKind::Gate => Color::new(0.35, 0.35, 0.3, 1.0),
        BuildingKind::Boiler | BuildingKind::SteamEngine => Color::new(0.15, 0.4, 0.35, 1.0),
        BuildingKind::SolarPanel => Color::new(0.1, 0.15, 0.45, 1.0), // deep blue (distinct from assembler)
        BuildingKind::NuclearReactor => Color::new(0.2, 0.5, 0.3, 1.0), // green glow
        BuildingKind::StorageChest => Color::new(0.5, 0.4, 0.2, 1.0), // wooden brown
        _ => Color::new(0.3, 0.3, 0.3, 1.0),
    }
}
