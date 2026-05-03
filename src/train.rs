//! Simplified train/rail logistics system.
//!
//! Trains are autonomous entities that move along placed rail tiles between
//! train stops. They carry items in a cargo buffer and load/unload at stops
//! based on their schedule.
//!
//! # Simplification
//!
//! Rather than full pathfinding on a rail graph, trains move in straight lines
//! along connected rail tiles. They have a schedule (list of stop positions)
//! and move toward the next stop each tick. Loading/unloading happens via
//! adjacent inserters (same as any other building).

use serde::{Deserialize, Serialize};

use crate::building::Buildings;
use crate::constants::*;
use crate::grid::Grid;
use crate::types::*;

/// A train entity moving on rails.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Train {
    /// World-space position.
    pub x: f32,
    pub y: f32,
    /// Current movement direction.
    pub direction: Direction,
    /// Speed in world pixels per tick.
    pub speed: f32,
    /// Schedule: list of train stop grid positions to visit in order.
    pub schedule: Vec<GridPos>,
    /// Current schedule index (which stop we're heading to).
    pub schedule_index: usize,
    /// Cargo buffer (items being transported).
    pub cargo: Vec<Resource>,
    /// Max cargo capacity.
    pub cargo_capacity: usize,
    /// Whether the train is currently waiting at a stop.
    pub waiting: bool,
    /// Ticks remaining at current stop (for loading/unloading time).
    pub wait_ticks: u32,
    /// Whether this train is alive/active.
    pub alive: bool,
}

/// Container for all trains.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Trains {
    pub list: Vec<Train>,
}

impl Trains {
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawns a new train at a train stop position.
    pub fn spawn_train(&mut self, stop_pos: GridPos) {
        self.list.push(Train {
            x: stop_pos.x as f32 * TILE_SIZE + TILE_SIZE * 0.5,
            y: stop_pos.y as f32 * TILE_SIZE + TILE_SIZE * 0.5,
            direction: Direction::East,
            speed: 2.0, // 2 pixels per tick = fast
            schedule: Vec::new(),
            schedule_index: 0,
            cargo: Vec::new(),
            cargo_capacity: 40,
            waiting: true,
            wait_ticks: 60,
            alive: true,
        });
    }
}

/// Ticks all trains: movement toward next stop, waiting at stops.
pub fn tick_trains(_grid: &Grid, _buildings: &Buildings, trains: &mut Trains) {
    for train in &mut trains.list {
        if !train.alive || train.schedule.is_empty() {
            continue;
        }

        if train.waiting {
            if train.wait_ticks > 0 {
                train.wait_ticks -= 1;
            } else {
                // Done waiting — move to next stop.
                train.waiting = false;
                train.schedule_index = (train.schedule_index + 1) % train.schedule.len();
            }
            continue;
        }

        // Move toward current target stop.
        let target = train.schedule[train.schedule_index];
        let target_world_x = target.x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
        let target_world_y = target.y as f32 * TILE_SIZE + TILE_SIZE * 0.5;

        let dx = target_world_x - train.x;
        let dy = target_world_y - train.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < train.speed * 2.0 {
            // Arrived at stop.
            train.x = target_world_x;
            train.y = target_world_y;
            train.waiting = true;
            train.wait_ticks = 100; // 5 seconds at stop for loading
        } else {
            // Move toward target.
            let nx = dx / dist;
            let ny = dy / dist;
            train.x += nx * train.speed;
            train.y += ny * train.speed;

            // Update direction for rendering.
            if nx.abs() > ny.abs() {
                train.direction = if nx > 0.0 { Direction::East } else { Direction::West };
            } else {
                train.direction = if ny > 0.0 { Direction::South } else { Direction::North };
            }
        }
    }
}
