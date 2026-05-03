//! Power generation and distribution.
//!
//! Uses a **global power pool** model for simplicity and performance:
//! - All power-producing buildings add to a single pool each tick.
//! - All power-consuming buildings draw from the same pool.
//! - If demand > supply, a **brownout ratio** (0.0–1.0) slows all machines proportionally.
//!
//! This avoids expensive graph/network calculations while still creating meaningful
//! power management gameplay.

use crate::building::Buildings;
use crate::constants::*;
use crate::daynight::DayNightState;
use crate::types::*;

/// Power state for the entire factory.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct PowerState {
    /// Total power supply this tick (kW).
    pub supply: f32,
    /// Total power demand this tick (kW).
    pub demand: f32,
    /// Satisfaction ratio (0.0–1.0). Machines run at this fraction of full speed.
    /// 1.0 = fully powered, 0.5 = half speed, 0.0 = no power.
    pub satisfaction: f32,
}

/// Recalculates power supply and demand.
///
/// Call once per tick (or every few ticks — power changes slowly).
pub fn update_power(buildings: &mut Buildings, power: &mut PowerState, daynight: &DayNightState) {
    let mut supply = 0.0f32;
    let mut demand = 0.0f32;

    let ids = buildings.alive_ids();

    for bid in &ids {
        let building = match buildings.get(*bid) {
            Some(b) => b,
            None => continue,
        };

        match building.kind {
            // --- Power producers ---
            BuildingKind::SteamEngine => {
                // Steam engine produces power if it has fuel in its buffer.
                if let Some(ms) = &building.machine_state {
                    if ms.fuel_ticks > 0 || !ms.input_buffer.is_empty() {
                        supply += STEAM_ENGINE_POWER;
                    }
                }
            }
            BuildingKind::SolarPanel => {
                // Solar output depends on daylight.
                supply += SOLAR_PANEL_POWER * daynight.solar_multiplier();
            }
            BuildingKind::NuclearReactor => {
                // Nuclear reactor produces massive power when fueled.
                if let Some(ms) = &building.machine_state {
                    if ms.fuel_ticks > 0 || !ms.input_buffer.is_empty() {
                        supply += NUCLEAR_REACTOR_POWER;
                    }
                }
            }

            // --- Power consumers ---
            _ if building.kind.needs_power() => {
                let draw = match building.kind {
                    BuildingKind::Miner => MINER_POWER_DRAW,
                    BuildingKind::PumpJack | BuildingKind::WaterPump => MINER_POWER_DRAW,
                    BuildingKind::ElectricFurnace => ELECTRIC_SMELTER_POWER_DRAW,
                    BuildingKind::AssemblerT1 => ASSEMBLER_POWER_DRAW,
                    BuildingKind::AssemblerT2 => ASSEMBLER_POWER_DRAW * 1.3,
                    BuildingKind::AssemblerT3 => ASSEMBLER_POWER_DRAW * 1.6,
                    BuildingKind::ChemicalPlant => CHEMICAL_PLANT_POWER_DRAW,
                    BuildingKind::OilRefinery => REFINERY_POWER_DRAW,
                    BuildingKind::Lab => LAB_POWER_DRAW,
                    BuildingKind::LaserTurret => LASER_TURRET_POWER_DRAW,
                    BuildingKind::Radar => RADAR_POWER_DRAW,
                    BuildingKind::Centrifuge => CENTRIFUGE_POWER_DRAW,
                    BuildingKind::Roboport => ROBOPORT_POWER_DRAW,
                    BuildingKind::Beacon => BEACON_POWER_DRAW,
                    BuildingKind::RocketSilo => ROCKET_SILO_POWER_DRAW,
                    BuildingKind::InserterRegular
                    | BuildingKind::InserterLong
                    | BuildingKind::InserterFast
                    | BuildingKind::InserterStack => 10.0, // inserters use minimal power
                    _ => 50.0,
                };
                demand += draw;
            }
            _ => {}
        }
    }

    power.supply = supply;
    power.demand = demand;
    power.satisfaction = if demand <= 0.0 {
        1.0
    } else if supply >= demand {
        1.0
    } else {
        (supply / demand).clamp(0.0, 1.0)
    };

    // Steam engines consume coal from their input buffer.
    for bid in &ids {
        let building = match buildings.get(*bid) {
            Some(b) => b,
            None => continue,
        };
        if building.kind != BuildingKind::SteamEngine {
            continue;
        }

        let ms = match &building.machine_state {
            Some(ms) => ms,
            None => continue,
        };

        if ms.fuel_ticks > 0 {
            // Burn fuel.
            let building = buildings.get_mut(*bid).unwrap();
            let ms = building.machine_state.as_mut().unwrap();
            ms.fuel_ticks -= 1;
        } else if !ms.input_buffer.is_empty() {
            // Load new fuel.
            let building = buildings.get_mut(*bid).unwrap();
            let ms = building.machine_state.as_mut().unwrap();
            // Consume first coal-like item from input buffer.
            if let Some(pos) = ms.input_buffer.iter().position(|&r| r == Resource::Coal) {
                ms.input_buffer.remove(pos);
                ms.fuel_ticks = COAL_FUEL_TICKS;
            }
        }
    }

    // Boilers also consume coal.
    for bid in &ids {
        let building = match buildings.get(*bid) {
            Some(b) => b,
            None => continue,
        };
        if building.kind != BuildingKind::Boiler {
            continue;
        }

        let ms = match &building.machine_state {
            Some(ms) => ms,
            None => continue,
        };

        if ms.fuel_ticks > 0 {
            let building = buildings.get_mut(*bid).unwrap();
            let ms = building.machine_state.as_mut().unwrap();
            ms.fuel_ticks -= 1;
        } else if !ms.input_buffer.is_empty() {
            let building = buildings.get_mut(*bid).unwrap();
            let ms = building.machine_state.as_mut().unwrap();
            if let Some(pos) = ms.input_buffer.iter().position(|&r| r == Resource::Coal) {
                ms.input_buffer.remove(pos);
                ms.fuel_ticks = COAL_FUEL_TICKS;
            }
        }
    }

    // Nuclear reactors consume fuel cells (much longer burn time).
    for bid in &ids {
        let building = match buildings.get(*bid) {
            Some(b) => b,
            None => continue,
        };
        if building.kind != BuildingKind::NuclearReactor {
            continue;
        }
        let ms = match &building.machine_state {
            Some(ms) => ms,
            None => continue,
        };

        if ms.fuel_ticks > 0 {
            let building = buildings.get_mut(*bid).unwrap();
            let ms = building.machine_state.as_mut().unwrap();
            ms.fuel_ticks -= 1;
        } else if !ms.input_buffer.is_empty() {
            let building = buildings.get_mut(*bid).unwrap();
            let ms = building.machine_state.as_mut().unwrap();
            if let Some(pos) = ms.input_buffer.iter().position(|&r| r == Resource::NuclearFuelCell) {
                ms.input_buffer.remove(pos);
                ms.fuel_ticks = NUCLEAR_FUEL_CELL_TICKS;
            }
        }
    }
}
