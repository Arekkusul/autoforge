//! Simplified fluid system for oil processing.
//!
//! Fluids are tracked per-building (refineries, chemical plants, pump jacks, storage tanks).
//! Pipe connections are implicit — buildings with fluid outputs connect to adjacent
//! buildings with fluid inputs if they share a pipe network. For simplicity, fluid
//! transfer is instantaneous within a connected group (no pressure simulation).
//!
//! # Design choice
//!
//! A full pressure-based pipe simulation would be expensive and complex. Instead,
//! we use a simplified model where each fluid-producing building pushes output to
//! adjacent fluid-consuming buildings directly. Pipes are visual connectors that
//! validate adjacency.

use crate::building::Buildings;
use crate::grid::Grid;
use crate::types::*;

/// Fluid storage for a building (stored inline alongside MachineState).
/// For simplicity, we track fluids as a resource count in the machine's
/// input/output buffers using special Resource variants.
///
/// Fluid buildings use their machine_state buffers:
/// - PumpJack: produces items representing fluid units (1 item = 100 units)
/// - Refinery: consumes oil items, produces petroleum/heavy oil/light oil items
/// - Chemical Plant: consumes petroleum + other items, produces products

/// Ticks fluid production for pump jacks.
///
/// Pump jacks work like miners but for oil — they extract crude oil items from
/// oil deposit tiles. Oil never depletes.
pub fn tick_pump_jacks(grid: &Grid, buildings: &mut Buildings) {
    let ids = buildings.alive_ids();

    for bid in ids {
        let building = match buildings.get(bid) {
            Some(b) => b,
            None => continue,
        };
        if building.kind != BuildingKind::PumpJack {
            continue;
        }

        let pos = building.pos;
        let ms = match &building.machine_state {
            Some(ms) => ms,
            None => continue,
        };

        // Check if on an oil deposit.
        let on_oil = grid
            .get_tile(pos)
            .and_then(|t| t.deposit)
            .map(|d| d == OreDeposit::Oil)
            .unwrap_or(false);

        if !on_oil {
            continue;
        }

        // If idle and output not full, start extracting.
        if ms.progress_ticks == 0 && ms.output_buffer.len() < 4 {
            let building = buildings.get_mut(bid).unwrap();
            let ms = building.machine_state.as_mut().unwrap();
            ms.progress_ticks = 60; // 3 seconds per oil unit
            ms.total_ticks = 60;
        }

        // Count down.
        let building = buildings.get_mut(bid).unwrap();
        let ms = building.machine_state.as_mut().unwrap();
        if ms.progress_ticks > 0 {
            ms.progress_ticks -= 1;
            if ms.progress_ticks == 0 {
                // For now, represent crude oil as Coal items (placeholder).
                // In a full implementation, we'd have a CrudeOil resource.
                // The refinery would consume these and output other fluid items.
                if ms.output_buffer.len() < 4 {
                    // Use a placeholder — the recipe system will handle conversion.
                    ms.output_buffer.push(Resource::Coal); // TODO: CrudeOil resource
                }
            }
        }
    }
}
