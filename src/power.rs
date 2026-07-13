//! Power generation and distribution.
//!
//! Uses a **global power pool** model for simplicity and performance:
//! - All power-producing buildings add to a single pool each tick.
//! - All power-consuming buildings draw from the same pool.
//! - If demand > supply, a **brownout ratio** (0.0–1.0) slows all machines proportionally.
//!
//! This avoids expensive graph/network calculations while still creating meaningful
//! power management gameplay.

use crate::building::{self, Buildings};
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
    /// Energy currently buffered in accumulators (kJ-equivalent units).
    #[serde(default)]
    pub stored_energy: f32,
}

/// Resolves a supply/demand gap against accumulator storage for one tick.
///
/// - When supply exceeds demand, the surplus charges accumulators up to
///   `capacity` (limited by a per-tick charge rate).
/// - When demand exceeds supply, accumulators discharge to cover the shortfall
///   (limited by a per-tick discharge rate), raising effective supply.
///
/// Returns `(effective_supply, new_stored)`. Kept pure so it can be unit-tested
/// without constructing a world.
pub fn apply_accumulators(
    supply: f32,
    demand: f32,
    stored: f32,
    capacity: f32,
    rate: f32,
) -> (f32, f32) {
    if supply >= demand {
        // Charge from surplus.
        let surplus = supply - demand;
        let room = (capacity - stored).max(0.0);
        let charged = surplus.min(rate).min(room);
        (supply - charged, (stored + charged).min(capacity))
    } else {
        // Discharge to cover the deficit.
        let deficit = demand - supply;
        let drawn = deficit.min(rate).min(stored);
        (supply + drawn, (stored - drawn).max(0.0))
    }
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
                let base_draw = match building.kind {
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
                // Apply module power multiplier if machine has modules installed.
                let power_mult = if let Some(ms) = &building.machine_state {
                    let (_, pm, _) = building::module_effects(&ms.modules);
                    pm
                } else {
                    1.0
                };
                demand += base_draw * power_mult;
            }
            _ => {}
        }
    }

    // Accumulators smooth the supply/demand gap: charge on surplus, discharge on
    // deficit. Capacity scales with the number of accumulators placed.
    let accumulator_count = ids
        .iter()
        .filter(|&&bid| buildings.get(bid).map(|b| b.kind) == Some(BuildingKind::Accumulator))
        .count();
    let capacity = accumulator_count as f32 * ACCUMULATOR_CAPACITY;
    let charge_rate = capacity.max(0.0) * ACCUMULATOR_RATE_FRACTION;
    let (effective_supply, new_stored) =
        apply_accumulators(supply, demand, power.stored_energy, capacity, charge_rate);
    power.stored_energy = new_stored;

    power.supply = supply;
    power.demand = demand;
    power.satisfaction = if demand <= 0.0 || effective_supply >= demand {
        1.0
    } else {
        (effective_supply / demand).clamp(0.0, 1.0)
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
            // Load new fuel (coal, rocket fuel, or any solid fuel).
            let building = buildings.get_mut(*bid).unwrap();
            let ms = building.machine_state.as_mut().unwrap();
            crate::machine::refuel_from_buffer(ms);
        }
    }

    // Boilers also consume solid fuel.
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
            crate::machine::refuel_from_buffer(ms);
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
            if let Some(pos) = ms
                .input_buffer
                .iter()
                .position(|&r| r == Resource::NuclearFuelCell)
            {
                ms.input_buffer.remove(pos);
                ms.fuel_ticks = NUCLEAR_FUEL_CELL_TICKS;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::apply_accumulators;

    #[test]
    fn surplus_charges_up_to_the_rate_limit() {
        // 100 surplus, but only 40/tick may charge; plenty of room.
        let (eff, stored) = apply_accumulators(150.0, 50.0, 0.0, 1000.0, 40.0);
        assert_eq!(stored, 40.0);
        // Effective supply drops by the amount diverted into storage.
        assert_eq!(eff, 110.0);
    }

    #[test]
    fn surplus_never_exceeds_capacity() {
        let (_eff, stored) = apply_accumulators(1000.0, 0.0, 990.0, 1000.0, 500.0);
        assert_eq!(stored, 1000.0);
    }

    #[test]
    fn deficit_discharges_to_cover_shortfall() {
        // Demand 100, supply 60 -> 40 deficit, storage covers it.
        let (eff, stored) = apply_accumulators(60.0, 100.0, 500.0, 1000.0, 100.0);
        assert_eq!(eff, 100.0);
        assert_eq!(stored, 460.0);
    }

    #[test]
    fn discharge_is_capped_by_stored_energy() {
        // Only 10 stored, deficit is 40 -> can only add 10.
        let (eff, stored) = apply_accumulators(60.0, 100.0, 10.0, 1000.0, 100.0);
        assert_eq!(eff, 70.0);
        assert_eq!(stored, 0.0);
    }

    #[test]
    fn balanced_grid_leaves_storage_untouched() {
        let (eff, stored) = apply_accumulators(100.0, 100.0, 300.0, 1000.0, 50.0);
        assert_eq!(eff, 100.0);
        assert_eq!(stored, 300.0);
    }
}
