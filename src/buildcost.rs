//! Building costs — what resources are needed to place each building type.
//!
//! Players must have sufficient resources in their inventory to place buildings.
//! This creates meaningful resource management and prevents infinite spam.

use crate::types::*;
use std::collections::HashMap;

/// Returns the resource cost to place a building of the given kind.
///
/// Returns an empty slice for buildings that are free (like removing).
pub fn building_cost(kind: BuildingKind) -> &'static [(Resource, u32)] {
    match kind {
        // Logistics — cheap
        BuildingKind::BeltYellow => &[(Resource::IronPlate, 1)],
        BuildingKind::BeltRed => &[(Resource::IronPlate, 1), (Resource::Gear, 1)],
        BuildingKind::BeltBlue => &[(Resource::IronPlate, 1), (Resource::Gear, 1), (Resource::GreenCircuit, 1)],
        BuildingKind::UndergroundBeltYellow => &[(Resource::IronPlate, 5), (Resource::Gear, 5)],
        BuildingKind::UndergroundBeltRed => &[(Resource::IronPlate, 10), (Resource::Gear, 10)],
        BuildingKind::UndergroundBeltBlue => &[(Resource::IronPlate, 10), (Resource::Gear, 10), (Resource::GreenCircuit, 5)],
        BuildingKind::Splitter => &[(Resource::IronPlate, 5), (Resource::GreenCircuit, 2)],

        // Inserters
        BuildingKind::InserterRegular => &[(Resource::IronPlate, 1), (Resource::Gear, 1), (Resource::GreenCircuit, 1)],
        BuildingKind::InserterLong => &[(Resource::IronPlate, 2), (Resource::Gear, 1), (Resource::GreenCircuit, 1)],
        BuildingKind::InserterFast => &[(Resource::IronPlate, 2), (Resource::Gear, 2), (Resource::GreenCircuit, 2)],
        BuildingKind::InserterStack => &[(Resource::IronPlate, 5), (Resource::Gear, 5), (Resource::GreenCircuit, 5)],

        // Storage
        BuildingKind::StorageChest => &[(Resource::IronPlate, 4), (Resource::Stone, 2)],

        // Production — moderate
        BuildingKind::Miner => &[(Resource::IronPlate, 3), (Resource::Gear, 3)],
        BuildingKind::PumpJack => &[(Resource::IronPlate, 5), (Resource::Gear, 5), (Resource::GreenCircuit, 2)],
        BuildingKind::WaterPump => &[(Resource::IronPlate, 2), (Resource::Pipe, 2)],
        BuildingKind::StoneFurnace => &[(Resource::Stone, 5)],
        BuildingKind::SteelFurnace => &[(Resource::SteelPlate, 5), (Resource::StoneBrick, 5)],
        BuildingKind::ElectricFurnace => &[(Resource::IronPlate, 10), (Resource::GreenCircuit, 3), (Resource::StoneBrick, 5)],
        BuildingKind::AssemblerT1 => &[(Resource::IronPlate, 5), (Resource::Gear, 5), (Resource::GreenCircuit, 3)],
        BuildingKind::AssemblerT2 => &[(Resource::IronPlate, 9), (Resource::Gear, 5), (Resource::GreenCircuit, 5)],
        BuildingKind::AssemblerT3 => &[(Resource::SteelPlate, 5), (Resource::Gear, 10), (Resource::GreenCircuit, 10)],
        BuildingKind::ChemicalPlant => &[(Resource::SteelPlate, 5), (Resource::Gear, 5), (Resource::Pipe, 5), (Resource::GreenCircuit, 5)],
        BuildingKind::OilRefinery => &[(Resource::SteelPlate, 10), (Resource::Gear, 10), (Resource::Pipe, 10), (Resource::GreenCircuit, 10)],
        BuildingKind::Centrifuge => &[(Resource::SteelPlate, 50), (Resource::Gear, 50), (Resource::GreenCircuit, 50)],
        BuildingKind::RocketSilo => &[(Resource::SteelPlate, 200), (Resource::Gear, 100), (Resource::GreenCircuit, 100), (Resource::Concrete, 200)],
        BuildingKind::Lab => &[(Resource::IronPlate, 5), (Resource::Gear, 5), (Resource::GreenCircuit, 5)],

        // Power
        BuildingKind::Boiler => &[(Resource::Stone, 5), (Resource::IronPlate, 2)],
        BuildingKind::SteamEngine => &[(Resource::IronPlate, 5), (Resource::Gear, 5), (Resource::Pipe, 5)],
        BuildingKind::SolarPanel => &[(Resource::SteelPlate, 5), (Resource::CopperPlate, 10), (Resource::GreenCircuit, 10)],
        BuildingKind::Accumulator => &[(Resource::IronPlate, 5), (Resource::Battery, 5)],
        BuildingKind::NuclearReactor => &[(Resource::SteelPlate, 100), (Resource::GreenCircuit, 100), (Resource::Concrete, 100)],

        // Military
        BuildingKind::GunTurret => &[(Resource::IronPlate, 10), (Resource::Gear, 5), (Resource::CopperPlate, 5)],
        BuildingKind::LaserTurret => &[(Resource::SteelPlate, 10), (Resource::GreenCircuit, 10), (Resource::Battery, 5)],
        BuildingKind::Wall => &[(Resource::StoneBrick, 3)],
        BuildingKind::Gate => &[(Resource::StoneBrick, 3), (Resource::IronPlate, 2), (Resource::GreenCircuit, 1)],
        BuildingKind::Radar => &[(Resource::IronPlate, 5), (Resource::Gear, 5), (Resource::GreenCircuit, 5)],

        // Fluids
        BuildingKind::PipeSegment => &[(Resource::IronPlate, 1)],
        BuildingKind::UndergroundPipe => &[(Resource::IronPlate, 5), (Resource::Pipe, 5)],
        BuildingKind::StorageTank => &[(Resource::IronPlate, 10), (Resource::SteelPlate, 5)],

        // Trains
        BuildingKind::RailStraight => &[(Resource::Stone, 1), (Resource::SteelPlate, 1)],
        BuildingKind::RailCurved => &[(Resource::Stone, 2), (Resource::SteelPlate, 2)],
        BuildingKind::TrainStop => &[(Resource::IronPlate, 5), (Resource::SteelPlate, 3), (Resource::GreenCircuit, 3)],
        BuildingKind::RailSignal => &[(Resource::IronPlate, 2), (Resource::GreenCircuit, 1)],

        // Robots
        BuildingKind::Roboport => &[(Resource::SteelPlate, 20), (Resource::GreenCircuit, 20), (Resource::Gear, 20)],
        BuildingKind::Beacon => &[(Resource::SteelPlate, 10), (Resource::GreenCircuit, 10), (Resource::CopperPlate, 10)],
    }
}

/// Checks if the player inventory has enough resources for a building.
pub fn can_afford(inventory: &HashMap<Resource, u32>, kind: BuildingKind) -> bool {
    for &(resource, count) in building_cost(kind) {
        let have = inventory.get(&resource).copied().unwrap_or(0);
        if have < count {
            return false;
        }
    }
    true
}

/// Deducts building cost from inventory. Call only after [`can_afford`] returns true.
pub fn pay_cost(inventory: &mut HashMap<Resource, u32>, kind: BuildingKind) {
    for &(resource, count) in building_cost(kind) {
        let entry = inventory.entry(resource).or_insert(0);
        *entry = entry.saturating_sub(count);
    }
}

/// Refunds building cost to inventory (when removing a building).
pub fn refund_cost(inventory: &mut HashMap<Resource, u32>, kind: BuildingKind) {
    for &(resource, count) in building_cost(kind) {
        *inventory.entry(resource).or_insert(0) += count;
    }
}
