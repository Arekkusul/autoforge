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
use crate::grid::Grid;
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

/// Computes how much of a solar panel's sunlight reaches it past adjacent
/// obstructions (1.0 = fully unobstructed, 0.0 = fully shadowed).
///
/// Only the four cardinal neighbors are checked; each building there whose
/// [`BuildingKind::casts_shadow`] is `true` blocks
/// [`SOLAR_SHADOW_PER_OBSTRUCTION`] of the panel's output. Kept pure (no
/// [`DayNightState`] dependency) so obstruction and daylight can be tested
/// independently.
pub fn solar_shadow_factor(grid: &Grid, buildings: &Buildings, pos: GridPos) -> f32 {
    let obstructions = Direction::all()
        .iter()
        .filter(|&&dir| {
            grid.get_tile(pos.neighbor(dir))
                .and_then(|tile| tile.building)
                .and_then(|bid| buildings.get(bid))
                .is_some_and(|b| b.kind.casts_shadow())
        })
        .count();

    (1.0 - obstructions as f32 * SOLAR_SHADOW_PER_OBSTRUCTION).clamp(0.0, 1.0)
}

/// Recalculates power supply and demand.
///
/// Call once per tick (or every few ticks — power changes slowly).
pub fn update_power(
    buildings: &mut Buildings,
    power: &mut PowerState,
    daynight: &DayNightState,
    grid: &Grid,
) {
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
                // Solar output depends on daylight and on shadows cast by
                // adjacent structures.
                let shadow = solar_shadow_factor(grid, buildings, building.pos);
                supply += SOLAR_PANEL_POWER * daynight.solar_multiplier() * shadow;
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
    //
    // Each `bid` in `ids` is a snapshot taken at the top of this function; a
    // building can only vanish between the snapshot and here if this
    // function itself removed it (it never does), but the lookup is kept
    // fallible rather than `.unwrap()`ed so a future change can't turn a
    // missing entity into a panic.
    for bid in &ids {
        let Some(building) = buildings.get_mut(*bid) else {
            continue;
        };
        if building.kind != BuildingKind::SteamEngine {
            continue;
        }
        let Some(ms) = building.machine_state.as_mut() else {
            continue;
        };

        if ms.fuel_ticks > 0 {
            // Burn fuel.
            ms.fuel_ticks -= 1;
        } else if !ms.input_buffer.is_empty() {
            // Load new fuel (coal, rocket fuel, or any solid fuel).
            crate::machine::refuel_from_buffer(ms);
        }
    }

    // Boilers also consume solid fuel.
    for bid in &ids {
        let Some(building) = buildings.get_mut(*bid) else {
            continue;
        };
        if building.kind != BuildingKind::Boiler {
            continue;
        }
        let Some(ms) = building.machine_state.as_mut() else {
            continue;
        };

        if ms.fuel_ticks > 0 {
            ms.fuel_ticks -= 1;
        } else if !ms.input_buffer.is_empty() {
            crate::machine::refuel_from_buffer(ms);
        }
    }

    // Nuclear reactors consume fuel cells (much longer burn time).
    for bid in &ids {
        let Some(building) = buildings.get_mut(*bid) else {
            continue;
        };
        if building.kind != BuildingKind::NuclearReactor {
            continue;
        }
        let Some(ms) = building.machine_state.as_mut() else {
            continue;
        };

        if ms.fuel_ticks > 0 {
            ms.fuel_ticks -= 1;
        } else if !ms.input_buffer.is_empty() {
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
    use super::*;
    use crate::building::{Building, MachineState};
    use crate::grid::Grid;

    fn engine_building(pos: GridPos) -> Building {
        Building {
            kind: BuildingKind::SteamEngine,
            pos,
            direction: Direction::East,
            machine_state: Some(MachineState::new()),
            hp: 1.0,
            max_hp: 1.0,
            underground_pair: None,
        }
    }

    fn world() -> (Grid, Buildings) {
        (Grid::new(16, 16), Buildings::new())
    }

    fn solar_building(pos: GridPos) -> Building {
        Building {
            kind: BuildingKind::SolarPanel,
            pos,
            direction: Direction::East,
            machine_state: None,
            hp: 1.0,
            max_hp: 1.0,
            underground_pair: None,
        }
    }

    /// A tall structure used to test shadow-casting onto adjacent solar panels.
    fn obstruction_building(pos: GridPos) -> Building {
        Building {
            kind: BuildingKind::NuclearReactor,
            pos,
            direction: Direction::East,
            machine_state: None,
            hp: 1.0,
            max_hp: 1.0,
            underground_pair: None,
        }
    }

    // --- Stale-id hardening: a building removed (undo/demolition) between
    // ticks must never be able to panic `update_power`. ---

    #[test]
    fn update_power_survives_engine_demolished_mid_burn() {
        let (mut grid, mut buildings) = world();
        let pos = GridPos::new(1, 1);
        let engine = buildings.place(engine_building(pos), &mut grid).unwrap();
        buildings
            .get_mut(engine)
            .unwrap()
            .machine_state
            .as_mut()
            .unwrap()
            .fuel_ticks = 10;

        let mut power = PowerState::default();
        let daynight = DayNightState::default();

        // One tick while alive: burns fuel, contributes supply.
        update_power(&mut buildings, &mut power, &daynight, &grid);
        assert_eq!(power.supply, STEAM_ENGINE_POWER);

        // Undo/demolition removes the engine; a stale local variable (`engine`)
        // is all that is left pointing at it.
        buildings.remove(engine, &mut grid);

        // Must not panic, and supply must correctly drop to zero.
        update_power(&mut buildings, &mut power, &daynight, &grid);
        assert_eq!(
            power.supply, 0.0,
            "demolished engine no longer contributes power"
        );
    }

    #[test]
    fn update_power_survives_boiler_demolished_with_fuel_in_buffer() {
        let (mut grid, mut buildings) = world();
        let pos = GridPos::new(1, 1);
        let boiler = buildings
            .place(
                Building {
                    kind: BuildingKind::Boiler,
                    pos,
                    direction: Direction::East,
                    machine_state: Some(MachineState::new()),
                    hp: 1.0,
                    max_hp: 1.0,
                    underground_pair: None,
                },
                &mut grid,
            )
            .unwrap();
        buildings
            .get_mut(boiler)
            .unwrap()
            .machine_state
            .as_mut()
            .unwrap()
            .input_buffer
            .push(Resource::Coal);
        buildings.remove(boiler, &mut grid);

        let mut power = PowerState::default();
        let daynight = DayNightState::default();
        // Must not panic when the id captured above no longer resolves.
        update_power(&mut buildings, &mut power, &daynight, &grid);
        assert_eq!(power.supply, 0.0);
    }

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

    // --- Solar panel power: light level and shadow obstruction. ---

    #[test]
    fn solar_panel_output_scales_with_time_of_day() {
        let (mut grid, mut buildings) = world();
        buildings
            .place(solar_building(GridPos::new(5, 5)), &mut grid)
            .unwrap();
        let mut power = PowerState::default();

        let midnight = DayNightState {
            time: DAY_DURATION_SECS + NIGHT_DURATION_SECS * 0.5,
        };
        update_power(&mut buildings, &mut power, &midnight, &grid);
        assert_eq!(power.supply, 0.0, "no sunlight at night means no supply");

        let midday = DayNightState {
            time: DAY_DURATION_SECS * 0.5,
        };
        update_power(&mut buildings, &mut power, &midday, &grid);
        assert_eq!(
            power.supply, SOLAR_PANEL_POWER,
            "unobstructed panel hits full rated output at midday"
        );
    }

    #[test]
    fn solar_shadow_factor_is_full_with_no_neighboring_structures() {
        let (grid, buildings) = world();
        assert_eq!(
            solar_shadow_factor(&grid, &buildings, GridPos::new(5, 5)),
            1.0
        );
    }

    #[test]
    fn solar_shadow_factor_reduces_per_adjacent_tall_structure() {
        let (mut grid, mut buildings) = world();
        let pos = GridPos::new(5, 5);
        buildings
            .place(
                obstruction_building(pos.neighbor(Direction::North)),
                &mut grid,
            )
            .unwrap();

        let factor = solar_shadow_factor(&grid, &buildings, pos);
        assert!((factor - 0.8).abs() < 1e-6);
    }

    #[test]
    fn solar_shadow_factor_floors_out_when_boxed_in_on_all_sides() {
        let (mut grid, mut buildings) = world();
        let pos = GridPos::new(5, 5);
        for dir in Direction::all() {
            buildings
                .place(obstruction_building(pos.neighbor(dir)), &mut grid)
                .unwrap();
        }

        let factor = solar_shadow_factor(&grid, &buildings, pos);
        assert!((factor - 0.2).abs() < 1e-6);
    }

    #[test]
    fn solar_shadow_factor_ignores_flat_infrastructure() {
        let (mut grid, mut buildings) = world();
        let pos = GridPos::new(5, 5);
        buildings
            .place(
                Building {
                    kind: BuildingKind::BeltYellow,
                    pos: pos.neighbor(Direction::North),
                    direction: Direction::East,
                    machine_state: None,
                    hp: 1.0,
                    max_hp: 1.0,
                    underground_pair: None,
                },
                &mut grid,
            )
            .unwrap();

        assert_eq!(solar_shadow_factor(&grid, &buildings, pos), 1.0);
    }

    #[test]
    fn shadowed_solar_panel_produces_less_power_than_an_unshadowed_one() {
        let (mut grid, mut buildings) = world();
        let open_pos = GridPos::new(2, 2);
        let shaded_pos = GridPos::new(8, 8);
        buildings
            .place(solar_building(open_pos), &mut grid)
            .unwrap();
        buildings
            .place(solar_building(shaded_pos), &mut grid)
            .unwrap();
        buildings
            .place(
                obstruction_building(shaded_pos.neighbor(Direction::North)),
                &mut grid,
            )
            .unwrap();
        buildings
            .place(
                obstruction_building(shaded_pos.neighbor(Direction::South)),
                &mut grid,
            )
            .unwrap();

        let mut power = PowerState::default();
        let midday = DayNightState {
            time: DAY_DURATION_SECS * 0.5,
        };
        update_power(&mut buildings, &mut power, &midday, &grid);

        // One unobstructed panel (100%) + one boxed on 2 of 4 sides (60%).
        let expected = SOLAR_PANEL_POWER + SOLAR_PANEL_POWER * 0.6;
        assert!((power.supply - expected).abs() < 1e-4);
    }
}
