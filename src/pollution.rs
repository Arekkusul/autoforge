//! Pollution generation and diffusion.
//!
//! Active machines generate pollution. Pollution diffuses across the grid each tick
//! and is absorbed by forest tiles. When pollution reaches enemy nests, it triggers
//! attack waves.

use crate::building::Buildings;
use crate::constants::*;
use crate::grid::Grid;
use crate::types::*;

/// Generates pollution from active machines and diffuses it across the grid.
pub fn tick_pollution(grid: &mut Grid, buildings: &Buildings) {
    let _width = grid.width;
    let _height = grid.height;

    // Step 1: Generate pollution from active machines.
    for (_bid, building) in buildings.iter() {
        if building.kind.needs_power() || building.kind.needs_fuel() {
            if let Some(ms) = &building.machine_state {
                if ms.progress_ticks > 0 {
                    // Machine is actively working — generate pollution.
                    if let Some(tile) = grid.get_tile_mut(building.pos) {
                        tile.pollution += POLLUTION_PER_KW_PER_TICK * 100.0;
                    }
                }
            }
        }
    }

    // Step 2: Diffuse pollution using sparse iteration.
    // Only process tiles that actually have pollution (skip the vast majority of empty tiles).
    // Collect polluted tile indices first to avoid full grid scan in the diffusion pass.
    let tiles = grid.tiles();
    let mut polluted_indices: Vec<usize> = Vec::with_capacity(256);
    for (idx, tile) in tiles.iter().enumerate() {
        if tile.pollution >= 0.01 {
            polluted_indices.push(idx);
        }
    }

    // Compute deltas only for polluted tiles and their neighbors.
    let mut deltas: Vec<(usize, f32)> = Vec::with_capacity(polluted_indices.len() * 5);
    for &idx in &polluted_indices {
        let pos = grid.index_to_pos(idx);
        let pollution = tiles[idx].pollution;
        let spread = pollution * POLLUTION_DIFFUSION_RATE;

        deltas.push((idx, -spread));

        for dir in Direction::all() {
            let npos = pos.neighbor(dir);
            if let Some(nidx) = grid.pos_to_index(npos) {
                deltas.push((nidx, spread * 0.25));
            }
        }
    }

    // Apply deltas.
    for (idx, delta) in deltas {
        let tiles = grid.tiles_mut();
        tiles[idx].pollution += delta;
    }

    // Tree absorption + clamp (only on polluted tiles).
    // Re-scan since new tiles may have gotten pollution from diffusion.
    let tiles = grid.tiles_mut();
    for tile in tiles.iter_mut() {
        if tile.pollution > 0.0 {
            if tile.terrain.has_trees() {
                tile.pollution = (tile.pollution - TREE_ABSORPTION_PER_TICK).max(0.0);
            }
            if tile.pollution < 0.001 {
                tile.pollution = 0.0;
            }
        }
    }
}

/// Calculates total pollution on the map (used for enemy wave thresholds).
pub fn total_pollution(grid: &Grid) -> f32 {
    let mut total = 0.0;
    for tile in grid.tiles() {
        total += tile.pollution;
    }
    total
}
