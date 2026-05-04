//! Procedural map generation.
//!
//! Generates the world with biomes, ore deposits, water bodies, forests, and
//! enemy nest locations. Uses a simple xorshift64 PRNG seeded from system time
//! so no external crate is needed.

use crate::constants::*;
use crate::grid::Grid;
use crate::types::*;

/// Simple xorshift64 pseudo-random number generator.
///
/// Fast, deterministic, and good enough for map generation.
/// No external dependency required.
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Creates a new RNG with the given seed. Seed must not be zero.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    /// Returns the next pseudo-random `u64`.
    pub fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    /// Returns a random `i32` in the range `[min, max]` (inclusive).
    pub fn range_i32(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        let range = (max - min + 1) as u64;
        min + (self.next_u64() % range) as i32
    }

    /// Returns a random `f32` in `[0.0, 1.0)`.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() % 1_000_000) as f32 / 1_000_000.0
    }
}

/// Generates the full map: terrain, deposits, water, forests, and returns nest positions.
///
/// The map is organized so that the player spawns near the center with basic resources
/// (iron, copper, coal, stone) close by, while rarer resources (uranium, oil) are
/// placed further out to encourage expansion.
pub fn generate_map(grid: &mut Grid, seed: u64) -> Vec<GridPos> {
    let mut rng = Rng::new(seed);
    let cx = grid.width / 2;
    let cy = grid.height / 2;

    // --- Clear starting area ---
    // Ensure the center 20×20 area is clean grass for the player to build.
    for dy in -10..=10 {
        for dx in -10..=10 {
            let pos = GridPos::new(cx + dx, cy + dy);
            if let Some(tile) = grid.get_tile_mut(pos) {
                tile.terrain = Terrain::Grass;
            }
        }
    }

    // --- Generate water bodies ---
    generate_water(grid, &mut rng, cx, cy);

    // --- Generate forest patches ---
    generate_forests(grid, &mut rng, cx, cy);

    // --- Scatter crash debris near spawn (cliff tiles in a trail) ---
    // Creates a visual "crash path" leading to the ship at center.
    for i in 0..12 {
        let angle = 2.2 + rng.next_f32() * 0.3; // roughly northwest
        let dist = 12.0 + i as f32 * 4.0 + rng.next_f32() * 3.0;
        let dx = cx + (angle.cos() * dist) as i32;
        let dy = cy + (angle.sin() * dist) as i32;
        for oy in -1..=1 {
            for ox in -1..=1 {
                if rng.next_f32() > 0.5 { continue; }
                let pos = GridPos::new(dx + ox, dy + oy);
                if let Some(tile) = grid.get_tile_mut(pos) {
                    if tile.terrain == Terrain::Grass && tile.deposit.is_none() {
                        tile.terrain = Terrain::Cliff;
                    }
                }
            }
        }
    }

    // --- Generate desert patches (sandy areas, less pollution absorption) ---
    for _ in 0..15 {
        let dx = rng.range_i32(40, grid.width - 40);
        let dy = rng.range_i32(40, grid.height - 40);
        let radius = rng.range_i32(5, 15);
        for oy in -radius..=radius {
            for ox in -radius..=radius {
                if ox * ox + oy * oy > radius * radius {
                    continue;
                }
                let pos = GridPos::new(dx + ox, dy + oy);
                let dist_to_center = pos.distance(GridPos::new(cx, cy));
                if dist_to_center > 20.0 {
                    if let Some(tile) = grid.get_tile_mut(pos) {
                        if tile.terrain == Terrain::Grass && tile.deposit.is_none() {
                            tile.terrain = Terrain::Desert;
                        }
                    }
                }
            }
        }
    }

    // --- Generate ore deposits (2×2 nodes) ---
    // Starter patches near center — guaranteed resources for early game.
    place_ore_cluster_2x2(grid, &mut rng, OreDeposit::Iron, cx - 15, cy - 10, 3, 8000);
    place_ore_cluster_2x2(grid, &mut rng, OreDeposit::Copper, cx + 12, cy - 8, 3, 6000);
    place_ore_cluster_2x2(grid, &mut rng, OreDeposit::Coal, cx - 8, cy + 14, 2, 6000);
    place_ore_cluster_2x2(grid, &mut rng, OreDeposit::Stone, cx + 10, cy + 12, 2, 4000);

    // Common ores — scattered around the map (fewer, but each is 2×2).
    let common_ores = [
        (OreDeposit::Iron, 10),
        (OreDeposit::Copper, 10),
        (OreDeposit::Coal, 8),
        (OreDeposit::Stone, 6),
    ];
    for (ore, count) in common_ores {
        for _ in 0..count {
            let nodes = rng.range_i32(2, 5); // 2-5 nodes per cluster
            let ox = rng.range_i32(30, grid.width - 30);
            let oy = rng.range_i32(30, grid.height - 30);
            let amount = rng.range_i32(3000, 10000) as u32;
            place_ore_cluster_2x2(grid, &mut rng, ore, ox, oy, nodes, amount);
        }
    }

    // Mid-distance ores — Tin, Sulfur (moderate distance from center).
    let mid_ores = [
        (OreDeposit::Tin, 8, 50.0, 150.0),
        (OreDeposit::Sulfur, 6, 60.0, 160.0),
    ];
    for (ore, count, min_dist, max_dist) in mid_ores {
        for _ in 0..count {
            let angle = rng.next_f32() * std::f32::consts::TAU;
            let dist = min_dist + rng.next_f32() * (max_dist - min_dist);
            let ox = cx + (angle.cos() * dist) as i32;
            let oy = cy + (angle.sin() * dist) as i32;
            let nodes = rng.range_i32(2, 4);
            place_ore_cluster_2x2(grid, &mut rng, ore, ox, oy, nodes, 5000);
        }
    }

    // Rare ores — Gold, Crystal, Uranium (far from center).
    let rare_ores = [
        (OreDeposit::Gold, 5, 100.0, 200.0),
        (OreDeposit::Crystal, 5, 90.0, 190.0),
        (OreDeposit::Uranium, 4, 130.0, 220.0),
    ];
    for (ore, count, min_dist, max_dist) in rare_ores {
        for _ in 0..count {
            let angle = rng.next_f32() * std::f32::consts::TAU;
            let dist = min_dist + rng.next_f32() * (max_dist - min_dist);
            let ox = cx + (angle.cos() * dist) as i32;
            let oy = cy + (angle.sin() * dist) as i32;
            let nodes = rng.range_i32(1, 3);
            place_ore_cluster_2x2(grid, &mut rng, ore, ox, oy, nodes, 3000);
        }
    }

    // Oil wells — 2x2 footprint, medium-far distance.
    for _ in 0..10 {
        let angle = rng.next_f32() * std::f32::consts::TAU;
        let dist = rng.range_i32(60, 180) as f32;
        let ox = cx + (angle.cos() * dist) as i32;
        let oy = cy + (angle.sin() * dist) as i32;
        place_ore_node_2x2(grid, OreDeposit::Oil, ox, oy, u32::MAX);
    }

    // --- Generate enemy nests ---
    let mut nests = Vec::new();
    for _ in 0..INITIAL_NEST_COUNT {
        let angle = rng.next_f32() * std::f32::consts::TAU;
        let dist = NEST_MIN_DISTANCE + rng.next_f32() * 120.0;
        let nx = cx + (angle.cos() * dist) as i32;
        let ny = cy + (angle.sin() * dist) as i32;
        let pos = GridPos::new(nx, ny);
        if grid.in_bounds(pos) {
            if let Some(tile) = grid.get_tile(pos) {
                if tile.terrain.is_buildable() {
                    nests.push(pos);
                }
            }
        }
    }

    nests
}

/// Places a cluster of 2×2 ore nodes scattered around `(cx, cy)`.
///
/// `node_count` is how many 2×2 rocks to place in this cluster.
fn place_ore_cluster_2x2(
    grid: &mut Grid,
    rng: &mut Rng,
    ore: OreDeposit,
    cx: i32,
    cy: i32,
    node_count: i32,
    base_amount: u32,
) {
    for _ in 0..node_count {
        // Scatter nodes within a small radius of the cluster center.
        let ox = cx + rng.range_i32(-4, 4) * 2; // align to even coords for spacing
        let oy = cy + rng.range_i32(-4, 4) * 2;
        let variation = rng.range_i32(80, 120) as u32;
        let amount = base_amount * variation / 100;
        place_ore_node_2x2(grid, ore, ox, oy, amount);
    }
}

/// Places a single 2×2 ore node with `(ox, oy)` as the top-left corner.
///
/// All 4 tiles get the deposit type. The top-left tile is marked as
/// `ore_origin = true` so the renderer knows to draw the large rock sprite here.
fn place_ore_node_2x2(grid: &mut Grid, ore: OreDeposit, ox: i32, oy: i32, amount: u32) {
    // Check all 4 tiles are valid.
    let positions = [
        GridPos::new(ox, oy),
        GridPos::new(ox + 1, oy),
        GridPos::new(ox, oy + 1),
        GridPos::new(ox + 1, oy + 1),
    ];

    let all_valid = positions.iter().all(|pos| {
        grid.get_tile(*pos)
            .map(|t| t.terrain.is_buildable() && t.deposit.is_none())
            .unwrap_or(false)
    });

    if !all_valid {
        return;
    }

    for (i, pos) in positions.iter().enumerate() {
        if let Some(tile) = grid.get_tile_mut(*pos) {
            tile.deposit = Some(ore);
            tile.ore_amount = amount;
            tile.ore_origin = i == 0; // only top-left is the origin
        }
    }
}

/// Generates water bodies (lakes) on the map.
fn generate_water(grid: &mut Grid, rng: &mut Rng, cx: i32, cy: i32) {
    // Place one water source near the player for early-game steam power.
    place_water_body(grid, rng, cx + 20, cy, 4, 7);

    for _ in 0..WATER_BODY_COUNT {
        let wx = rng.range_i32(30, grid.width - 30);
        let wy = rng.range_i32(30, grid.height - 30);
        let min_r = rng.range_i32(3, 5);
        let max_r = rng.range_i32(6, 12);
        place_water_body(grid, rng, wx, wy, min_r, max_r);
    }
}

/// Places an irregularly shaped water body.
fn place_water_body(grid: &mut Grid, rng: &mut Rng, cx: i32, cy: i32, min_r: i32, max_r: i32) {
    for dy in -max_r..=max_r {
        for dx in -max_r..=max_r {
            let dist_sq = dx * dx + dy * dy;
            // Irregular edge: use varying radius per angle.
            let threshold = rng.range_i32(min_r * min_r, max_r * max_r);
            if dist_sq <= threshold {
                let pos = GridPos::new(cx + dx, cy + dy);
                let grid_center = GridPos::new(grid.width / 2, grid.height / 2);
                let dist_to_center = pos.distance(grid_center);
                if dist_to_center > 15.0 {
                    if let Some(tile) = grid.get_tile_mut(pos) {
                        tile.terrain = Terrain::Water;
                        tile.deposit = None;
                    }
                }
            }
        }
    }
}

/// Generates forest patches across the map.
fn generate_forests(grid: &mut Grid, rng: &mut Rng, cx: i32, cy: i32) {
    for _ in 0..TREE_PATCH_COUNT {
        let fx = rng.range_i32(15, grid.width - 15);
        let fy = rng.range_i32(15, grid.height - 15);
        let radius = rng.range_i32(4, 10);

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy > radius * radius {
                    continue;
                }
                // Sparse edges: skip some tiles randomly.
                if rng.next_f32() > 0.7 && (dx * dx + dy * dy) > (radius * radius / 2) {
                    continue;
                }
                let pos = GridPos::new(fx + dx, fy + dy);
                if let Some(tile) = grid.get_tile_mut(pos) {
                    // Don't place forest on water, starting area, or existing deposits.
                    let dist_to_center = pos.distance(GridPos::new(cx, cy));
                    if tile.terrain == Terrain::Grass && dist_to_center > 12.0 {
                        tile.terrain = Terrain::Forest;
                    }
                }
            }
        }
    }
}
